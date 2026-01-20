use tauri::{State, AppHandle};
use crate::state::{AppState, ENGINE};
use crate::processor::FileProcessor;
use crate::config::RuleManager;

#[tauri::command]
pub async fn manual_mask_cmd() -> Result<String, String> {
    let mut ctx = arboard::Clipboard::new().map_err(|e| e.to_string())?;
    let text = ctx.get_text().map_err(|e| e.to_string())?;
    let masked = ENGINE.mask_line(text.as_bytes());
    let masked_text = String::from_utf8_lossy(&masked).into_owned();
    ctx.set_text(masked_text).map_err(|e| e.to_string())?;
    Ok("脱敏已成功".into())
}

#[tauri::command]
pub async fn toggle_monitor(enabled: bool, state: State<'_, AppState>) -> Result<(), String> {
    let mut monitor = state.is_monitor_on.lock().unwrap();
    *monitor = enabled;
    Ok(())
}

#[tauri::command]
pub async fn process_file_gui(
    input_path: String,
    output_path: String,
    app_handle: AppHandle,
) -> Result<String, String> {
    FileProcessor::run_ordered_pipeline(input_path, output_path, app_handle)
}

#[tauri::command]
pub fn get_rules_stats() -> serde_json::Value {
    let rules = RuleManager::load_all_rules();
    serde_json::json!({
        "rule_count": rules.len(),
    })
}