#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod api;
mod common;
mod core;
mod infra;

// 🚀 修复核心：必须导入 Manager 才能使用 .manage()
use tauri::Manager; 
use crate::common::state::AppState;
use crate::core::engine::MaskEngine;
use crate::infra::config::loader::ConfigLoader;
use std::sync::{Arc, atomic::AtomicBool};
// 统一使用 parking_lot
// 🚀 显式从 parking_lot 导入
use parking_lot::{Mutex};
use log::{info, error, LevelFilter};
use {tauri_plugin_dialog, tauri_plugin_opener};  // ← 新增这一行导入 

#[global_allocator]
static GLOBAL: mimalloc::MiMalloc = mimalloc::MiMalloc;

fn main() {
    
    // 更健壮的初始化方式
    env_logger::builder()
        .filter_level(LevelFilter::Info)           // 默认 info 级别
        .filter_module("SafeMask", LevelFilter::Trace)  // 你的 crate 名强制 trace
        .target(env_logger::Target::Stdout)        // 强制输出到 stdout（终端更容易看到）
        .init();

    info!("🚀 env_logger 已初始化，级别: info+");

    info!("🚀 Tauri 应用启动中...");
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())   // ← 这里添加这一行！！
        .plugin(tauri_plugin_opener::init())
        .setup(|app| {
            info!("🛠️ Setup 开始...");
            let handle = app.handle();
            info!("📂 加载规则...");
            // 1. 加载规则 (🚀 修正点：直接获取 Vec<Rule>，不再需要 unwrap)
            let rules = ConfigLoader::load_all_rules(handle);
            info!("✅ 加载规则完成: {} 条", rules.len());

            // 初始化引擎...
            info!("🧠 引擎初始化完成");
            // 2. 初始化引擎实体
            let engine_instance = Arc::new(MaskEngine::new(rules));
            
            // 3. 构建全局状态
            // 🚀 这里显式使用 parking_lot 的构造方式
            let app_state = AppState {
                engine: Arc::new(parking_lot::RwLock::new(engine_instance)),  
                is_monitor_on: Arc::new(Mutex::new(true)),
                history: Arc::new(Mutex::new(Vec::new())),
                is_internal_changing: Arc::new(AtomicBool::new(false)),
                last_content: Arc::new(Mutex::new(String::new())),
            };
            // 4. 注入状态
            app.manage(app_state);
            // 注入状态...
            info!("🔗 状态注入完成");
            // 5. 启动剪贴板监听
            info!("🎧 启动剪贴板监听...");
            infra::clipboard::monitor::start_listener(handle.clone());
            info!("✅ Setup 完成");
            Ok(())
        })
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
        .expect("Tauri 运行失败");
}