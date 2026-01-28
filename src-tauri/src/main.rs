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
use tokio::sync::watch;  // 修复 watch::channel 找不到的问题
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
    // 创建 watch channel
    let (shutdown_tx, shutdown_rx) = watch::channel(());
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
            shutdown_tx,
            shutdown_rx,
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
            commands::copy_original_cmd,
            commands::clear_history_cmd,
            commands::get_app_info
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
            // let engine_for_monitor = engine_for_setup.clone();
            // let is_on_monitor = is_monitor_on.clone();
            // let last_c_monitor = last_content.clone();
            // let handle_clone = handle.clone();

            // std::thread::spawn(move || {
            //     let handler = GlobalClipboardHandler {
            //         app_handle: handle_clone,
            //         engine: engine_for_monitor,
            //         last_content: last_c_monitor,
            //         is_enabled: is_on_monitor,
            //     };
            //     const INITIAL_BACKOFF: Duration = Duration::from_secs(1);
            //     const MAX_BACKOFF: Duration = Duration::from_secs(30);
            //     let mut retry_count = 0u32;
            //     let mut backoff = INITIAL_BACKOFF;
            //     // // 基于事件驱动的高效监听器
            //     // match clipboard_master::Master::new(handler) {
            //     //     Ok(mut master) => {
            //     //         if let Err(e) = master.run() {
            //     //             eprintln!("剪贴板监听失败: {:?}", e);
            //     //             // 这里可以考虑重试机制或通知主线程
            //     //         }
            //     //     }
            //     //     Err(e) => {
            //     //         eprintln!("创建 Master 失败: {:?}", e);
            //     //     }
            //     // }
            //     loop {
            //         println!("[Clipboard] 尝试启动监听器 (第 {} 次尝试)", retry_count + 1);

            //         match clipboard_master::Master::new(handler.clone()) {  // 注意：handler 需要实现 Clone 或 Arc 包裹
            //             Ok(mut master) => {
            //                 println!("[Clipboard] Master 创建成功，即将进入监听循环 (尝试 #{})", retry_count + 1);

            //                 if let Err(e) = master.run() {
            //                     eprintln!("[Clipboard] run() 异常退出: {:?}", e);
            //                 } else {
            //                     println!("[Clipboard] run() 正常退出（可能是外部信号）");
            //                 }

            //                 // run() 退出后，认为需要重试
            //                 retry_count += 1;
            //             }

            //             Err(e) => {
            //                 eprintln!("[Clipboard] 创建 Master 失败 (尝试 #{}): {:?}", retry_count + 1, e);
            //                 retry_count += 1;
            //             }
            //         }

            //         // 指数退避 + 随机抖动（jitter），防止所有实例同时重试
            //         let sleep_duration = backoff.min(MAX_BACKOFF);
            //         let jitter = Duration::from_millis((rand::random::<u64>() % 500) as u64); // 需要引入 rand crate
            //         let total_sleep = sleep_duration + jitter;

            //         println!(
            //             "[Clipboard] 将在 {} 秒后重试 (当前 backoff: {:?})",
            //             total_sleep.as_secs_f32(),
            //             backoff
            //         );
            //         thread::sleep(total_sleep);
            //         // 指数增长 backoff
            //         backoff = backoff * 2;
            //         if backoff > MAX_BACKOFF {
            //             backoff = MAX_BACKOFF;
            //         }
            //         // 设置最大重试次数，防止无限循环
            //         if retry_count >= 50 {
            //             eprintln!("[Clipboard] 达到最大重试次数 ({})，永久停止监听", retry_count);
            //             break;
            //         }
            //     }
            // });
             // 修改 setup 闭包内的监听部分
            let handle_clone = app.handle().clone();
            let engine_for_monitor = engine_for_setup.clone();
            let is_on_monitor = is_monitor_on.clone();
            let last_c_monitor = last_content.clone();

            // 使用标准线程，不阻塞主线程，也不占用异步 Runtime
            std::thread::spawn(move || {
                let mut retry_count = 0;
                loop {
                    println!("[Clipboard] 启动监听器 (第 {} 次尝试)", retry_count + 1);

                    let handler = GlobalClipboardHandler {
                        app_handle: handle_clone.clone(),
                        engine: engine_for_monitor.clone(),
                        last_content: last_c_monitor.clone(),
                        is_enabled: is_on_monitor.clone(),
                    };

                     // 给系统窗口一点缓冲时间，防止抢占主线程初始化
                    std::thread::sleep(std::time::Duration::from_millis(500));

                    match clipboard_master::Master::new(handler) {
                        Ok(mut master) => {
                            // 这里会一直阻塞，直到出错或进程结束
                            if let Err(e) = master.run() {
                                eprintln!("[Clipboard] 监听异常退出: {:?}", e);
                            }
                        }
                        Err(e) => eprintln!("[Clipboard] 创建失败: {:?}", e),
                    }

                    // 退避重试
                    std::thread::sleep(std::time::Duration::from_secs(2));
                    retry_count += 1;
                    if retry_count > 50 { break; }
                }
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
                // 直接获取 state，它是通过 .manage 注入的 AppState 的引用
                let state = window.state::<AppState>();
                // shutdown_tx 是 watch::Sender，直接调用 send 即可，不需要 try_lock
                let _ = state.shutdown_tx.send(());
                let _ = window.emit("request-close", "SIGNAL_CLOSE");
            }
        })
        .run(tauri::generate_context!())
        .expect("SafeMask Tauri 应用启动异常");
}