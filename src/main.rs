mod engine;
mod config;

use anyhow::{Context, Result};
use arboard::Clipboard;
use clap::Parser;
use engine::MaskEngine;
use memmap2::Mmap;
use std::collections::BTreeMap;
use once_cell::sync::Lazy;
use rayon::prelude::*;
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

// 流水线大块 (定义4MB)
const MACRO_CHUNK_SIZE: usize = 4 * 1024 *1024;

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
            handle_file_ordered_fast(path, args.output)?;
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
    // 脱敏处理
    let output = ENGINE.mask_line(input.as_bytes());
    // 转成utf-8类型的字符串
    let output_text = String::from_utf8_lossy(&output).into_owned();
    
    clipboard.set_text(output_text).context("无法回写剪贴板")?;
    println!("✅ 脱敏成功！内容已存回剪贴板。");
    Ok(())
}


/// 文件模式：通过 IndexedParallelIterator 保证行顺序
fn handle_file_ordered_fast(input_path: PathBuf, output_path: Option<PathBuf>) -> Result<()> {
    let global_start = Instant::now();

    // 基础参数构建
    let file = File::open(&input_path).with_context(|| format!("无法打开输入文件: {:?}", input_path))?;
    let mmap = unsafe { Mmap::map(&file)? };
    let file_size = mmap.len();
    let load_time = global_start.elapsed();

    println!("🚀 引擎加载成功 | 线程池大小: {} | 文件大小: {:.2} MB", 
             rayon::current_num_threads(),
             file_size / BUFFER_SIZE);

     // 1. 构建跨线程通道 (Channel)
    // 传输内容: (块序号, 该块脱敏后的字节数据)
    let (tx, rx) = crossbeam_channel::bounded::<(usize, Vec<u8>)>( rayon::current_num_threads() * 2);
    
    // 2. 创建 Stage 3: 专用有序写入线程
    let writer_handle = std::thread::spawn(move || -> Result<()> {
        let writer_target: Box<dyn Write> = if let Some(p) = output_path {
            Box::new(File::create(p)?)
        } else {
            Box::new(io::stdout())
        };
        let mut writer = BufWriter::with_capacity(BUFFER_SIZE, writer_target);

        let mut next_idx = 0;
        let mut pending_map: BTreeMap<usize, Vec<u8>> = BTreeMap::new(); // 优先级缓存，用于保序

        // 从通道接收处理好的块
        // 正确的接收方式
        drain_channel(rx, &mut pending_map, &mut writer, &mut next_idx)?;
        
        writer.flush()?;
        Ok(())
    });

     println!("🚀 Level 3 流水线启动 | 块大小: 4MB | 并行线程: {}", rayon::current_num_threads());

     // 3. Stage 1 & 2: 生产者与计算核心
    // 将 Mmap 划分为 Macro-Chunks 进行分发
    // 使用 par_chunks 结合 enumerate 获取块序号
    mmap.par_chunks(MACRO_CHUNK_SIZE)
    .enumerate()
    .for_each(|(idx, chunk)| {
        let mut chunk_output = Vec::with_capacity(chunk.len() + chunk.len()/10);

        // 处理每一行（简单版）
        let lines = chunk.split(|&b| b == b'\n').collect::<Vec<_>>();
        for (i, line) in lines.iter().enumerate() {
            if !line.is_empty() || i < lines.len() - 1 {
                let masked = ENGINE.mask_line(line);
                chunk_output.extend_from_slice(&masked);
                chunk_output.push(b'\n');
            }
        }

        if let Err(_) = tx.send((idx, chunk_output)) {
            eprintln!("通道关闭，丢弃块 {}", idx);
        }
    });

    // 只有这样，rx.recv() 才会收到 Err 并跳出 loop
    drop(tx); 
    // 等待写入线程完成工作
    writer_handle.join().expect("写入线程崩溃").expect("写入操作失败");
    let total_time = global_start.elapsed();
    println!("🚀 极限吞吐量: {:.2} MB/s", (file_size as f64 / 1024.0 / 1024.0) / total_time.as_secs_f64());

    // 5. 性能报告
       // 4. 性能报告
    let process_time = global_start.elapsed() - load_time;
    let total_time = global_start.elapsed();
    let throughput = (file_size as f64  / 1024.0 / 1024.0) / total_time.as_secs_f64();
    println!("\n--- ⚡ SafeMask 性能分析报告 ---");
    println!("📂 三阶段保序流水线并行处理耗时: {:?}", load_time);
    println!("⚙️  并行保序计算耗时: {:?}", process_time);
    println!("⏱️  总计运行时间    : {:?}", total_time);
    println!("🚀 平均保序吞吐量  : {:.2} MB/s", throughput);
    println!("--------------------------------------");



    Ok(())
}




fn drain_channel(
    rx: crossbeam_channel::Receiver<(usize, Vec<u8>)>,
    pending_map: &mut BTreeMap<usize, Vec<u8>>,
    writer: &mut BufWriter<Box<dyn Write>>,
    next_idx: &mut usize,
) -> Result<()> {
   loop {
        let (idx, data) = match rx.recv() {
            Ok((idx, data)) => (idx, data),
            Err(_) => break,           // 通道关闭，所有发送端都 drop 了
        };

        pending_map.insert(idx, data);

        while let Some(data) = pending_map.remove(next_idx) {
            writer.write_all(&data)?;
            *next_idx += 1;
        }
    }

    Ok(())
}