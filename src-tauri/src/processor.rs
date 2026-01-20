use std::collections::BTreeMap;
use std::io::{BufWriter, Write};
use std::fs::File;
use tauri::{AppHandle, Emitter};
use crate::state::{ProgressPayload, ENGINE, MACRO_CHUNK_SIZE, BUFFER_SIZE};
use rayon::prelude::*;

pub struct FileProcessor;

impl FileProcessor {
    /// 执行保序脱敏流水线
    pub fn run_ordered_pipeline(
        input_path: String,
        output_path: String,
        app_handle: AppHandle,
    ) -> Result<String, String> {
        let file = File::open(&input_path).map_err(|e| e.to_string())?;
        let mmap = unsafe { memmap2::Mmap::map(&file).map_err(|e| e.to_string())? };
        let file_size = mmap.len();
        let total_chunks = (file_size as f32 / MACRO_CHUNK_SIZE as f32).ceil() as usize;

        let (tx, rx) = crossbeam_channel::bounded::<(usize, Vec<u8>)>(rayon::current_num_threads() * 2);

        // 顺序写入线程
        let writer_handle = std::thread::spawn(move || -> Result<(), String> {
            let file_out = File::create(&output_path).map_err(|e| e.to_string())?;
            let mut writer = BufWriter::with_capacity(BUFFER_SIZE, file_out);
            let mut next_idx = 0;
            let mut pending_map = BTreeMap::new();
            let mut processed_count = 0;

            while let Ok((idx, data)) = rx.recv() {
                pending_map.insert(idx, data);
                while let Some(data) = pending_map.remove(&next_idx) {
                    writer.write_all(&data).map_err(|e| e.to_string())?;
                    next_idx += 1;
                    processed_count += 1;
                    
                    let _ = app_handle.emit("file-progress", ProgressPayload {
                        percentage: (processed_count as f32 / total_chunks as f32) * 100.0,
                        processed_mb: (processed_count * MACRO_CHUNK_SIZE) as f64 / 1024.0 / 1024.0,
                    });
                }
            }
            writer.flush().map_err(|e| e.to_string())?;
            Ok(())
        });

        // 并行计算
        mmap.par_chunks(MACRO_CHUNK_SIZE)
            .enumerate()
            .for_each(|(idx, chunk)| {
                let mut out = Vec::with_capacity(chunk.len() + 2048);
                for line in chunk.split(|&b| b == b'\n') {
                    if !line.is_empty() {
                        out.extend_from_slice(&ENGINE.mask_line(line));
                    }
                    out.push(b'\n');
                }
                let _ = tx.send((idx, out));
            });

        drop(tx);
        writer_handle.join().map_err(|_| "Writer thread panicked")??;
        Ok("文件处理成功".into())
    }
}