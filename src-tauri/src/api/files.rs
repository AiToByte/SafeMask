use crate::common::state::{AppState, MaskHistoryItem};
use crate::common::errors::{AppError, AppResult};
use crate::infra::fs::processor;
use tauri::{AppHandle, Emitter, State, Manager};
use serde::Serialize;
use log::info;
use uuid::Uuid;
use chrono::Local;
use std::path::{Path, PathBuf};

/// 用于文件记录的最大内容长度（字节，按 UTF-8 字符边界截断）
const MAX_RECORD_CONTENT_LEN: usize = 2000;

#[derive(Serialize)]
pub struct ProcessResponse {
    pub message: String,
    pub output_path: String,
    pub output_dir: String,
    pub duration: String,
    pub throughput: String,
}

#[tauri::command]
pub async fn process_file_gui(
    app: AppHandle,
    state: State<'_, AppState>,
    input_path: String,
) -> AppResult<ProcessResponse> {
    // 1. 校验并规范化输入路径（防任意路径写入 / 不存在 / 非文件）
    let input = validate_input_path(&input_path)?;

    // 2. 派生输出路径（stem 已由 file_name 拆分，天然不含路径分隔符）
    let stem = input
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("output");
    let ext = input
        .extension()
        .and_then(|s| s.to_str())
        .unwrap_or("log");
    let parent = input.parent().ok_or_else(|| {
        AppError::Config(format!("无法获取父目录: {}", input.display()))
    })?;

    // PDF 输出为纯文本
    let target_ext = if ext.eq_ignore_ascii_case("pdf") { "txt" } else { ext };
    let output_path = parent.join(format!("{}.masked.{}", stem, target_ext));
    let output_path_str = output_path.to_string_lossy().to_string();

    // 3. 获取引擎快照（读锁快进快出，避免跨 await 持锁）
    let engine_snapshot = {
        let guard = state.engine.read();
        guard.clone()
    };

    // 4. 后台执行文件处理
    let app_clone = app.clone();
    let input_for_thread = input.clone();
    let output_for_thread = output_path.clone();
    let stats = tauri::async_runtime::spawn_blocking(move || {
        processor::process_file(
            &input_for_thread,
            &output_for_thread,
            &engine_snapshot,
            move |progress| {
                let _ = app_clone.emit(
                    "file-progress",
                    serde_json::json!({ "percentage": progress * 100.0 }),
                );
            },
        )
    })
    .await
    .map_err(|e| AppError::Internal(format!("Runtime Error: {}", e)))?
    .map_err(|e| AppError::Internal(format!("Processing Error: {}", e)))?;

    // 5. 若启用记录写入器，异步写入文件处理记录
    {
        let app_state = app.state::<AppState>();
        let writer_opt = app_state.record_writer.read().clone();
        if let Some(writer) = writer_opt {
            let original_raw = tokio::fs::read_to_string(&input).await.unwrap_or_default();
            let masked_raw = tokio::fs::read_to_string(&output_path).await.unwrap_or_default();

            let file_item = MaskHistoryItem {
                id: Uuid::new_v4().to_string(),
                timestamp: Local::now().format("%H:%M:%S").to_string(),
                original: summarize_content(&original_raw),
                masked: summarize_content(&masked_raw),
                mode: "FILE".to_string(),
                entities: stats.entities.clone(),
            };
            writer.write(file_item).await;
            info!("[FileRecord] 文件处理记录已写入");
        }
    }

    Ok(ProcessResponse {
        message: "🛡️ 脱敏任务已成功完成".into(),
        output_path: output_path_str,
        output_dir: parent.to_string_lossy().to_string(),
        duration: format!("{:.2}s", stats.duration_secs),
        throughput: format!(
            "{:.2} MB/s",
            (stats.processed_bytes as f64 / 1024.0 / 1024.0) / stats.duration_secs.max(0.001)
        ),
    })
}

/// 校验输入路径并返回规范化的绝对路径
///
/// 拒绝：空路径、不存在、非常规文件。规范化会跟随 symlink 并解析 `.`/`..`，
/// 防止前端传入 `../etc/passwd` 一类的相对路径穿越。
fn validate_input_path(raw: &str) -> AppResult<PathBuf> {
    if raw.trim().is_empty() {
        return Err(AppError::Config("输入路径为空".into()));
    }
    let path = Path::new(raw);
    if !path.exists() {
        return Err(AppError::Config(format!("文件不存在: {}", raw)));
    }
    let canonical = path
        .canonicalize()
        .map_err(|e| AppError::Config(format!("无法解析路径 {}: {}", raw, e)))?;
    if !canonical.is_file() {
        return Err(AppError::Config(format!(
            "不是常规文件: {}",
            canonical.display()
        )));
    }
    Ok(canonical)
}

/// 按 UTF-8 字符边界截断字符串，避免落在多字节字符中间导致 panic
fn truncate_utf8_safe(s: &str, max_bytes: usize) -> &str {
    if s.len() <= max_bytes {
        return s;
    }
    let mut end = max_bytes;
    while end > 0 && !s.is_char_boundary(end) {
        end -= 1;
    }
    &s[..end]
}

/// 构造记录内容摘要：超长时按 UTF-8 边界截断并标注原始长度
fn summarize_content(raw: &str) -> String {
    if raw.len() <= MAX_RECORD_CONTENT_LEN {
        return raw.to_string();
    }
    let head = truncate_utf8_safe(raw, MAX_RECORD_CONTENT_LEN);
    format!("{}...\n[已截断，共 {} 字节]", head, raw.len())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn truncate_utf8_safe_ascii() {
        assert_eq!(truncate_utf8_safe("hello world", 5), "hello");
        assert_eq!(truncate_utf8_safe("hello", 100), "hello");
    }

    #[test]
    fn truncate_utf8_safe_multibyte_boundary() {
        // "中" = 3 bytes。max_bytes 落在字符中间时应回退到边界
        let s = "中文测试";
        // "中" 占 0..3, "文" 占 3..6
        assert_eq!(truncate_utf8_safe(s, 4), "中"); // 4 落在 "文" 中间 → 回退到 3
        assert_eq!(truncate_utf8_safe(s, 3), "中");
        assert_eq!(truncate_utf8_safe(s, 6), "中文");
        assert_eq!(truncate_utf8_safe(s, 0), "");
    }

    #[test]
    fn truncate_utf8_safe_mixed() {
        let s = "ab中cd";
        assert_eq!(truncate_utf8_safe(s, 3), "ab"); // 3 落在 "中" 中间 → 回退到 2
        assert_eq!(truncate_utf8_safe(s, 5), "ab中");
    }

    #[test]
    fn summarize_content_short() {
        assert_eq!(summarize_content("hello"), "hello");
    }

    #[test]
    fn summarize_content_truncates_at_boundary() {
        // 构造超过 MAX_RECORD_CONTENT_LEN 字节的中文字符串
        let big: String = "中".repeat(1000); // 3000 bytes
        let out = summarize_content(&big);
        assert!(out.starts_with("中"));
        assert!(out.contains("[已截断"));
        // 验证截断点在字符边界（没有替换字符）
        assert!(!out.contains('\u{FFFD}'));
    }

    #[test]
    fn validate_rejects_empty() {
        let err = validate_input_path("").unwrap_err();
        assert!(matches!(err, AppError::Config(_)));
    }

    #[test]
    fn validate_rejects_missing() {
        let err = validate_input_path("Z:/definitely/not/a/real/file/xyz_9f8e7d.txt").unwrap_err();
        assert!(matches!(err, AppError::Config(_)));
    }

    #[test]
    fn validate_rejects_directory() {
        // 用当前 crate 目录作为已知存在的目录
        let dir = std::env::current_dir().expect("cwd");
        let err = validate_input_path(dir.to_str().unwrap()).unwrap_err();
        assert!(matches!(err, AppError::Config(_)));
    }
}
