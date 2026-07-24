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
    // 同步脱敏标签包裹样式
    new_engine.set_wrapper_style(&state.settings.read().mask_wrapper_style);
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
        "version": env!("CARGO_PKG_VERSION"),
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

    // 同步脱敏标签包裹样式到引擎
    state.engine.read().set_wrapper_style(&new_settings.mask_wrapper_style);

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
    let mut recognizers = engine.registry().recognizer_names();
    if !engine.is_ai_enabled() {
        recognizers.retain(|n| *n != "ner_engine");
    }
    Ok(serde_json::json!({
        "rule_count": engine.rule_count(),
        "recognizers": recognizers,
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

/// 同步原生窗口标题栏/背景色到当前 UI 主题。
///
/// - `theme_id`: `"default"` | `"claude"`
/// - Windows 上额外设置 DWM 标题栏背景/文字色，避免系统标题栏仍是黑色
#[tauri::command]
pub async fn apply_window_chrome(
    window: tauri::Window,
    theme_id: String,
) -> AppResult<()> {
    let is_light = theme_id == "claude";

    // 1) Tauri 系统主题（影响标题栏按钮等系统控件）
    let theme = if is_light {
        tauri::Theme::Light
    } else {
        tauri::Theme::Dark
    };
    let _ = window.set_theme(Some(theme));

    // 2) 窗口背景色
    let (r, g, b) = if is_light {
        (0xF5u8, 0xF1u8, 0xE8u8) // #F5F1E8
    } else {
        (0x0Cu8, 0x0Bu8, 0x0Au8) // #0c0b0a
    };
    let _ = window.set_background_color(Some(tauri::window::Color(r, g, b, 255)));

    // 3) Windows DWM 标题栏配色（Win11 22H2+ 支持 CAPTION_COLOR）
    #[cfg(target_os = "windows")]
    {
        use std::mem::size_of;

        #[link(name = "dwmapi")]
        unsafe extern "system" {
            fn DwmSetWindowAttribute(
                hwnd: *mut std::ffi::c_void,
                dwAttribute: u32,
                pvAttribute: *const std::ffi::c_void,
                cbAttribute: u32,
            ) -> i32;
        }

        const DWMWA_USE_IMMERSIVE_DARK_MODE: u32 = 20;
        const DWMWA_CAPTION_COLOR: u32 = 35;
        const DWMWA_TEXT_COLOR: u32 = 36;

        if let Ok(hwnd) = window.hwnd() {
            let ptr = hwnd.0;
            if !ptr.is_null() {
                unsafe {
                    // 0 = 浅色标题栏，1 = 深色标题栏
                    let dark_mode: i32 = if is_light { 0 } else { 1 };
                    let _ = DwmSetWindowAttribute(
                        ptr,
                        DWMWA_USE_IMMERSIVE_DARK_MODE,
                        &dark_mode as *const _ as *const std::ffi::c_void,
                        size_of::<i32>() as u32,
                    );

                    // COLORREF = 0x00BBGGRR
                    let caption: u32 = if is_light {
                        0x00E8_F1_F5 // #F5F1E8
                    } else {
                        0x000A_0B_0C // #0c0b0a
                    };
                    let text: u32 = if is_light {
                        0x0029_393D // #3D3929
                    } else {
                        0x00FE_F3_F0 // 近白
                    };

                    let _ = DwmSetWindowAttribute(
                        ptr,
                        DWMWA_CAPTION_COLOR,
                        &caption as *const _ as *const std::ffi::c_void,
                        size_of::<u32>() as u32,
                    );
                    let _ = DwmSetWindowAttribute(
                        ptr,
                        DWMWA_TEXT_COLOR,
                        &text as *const _ as *const std::ffi::c_void,
                        size_of::<u32>() as u32,
                    );
                }
            }
        }
    }

    Ok(())
}

/// 从本地 YAML 文件批量导入自定义规则（可多文件）。
///
/// - 冲突策略固定为覆盖同名自定义规则；与内置同名则跳过
/// - 文件大小 / 数量 / 条数有上限
/// - 成功后原子写盘并热重载引擎
#[tauri::command]
pub async fn import_custom_rules(
    app: AppHandle,
    state: State<'_, AppState>,
    paths: Vec<String>,
) -> AppResult<crate::infra::config::rule_import::ImportRulesReport> {
    use crate::infra::config::rule_import::{
        self, ConflictPolicy, MAX_FILE_BYTES, MAX_FILES, MAX_RULES,
    };
    use std::path::PathBuf;

    if paths.is_empty() {
        return Err(crate::common::errors::AppError::Config("未选择任何文件".into()));
    }
    if paths.len() > MAX_FILES {
        return Err(crate::common::errors::AppError::Config(format!(
            "单次最多导入 {} 个文件",
            MAX_FILES
        )));
    }

    let mut parsed_files = Vec::new();
    let mut total_rules = 0usize;

    for p in &paths {
        let path = PathBuf::from(p);
        let ext = path
            .extension()
            .and_then(|s| s.to_str())
            .unwrap_or("")
            .to_ascii_lowercase();
        if ext != "yaml" && ext != "yml" {
            return Err(crate::common::errors::AppError::Config(format!(
                "仅支持 .yaml / .yml: {}",
                path.display()
            )));
        }
        let meta = std::fs::metadata(&path).map_err(|e| {
            crate::common::errors::AppError::Config(format!(
                "无法读取文件 {}: {}",
                path.display(),
                e
            ))
        })?;
        if meta.len() > MAX_FILE_BYTES {
            return Err(crate::common::errors::AppError::Config(format!(
                "文件过大（>{}KB）: {}",
                MAX_FILE_BYTES / 1024,
                path.display()
            )));
        }
        let content = std::fs::read_to_string(&path).map_err(|e| {
            crate::common::errors::AppError::Config(format!(
                "读取失败 {}: {}",
                path.display(),
                e
            ))
        })?;
        let source = path
            .file_name()
            .and_then(|s| s.to_str())
            .unwrap_or(p)
            .to_string();
        let file = rule_import::parse_rules_yaml(&content, &source)
            .map_err(crate::common::errors::AppError::Config)?;
        total_rules += file.rules.len();
        if total_rules > MAX_RULES {
            return Err(crate::common::errors::AppError::Config(format!(
                "单次最多导入 {} 条规则",
                MAX_RULES
            )));
        }
        parsed_files.push(file);
    }

    let wrapper = state.settings.read().mask_wrapper_style.clone();
    let existing = ConfigLoader::load_custom_rules_only(&app);
    let builtin = ConfigLoader::builtin_rule_names(&app);

    let mut report = rule_import::merge_import(
        existing,
        &builtin,
        parsed_files,
        &wrapper,
        ConflictPolicy::OverwriteCustom,
    );

    // 仅当有实际变更时写盘
    let changed = report.imported + report.overwritten;
    if changed > 0 {
        ConfigLoader::write_all_custom_rules(&app, report.merged_custom_rules.clone())?;
        reload_engine_internal(app, state).await?;
    }

    // 清空 skip 序列化字段后返回
    report.merged_custom_rules.clear();
    Ok(report)
}

/// 导出全部自定义规则为 YAML 文本（供前端保存到用户选择路径）。
#[tauri::command]
pub async fn export_custom_rules_yaml(app: AppHandle) -> AppResult<String> {
    use crate::core::rules::RuleGroup;
    let rules = ConfigLoader::load_custom_rules_only(&app);
    let yaml = serde_yaml::to_string(&RuleGroup {
        group: "CUSTOM".into(),
        rules,
    })
    .map_err(|e| crate::common::errors::AppError::Config(format!("导出序列化失败: {}", e)))?;
    Ok(yaml)
}

/// 返回官方规则导入模板 YAML。
#[tauri::command]
pub async fn get_rules_import_template() -> AppResult<String> {
    Ok(crate::infra::config::rule_import::rules_import_template_yaml().to_string())
}

/// 将文本写入用户通过 save 对话框选择的路径（仅用于导出模板/规则）。
#[tauri::command]
pub async fn save_text_to_path(path: String, content: String) -> AppResult<()> {
    use std::path::PathBuf;
    let p = PathBuf::from(&path);
    let ext = p
        .extension()
        .and_then(|s| s.to_str())
        .unwrap_or("")
        .to_ascii_lowercase();
    if ext != "yaml" && ext != "yml" && ext != "txt" {
        return Err(crate::common::errors::AppError::Config(
            "仅允许写入 .yaml / .yml / .txt".into(),
        ));
    }
    if content.len() > 2 * 1024 * 1024 {
        return Err(crate::common::errors::AppError::Config(
            "导出内容过大".into(),
        ));
    }
    std::fs::write(&p, content.as_bytes()).map_err(|e| {
        crate::common::errors::AppError::Config(format!("写入失败: {}", e))
    })?;
    Ok(())
}

