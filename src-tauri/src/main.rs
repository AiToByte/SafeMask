#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod api;
mod common;
mod core;
mod infra;

use crate::common::state::AppState;
use crate::core::engine::MaskEngine;
use crate::infra::config::loader::ConfigLoader;
use std::sync::{Arc, atomic::AtomicBool};
// 统一使用 parking_lot
// 🚀 显式从 parking_lot 导入
use parking_lot::{Mutex, RwLock};
use log::{info, error, LevelFilter};
use {tauri_plugin_dialog, tauri_plugin_opener};  // ← 新增这一行导入 
use tauri::{
    AppHandle,                  // ← 新增，用于闭包参数类型
    Emitter,
    Manager,
    image::Image,
    menu::{MenuBuilder, MenuItemBuilder},  // ← 一次性导入 MenuBuilder 和 MenuItemBuilder
    tray::{TrayIconEvent}, // ← TrayIconEvent 用于 match
};
use std::path::Path;

#[global_allocator]
static GLOBAL: mimalloc::MiMalloc = mimalloc::MiMalloc;

/// 应用程序入口函数
/// 职责：初始化日志、创建 Tauri 应用构建器、注册插件和命令、启动应用
fn main() {
    // 初始化日志系统（放在最前面，便于后续所有模块都能输出日志）
    init_logger();

    info!("🚀 Tauri 应用启动中...");

    if let Err(e) = tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_opener::init())
        .setup(setup_application)
        .invoke_handler(tauri::generate_handler![
            api::system::get_rules_stats,
            api::system::get_all_detailed_rules,
            api::system::save_rule_api,
            api::system::delete_rule_api,
            api::system::get_mask_history,
            api::system::clear_history_cmd,
            api::system::toggle_monitor,
            api::system::copy_original_cmd,
            api::system::get_app_info,
            api::text::mask_text,
            api::files::process_file_gui,
        ])
        .run(tauri::generate_context!())
    {
        error!("Tauri 运行失败: {}", e);
        std::process::exit(1);
    }
}

/// 初始化日志系统
/// - 默认级别：Info
/// - 对本项目（SafeMask）模块强制使用 Trace 级别，便于调试
/// - 输出到标准输出（stdout）
fn init_logger() {
    env_logger::builder()
        .filter_level(LevelFilter::Info)
        .filter_module("SafeMask", LevelFilter::Trace)
        .target(env_logger::Target::Stdout)
        .init();

    info!("🚀 env_logger 已初始化，级别: info+ (SafeMask 模块为 trace)");
}

/// Tauri 应用初始化核心逻辑
/// 所有需要在应用启动时完成的初始化工作都集中在此函数中
fn setup_application(app: &mut tauri::App) -> Result<(), Box<dyn std::error::Error>> {
    info!("🛠️ Setup 开始...");

    let handle = app.handle();

    // 1. 加载所有规则（系统规则 + 用户自定义规则）
    info!("📂 加载规则...");
    let rules = ConfigLoader::load_all_rules(&handle);
    info!("✅ 加载规则完成: {} 条", rules.len());

    // 2. 创建脱敏引擎实例
    info!("🧠 初始化脱敏引擎...");
    let engine_instance = Arc::new(MaskEngine::new(rules));
    info!("✅ 引擎初始化完成");

    // 3. 构建并注入全局应用状态
    info!("🔗 准备全局状态...");
    let app_state = AppState {
        engine: Arc::new(RwLock::new(engine_instance)),
        is_monitor_on: Arc::new(Mutex::new(true)),
        history: Arc::new(Mutex::new(Vec::new())),
        is_internal_changing: Arc::new(AtomicBool::new(false)),
        last_content: Arc::new(Mutex::new(String::new())),
    };

    app.manage(app_state);
    info!("✅ 全局状态注入完成");

    // 4. 启动剪贴板实时监控（自动脱敏）
    info!("🎧 启动剪贴板监听...");
    infra::clipboard::monitor::start_listener(handle.clone());
    info!("✅ 剪贴板监听已启动");

    // 5. 设置窗口关闭拦截（显示退出确认对话框）
    info!("🪟 设置窗口关闭拦截...");
    init_window_close_handler(handle.clone())?;

    info!("🎉 Setup 完成！SafeMask 已就绪");
    // 创建托盘...
    setup_system_tray(app)?;
    Ok(())
}

/// 为主窗口注册关闭事件拦截
/// 当用户点击窗口关闭按钮时，不直接退出，而是发出 "request-close" 事件给前端
/// 让前端显示退出确认对话框（最小化到托盘 / 彻底退出）
fn init_window_close_handler(handle: tauri::AppHandle) -> Result<(), Box<dyn std::error::Error>> {
    let window = handle
        .get_webview_window("main")
        .ok_or("未找到主窗口 'main'")?;

    // Clone 给闭包使用（cheap 操作）
    let window_for_closure = window.clone();

    window.on_window_event(move |event| {
        if let tauri::WindowEvent::CloseRequested { api, .. } = event {
            api.prevent_close();

            // 使用克隆的 window 发出事件
            let _ = window_for_closure.emit("request-close", ());
            info!("捕获到关闭请求，已转发给前端处理");
        }
    });
    Ok(())
}

fn setup_system_tray(app: &tauri::App) -> Result<(), Box<dyn std::error::Error>> {
    info!("🛡️ 初始化系统托盘图标...");

    let _handle = app.handle().clone();

    // 加载图标（建议用资源路径，更可靠）
    let icon_path = app.path().resource_dir()?.join("icons/32x32.png");
    let icon = Image::from_path(&icon_path)
        .map_err(|e| format!("托盘图标加载失败 {}: {}", icon_path.display(), e))?;

    // ────────────────────────────────
    // 创建菜单项
    // ────────────────────────────────
    let show_item = MenuItemBuilder::with_id("show", "显示窗口")
        .build(app)?;

    let quit_item = MenuItemBuilder::with_id("quit", "退出程序")
        .build(app)?;

    // 构建菜单
    let menu = MenuBuilder::new(app)
        .items(&[&show_item, &quit_item])
        .build()?;

    // ────────────────────────────────
    // 创建托盘 + 附加菜单 + 事件处理
    // ────────────────────────────────
    let tray_id = "safemask-main-tray";

    let _tray = TrayIconBuilder::with_id(tray_id)
        .icon(icon)
        .tooltip("SafeMask - 隐私保护中")
        .menu(&menu)
        .on_menu_event(move |app: &AppHandle, event| {
            match event.id().as_ref() {
                "show" => {
                    if let Some(window) = app.get_webview_window("main") {
                        let _ = window.show();
                        let _ = window.set_focus();
                        info!("托盘菜单：显示主窗口");
                    }
                }
                "quit" => {
                    info!("托盘菜单：用户选择退出");
                    app.exit(0);
                }
                _ => {}
            }
        })
        // 左键点击直接显示窗口（推荐！）
        .on_tray_icon_event(move |tray, event| {   // 注意：第一个参数是 &TrayIcon，不是 &AppHandle
            use tauri::tray::TrayIconEvent;       

            if let TrayIconEvent::Click { button, .. } = event {
                if button == tauri::tray::MouseButton::Left {
                    if let Some(window) = tray.app_handle().get_webview_window("main") {
                        if window.is_visible().unwrap_or(false) {
                            let _ = window.hide();
                        } else {
                            let _ = window.show();
                            let _ = window.set_focus();
                        }
                        info!("托盘左键：切换窗口可见性");
                    }
                }
            }
        })
        .build(app)?;

    info!("✅ 系统托盘已初始化 (带菜单 & 左键切换)");

    Ok(())
}