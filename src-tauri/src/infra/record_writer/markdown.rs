use std::path::PathBuf;
use std::fs;
use std::io::Write;
use async_trait::async_trait;
use chrono::Local;
use tokio::sync::mpsc;
use tokio::time::{interval, Duration};
use log::{info, error};
use crate::common::state::MaskHistoryItem;
use super::RecordWriter;

/// 单文件最大记录数
const MAX_RECORDS_PER_FILE: u32 = 150;
/// 后台 flush 间隔（秒）
const FLUSH_INTERVAL_SECS: u64 = 5;
/// 内存缓冲区触发 flush 的条数
const BATCH_FLUSH_THRESHOLD: usize = 10;

// ── 后台写入任务的内部状态 ──

struct WriterState {
    output_dir: PathBuf,
    current_date: String,
    current_seq: u32,
    current_count: u32,
}

impl WriterState {
    fn new(output_dir: PathBuf) -> Self {
        let today = today_str();
        let year = Local::now().format("%Y").to_string();
        let dir = output_dir.join(&year);
        let mut max_seq = 0u32;
        let mut count = 0u32;

        if dir.exists() && let Ok(entries) = fs::read_dir(&dir) {
            for entry in entries.flatten() {
                let name = entry.file_name().to_string_lossy().to_string();
                if name.starts_with(&today) && name.ends_with(".md")
                    && let Some(seq_str) = name
                        .strip_suffix(".md")
                        .and_then(|s| s.rsplit('-').next())
                        && let Ok(seq) = seq_str.parse::<u32>()
                        && seq > max_seq
                {
                    max_seq = seq;
                    if let Ok(content) = fs::read_to_string(entry.path()) {
                        let sep_count = content.matches("\n---\n").count() as u32;
                        count = sep_count;
                    }
                }
            }
        }

        if max_seq == 0 { max_seq = 1; }

        Self { output_dir, current_date: today, current_seq: max_seq, current_count: count }
    }
}

fn today_str() -> String {
    Local::now().format("%Y-%m-%d").to_string()
}

// ── MarkdownRecordWriter ──

pub struct MarkdownRecordWriter {
    sender: mpsc::UnboundedSender<MaskHistoryItem>,
}

impl MarkdownRecordWriter {
    /// 创建一个新写入器，返回 (writer, background_task)。
    /// 调用者必须在 Tokio 运行时环境中 spawn background_task。
    pub fn new(output_dir: PathBuf) -> (Self, impl Future<Output = ()>) {
        let (tx, rx) = mpsc::unbounded_channel();
        let future = writer_task(output_dir, rx);
        (Self { sender: tx }, future)
    }
}

#[async_trait]
impl RecordWriter for MarkdownRecordWriter {
    async fn write(&self, item: MaskHistoryItem) {
        let _ = self.sender.send(item);
    }

    async fn flush(&self) {
        // background task handles it
    }
}

// ── 后台 writer 任务 ──

async fn writer_task(output_dir: PathBuf, mut rx: mpsc::UnboundedReceiver<MaskHistoryItem>) {
    let mut state = WriterState::new(output_dir);
    let mut buffer: Vec<MaskHistoryItem> = Vec::with_capacity(BATCH_FLUSH_THRESHOLD);
    let mut timer = interval(Duration::from_secs(FLUSH_INTERVAL_SECS));
    // 跳过第一次立即 tick
    timer.tick().await;

    loop {
        tokio::select! {
            Some(item) = rx.recv() => {
                buffer.push(item);
                // 尽可能多地收取已积压的条目
                while let Ok(item) = rx.try_recv() {
                    buffer.push(item);
                    if buffer.len() >= BATCH_FLUSH_THRESHOLD {
                        do_flush(&mut state, &mut buffer);
                    }
                }
                if buffer.len() >= BATCH_FLUSH_THRESHOLD {
                    do_flush(&mut state, &mut buffer);
                }
            }
            _ = timer.tick() => {
                if !buffer.is_empty() {
                    do_flush(&mut state, &mut buffer);
                }
            }
        }
    }
}

/// 批量写入缓冲中的所有记录
fn do_flush(state: &mut WriterState, buffer: &mut Vec<MaskHistoryItem>) {
    if buffer.is_empty() { return; }

    let today = today_str();
    // 日期变更 → 重置 seq
    if today != state.current_date {
        state.current_date = today;
        state.current_seq = 1;
        state.current_count = 0;
    }

    for item in buffer.drain(..) {
        // 当前文件已满 → 切新文件
        if state.current_count >= MAX_RECORDS_PER_FILE {
            state.current_seq += 1;
            state.current_count = 0;
        }

        let path = build_path(state);
        let record = format_record(&item, state.current_count + 1);
        let is_new = !path.exists() || path.metadata().map(|m| m.len() == 0).unwrap_or(false);

        if let Some(parent) = path.parent() && let Err(e) = fs::create_dir_all(parent) {
            error!("[RecordWriter] 无法创建记录目录 '{}': {} (路径: {}). 请确保应用有写入权限",
                parent.display(), e, path.display());
            continue;
        }

        match fs::OpenOptions::new().create(true).append(true).open(&path) {
            Ok(mut file) => {
                if is_new {
                    let header = format!("---\ndate: {}\n---\n\n", state.current_date);
                    if let Err(e) = file.write_all(header.as_bytes()) {
                        error!("[RecordWriter] 写入文件头失败 {}: {}", path.display(), e);
                        continue;
                    }
                }
                if let Err(e) = file.write_all(record.as_bytes()) {
                    error!("[RecordWriter] 写入记录失败 {}: {}", path.display(), e);
                    continue;
                }
                state.current_count += 1;
            }
            Err(e) => {
                error!("[RecordWriter] 无法打开文件 {}: {} (权限或路径错误)", path.display(), e);
            }
        }
    }

    info!("[RecordWriter] 已写入记录到 {}-{:03}.md",
        state.current_date, state.current_seq);
}

fn build_path(state: &WriterState) -> PathBuf {
    let year = state.current_date[..4].to_string();
    state.output_dir
        .join(&year)
        .join(format!("{}-{:03}.md", state.current_date, state.current_seq))
}

fn format_record(item: &MaskHistoryItem, record_num: u32) -> String {
    let mut s = String::new();
    s.push_str(&format!("## 记录 {}\n\n", record_num));
    s.push_str("### 原始内容\n```\n");
    s.push_str(&item.original);
    s.push_str("\n```\n\n");
    s.push_str("### 脱敏后内容\n```\n");
    s.push_str(&item.masked);
    s.push_str("\n```\n\n");

    if !item.entities.is_empty() {
        s.push_str("### 识别实体\n");
        s.push_str("| 类型 | 起始 | 结束 | 脱敏值 |\n");
        s.push_str("|------|------|------|--------|\n");
        for e in &item.entities {
            s.push_str(&format!("| {} | {} | {} | {} |\n",
                e.entity_type, e.start, e.end, e.mask_label));
        }
        s.push('\n');
    }

    s.push_str("### 统计\n");
    s.push_str(&format!("- 模式: {}\n", item.mode));
    s.push_str(&format!("- 实体数: {}\n", item.entities.len()));
    s.push_str(&format!("- 时间: {}\n", item.timestamp));
    s.push_str("\n---\n\n");
    s
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::common::state::EntitySpanBrief;

    fn make_item(original: &str, masked: &str, mode: &str, entities: Vec<EntitySpanBrief>) -> MaskHistoryItem {
        MaskHistoryItem {
            id: "test-id".into(),
            timestamp: "14:30:00".into(),
            original: original.into(),
            masked: masked.into(),
            mode: mode.into(),
            entities,
        }
    }

    #[test]
    fn test_format_record() {
        let item = make_item(
            "我的手机号是13800138000",
            "我的手机号是138****8000",
            "SHADOW",
            vec![EntitySpanBrief {
                start: 6,
                end: 17,
                entity_type: "PHONE".into(),
                mask_label: "138****8000".into(),
            }],
        );
        let result = format_record(&item, 1);
        assert!(result.contains("## 记录 1"));
        assert!(result.contains("原始内容"));
        assert!(result.contains("脱敏后内容"));
        assert!(result.contains("识别实体"));
        assert!(result.contains("PHONE"));
        assert!(result.contains("SHADOW"));
        assert!(result.contains("---"));
    }

    #[test]
    fn test_build_path() {
        let state = WriterState {
            output_dir: PathBuf::from("/tmp/records"),
            current_date: "2026-07-15".into(),
            current_seq: 1,
            current_count: 0,
        };
        let path = build_path(&state);
        let s = path.to_string_lossy();
        assert!(s.contains("2026-07-15-001.md"), "path should contain date-seq: {}", s);
    }

    #[test]
    fn test_today_str() {
        let s = today_str();
        assert_eq!(s.len(), 10);
        assert_eq!(s.chars().filter(|&c| c == '-').count(), 2);
    }
}
