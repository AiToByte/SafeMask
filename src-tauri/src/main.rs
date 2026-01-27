// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod engine;
mod config;
mod clipboard;
mod state;
mod commands;
mod processor;

use std::sync::{Arc, Mutex, RwLock};
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
    // 1. 初始化应用状态容器
    // 注意：初始引擎为空，待 setup 阶段获取到资源路径后再注入真实规则
    let initial_engine = Arc::new(RwLock::new(MaskEngine::new(vec![])));
    let is_monitor_on = Arc::new(Mutex::new(true));
    let last_content = Arc::new(Mutex::new(String::new()));
    let history = Arc::new(Mutex::new(Vec::new()));

    // 为闭包克隆引用
    let engine_for_setup = initial_engine.clone();

    // 2. 启动并构建应用
    tauri::Builder::default()
        // --- 注册 Tauri 官方插件 ---
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_notification::init())
        // --- 方案二：全局热键处理器 (Tauri v2 规范) ---
        .plugin(ShortcutBuilder::new()
            .with_handler(move |app, shortcut, _event| {
                // 监听 Alt + Shift + S 执行手动脱敏
                if shortcut.matches(Modifiers::ALT | Modifiers::SHIFT, Code::KeyS) {
                    // 获取当前窗口状态并调用脱敏指令
                    let state = app.state::<AppState>();
                    tauri::async_runtime::block_on(commands::manual_mask_cmd(state)).ok();
                    let _ = app.emit("masked-event", "🚀 热键触发：剪贴板内容已清洗");
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
        })
        // --- 注册前端 JS 可调用的 Rust 命令 ---
        .invoke_handler(tauri::generate_handler![
            commands::manual_mask_cmd,
            commands::toggle_monitor,
            commands::process_file_gui,
            commands::get_rules_stats,
            commands::get_mask_history,
            commands::save_rule_api,
            commands::get_all_detailed_rules,
            commands::delete_rule_api,
            commands::copy_original_cmd
        ])
        // --- 应用引导初始化 (Setup) ---
        .setup(move |app| {
            let handle = app.handle();

            // 🚀 A. 动态加载规则：解决打包后路径找不到的问题
            // 通过 AppHandle 获取资源目录中的 rules/ 和 custom/
            let rules = RuleManager::load_all_rules(handle);
            {
                let mut engine_lock = engine_for_setup.write().unwrap();
                *engine_lock = MaskEngine::new(rules);
                println!("✅ 引擎初始化完成，已加载最新脱敏规则");
            }

            // 🚀 B. 创建系统托盘菜单与图标
            let quit_i = MenuItemBuilder::with_id("quit", "彻底退出 SafeMask").build(app)?;
            let show_i = MenuItemBuilder::with_id("show", "显示主界面").build(app)?;
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

            // 🚀 C. 方案一：启动系统级原生剪贴板监听线程
            let engine_for_monitor = engine_for_setup.clone();
            let is_on_monitor = is_monitor_on.clone();
            let last_c_monitor = last_content.clone();
            let handle_clone = handle.clone();

            std::thread::spawn(move || {
                let handler = GlobalClipboardHandler {
                    app_handle: handle_clone,
                    engine: engine_for_monitor,
                    last_content: last_c_monitor,
                    is_enabled: is_on_monitor,
                };
                // 基于事件驱动的高效监听器
                clipboard_master::Master::new(handler)
                    .expect("Master creation failed")
                    .run()
                    .expect("Clipboard listener loop failed");
            });

            // 🚀 D. 注册全局热键监听
            let shortcut = Shortcut::new(Some(Modifiers::ALT | Modifiers::SHIFT), Code::KeyS);
            app.global_shortcut().register(shortcut)?;

            Ok(())
        })
        // --- 核心：拦截窗口关闭请求 ---
        .on_window_event(|window, event| {
            if let WindowEvent::CloseRequested { api, .. } = event {
                // 阻止窗口真正销毁，将决策权交给前端 Vue (ExitConfirm.vue)
                api.prevent_close();
                let _ = window.emit("request-close", "SIGNAL_CLOSE");
            }
        })
        .run(tauri::generate_context!())
        .expect("SafeMask Tauri 应用启动异常");
}