use crate::common::state::{AppState, MaskHistoryItem};
use crate::common::errors::AppResult;
use crate::core::rules::Rule;
use crate::core::hybrid_engine::HybridEngine;
use crate::infra::config::loader::ConfigLoader;
// 🚀 核心修复：必须引入 Emitter 才能使用 .emit() 方法
use tauri::{AppHandle, State, Emitter}; 
use std::sync::Arc;
use crate::core::config::AppSettings;
use crate::core::download_auth;
use crate::infra::record_writer::{RecordWriter, MarkdownRecordWriter};
use log::{info, error};
use std::sync::atomic::Ordering;

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

/// 内部函数：重新加载规则并替换引擎（保留 AI 引擎）
async fn reload_engine_internal(app: AppHandle, state: State<'_, AppState>) -> AppResult<()> {
    let rules = ConfigLoader::load_all_rules(&app);
    let models_dir = state.models_dir.clone();
    let mut new_engine = HybridEngine::from_rules(rules);
    // 🚀 重新启用 AI 引擎，确保 reload 后 AI 识别器不丢失
    new_engine.enable_ai_engine(&models_dir);
    let new_engine = Arc::new(new_engine);

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
        "version": "1.2.4",
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

/// 获取当前应用配置（自动注入 Worker 代理下载 URL，保证前端可见性）
#[tauri::command]
pub async fn get_app_settings(state: State<'_, AppState>) -> AppResult<serde_json::Value> {
    let mut settings = state.settings.read().clone();
    // 清除内存中可能残留的旧 Worker 代理 URL
    settings.model_download_urls.retain(|u| !u.contains(download_auth::WORKER_BASE_URL));
    // 前置插入实时生成的 Worker 代理 URL（含 HMAC 令牌）
    let worker_url = download_auth::generate_worker_url(&state.device_id);
    settings.model_download_urls.insert(0, worker_url);

    // 1. 将 settings 序列化为 JSON Value
    let mut json = serde_json::to_value(&settings)
        .map_err(|e| crate::common::errors::AppError::Internal(e.to_string()))?;

    // 2. 🚀 关键修复：由于 AppSettings 内有 skip_serializing，我们在这里手动注入 model_download_urls
    // 这样既防止了写入本地 settings.yaml 导致旧 URL 固化，又保证了前端 API 100% 能拿到下载链接
    if let Some(obj) = json.as_object_mut() {
        obj.insert(
            "model_download_urls".to_string(),
            serde_json::to_value(&settings.model_download_urls)
                .map_err(|e| crate::common::errors::AppError::Internal(e.to_string()))?,
        );
    }

    Ok(json)
}

/// 更新应用配置 (新增命令，用于设置页面保存所有开关)
#[tauri::command]
pub async fn update_app_settings(
    app: AppHandle,
    state: State<'_, AppState>,
    mut new_settings: AppSettings,
) -> AppResult<serde_json::Value> {
    info!("[update_app_settings] INVOKED: record_writer_enabled={}", new_settings.record_writer_enabled);
    // 剥离前端可能回传的 Worker 代理 URL，防止污染持久化存储
    new_settings.model_download_urls.retain(|u| !u.contains(download_auth::WORKER_BASE_URL));
    let old_shortcut = state.settings.read().magic_paste_shortcut.clone();
    let shortcut_changed = old_shortcut != new_settings.magic_paste_shortcut;

    // 缓存旧值，在写入 state 前记录（用于后续 dirty check）
    let old_writer_enabled = state.settings.read().record_writer_enabled;

    {
        let mut guard = state.settings.write();
        *guard = new_settings.clone();
    }

    ConfigLoader::save_settings(&app, &new_settings)?;

    if shortcut_changed {
        crate::infra::config::shortcut_manager::ShortcutManager::reload_magic_shortcut(
            &app, 
            &new_settings.magic_paste_shortcut
        ).map_err(crate::common::errors::AppError::Internal)?;
    }

    // 无条件重建记录写入器（无论配置是否变化，确保 writer 与 state 一致）
    info!("[RecordWriter] 保存触发重建 (old={}, new={})",
        old_writer_enabled, new_settings.record_writer_enabled);
    rebuild_record_writer(&state).await?;

    // 返回记录目录路径用于前端诊断显示
    let records_dir = crate::infra::record_writer::default_records_dir();
    Ok(serde_json::json!({
        "message": "设置更新成功",
        "records_dir": records_dir.to_string_lossy(),
        "writer_enabled": new_settings.record_writer_enabled,
        "records_dir_exists": records_dir.exists()
    }))
}

/// 重建记录写入器：flush 旧的 → 根据新配置创建或移除
/// 返回 Err 表示输出目录不可写（前端会弹出错误提示）
pub async fn rebuild_record_writer(state: &State<'_, AppState>) -> AppResult<()> {
    // flush old (clone Arc out of lock first to avoid parking_lot guard across await)
    let old_writer = state.record_writer.read().clone();
    if let Some(writer) = old_writer {
        writer.flush().await;
    }

    // build new
    let enabled = {
        let s = state.settings.read();
        s.record_writer_enabled
    };
    let writer: Option<Arc<dyn RecordWriter>> = if enabled {
        let output_dir = crate::infra::record_writer::default_records_dir();
        info!("[RecordWriter] 重建记录写入器, 输出目录: {}", output_dir.display());
        // 立即创建目录并验证可写性，失败则向前端返回错误
        std::fs::create_dir_all(&output_dir).map_err(|e| {
            let msg = format!("无法创建记录目录 {}: {}", output_dir.display(), e);
            error!("[RecordWriter] {}", msg);
            crate::common::errors::AppError::Internal(msg)
        })?;
        // 写入测试文件验证可写性（写入后立即删除）
        let test_file = output_dir.join(".write_test");
        match std::fs::write(&test_file, b"test") {
            Ok(_) => { let _ = std::fs::remove_file(&test_file); },
            Err(e) => {
                let msg = format!("记录目录不可写 {}: {}", output_dir.display(), e);
                error!("[RecordWriter] {}", msg);
                return Err(crate::common::errors::AppError::Internal(msg));
            }
        }
        let (writer, task) = MarkdownRecordWriter::new(output_dir);
        tokio::spawn(task);
        Some(Arc::new(writer))
    } else {
        None
    };
    *state.record_writer.write() = writer;
    info!("[RecordWriter] 记录写入器状态已更新 (enabled={})", enabled);
    Ok(())
}

/// 获取记录目录诊断信息（用于前端调试）
#[tauri::command]
pub async fn get_records_dir_info(state: State<'_, AppState>) -> AppResult<serde_json::Value> {
    let records_dir = crate::infra::record_writer::default_records_dir();
    let writer_enabled = state.settings.read().record_writer_enabled;
    let has_writer = state.record_writer.read().is_some();
    Ok(serde_json::json!({
        "dir": records_dir.to_string_lossy(),
        "exists": records_dir.exists(),
        "writer_enabled": writer_enabled,
        "has_writer": has_writer,
    }))
}

/// 实时测试单条规则的有效性
#[tauri::command]
pub async fn test_rule_logic(pattern: String, mask: String, test_text: String) -> AppResult<String> {
    let re = regex::bytes::RegexBuilder::new(&pattern)
        .unicode(false)
        .build()
        .map_err(|e| {
        crate::common::errors::AppError::Config(format!("正则语法错误: {}", e))
    })?;

    let input_bytes = test_text.as_bytes();
    let result = re.replace_all(input_bytes, mask.as_bytes());
    
    Ok(String::from_utf8_lossy(&result).to_string())
}


#[tauri::command]
pub async fn set_recording_mode(state: State<'_, AppState>, enabled: bool) -> AppResult<()> {
    state.is_recording_mode.store(enabled, Ordering::SeqCst);
    info!("🚀 录制模式状态更新: {}", enabled);
    Ok(())
}

/// 获取 AI 引擎状态
///
/// 返回 AI 引擎的当前状态，包括：
/// - 模型加载状态 (not_loaded/loading/ready/error)
/// - 可用模型数量
/// - 模型信息 (名称、大小、支持的实体类型)
/// - 模型目录路径
#[tauri::command]
pub async fn get_ai_engine_status(state: State<'_, AppState>) -> AppResult<serde_json::Value> {
    let engine = state.engine.read();
    Ok(engine.ai_status())
}

/// 获取完整的引擎信息
///
/// 返回混合引擎的完整信息，包括：
/// - 规则数量
/// - 已注册的识别器列表
/// - AI 引擎状态
/// - 脱敏配置
#[tauri::command]
pub async fn get_engine_info(state: State<'_, AppState>) -> AppResult<serde_json::Value> {
    let engine = state.engine.read();
    Ok(serde_json::json!({
        "rule_count": engine.rule_count(),
        "recognizers": engine.registry().recognizer_names(),
        "ai_status": engine.ai_status(),
    }))
}

/// 启用/停用 AI 引擎
#[tauri::command]
pub async fn toggle_ai_engine(state: State<'_, AppState>, enabled: bool) -> AppResult<bool> {
    let engine = state.engine.write();
    let result = engine.set_ai_enabled(enabled);
    Ok(result)
}

/// 获取已注册的识别器列表
#[tauri::command]
pub async fn get_registered_recognizers(state: State<'_, AppState>) -> AppResult<Vec<String>> {
    let engine = state.engine.read();
    Ok(engine.registry().recognizer_names().iter().map(|s| s.to_string()).collect())
}