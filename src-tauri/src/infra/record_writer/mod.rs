//! Pluggable record writer interface for persisting original→masked mappings
use std::path::PathBuf;
use crate::common::state::MaskHistoryItem;

#[async_trait::async_trait]
pub trait RecordWriter: Send + Sync {
    /// 写入一条记录（非阻塞——内部 buffer，异步 flush）
    async fn write(&self, item: MaskHistoryItem);
    /// 强制刷入所有缓冲记录
    async fn flush(&self);
}

/// 返回 exe 所在目录下的 records 文件夹路径
pub fn default_records_dir() -> PathBuf {
    std::env::current_exe()
        .ok()
        .and_then(|p| p.parent().map(|d| d.to_path_buf()))
        .unwrap_or_else(|| PathBuf::from("."))
        .join("records")
}

pub mod markdown;
pub use markdown::MarkdownRecordWriter;
