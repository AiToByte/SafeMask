use crate::common::state::AppState;
use crate::common::events::AppEvents;
use crate::common::errors::AppResult;
use crate::infra::fs::processor;
use tauri::{AppHandle, Emitter, State};

#[tauri::command]
pub async fn process_file_gui(
    app: AppHandle,
    state: State<'_, AppState>,
    input_path: String,
    output_path: String,
) -> AppResult<String> {
    use crate::common::errors::AppError; // 确保引入了 AppError

    // 1. 获取引擎快照
    let engine_snapshot = {
        let guard = state.engine.read(); // 去掉 .expect()
        guard.clone()
    };

    let app_clone = app.clone();
    
    // 2. 运行 IO 任务
    let stats = tauri::async_runtime::spawn_blocking(move || {
        processor::process_file(
            input_path, 
            output_path, 
            &engine_snapshot, 
            move |progress| {
                let _ = app_clone.emit(AppEvents::FILE_PROGRESS, serde_json::json!({ "percentage": progress * 100.0 }));
            }
        )
    })
    .await
    // 处理第一层错误：JoinError (线程池或运行时错误)
    .map_err(|e| AppError::Internal(format!("Runtime Error: {}", e)))?
    // 处理第二层错误：anyhow::Error (文件处理业务错误)
    .map_err(|e| AppError::Internal(format!("Processing Error: {}", e)))?; 

    Ok(format!(
        "处理完成! 耗时: {:.2}s, 吞吐量: {:.2} MB/s",
        stats.duration_secs,
        (stats.processed_bytes as f64 / 1024.0 / 1024.0) / stats.duration_secs
    ))
}