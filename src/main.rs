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
use std::io::{self, Write};
use std::path::PathBuf;

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
}


fn main() -> Result<()> {
    let args = Args::parse();

    match args.mode.as_str() {
        "clipboard" => handle_clipboard()?,
        "file" => {
            let path = args.path.context("file模式必须使用 --path 指定路径")?;
            handle_file_parallel(path)?;
        }
        _ => println!("❌ 未知模式。 请使用 --help 查看用法。")
    }
    Ok(())
}


/// 剪贴板模式逻辑
fn handle_clipboard() -> Result<()> {
    let mut clipboard = Clipboard::new().context("无法连接剪贴板")?;
    let input = clipboard.get_text().context("剪贴板中没有文本内容")?;

    println!("🚀 正在处理剪贴板数据 (长度: {})...", input.len());
    
    // 执行脱敏
    let output = ENGINE.mask_line(&input);
    
    clipboard.set_text(output.to_string()).context("无法写回剪贴板")?;
    println!("✅ 脱敏成功！");
    Ok(())
}

/// 文件模式逻辑：利用 Mmap + Rayon 并行块处理
fn handle_file_parallel(path: PathBuf) -> Result<()> {
    let file = File::open(&path).with_context(|| format!("无法打开文件: {:?}", path))?;
    let mmap = unsafe { Mmap::map(&file)? };

    println!("🚀 开启多核并行处理 (文件大小: {} bytes)", mmap.len());

    // 性能关键：par_split 按换行符切分数据块
    // map_chunk_size(1024) 减少细小任务调度带来的线程开销
    let processed_lines: Vec<String> = mmap
        .par_split(|&b| b == b'\n')
        .map(|chunk| {
            // 注意：大规模生产环境建议处理非 UTF-8 的兼容性，此处使用 Lossy 保证安全
            let line = String::from_utf8_lossy(chunk);
            ENGINE.mask_line(&line).into_owned()
        })
        .collect();

    // 高效批量写入输出
    let stdout = io::stdout();
    let mut handle = io::BufWriter::with_capacity(128 * 1024, stdout.lock());
    for line in processed_lines {
        writeln!(handle, "{}", line)?;
    }
    handle.flush()?;
    
    println!("✅ 文件脱敏处理完成。");
    Ok(())
}