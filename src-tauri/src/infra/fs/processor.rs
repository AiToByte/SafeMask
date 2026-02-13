use crate::core::engine::MaskEngine;
use anyhow::{Context, Result};
use crossbeam_channel::{bounded};
use memmap2::MmapOptions;
use rayon::prelude::*;
use std::collections::BTreeMap;
use std::fs::File;
use std::io::{BufWriter, Read, Write};
use std::path::Path;
use std::sync::atomic::{AtomicU64, AtomicUsize, Ordering};
use std::sync::Arc;
use std::time::Instant;
use calamine::{Reader, Xlsx, open_workbook, Data}; // 🚀 修正：calamine 使用 Data 而非 DataType
use rust_xlsxwriter::{Workbook};
use zip::write::SimpleFileOptions; // 🚀 修正：zip v2.x 推荐使用 SimpleFileOptions
use quick_xml::events::{Event, BytesText};
use quick_xml::reader::Reader as XmlReader;
use quick_xml::writer::Writer as XmlWriter;
use std::io::Cursor;

const CHUNK_SIZE: usize = 8 * 1024 * 1024; // 增加到 8MB 提升吞吐
const MAX_IN_FLIGHT: usize = 32;           // 限制内存中积压的块数 (约 256MB)

#[derive(Debug, Clone)]
pub struct ProcessStats {
    #[allow(dead_code)]
    pub total_lines: u64,
    pub processed_bytes: u64,
    pub duration_secs: f64,
}

/// 统筹分发函数：根据后缀名决定处理策略
pub fn process_file<P: AsRef<Path>>(
    input_path: P,
    output_path: P,
    engine: &Arc<MaskEngine>,
    progress_callback: impl Fn(f64) + Sync + Send + 'static,
) -> Result<ProcessStats> {
    let input = input_path.as_ref();
    let ext = input.extension().and_then(|s| s.to_str()).unwrap_or("").to_lowercase();

    match ext.as_str() {
        // 1. Office Word 文档
        "docx" => process_docx(input, output_path.as_ref(), engine, progress_callback),
        
        // 2. Office Excel 表格
         "xlsx" | "xls" | "xlsm" | "xlsb" => process_xlsx(input, output_path.as_ref(), engine, progress_callback),

        // 3. PDF 文档 (通常输出为脱敏后的文本，因为 PDF 逆向修改容易乱码)
        "pdf" | "doc" => process_pdf(input, output_path.as_ref(), engine, progress_callback),

        // 4. 默认：高性能纯文本流水线 (Log, Txt, Csv, Json, etc.)
        _ => process_text_file_mmap(input, output_path.as_ref(), engine, progress_callback),
    }
}

/// Word 脱敏：基于 ZIP 结构直接替换 XML 中的文本节点
fn process_docx(input: &Path, output: &Path, engine: &Arc<MaskEngine>, cb: impl Fn(f64)) -> Result<ProcessStats> {
    let start = Instant::now();
    
    // 1. 以只读模式打开输入文件
    let reader = File::open(input).context("无法读取原文件")?;
    let mut archive = zip::ZipArchive::new(reader)?;
    
    // 2. 创建完全独立的新输出文件
    let writer_file = File::create(output).context("无法创建脱敏结果文件")?;
    let mut zip_writer = zip::ZipWriter::new(writer_file);

    let total = archive.len();

    for i in 0..total {
        let mut entry = archive.by_index(i)?;
        let name = entry.name().to_string();
        
        // 复制原文件的压缩选项
        let options = SimpleFileOptions::default()
            .compression_method(entry.compression())
            .unix_permissions(entry.unix_mode().unwrap_or(0o755));

        // 核心：只处理包含文本的 XML
        if name.ends_with("document.xml") || name.ends_with("header.xml") || name.ends_with("footer.xml") {
            let mut buffer = Vec::new();
            entry.read_to_end(&mut buffer)?;
            
            // 🚀 核心改进：XML 标签感知脱敏
            // 我们不能直接 mask 整个 XML 字节流，否则会破坏标签（如 <w:t> 变成 <MASK>）
            let processed_xml = mask_xml_content(&buffer, engine)?;
            
            zip_writer.start_file(name, options)?;
            zip_writer.write_all(&processed_xml)?;
        } else {
            // 图片、样式等非文本内容直接拷贝
            zip_writer.raw_copy_file(entry)?;
        }
        cb((i as f64 / total as f64) * 0.9);
    }

    // 🚀 确保写入完成并刷新到磁盘
    zip_writer.finish()?;
    cb(1.0);

    Ok(ProcessStats {
        total_lines: 0, 
        processed_bytes: std::fs::metadata(input)?.len(),
        duration_secs: start.elapsed().as_secs_f64()
    })
}

fn process_xlsx(input: &Path, output: &Path, engine: &Arc<MaskEngine>, cb: impl Fn(f64)) -> Result<ProcessStats> {
    let start = std::time::Instant::now();
    let mut workflow: Xlsx<_> = open_workbook(input)?;
    let mut new_workbook = Workbook::new();

    let sheet_names = workflow.sheet_names().to_vec();
    for (idx, name) in sheet_names.iter().enumerate() {
        if let Ok(range) = workflow.worksheet_range(name) {
            let sheet = new_workbook.add_worksheet();
            sheet.set_name(name)?;
            
            for (r, row) in range.rows().enumerate() {
                for (c, cell) in row.iter().enumerate() {
                    match cell {
                        // 🚀 修正：使用 Data::String 而非 DataType::String
                        Data::String(s) => {
                            let masked = engine.mask_line(s.as_bytes());
                            // 🚀 修正：into_owned() 将 Cow<str> 转为 String，满足 write_string 要求
                            sheet.write_string(r as u32, c as u16, String::from_utf8_lossy(&masked).into_owned())?;
                        },
                        Data::Float(f) => { sheet.write_number(r as u32, c as u16, *f)?; },
                        Data::Int(i) => { sheet.write_number(r as u32, c as u16, *i as f64)?; },
                        Data::Bool(b) => { sheet.write_boolean(r as u32, c as u16, *b)?; },
                        _ => {}
                    }
                }
            }
        }
        cb((idx as f64 / sheet_names.len() as f64) * 0.9);
    }

    new_workbook.save(output.to_string_lossy().as_ref())?;
    cb(1.0);
    Ok(ProcessStats { 
        total_lines: 0, 
        processed_bytes: std::fs::metadata(input)?.len(), 
        duration_secs: start.elapsed().as_secs_f64() 
    })
}

/// PDF 脱敏实现
fn process_pdf(input: &Path, output: &Path, engine: &Arc<MaskEngine>, cb: impl Fn(f64)) -> Result<ProcessStats> {
    let start = std::time::Instant::now();
    // 使用 pdf_extract 或类似工具提取文本
    // 注意：对于 .doc，pdf_extract 可能不支持，这里主要针对 PDF
    let content = if input.extension().map_or(false, |e| e == "pdf") {
        pdf_extract::extract_text(input).unwrap_or_default()
    } else {
        // 如果是 .doc，目前做简单读取或提示
        "目前 .doc 仅支持另存为 .docx 后进行格式保留脱敏".to_string()
    };
    cb(0.5);
    
    let masked = engine.mask_line(content.as_bytes());
    std::fs::write(output, &masked)?;
    
    cb(1.0);
    Ok(ProcessStats {
        total_lines: 0,
        processed_bytes: content.len() as u64,
        duration_secs: start.elapsed().as_secs_f64()
    })
}

pub fn process_text_file_mmap<P: AsRef<Path>>(
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
        let mut writer = BufWriter::with_capacity(4 * 1024 * 1024, output_file);
        
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

/// 🚀 XML 深度脱敏：只针对文本节点进行脱敏，保护 XML 标签
fn mask_xml_content(xml_data: &[u8], engine: &Arc<MaskEngine>) -> Result<Vec<u8>> {
    let mut reader = XmlReader::from_reader(xml_data);
    let mut writer = XmlWriter::new(Cursor::new(Vec::new()));
    let mut buf = Vec::new();

    loop {
        match reader.read_event_into(&mut buf) {
            Ok(Event::Text(e)) => {
                // 仅对文本内容执行脱敏引擎
                let raw_text = e.unescape()?;
                let masked_bytes = engine.mask_line(raw_text.as_bytes());
                
                // 将脱敏后的文本写回
                let masked_text = String::from_utf8_lossy(&masked_bytes);
                writer.write_event(Event::Text(BytesText::new(&masked_text)))?;
            },
            Ok(Event::Eof) => break,
            Ok(e) => {
                // 标签本身（Start, End, Empty）原样写回，不进行脱敏
                writer.write_event(e)?;
            },
            Err(e) => return Err(anyhow::anyhow!("XML 解析错误: {}", e)),
        }
        buf.clear();
    }

    Ok(writer.into_inner().into_inner())
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