use crate::common::state::{AppState, MaskHistoryItem};
use crate::common::errors::AppResult;
use crate::core::rules::Rule;
use crate::core::engine::MaskEngine;
use crate::infra::config::loader::ConfigLoader;
// 🚀 核心修复：必须引入 Emitter 才能使用 .emit() 方法
use tauri::{AppHandle, State, Emitter}; 
use std::sync::Arc;
use crate::core::config::AppSettings;
use log::{info, error};
use regex::bytes::Regex;

/// 获取规则统计信息 (仪表盘使用)
#[tauri::command]
pub async fn get_rules_stats(app: AppHandle) -> AppResult<serde_json::Value> {
    let rules = ConfigLoader::load_all_rules(&app);
    Ok(serde_json::json!({
        "rule_count": rules.len(),
    }))
}

/// 获取所有详细规则列表
#[tauri::command]
pub async fn get_all_detailed_rules(app: AppHandle) -> AppResult<Vec<Rule>> {
    Ok(ConfigLoader::load_all_rules(&app))
}

/// 保存或更新规则
#[tauri::command]
pub async fn save_rule_api(app: AppHandle, state: State<'_, AppState>, rule: Rule) -> AppResult<String> {
    ConfigLoader::save_custom_rule(&app, rule)?;
    reload_engine_internal(app, state).await?;
    Ok("规则已保存并应用".into())
}

/// 删除规则
#[tauri::command]
pub async fn delete_rule_api(app: AppHandle, state: State<'_, AppState>, name: String) -> AppResult<String> {
    ConfigLoader::delete_custom_rule(&app, &name)?;
    reload_engine_internal(app, state).await?;
    Ok("规则已删除".into())
}

/// 内部函数：重新加载规则并替换引擎
async fn reload_engine_internal(app: AppHandle, state: State<'_, AppState>) -> AppResult<()> {
    let rules = ConfigLoader::load_all_rules(&app);
    let new_engine = Arc::new(MaskEngine::new(rules));
    
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
#[tauri::command]
pub async fn copy_original_cmd(state: State<'_, AppState>, text: String) -> AppResult<()> {
    {
        let mut last = state.last_content.lock();
        *last = text.clone();
    }
    let mut cb = arboard::Clipboard::new().map_err(|e| crate::common::errors::AppError::Clipboard(e.to_string()))?;
    cb.set_text(text).map_err(|e| crate::common::errors::AppError::Clipboard(e.to_string()))?;
    Ok(())
}

/// 获取应用元数据
#[tauri::command]
pub fn get_app_info() -> serde_json::Value {
    serde_json::json!({
        "version": "1.1.3",
        "author": "XiaoSheng",
        "github": "https://github.com/AiToByte/SafeMask",
        "description": "极致性能的本地隐私脱敏引擎"
    })
}

#[tauri::command]
pub async fn toggle_always_on_top(window: tauri::Window, enabled: bool) -> AppResult<()> {
    window.set_always_on_top(enabled)
        .map_err(|e| crate::common::errors::AppError::Internal(e.to_string()))?;
    Ok(())
}

#[tauri::command]
pub async fn update_magic_shortcut_api(
    app: tauri::AppHandle,
    state: tauri::State<'_, AppState>,
    new_shortcut: String
) -> AppResult<String> {
    crate::infra::config::shortcut_manager::ShortcutManager::reload_magic_shortcut(&app, &new_shortcut)
        .map_err(|e| crate::common::errors::AppError::Config(e))?;

    {
        let mut settings = state.settings.write();
        settings.magic_paste_shortcut = new_shortcut.clone();
        crate::infra::config::loader::ConfigLoader::save_settings(&app, &settings)?;
    }
    Ok("快捷键已更新并生效".into())
}

/// 切换脱敏宇宙模式
#[tauri::command]
pub async fn toggle_vault_mode(app: tauri::AppHandle, state: tauri::State<'_, AppState>) -> AppResult<bool> {
    // 🚀 修复：直接初始化，避免未使用赋值警告
    let current_mode = {
        let mut settings = state.settings.write();
        settings.shadow_mode_enabled = !settings.shadow_mode_enabled;
        let mode = settings.shadow_mode_enabled;
        crate::infra::config::loader::ConfigLoader::save_settings(&app, &settings)?;
        mode
    };

    let mode_name = if current_mode { "SHADOW" } else { "SENTRY" };
    let _ = app.emit("mode-switch-event", mode_name);
    
    Ok(current_mode)
}

/// 获取当前应用配置
#[tauri::command]
pub async fn get_app_settings(state: State<'_, AppState>) -> AppResult<AppSettings> {
    Ok(state.settings.read().clone())
}

/// 更新应用配置 (新增命令，用于设置页面保存所有开关)
#[tauri::command]
pub async fn update_app_settings(
    app: AppHandle,
    state: State<'_, AppState>,
    new_settings: AppSettings,
) -> AppResult<String> {
    let old_shortcut = state.settings.read().magic_paste_shortcut.clone();
    let shortcut_changed = old_shortcut != new_settings.magic_paste_shortcut;

    {
        let mut guard = state.settings.write();
        *guard = new_settings.clone();
    }

    ConfigLoader::save_settings(&app, &new_settings)?;

    if shortcut_changed {
        crate::infra::config::shortcut_manager::ShortcutManager::reload_magic_shortcut(
            &app, 
            &new_settings.magic_paste_shortcut
        ).map_err(|e| crate::common::errors::AppError::Internal(e))?;
    }

    Ok("设置更新成功".into())
}

/// 实时测试单条规则的有效性
#[tauri::command]
pub async fn test_rule_logic(pattern: String, mask: String, test_text: String) -> AppResult<String> {
    let re = Regex::new(&pattern).map_err(|e| {
        crate::common::errors::AppError::Config(format!("正则语法错误: {}", e))
    })?;

    let input_bytes = test_text.as_bytes();
    let result = re.replace_all(input_bytes, mask.as_bytes());
    
    Ok(String::from_utf8_lossy(&result).to_string())
}