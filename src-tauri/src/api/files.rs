use crate::common::state::AppState;
use crate::common::errors::AppResult;
use crate::infra::fs::processor;
use tauri::{AppHandle, Emitter, State};
use serde::Serialize;
use serde_json::json; // 🚀 显式导入 json 宏

#[derive(Serialize)]
pub struct ProcessResponse {
    pub message: String,
    pub output_path: String,
    pub output_dir: String, // 🚀 新增：方便前端打开目录
    pub duration: String,
    pub throughput: String,
}

#[tauri::command]
pub async fn process_file_gui(
    app: AppHandle,
    state: State<'_, AppState>,
    input_path: String
) -> AppResult<ProcessResponse> {
    use crate::common::errors::AppError; // 确保引入了 AppError
    use std::path::Path;
    
    let input = Path::new(&input_path);
     // 🚀 核心逻辑：智能后缀名处理
    let ext = input.extension().and_then(|s| s.to_str()).unwrap_or("log");
    let stem = input.file_stem().and_then(|s| s.to_str()).unwrap_or("output");
    let parent = input.parent().unwrap_or_else(|| Path::new(""));

    // 如果是 PDF，后缀改为 txt（对应第二步的逻辑）
    let target_ext = if ext.to_lowercase() == "pdf" { "txt" } else { ext };
    let output_file_name = format!("{}.masked.{}", stem, target_ext);
    let output_path = parent.join(output_file_name);
    let output_path_str = output_path.to_string_lossy().to_string();
    // 1. 获取引擎快照
    let engine_snapshot = {
        let guard = state.engine.read(); // 去掉 .expect()
        guard.clone()
    };

    let app_clone = app.clone();
    let input_path_for_thread = input_path.clone();
    let output_path_for_thread = output_path_str.clone();
    // 2. 运行 IO 任务
    let stats = tauri::async_runtime::spawn_blocking(move || {
        processor::process_file(
            &input_path_for_thread, 
            &output_path_for_thread, 
            &engine_snapshot, 
            move |progress| {
                let _ = app_clone.emit("file-progress", serde_json::json!({ "percentage": progress * 100.0 }));
            }
        )
    })
    .await
    // 处理第一层错误：JoinError (线程池或运行时错误)
    .map_err(|e| AppError::Internal(format!("Runtime Error: {}", e)))?
    // 处理第二层错误：anyhow::Error (文件处理业务错误)
    .map_err(|e| AppError::Internal(format!("Processing Error: {}", e)))?; 

     // 🚀 返回结构化数据
     Ok(ProcessResponse {
        message: "🛡️ 脱敏任务已成功完成".into(),
        output_path: output_path_str,
        output_dir: parent.to_string_lossy().to_string(),
        duration: format!("{:.2}s", stats.duration_secs),
        throughput: format!("{:.2} MB/s", (stats.processed_bytes as f64 / 1024.0 / 1024.0) / stats.duration_secs),
    })
}