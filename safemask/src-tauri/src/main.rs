// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod engine;
mod config;
mod clipboard;

use std::sync::{Arc, Mutex};
use std::collections::BTreeMap;
use std::io::{BufWriter, Write};
use std::fs::File;

use tauri::{Manager, State, AppHandle, Emitter};
use tauri_plugin_global_shortcut::{GlobalShortcutExt, Shortcut, Modifiers, Code, Builder as ShortcutBuilder};
use crate::engine::MaskEngine;
use crate::config::RuleManager;
use crate::clipboard::GlobalClipboardHandler;

use rayon::prelude::{ParallelSlice, ParallelIterator, IndexedParallelIterator};

// 使用 mimalloc 提升内存分配性能
#[global_allocator]
static GLOBAL: mimalloc::MiMalloc = mimalloc::MiMalloc;

// 常量配置
const MACRO_CHUNK_SIZE: usize = 4 * 1024 * 1024; // 4MB
const BUFFER_SIZE: usize = 8 * 1024 * 1024;    // 8MB

/// 进度负载
#[derive(serde::Serialize, Clone)]
struct ProgressPayload {
    percentage: f32,
    processed_mb: f64,
}

/// 应用全局状态，将在 Tauri 生命周期内持续存在
struct AppState {
    // 包装引擎，使其跨线程安全
    engine: Arc<MaskEngine>,
    // 记录最后一次脱敏后的内容，防止“处理 -> 写回 -> 再次检测到变化”的死循环
    #[allow(dead_code)]
    last_content: Arc<Mutex<String>>,
    // 控制自动监控的开关
    is_monitor_on: Arc<Mutex<bool>>,
}

// --- Tauri Commands ---

/// 前端调用命令：单次脱敏文本
/// 指令：手动触发当前剪贴板脱敏 (供前端按钮调用)
/// 命令：手动脱敏并返回预览（供前端按钮或逻辑调用）
#[tauri::command]
async fn manual_mask_cmd(state: State<'_, AppState>) -> Result<String, String> {
    let mut ctx = arboard::Clipboard::new().map_err(|e| e.to_string())?;
    let text = ctx.get_text().map_err(|e| e.to_string())?;
    let masked = state.engine.mask_line(text.as_bytes());
    let masked_text = String::from_utf8_lossy(&masked).into_owned();
    ctx.set_text(masked_text.clone()).map_err(|e| e.to_string())?;
    Ok("脱敏已成功".into())
}

/// 命令：控制自动监听开关
#[tauri::command]
async fn toggle_monitor(enabled: bool, state: State<'_, AppState>) -> Result<(), String> {
    let mut monitor = state.is_monitor_on.lock().unwrap();
    *monitor = enabled;
    Ok(())
}

/// 命令：大文件保序脱敏流水线（带 GUI 进度回传）
#[tauri::command]
async fn process_file_gui(
    input_path: String,
    output_path: String,
    app_handle: AppHandle,
    state: State<'_, AppState>,
) -> Result<String, String> {
    let engine = state.engine.clone();
    let file = File::open(&input_path).map_err(|e| e.to_string())?;
    let mmap = unsafe { memmap2::Mmap::map(&file).map_err(|e| e.to_string())? };
    let file_size = mmap.len();
    let total_chunks = (file_size as f32 / MACRO_CHUNK_SIZE as f32).ceil() as usize;

    let (tx, rx) = crossbeam_channel::bounded::<(usize, Vec<u8>)>(rayon::current_num_threads() * 2);

    // 独立写入线程：保证输出顺序与输入完全一致
    let writer_handle = std::thread::spawn(move || -> Result<(), String> {
        let file_out = File::create(&output_path).map_err(|e| e.to_string())?;
        let mut writer = BufWriter::with_capacity(BUFFER_SIZE, file_out);
        let mut next_idx = 0;
        let mut pending_map = BTreeMap::new();
        let mut processed_count = 0;

        while let Ok((idx, data)) = rx.recv() {
            pending_map.insert(idx, data);
            while let Some(data) = pending_map.remove(&next_idx) {
                writer.write_all(&data).map_err(|e| e.to_string())?;
                next_idx += 1;
                processed_count += 1;
                
                // 发送进度到前端
                let _ = app_handle.emit("file-progress", ProgressPayload {
                    percentage: (processed_count as f32 / total_chunks as f32) * 100.0,
                    processed_mb: (processed_count * MACRO_CHUNK_SIZE) as f64 / 1024.0 / 1024.0,
                });
            }
        }
        writer.flush().map_err(|e| e.to_string())?;
        Ok(())
    });

    // 并行计算集群
    mmap.par_chunks(MACRO_CHUNK_SIZE)
        .enumerate()
        .for_each(|(idx, chunk)| {
            let mut out = Vec::with_capacity(chunk.len() + 2048);
            for line in chunk.split(|&b| b == b'\n') {
                if !line.is_empty() {
                    out.extend_from_slice(&engine.mask_line(line));
                }
                out.push(b'\n');
            }
            let _ = tx.send((idx, out));
        });

    drop(tx);
    writer_handle.join().map_err(|_| "写入线程异常退出")??;
    Ok("文件脱敏处理完成".into())
}


fn main() {
    // 1. 加载规则并初始化引擎
    let rules = RuleManager::load_all_rules();
    let engine = Arc::new(MaskEngine::new(rules));
    
    // 2. 初始化持久状态
    let is_monitor_on = Arc::new(Mutex::new(true));
    let last_content = Arc::new(Mutex::new(String::new()));

    // 预定义热键：Alt + Shift + D
    let shortcut = Shortcut::new(Some(Modifiers::ALT | Modifiers::SHIFT), Code::KeyS);


     tauri::Builder::default()
        // 修复：Tauri v2 的快捷键处理器需要在 Builder 中声明
        .plugin(ShortcutBuilder::new()
            .with_handler(move |app, s, _event| {
                // 检查按下的快捷键是否匹配我们的脱敏快捷键
                if s.matches(Modifiers::ALT | Modifiers::SHIFT, Code::KeyD) {
                    let state = app.state::<AppState>();
                    let mut ctx = arboard::Clipboard::new().unwrap();
                    if let Ok(text) = ctx.get_text() {
                        let masked = state.engine.mask_line(text.as_bytes());
                        let masked_text = String::from_utf8_lossy(&masked).into_owned();
                        let _ = ctx.set_text(masked_text);
                        let _ = app.emit("masked-event", "🚀 热键触发：隐私已清洗");
                    }
                }
            })
            .build()
        )
        .plugin(tauri_plugin_notification::init())
        .manage(AppState {
            engine: engine.clone(),
            is_monitor_on: is_monitor_on.clone(),
            last_content: last_content.clone(),
        })
        .invoke_handler(tauri::generate_handler![
            manual_mask_cmd,
            toggle_monitor,
            process_file_gui
        ])
        .setup(move |app| {
            // 注册快捷键
            app.global_shortcut().register(shortcut)?;

            // 启动后台监听线程
            let handle = app.handle().clone();
            let engine_monitor = engine.clone();
            let is_on_monitor = is_monitor_on.clone();
            let last_content_monitor = last_content.clone();
            
            std::thread::spawn(move || {
                let handler = GlobalClipboardHandler {
                    app_handle: handle,
                    engine: engine_monitor,
                    last_content: last_content_monitor,
                    is_enabled: is_on_monitor,
                };
                clipboard_master::Master::new(handler)
                    .expect("无法初始化监听器")
                    .run()
                    .expect("监听器异常退出");
            });

            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("Tauri 应用启动失败");
}

