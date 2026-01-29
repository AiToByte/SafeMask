// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod engine;
mod config;
mod clipboard;
mod state;
mod commands;
mod processor;

use std::sync::{Arc, Mutex, RwLock};
use std::sync::atomic::AtomicBool;
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
use crate::config::RuleManager;
#[global_allocator]
static GLOBAL: mimalloc::MiMalloc = mimalloc::MiMalloc;
fn main() {
     // 1. 初始化核心状态变量
    // engine 使用 RwLock 支持运行时热重载规则
    let initial_engine = Arc::new(RwLock::new(MaskEngine::new(vec![])));
    let is_monitor_on = Arc::new(Mutex::new(true));
    let last_content = Arc::new(Mutex::new(String::new()));
    let history = Arc::new(Mutex::new(Vec::new()));

    // 🚀 核心优化：原子标记位。用于抑制写回时的自触发循环。
    let is_internal_changing = Arc::new(AtomicBool::new(false));
    
    // 用于通知异步任务停止的通道
    let (shutdown_tx, shutdown_rx) = tokio::sync::watch::channel(());

    // 克隆引用用于 setup 闭包
    let engine_ref = initial_engine.clone();

    // 2. 构建并启动 Tauri 应用
    tauri::Builder::default()
        // --- 注册官方插件 ---
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_notification::init())
        
        // --- 注册热键插件：Alt + Shift + S 手动脱敏 ---
        .plugin(ShortcutBuilder::new()
            .with_handler(move |app, shortcut, _event| {
                if shortcut.matches(Modifiers::ALT | Modifiers::SHIFT, Code::KeyS) {
                    // 异步调用手动脱敏指令
                    let handle = app.clone();
                    tauri::async_runtime::spawn(async move {
                        let state_cmd = handle.state::<AppState>();
                        if let Ok(_) = commands::manual_mask_cmd(state_cmd).await {
                            let _ = handle.emit("masked-event", "🚀 热键触发：剪贴板隐私已清洗");
                        }
                    });
                }
            })
            .build()
        )

        // --- 注入全局状态 (State) ---
        .manage(AppState {
            engine: initial_engine,
            is_monitor_on: is_monitor_on.clone(),
            last_content: last_content.clone(),
            history: history.clone(),
            is_internal_changing: is_internal_changing.clone(),
            shutdown_tx,
            shutdown_rx,
        })

        // --- 注册前端 JS 可调用的 Rust 指令 ---
        .invoke_handler(tauri::generate_handler![
            commands::manual_mask_cmd,
            commands::toggle_monitor,
            commands::process_file_gui,
            commands::get_rules_stats,
            commands::get_mask_history,
            commands::save_rule_api,
            commands::get_all_detailed_rules,
            commands::delete_rule_api,
            commands::copy_original_cmd,
            commands::clear_history_cmd,
            commands::get_app_info
        ])

        // --- 应用引导与后台监听线程初始化 ---
        .setup(move |app| {
            let handle = app.handle();

            // A. 加载脱敏规则
            let rules = RuleManager::load_all_rules(handle);
            {
                let mut engine_lock = engine_ref.write().unwrap();
                *engine_lock = MaskEngine::new(rules);
                println!("✅ SafeMask Engine Initialized with {} rules.", RuleManager::load_all_rules(handle).len());
            }

            // B. 构建托盘图标与菜单
            let quit_i = MenuItemBuilder::with_id("quit", "退出程序").build(app)?;
            let show_i = MenuItemBuilder::with_id("show", "显示控制台").build(app)?;
            let menu = MenuBuilder::new(app).items(&[&show_i, &quit_i]).build()?;

            let _tray = TrayIconBuilder::new()
                .icon(app.default_window_icon().unwrap().clone())
                .menu(&menu)
                .show_menu_on_left_click(false) 
                .on_menu_event(|app, event| match event.id().as_ref() {
                    "quit" => app.exit(0),
                    "show" => {
                        if let Some(window) = app.get_webview_window("main") {
                            let _ = window.show();
                            let _ = window.set_focus();
                        }
                    }
                    _ => {}
                })
                .on_tray_icon_event(|tray, event| {
                    if let TrayIconEvent::Click { button: MouseButton::Left, .. } = event {
                        let app = tray.app_handle();
                        if let Some(window) = app.get_webview_window("main") {
                            let _ = window.show();
                            let _ = window.set_focus();
                        }
                    }
                })
                .build(app)?;

            // 🚀 C. 启动“长连接上下文”剪贴板监听线程
            let handle_for_listener = handle.clone();
            std::thread::spawn(move || {
                // 开启无限循环监听，带崩溃重启机制
                let mut retry_count = 0;
                loop {
                    // 🚀 修复点 2: 处理 Master::new 返回的 Result
                    let handler = GlobalClipboardHandler::new(handle_for_listener.clone());
                    
                    match clipboard_master::Master::new(handler) {
                        Ok(mut master) => {
                            println!("[Clipboard] 监听服务启动成功 (Session #{})", retry_count + 1);
                            if let Err(e) = master.run() {
                                eprintln!("[Clipboard] 运行中异常: {:?}", e);
                            }
                        }
                        Err(e) => {
                            eprintln!("[Clipboard] 无法创建监听器: {:?}", e);
                        }
                    }

                    std::thread::sleep(std::time::Duration::from_secs(2));
                    retry_count += 1;
                    if retry_count > 100 { break; }
                }
            });

            // D. 注册全局热键（Alt + Shift + S）
            let shortcut = Shortcut::new(Some(Modifiers::ALT | Modifiers::SHIFT), Code::KeyS);
            let _ = app.global_shortcut().register(shortcut);

            Ok(())
        })

        // --- 窗口事件拦截：实现“最小化到托盘”的确认逻辑 ---
        .on_window_event(|window, event| {
            if let WindowEvent::CloseRequested { api, .. } = event {
                // 阻止窗口直接退出
                api.prevent_close();
                // 向前端发送信号，触发 ExitConfirm.vue 弹窗
                let _ = window.emit("request-close", "SIGNAL_CLOSE");
            }
        })

        .run(tauri::generate_context!())
        .expect("SafeMask: 启动过程中发生致命错误");
}

// 辅助扩展：为了能让 Handler 在多轮重试中保持状态
impl GlobalClipboardHandler {
    fn clone_context(&self) -> Self {
        Self::new(self.app_handle.clone())
    }
}