use tauri::{State, AppHandle};
use crate::state::{AppState, MaskHistoryItem};
use crate::processor::FileProcessor;
use crate::config::{Rule, RuleManager};
use arboard::Clipboard;
#[tauri::command]
pub async fn manual_mask_cmd(state: State<'_, AppState>) -> Result<String, String> {
    let mut ctx = arboard::Clipboard::new().map_err(|e| e.to_string())?;
    let text = ctx.get_text().map_err(|e| e.to_string())?;

    // 🚀 从 State 中获取引擎
    let engine = state.engine.read().unwrap();
    let masked = engine.mask_line(text.as_bytes());

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
    state: State<'_, AppState>, // 🚀 获取 State
) -> Result<String, String> {
     // 🚀 将 engine 引用传给处理器
    FileProcessor::run_ordered_pipeline(input_path, output_path, app_handle, state.engine.clone())
}

#[tauri::command]
pub fn get_rules_stats(app_handle: AppHandle) -> serde_json::Value {
    // 🚀 传入 app_handle 解决路径问题
    let rules = RuleManager::load_all_rules(&app_handle);
    serde_json::json!({ "rule_count": rules.len() })
}

#[tauri::command]
pub async fn get_mask_history(state: State<'_, AppState>) -> Result<Vec<MaskHistoryItem>, String> {
    let history = state.history.lock().unwrap();
    Ok(history.clone())
}


#[tauri::command]
pub async fn save_rule_api(rule: Rule) -> Result<String, String> {
    RuleManager::save_custom_rule(rule).map_err(|e| e.to_string())?;
    // 💡 保存后，建议通过某种方式通知引擎重新加载，这里我们先返回成功
    Ok("规则已保存至 custom 目录".into())
}

#[tauri::command]
pub async fn copy_original_cmd(text: String, state: tauri::State<'_, AppState>) -> Result<(), String> {
    let mut ctx = Clipboard::new().map_err(|e| e.to_string())?;
    
    // 🚀 核心优化：在写入剪贴板前，先把内容注入到 last_content 缓存中
    // 这样后台监听线程检测到内容变化时，会发现 current == last，从而直接跳过脱敏
    {
        let mut last = state.last_content.lock().unwrap();
        *last = text.clone();
    }

    // 执行真实的剪贴板写入
    ctx.set_text(text).map_err(|e| e.to_string())?;
    
    Ok(())
}

#[tauri::command]
pub fn get_all_detailed_rules(app_handle: AppHandle) -> Vec<Rule> {
    RuleManager::load_all_rules(&app_handle)
}

#[tauri::command]
pub async fn delete_rule_api(name: String) -> Result<String, String> {
    RuleManager::delete_custom_rule(name).map_err(|e| e.to_string())?;
    Ok("规则已删除".into())
}