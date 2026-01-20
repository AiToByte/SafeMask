// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod engine;
mod config;
mod clipboard;
mod state;
mod commands;
mod processor;

use std::sync::{Arc, Mutex};
// 修复核心：显式导入 Emitter Trait
use tauri::{Emitter}; 
use tauri_plugin_global_shortcut::{GlobalShortcutExt, Shortcut, Modifiers, Code, Builder as ShortcutBuilder};

use crate::state::{AppState};
use crate::clipboard::GlobalClipboardHandler;
use crate::engine::MaskEngine;

#[global_allocator]
static GLOBAL: mimalloc::MiMalloc = mimalloc::MiMalloc;

fn main() {
    // 1. 准备全局状态
    let is_monitor_on = Arc::new(Mutex::new(true));
    let last_content = Arc::new(Mutex::new(String::new()));

    // 2. 启动并构建应用
    tauri::Builder::default()
        .plugin(ShortcutBuilder::new()
            .with_handler(move |app, s, _event| {
                // 热键逻辑：Alt + Shift + S
                if s.matches(Modifiers::ALT | Modifiers::SHIFT, Code::KeyS) {
                    tauri::async_runtime::block_on(commands::manual_mask_cmd()).ok();
                    let _ = app.emit("masked-event", "🚀 热键触发：隐私已清洗");
                }
            })
            .build()
        )
        .plugin(tauri_plugin_notification::init())
        .manage(AppState {
            engine: Arc::new(MaskEngine::new(vec![])), // 占位，实际逻辑通过 Lazy ENGINE
            is_monitor_on: is_monitor_on.clone(),
            last_content: last_content.clone(),
        })
        .invoke_handler(tauri::generate_handler![
            commands::manual_mask_cmd,
            commands::toggle_monitor,
            commands::process_file_gui,
            commands::get_rules_stats
        ])
        .setup(move |app| {
            let handle = app.handle().clone();
            
            // 方案一：启动系统级原生监听线程
            let is_on = is_monitor_on.clone();
            let last_c = last_content.clone();
            std::thread::spawn(move || {
                let handler = GlobalClipboardHandler {
                    app_handle: handle,
                    engine: Arc::new(MaskEngine::new(crate::config::RuleManager::load_all_rules())),
                    last_content: last_c,
                    is_enabled: is_on,
                };
                clipboard_master::Master::new(handler)
                    .expect("Failed to create Master")
                    .run()
                    .expect("Clipboard listener failed");
            });

            // 注册全局热键
            let shortcut = Shortcut::new(Some(Modifiers::ALT | Modifiers::SHIFT), Code::KeyS);
            app.global_shortcut().register(shortcut)?;

            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("Tauri 应用启动失败");
}