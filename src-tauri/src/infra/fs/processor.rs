use crate::core::engine::MaskEngine;
use anyhow::{Context, Result};
use crossbeam_channel::{bounded};
use memmap2::MmapOptions;
use rayon::prelude::*;
use std::collections::BTreeMap;
use std::fs::File;
use std::io::{BufWriter, Write};
use std::path::Path;
use std::sync::atomic::{AtomicU64, AtomicUsize, Ordering};
use std::sync::Arc;
use std::time::Instant;

const CHUNK_SIZE: usize = 8 * 1024 * 1024; // 增加到 8MB 提升吞吐
const MAX_IN_FLIGHT: usize = 32;           // 限制内存中积压的块数 (约 256MB)

#[derive(Debug, Clone)]
pub struct ProcessStats {
    #[allow(dead_code)]
    pub total_lines: u64,
    pub processed_bytes: u64,
    pub duration_secs: f64,
}

pub fn process_file<P: AsRef<Path>>(
    input_path: P,
    output_path: P,
    engine: &Arc<MaskEngine>,
    progress_callback: impl Fn(f64) + Sync + Send + 'static,
) -> Result<ProcessStats> {
    let start_time = Instant::now();

    let file = File::open(&input_path).context("无法打开输入文件")?;
    let file_len = file.metadata()?.len();

    if file_len == 0 {
        File::create(&output_path)?;
        progress_callback(1.0);
        return Ok(ProcessStats { total_lines: 0, processed_bytes: 0, duration_secs: 0.0 });
    }

    let mmap = unsafe { MmapOptions::new().map(&file)? };
    let output_path_buf = output_path.as_ref().to_path_buf();

    // 管道定义：(块索引, 数据内容, 行数)
    let (result_tx, result_rx) = bounded::<(usize, Vec<u8>, u64)>(MAX_IN_FLIGHT);
    // 背压控制：控制读取速度
    let (backpressure_tx, backpressure_rx) = bounded::<()>(MAX_IN_FLIGHT);

    let processed_bytes = Arc::new(AtomicUsize::new(0));
    let total_lines = Arc::new(AtomicU64::new(0));
    let p_bytes_clone = processed_bytes.clone();
    let p_total_lines = total_lines.clone();
    
    let progress_arc = Arc::new(progress_callback);
    let progress_for_writer = progress_arc.clone();

    // 消费者线程：保序写入磁盘
    let writer_handle = std::thread::spawn(move || -> Result<()> {
        let output_file = File::create(output_path_buf).context("无法创建输出文件")?;
        let mut writer = BufWriter::with_capacity(2 * 1024 * 1024, output_file);
        
        let mut pending_chunks = BTreeMap::new();
        let mut next_idx = 0;

        for (idx, data, lines) in result_rx {
            pending_chunks.insert(idx, data);
            p_total_lines.fetch_add(lines, Ordering::Relaxed);

            while let Some(chunk_data) = pending_chunks.remove(&next_idx) {
                writer.write_all(&chunk_data)?;
                let current_bytes = p_bytes_clone.fetch_add(chunk_data.len(), Ordering::Relaxed);
                
                // 消耗背压信号，允许生产者继续
                let _ = backpressure_rx.recv(); 

                // 进度回调节流
                if next_idx % 4 == 0 {
                    progress_for_writer(current_bytes as f64 / file_len as f64);
                }
                next_idx += 1;
            }
        }
        writer.flush()?;
        Ok(())
    });

    // 生产者逻辑：并行脱敏
    let chunk_iter = SplitLinesIterator::new(&mmap, CHUNK_SIZE);
    chunk_iter.par_bridge().for_each(|(idx, chunk)| {
        // 等待背压许可
        if backpressure_tx.send(()).is_err() { return; }

        let result = engine.mask_line(chunk);
        let lines = bytecount::count(result.as_ref(), b'\n') as u64;

        if result_tx.send((idx, result.into_owned(), lines)).is_err() { return; }
    });

    drop(result_tx);
    drop(backpressure_tx);

    writer_handle.join().map_err(|_| anyhow::anyhow!("写入线程崩溃"))??;
    progress_arc(1.0);

    Ok(ProcessStats {
        total_lines: total_lines.load(Ordering::SeqCst),
        processed_bytes: processed_bytes.load(Ordering::SeqCst) as u64,
        duration_secs: start_time.elapsed().as_secs_f64(),
    })
}

struct SplitLinesIterator<'a> {
    data: &'a [u8],
    pos: usize,
    chunk_size: usize,
    idx: usize,
}

impl<'a> SplitLinesIterator<'a> {
    fn new(data: &'a [u8], chunk_size: usize) -> Self {
        Self { data, pos: 0, chunk_size, idx: 0 }
    }
}

impl<'a> Iterator for SplitLinesIterator<'a> {
    type Item = (usize, &'a [u8]);
    fn next(&mut self) -> Option<Self::Item> {
        if self.pos >= self.data.len() { return None; }
        let start = self.pos;
        let mut end = (start + self.chunk_size).min(self.data.len());
        if end < self.data.len() {
            if let Some(nl) = memchr::memchr(b'\n', &self.data[end..]) {
                end += nl + 1;
            } else {
                end = self.data.len();
            }
        }
        self.pos = end;
        let res = (self.idx, &self.data[start..end]);
        self.idx += 1;
        Some(res)
    }
}