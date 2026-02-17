#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod api;
mod common;
mod core;
mod infra;

use crate::common::state::AppState;
use crate::core::engine::MaskEngine;
use crate::infra::config::loader::ConfigLoader;
use crate::infra::config::shortcut_manager::ShortcutManager;
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
    menu::{MenuBuilder, MenuItemBuilder}  // ← 一次性导入 MenuBuilder 和 MenuItemBuilder
};

// 🚀 核心修复：导入快捷键插件相关的类型和扩展 Trait
use tauri_plugin_global_shortcut::{GlobalShortcutExt, Shortcut, Modifiers, Code, ShortcutState};

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
        .plugin(tauri_plugin_notification::init())
         // 1. 注册快捷键插件（这里由于插件机制，必须在 Builder 链中声明）
        .plugin(init_shortcut_plugin()) 
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
            api::system::toggle_always_on_top,
            api::system::update_app_settings,
            api::system::get_app_settings,
            api::system::toggle_vault_mode,
            api::system::test_rule_logic,
            api::system::set_recording_mode,  // 🚀 新增命令
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
    // 2. 初始化全局状态 (配置、引擎、AppState)
    init_app_state(handle)?;

    // 3. 初始化快捷键注册
    init_shortcut_service(handle)?;

    // 4. 初始化后台服务 (剪贴板监听)
    init_background_services(handle)?;

    // 5. 初始化窗口与托盘
    setup_window_handlers(handle)?;
    setup_system_tray(app)?;

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

    let _tray = tauri::tray::TrayIconBuilder::with_id(tray_id)
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

            if let tauri::tray::TrayIconEvent::Click { button, .. } = event {
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


// ─────────────────────────────────────────────────────────────────────────────
// 工具初始化方法定义
// ─────────────────────────────────────────────────────────────────────────────

/// 初始化全局状态：加载配置、编译引擎、管理 AppState
fn init_app_state(handle: &AppHandle) -> Result<(), Box<dyn std::error::Error>> {
    info!("🛠️ [Init] 正在初始化应用状态...");

    // 加载持久化设置
    let settings = ConfigLoader::load_settings(handle);

    // 加载并编译规则引擎
    let rules = ConfigLoader::load_all_rules(handle);
    let engine = Arc::new(MaskEngine::new(rules));

    // 构建核心状态机
    let app_state = AppState {
        engine: Arc::new(RwLock::new(engine)),
        settings: Arc::new(RwLock::new(settings)),
        shadow_store: Arc::new(RwLock::new(crate::common::state::ShadowClipboard::default())),
        is_magic_pasting: Arc::new(AtomicBool::new(false)),
        is_monitor_on: Arc::new(Mutex::new(true)),
        history: Arc::new(Mutex::new(Vec::new())),
        last_content: Arc::new(Mutex::new(String::new())),
        is_recording_mode: Arc::new(AtomicBool::new(false)),  // 🚀 新增录制模式状态
    };

    // 托管状态
    handle.manage(app_state);
    info!("✅ [Init] 全局状态注入完成");
    Ok(())
}

fn init_shortcut_plugin() -> tauri::plugin::TauriPlugin<tauri::Wry> {
    tauri_plugin_global_shortcut::Builder::new()
        .with_handler(move |app_handle, shortcut, event| {
            // 🚀 这里的 ShortcutState 现在可以被正确解析了
            if event.state() == ShortcutState::Pressed {
                let h = app_handle.clone();
                let state = h.state::<AppState>();
                let settings = state.settings.read().clone();

                // 校验动态快捷键 (Alt+V)
                if let Ok(magic_v) = ShortcutManager::parse_shortcut_string(&settings.magic_paste_shortcut) {
                    if shortcut == &magic_v {
                        tauri::async_runtime::spawn(async move {
                            crate::infra::clipboard::magic_paste::MagicPaster::execute(&h).await;
                        });
                        return;
                    }
                }

                 // --- 🚀 逻辑 B: 模式一键切换 (统一使用管理器解析) ---
                if let Ok(mode_toggle) = ShortcutManager::parse_shortcut_string("Alt+M") {
                    if shortcut == &mode_toggle {
                        let h2 = app_handle.clone();
                        tauri::async_runtime::spawn(async move {
                            if let Ok(new_mode_is_shadow) = crate::api::system::toggle_vault_mode(h2.clone(), h2.state()).await {
                                let payload = if new_mode_is_shadow { "SHADOW" } else { "SENTRY" };
                                let _ = h2.emit("mode-switch-event", payload);
                            }
                        });
                    }
                }
            }
        })
        .build()
}

/// [工具方法] 执行初始快捷键在系统中的注册
fn init_shortcut_service(handle: &AppHandle) -> Result<(), Box<dyn std::error::Error>> {
    let settings = handle.state::<crate::common::state::AppState>().settings.read().clone();
    
    // 1. 注册动态的安全粘贴快捷键 (如 Alt+V)
    ShortcutManager::reload_magic_shortcut(handle, &settings.magic_paste_shortcut)
        .map_err(|e| e.to_string())?;

    // 2. 🚀 注册静态模式切换键 Alt+M (同样使用管理器解析)
    if let Ok(alt_m_shortcut) = ShortcutManager::parse_shortcut_string("Alt+M") {
        let _ = handle.global_shortcut().register(alt_m_shortcut);
    } else {
        error!("无法解析默认切换快捷键 Alt+M");
    }

    Ok(())
}

/// 启动后台常驻服务
fn init_background_services(handle: &AppHandle) -> Result<(), Box<dyn std::error::Error>> {
    info!("🎧 [Init] 正在启动后台哨兵服务...");
    crate::infra::clipboard::monitor::start_listener(handle.clone());
    Ok(())
}

/// 处理窗口特有事件（如关闭拦截）
fn setup_window_handlers(handle: &AppHandle) -> Result<(), Box<dyn std::error::Error>> {
    if let Some(window) = handle.get_webview_window("main") {
        let w = window.clone();
        window.on_window_event(move |event| {
            if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                api.prevent_close();
                let _ = w.emit("request-close", ());
            }
        });
    }
    Ok(())
}