mod engine;
mod config;

use anyhow::{Context, Result};
use arboard::Clipboard;
use clap::Parser;
use engine::MaskEngine;
use memmap2::Mmap;
use once_cell::sync::Lazy;
use rayon::prelude::*;
use std::collections::BTreeMap;
use std::fs::File;
use std::io::{self, BufWriter, Write};
use std::path::PathBuf;
use std::time::Instant;

// 使用 mimalloc 替代默认分配器，在高并发 String 操作下性能更佳
#[global_allocator]
static GLOBAL: mimalloc::MiMalloc = mimalloc::MiMalloc;

/// 全局静态引擎，确保规则只加载和编译一次
static ENGINE: Lazy<MaskEngine> = Lazy::new(|| {
    let rules = config::load_all_rules("rules");
    MaskEngine::new(rules)
});

#[cfg(target_os = "windows")]
const BUFFER_SIZE: usize = 8 * 1024 * 1024; // Windows 侧重减少系统调用

#[cfg(target_os = "macos")]
const BUFFER_SIZE: usize = 16 * 1024 * 1024; // macOS 侧重喂饱高速 NVMe

#[cfg(target_os = "linux")]
const BUFFER_SIZE: usize = 4 * 1024 * 1024; // Linux 内核高效，4MB 即可保持极低内存占用

#[cfg(not(any(target_os = "windows", target_os = "macos", target_os = "linux")))]
const BUFFER_SIZE: usize = 1024 * 1024; // 其他系统默认 1MB

// 4MB 并行块
const MACRO_CHUNK_SIZE: usize = 4 * 1024 * 1024; // 4MB 并行块

#[derive(Parser, Debug)]
#[command(name = "safemask", version = "0.4.1", about = "High-performance Data Masking Tool")]
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
            handle_file_pipeline_ordered(path, args.output)?;
        }
        _ => println!("❌ 未知模式。 请使用 --help 查看用法。")
    }
    Ok(())
}

/// 剪切板处理逻辑
fn handle_clipboard() -> Result<()> {
    let mut clipboard = Clipboard::new().context("无法连接剪贴板")?;
    let input = clipboard.get_text().context("剪贴板空")?;
    println!("🚀 正在处理剪贴板数据...");
    let output_bytes = ENGINE.mask_line(input.as_bytes());
    let output_text = String::from_utf8_lossy(&output_bytes).into_owned();
    clipboard.set_text(output_text).context("回写失败")?;
    println!("✅ 脱敏成功！");
    Ok(())
}

/// 文件模式：三阶段保序流水线 (Mmap -> Rayon -> BTreeMap -> Writer)
fn handle_file_pipeline_ordered(input_path: PathBuf, output_path: Option<PathBuf>) -> Result<()> {
    let global_start = Instant::now();
    let file = File::open(&input_path)?;
    let mmap = unsafe { Mmap::map(&file)? };
    let file_size = mmap.len();

    // 1. 构建跨线程通信通道
    let (tx, rx) = crossbeam_channel::bounded::<(usize, Vec<u8>)>(rayon::current_num_threads() * 2);

    // 2. 启动 Stage 3: 顺序写入线程
    let writer_handle = std::thread::spawn(move || -> Result<()> {
        let writer_target: Box<dyn Write> = if let Some(p) = output_path {
            Box::new(File::create(p)?)
        } else {
            Box::new(io::stdout())
        };
        let mut writer = BufWriter::with_capacity(BUFFER_SIZE, writer_target);
        let mut next_idx = 0;
        let mut pending_map: BTreeMap<usize, Vec<u8>> = BTreeMap::new();

        while let Ok((idx, data)) = rx.recv() {
            pending_map.insert(idx, data);
            while let Some(data) = pending_map.remove(&next_idx) {
                writer.write_all(&data)?;
                next_idx += 1;
            }
        }
        writer.flush()?;
        Ok(())
    });

    println!("🚀 流水线启动 | 核心数: {} | 文件: {:.2} MB", 
             rayon::current_num_threads(), file_size as f64 / 1024.0 / 1024.0);

    // 3. Stage 1 & 2: 生产者与并行计算
    mmap.par_chunks(MACRO_CHUNK_SIZE)
        .enumerate()
        .for_each(|(idx, chunk)| {
            // 预分配内存：原始块大小 + 5% 缓冲区用于存放脱敏标签
            let mut chunk_output = Vec::with_capacity(chunk.len() + chunk.len() / 20);
            // 块内按行切割并脱敏
            for line in chunk.split(|&b| b == b'\n') {
                if !line.is_empty() {
                    let masked = ENGINE.mask_line(line);
                    chunk_output.extend_from_slice(&masked);
                }
                chunk_output.push(b'\n'); // 保持行结构
            }
            let _ = tx.send((idx, chunk_output));
        });

    // 4. 关闭通道并等待结束 
    // 必须手动释放 tx，否则 rx 会死锁
    drop(tx);
    writer_handle.join().unwrap()?;

    let total_time = global_start.elapsed();
    println!("\n--- ⚡ SafeMask v{} 性能报告 ---", env!("CARGO_PKG_VERSION"));
    println!("⏱️  总执行时间: {:?}", total_time);
    println!("🚀 极限吞吐量: {:.2} MB/s", (file_size as f64 / 1024.0 / 1024.0) / total_time.as_secs_f64());
    Ok(())
}


// /// 文件模式：通过 IndexedParallelIterator 保证行顺序
// #[allow(dead_code)]
// fn handle_file_ordered(input_path: PathBuf, output_path: Option<PathBuf>) -> Result<()> {
//     let global_start = Instant::now();

//     // 1. 内存映射输入文件 (读取最快方案)
//     let file = File::open(&input_path).with_context(|| format!("无法打开输入文件: {:?}", input_path))?;
//     let mmap = unsafe { Mmap::map(&file)? };
//     let file_size = mmap.len();
//     let load_time = global_start.elapsed();

//     println!("🚀 引擎加载成功 | 线程池大小: {} | 文件大小: {:.2} MB", 
//              rayon::current_num_threads(),
//              file_size as f64 / 1024.0 / 1024.0);

//     // 2. 并行映射 (Map) + 保序收集 (Collect)
//     // Rayon 的 collect 会保证最终生成的 Vec 顺序与原始切分顺序完全一致
//     let processed_results: Vec<String> = mmap
//         .par_split(|&b| b == b'\n')
//         .map(|chunk| {
//             // 将字节切片转换为字符串（零拷贝尝试）
//             let line = String::from_utf8_lossy(chunk);
//             // 核心脱敏计算 (CPU 密集型)
//             ENGINE.mask_line(&line).into_owned()
//         })
//         .collect();

//     let process_time = global_start.elapsed() - load_time;

//     // 3. 顺序写入 (Sequential Write)
//     // 此时已经得到了有序的 Vec<String>，直接顺序写入磁盘
//     let writer_target: Box<dyn Write> = if let Some(out_p) = output_path {
//         Box::new(File::create(&out_p)?)
//     } else {
//         Box::new(io::stdout())
//     };

//     let mut writer = BufWriter::with_capacity(1024 * 1024, writer_target);
//     for line in processed_results {
//         writeln!(writer, "{}", line)?;
//     }
//     writer.flush()?;

//     // 4. 性能报告
//     let total_time = global_start.elapsed();
//     let throughput = (file_size as f64 / 1024.0 / 1024.0) / total_time.as_secs_f64();

//     println!("\n--- ⚡ SafeMask 性能分析报告 ---");
//     println!("📂 IO 读取/映射耗时: {:?}", load_time);
//     println!("⚙️  并行保序计算耗时: {:?}", process_time);
//     println!("⏱️  总计运行时间    : {:?}", total_time);
//     println!("🚀 平均保序吞吐量  : {:.2} MB/s", throughput);
//     println!("--------------------------------------");

//     Ok(())
// }