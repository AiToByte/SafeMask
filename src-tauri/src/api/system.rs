use crate::common::state::{AppState, MaskHistoryItem};
use crate::common::errors::AppResult;
use crate::core::rules::Rule;
use crate::core::engine::MaskEngine;
use crate::infra::config::loader::ConfigLoader;
use tauri::{AppHandle, State}; // 🚀 确保引入 Manager 以便使用 .state()
use std::sync::Arc;

/// 获取规则统计信息 (仪表盘使用)
#[tauri::command]
pub async fn get_rules_stats(app: AppHandle) -> AppResult<serde_json::Value> {
    // 🚀 修复：load_all_rules 返回的是 Vec<Rule>，不需要 '?'
    let rules = ConfigLoader::load_all_rules(&app);
    Ok(serde_json::json!({
        "rule_count": rules.len(),
    }))
}

/// 获取所有详细规则列表 (规则管理页面使用)
#[tauri::command]
pub async fn get_all_detailed_rules(app: AppHandle) -> AppResult<Vec<Rule>> {
    // 🚀 修复：包装在 Ok() 中返回
    Ok(ConfigLoader::load_all_rules(&app))
}


/// 保存或更新规则
#[tauri::command]
pub async fn save_rule_api(app: AppHandle, state: State<'_, AppState>, rule: Rule) -> AppResult<String> {
    // 1. 持久化到 YAML
    ConfigLoader::save_custom_rule(rule)?;
    
    // 2. 触发引擎热重载，使规则立即生效
    reload_engine_internal(app, state).await?;
    
    Ok("规则已保存并应用".into())
}

/// 删除规则
#[tauri::command]
pub async fn delete_rule_api(app: AppHandle, state: State<'_, AppState>, name: String) -> AppResult<String> {
    ConfigLoader::delete_custom_rule(&name)?;
    reload_engine_internal(app, state).await?;
    Ok("规则已删除".into())
}

/// 内部函数：重新加载规则并替换引擎
async fn reload_engine_internal(app: AppHandle, state: State<'_, AppState>) -> AppResult<()> {
    // 🚀 修复：load_all_rules 返回的是 Vec<Rule>，不需要 '?'
    let rules = ConfigLoader::load_all_rules(&app);
    let new_engine = Arc::new(MaskEngine::new(rules));
    
    // parking_lot 不需要 unwrap
    let mut guard = state.engine.write();
    *guard = new_engine; 
    Ok(())
}

/// 获取脱敏历史记录
#[tauri::command]
pub async fn get_mask_history(state: State<'_, AppState>) -> AppResult<Vec<MaskHistoryItem>> {
    Ok(state.history.lock().clone())
}

/// 清除历史记录
#[tauri::command]
pub async fn clear_history_cmd(state: State<'_, AppState>) -> AppResult<()> {
    state.history.lock().clear();
    Ok(())
}

/// 切换监控开关
#[tauri::command]
pub async fn toggle_monitor(state: State<'_, AppState>, enabled: bool) -> AppResult<()> {
    *state.is_monitor_on.lock() = enabled;
    Ok(())
}

/// 复制原文 (绕过脱敏监听)
/// 原理：将内容存入 last_content 缓存，监听器发现内容一致时会自动跳过
#[tauri::command]
pub async fn copy_original_cmd(state: State<'_, AppState>, text: String) -> AppResult<()> {
    // 1. 设置去重缓存
    {
        let mut last = state.last_content.lock();
        *last = text.clone();
    }
    
    // 2. 写入剪贴板
    let mut cb = arboard::Clipboard::new().map_err(|e| crate::common::errors::AppError::Clipboard(e.to_string()))?;
    cb.set_text(text).map_err(|e| crate::common::errors::AppError::Clipboard(e.to_string()))?;
    
    Ok(())
}

/// 获取应用元数据
#[tauri::command]
pub fn get_app_info() -> serde_json::Value {
    serde_json::json!({
        "version": "1.0.2",
        "author": "XiaoSheng",
        "github": "https://github.com/AiToByte/SafeMask",
        "description": "极致性能的本地隐私脱敏引擎"
    })
}

#[tauri::command]
pub async fn toggle_always_on_top(window: tauri::Window, enabled: bool) -> AppResult<()> {
    // 调用窗口的原生方法设置置顶
    window.set_always_on_top(enabled)
        .map_err(|e| crate::common::errors::AppError::Internal(e.to_string()))?;
    Ok(())
}