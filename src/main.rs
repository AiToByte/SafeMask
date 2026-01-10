mod engine;
mod config;

use anyhow::{Context, Result};
use arboard::Clipboard;
use clap::Parser;
use engine::MaskEngine;
use memmap2::Mmap;
use once_cell::sync::Lazy;
use rayon::prelude::*;
use std::fs::File;
use std::io::{self, BufWriter, Write};
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::time::Instant;

// 使用 mimalloc 替代默认分配器，在高并发 String 操作下性能更佳
#[global_allocator]
static GLOBAL: mimalloc::MiMalloc = mimalloc::MiMalloc;

/// 全局静态引擎，确保规则只加载和编译一次
static ENGINE: Lazy<MaskEngine> = Lazy::new(|| {
    let rules = config::load_all_rules("rules");
    MaskEngine::new(rules)
});

#[derive(Parser, Debug)]
#[command(name = "safemask", version = "0.2.0", about = "High-performance Data Masking Tool")]
struct Args {
    /// 模式: clipboard (默认) 或 file
    #[arg(short, long, default_value = "clipboard")]
    mode: String,

    /// 文件路径 (仅 file 模式下有效)
    #[arg(short, long)]
    path: Option<PathBuf>,

     /// [输出] 文件路径 (可选，指定后将直接写文件而不经过 stdout)
    #[arg(short, long)]
    output: Option<PathBuf>,
}


fn main() -> Result<()> {
    let args = Args::parse();

    match args.mode.as_str() {
        "clipboard" => handle_clipboard()?,
        "file" => {
            let path = args.path.context("file模式必须使用 --path 指定路径")?;
            handle_file_ordered(path, args.output)?;
        }
        _ => println!("❌ 未知模式。 请使用 --help 查看用法。")
    }
    Ok(())
}

/// 剪切板处理逻辑
fn handle_clipboard() -> Result<()> {
    let mut clipboard = Clipboard::new().context("无法连接剪贴板")?;
    let input = clipboard.get_text().context("剪贴板中没有文本内容")?;

    println!("🚀 正在处理剪贴板数据 (长度: {})...", input.len());
    let output = ENGINE.mask_line(&input);
    
    clipboard.set_text(output.to_string()).context("无法回写剪贴板")?;
    println!("✅ 脱敏成功！内容已存回剪贴板。");
    Ok(())
}

/// 核心文件处理函数：并行扫描 + 直接流式写入, 不保证顺序
#[allow(dead_code)]
fn handle_file_parallel(input_path: PathBuf, output_path: Option<PathBuf>) -> Result<()> {
    let global_start = Instant::now();

    // 1. 内存映射输入文件
    let file = File::open(&input_path).with_context(|| format!("无法打开输入文件: {:?}", input_path))?;
    let mmap = unsafe { Mmap::map(&file)? };
    let file_size = mmap.len();
    let load_time = global_start.elapsed();

    // 2. 初始化输出流
    // 使用 Box<dyn Write + Send> 实现多态输出（文件或标准输出）
    let writer_raw: Box<dyn Write + Send> = if let Some(out_p) = output_path {
        Box::new(File::create(&out_p).with_context(|| format!("无法创建输出文件: {:?}", out_p))?)
    } else {
        Box::new(io::stdout())
    };

    // 使用 1MB 的大容量缓冲区，并用 Mutex 包装以支持并发写入
    let writer = Arc::new(Mutex::new(BufWriter::with_capacity(1024 * 1024, writer_raw)));

    println!("🚀 引擎就绪 | 线程数: {} | 文件大小: {:.2} MB", 
             rayon::current_num_threads(),
             file_size as f64 / 1024.0 / 1024.0);

    // 3. 并行流水线处理
    // 注意：这里不再 collect() 到 Vec，而是直接 for_each 写入
    mmap.par_split(|&b| b == b'\n')
        .for_each(|chunk| {
            // 将字节切片转换为字符串（零拷贝尝试）
            let line = String::from_utf8_lossy(chunk);
            
            // 执行脱敏引擎逻辑
            let masked = ENGINE.mask_line(&line);
            
            // 写入缓冲区（带锁保护）
            // 在高计算占比的任务中，锁竞争会被正则计算的耗时稀释
            let mut w = writer.lock().expect("写入锁冲突");
            let _ = writeln!(w, "{}", masked);
        });

    // 4. 强制刷新缓冲区并关闭
    let mut final_w = writer.lock().unwrap();
    final_w.flush()?;

    // 5. 性能报告
    let total_time = global_start.elapsed();
    let pure_calc_time = total_time - load_time;
    let throughput = (file_size as f64 / 1024.0 / 1024.0) / total_time.as_secs_f64();

    println!("\n--- ⚡ SafeMask 性能报告 ---");
    println!("📂 IO 加载耗时   : {:?}", load_time);
    println!("⚙️  核心处理耗时  : {:?}", pure_calc_time);
    println!("⏱️  总计运行时间  : {:?}", total_time);
    println!("🚀 平均处理吞吐  : {:.2} MB/s", throughput);
    println!("----------------------------");

    Ok(())
}


/// 文件模式：通过 IndexedParallelIterator 保证行顺序
fn handle_file_ordered(input_path: PathBuf, output_path: Option<PathBuf>) -> Result<()> {
    let global_start = Instant::now();

    // 1. 内存映射输入文件 (读取最快方案)
    let file = File::open(&input_path).with_context(|| format!("无法打开输入文件: {:?}", input_path))?;
    let mmap = unsafe { Mmap::map(&file)? };
    let file_size = mmap.len();
    let load_time = global_start.elapsed();

    println!("🚀 引擎加载成功 | 线程池大小: {} | 文件大小: {:.2} MB", 
             rayon::current_num_threads(),
             file_size as f64 / 1024.0 / 1024.0);

    // 2. 并行映射 (Map) + 保序收集 (Collect)
    // Rayon 的 collect 会保证最终生成的 Vec 顺序与原始切分顺序完全一致
    let processed_results: Vec<String> = mmap
        .par_split(|&b| b == b'\n')
        .map(|chunk| {
            // 将字节切片转换为字符串（零拷贝尝试）
            let line = String::from_utf8_lossy(chunk);
            // 核心脱敏计算 (CPU 密集型)
            ENGINE.mask_line(&line).into_owned()
        })
        .collect();

    let process_time = global_start.elapsed() - load_time;

    // 3. 顺序写入 (Sequential Write)
    // 此时已经得到了有序的 Vec<String>，直接顺序写入磁盘
    let writer_target: Box<dyn Write> = if let Some(out_p) = output_path {
        Box::new(File::create(&out_p)?)
    } else {
        Box::new(io::stdout())
    };

    let mut writer = BufWriter::with_capacity(1024 * 1024, writer_target);
    for line in processed_results {
        writeln!(writer, "{}", line)?;
    }
    writer.flush()?;

    // 4. 性能报告
    let total_time = global_start.elapsed();
    let throughput = (file_size as f64 / 1024.0 / 1024.0) / total_time.as_secs_f64();

    println!("\n--- ⚡ SafeMask 性能分析报告 ---");
    println!("📂 IO 读取/映射耗时: {:?}", load_time);
    println!("⚙️  并行保序计算耗时: {:?}", process_time);
    println!("⏱️  总计运行时间    : {:?}", total_time);
    println!("🚀 平均保序吞吐量  : {:.2} MB/s", throughput);
    println!("--------------------------------------");

    Ok(())
}