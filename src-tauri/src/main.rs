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
use tauri_plugin_global_shortcut::{GlobalShortcutExt, Shortcut, Modifiers, Code, Builder as ShortcutBuilder};

use crate::state::{AppState};
use crate::clipboard::GlobalClipboardHandler;
use crate::engine::MaskEngine;
use tauri::{
    menu::{MenuBuilder, MenuItemBuilder},
    tray::{MouseButton, TrayIconBuilder, TrayIconEvent},
    Emitter, Manager, WindowEvent
};

#[global_allocator]
static GLOBAL: mimalloc::MiMalloc = mimalloc::MiMalloc;

fn main() {
    // 1. 加载规则
    let rules = crate::config::RuleManager::load_all_rules();
    
    // 2. 🚀 定义核心引擎变量 (确保变量名是 engine_arc 且在最前面)
    let engine_arc = Arc::new(MaskEngine::new(rules));
    
    // 3. 初始化共享状态
    let is_monitor_on = Arc::new(Mutex::new(true));
    let last_content = Arc::new(Mutex::new(String::new()));
    let history = Arc::new(Mutex::new(Vec::new()));

    // 4. 为不同的闭包（Closure）准备克隆引用
    let engine_for_setup = engine_arc.clone();
    let is_monitor_on_setup = is_monitor_on.clone();
    let last_content_setup = last_content.clone();
    let history_setup = history.clone();

    // 2. 启动并构建应用
    tauri::Builder::default()
        // 🚀 新增：使用 tauri-plugin-dialog
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_opener::init())
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
            engine: engine_for_setup, // 占位，实际逻辑通过 Lazy ENGINE
            is_monitor_on: is_monitor_on_setup,
            last_content: last_content_setup,
            history: history_setup,
        })
        .invoke_handler(tauri::generate_handler![
            commands::manual_mask_cmd,
            commands::toggle_monitor,
            commands::process_file_gui,
            commands::get_rules_stats,
            commands::get_mask_history
        ])
        .setup(move |app| {
            
           // 1. 创建托盘菜单
    let quit_i = MenuItemBuilder::with_id("quit", "退出 SafeMask").build(app)?;
    let show_i = MenuItemBuilder::with_id("show", "显示主界面").build(app)?;
    let menu = MenuBuilder::new(app).items(&[&show_i, &quit_i]).build()?;

    // 2. 初始化托盘图标
    let _tray = TrayIconBuilder::new()
        .icon(app.default_window_icon().unwrap().clone()) // 使用默认图标
        .menu(&menu)
        .show_menu_on_left_click(false) // 左键通常用来显示窗口，右键显式菜单A
        .on_menu_event(|app, event| match event.id().as_ref() {
            "quit" => { app.exit(0); }
            "show" => {
                if let Some(window) = app.get_webview_window("main") {
                    let _ = window.show();
                    let _ = window.set_focus();
                }
            }
            _ => {}
        })
        .on_tray_icon_event(|tray, event| {
            // 逻辑：左键点击托盘图标时还原窗口
            if let TrayIconEvent::Click { button: MouseButton::Left, .. } = event {
                let app = tray.app_handle();
                if let Some(window) = app.get_webview_window("main") {
                    let _ = window.show();
                    let _ = window.set_focus();
                }
            }
        })
        .build(app)?;


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
        // 🚀 新增：拦截窗口关闭按钮
        .on_window_event(|window, event| {
            if let WindowEvent::CloseRequested { api, .. } = event {
                // 1. 阻止立即关闭
                api.prevent_close();
                
                // 2. 打印调试信息（如果你在终端运行，能看到这个说明 Rust 拦截成功了）
                println!("⚠️ 检测到关闭请求，正在通知前端...");

                // 3. 使用全局发射（emit）确保所有监听者都能收到，payload 传一个简单的字符串
                let _ = window.emit("request-close", "OPEN_MODAL");
            }
        })
        .run(tauri::generate_context!())
        .expect("Tauri 应用启动失败");
}