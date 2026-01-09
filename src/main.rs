mod engine;

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


/// 全局单例引擎，避免重复编译正则
static ENGINE: Lazy<MaskEngine> = Lazy::new(MaskEngine::new);

#[derive(Parser, Debug)]
#[command(name = "safemask", version, about = "High-performance Data Masking Tool")]
struct Args {
    /// 模式: clipboard (默认) 或 file
    #[arg(short, long, default_value = "clipboard")]
    mode: String,

    /// 文件路径 (仅 file 模式)
    #[arg(short, long)]
    path: Option<PathBuf>,
}


fn main() -> Result<()> {
    let args = Args::parse();

    match args.mode.as_str() {
        "clipboard" => {
            let mut clipboard = Clipboard::new().context("初始化剪贴板失败")?;
            let input = clipboard.get_text().context("剪贴板为空")?;
            
            println!("🚀 正在处理剪贴板数据...");
            let output = ENGINE.mask_line(&input);
            
            clipboard.set_text(output.to_string()).context("回写剪贴板失败")?;
            println!("✅ 脱敏完成！");
        }
        "file" => {
            let path = args.path.context("请指定 --path")?;
            let file = File::open(&path)?;
            let mmap = unsafe { Mmap::map(&file)? };

            println!("🚀 Mmap 成功，开始并行脱敏 (文件大小: {} bytes)", mmap.len());

            // 利用 Rayon 并行处理字节流
            let output: Vec<String> = mmap
                .par_split(|&b| b == b'\n')
                .map(|chunk| {
                    let line = String::from_utf8_lossy(chunk);
                    ENGINE.mask_line(&line).into_owned()
                })
                .collect();

            let mut stdout = io::BufWriter::new(io::stdout());
            for line in output {
                writeln!(stdout, "{}", line)?;
            }
            stdout.flush()?;
        }
        _ => println!("未知模式"),
    }
    Ok(())
}