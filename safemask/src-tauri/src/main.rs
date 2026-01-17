// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod engine;
mod config;
mod clipboard;

use std::sync::{Arc, Mutex};
use tauri::{Manager, State, AppHandle};
use tauri_plugin_global_hotkey::{GlobalHotkeyExt, Hotkey};
use crate::engine::MaskEngine;
use crate::config::RuleManager;
use crate::clipboard::GlobalClipboardHandler;
use clipboard_master::Master;

// 使用 mimalloc 提升内存分配性能
#[global_allocator]
static GLOBAL: mimalloc::MiMalloc = mimalloc::MiMalloc;

/// 应用全局状态，将在 Tauri 生命周期内持续存在
struct AppState {
    // 包装引擎，使其跨线程安全
    engine: Arc<MaskEngine>,
    // 记录最后一次脱敏后的内容，防止“处理 -> 写回 -> 再次检测到变化”的死循环
    last_content: Arc<Mutex<String>>,
    // 控制自动监控的开关
    is_monitor_on: Arc<Mutex<bool>>,
}

/// 前端调用命令：单次脱敏文本
/// 指令：手动触发当前剪贴板脱敏 (供前端按钮调用)
#[tauri::command]
async fn manual_mask(state: State<'_, AppState>) -> Result<String, String> {
    let mut ctx = arboard::Clipboard::new().map_err(|e| e.to_string())?;
    let text = ctx.get_text().map_err(|e| e.to_string())?;
    
    let masked = state.engine.mask_line(text.as_bytes());
    let masked_text = String::from_utf8_lossy(&masked).into_owned();
    
    ctx.set_text(masked_text.clone()).map_err(|e| e.to_string())?;
    Ok("手动脱敏成功".to_string())
}

/// 前端调用命令：开启/关闭剪贴板监控
#[tauri::command]
async fn toggle_monitor(enabled: bool, state: State<'_, AppState>) -> Result<(), String> {
    let mut monitor = state.is_monitor_on.lock().await;
    *monitor = enabled;
    println!("🔔 自动监控状态: {}", enabled);
    Ok(())
}


fn main() {
    // 1. 初始化核心引擎
    let rules = RuleManager::load_all_rules();
    let engine = Arc::new(MaskEngine::new(rules));
    
    // 共享状态
    let is_monitor_on = Arc::new(Mutex::new(true));
    let last_content = Arc::new(Mutex::new(String::new()));

    let engine_clone = engine.clone();
    let is_monitor_clone = is_monitor_on.clone();
    let last_content_clone = last_content.clone();

    tauri::Builder::default()
        .plugin(tauri_plugin_global_hotkey::Builder::new().build())
        .plugin(tauri_plugin_notification::init())
        .manage(AppState {
            engine: engine.clone(),
            is_monitor_on,
            last_content,
        })
        .invoke_handler(tauri::generate_handler![manual_mask])
        .setup(move |app| {
            let app_handle = app.handle().clone();

            // --- 方案一：启动原生事件驱动监听线程 ---
            std::thread::spawn(move || {
                let handler = GlobalClipboardHandler {
                    app_handle,
                    engine: engine_clone,
                    last_content: last_content_clone,
                    is_enabled: is_monitor_clone,
                };
                // Master::new(handler).run() 会阻塞线程，监听系统剪贴板信号
                Master::new(handler).run().expect("无法启动剪贴板监听器");
            });

            // --- 方案二：注册全局热键 (Alt+Shift+D) ---
            let hotkey = Hotkey::new(
                Some(tauri_plugin_global_hotkey::Modifiers::ALT | tauri_plugin_global_hotkey::Modifiers::SHIFT),
                tauri_plugin_global_hotkey::Code::KeyD,
            );

            app.global_hotkey().register(hotkey, move |app, _event| {
                // 当按下热键时，执行脱敏
                let state = app.state::<AppState>();
                let mut ctx = arboard::Clipboard::new().unwrap();
                if let Ok(text) = ctx.get_text() {
                    let masked = state.engine.mask_line(text.as_bytes());
                    let masked_text = String::from_utf8_lossy(&masked).into_owned();
                    let _ = ctx.set_text(masked_text);
                    
                    // 发送系统通知
                    app.emit("masked-event", "🚀 热键触发：内容已安全脱敏").unwrap();
                }
            })?;

            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

