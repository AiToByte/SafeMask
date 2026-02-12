use crate::infra::clipboard::handler::GlobalClipboard;
use clipboard_master::{CallbackResult, ClipboardHandler};
use std::sync::Arc;
use tauri::{AppHandle, Manager};
// 🚀 导入 Tauri 的运行时句柄类型
use tauri::async_runtime::RuntimeHandle;
use std::time::Duration;
use log::{info, error};

#[allow(dead_code)]
struct ClipboardHandlerImpl {
    handler: Arc<GlobalClipboard>,
    // 🚀 修改此处：使用 RuntimeHandle 而不是 tokio::runtime::Handle
    rt: RuntimeHandle,
}

impl ClipboardHandler for ClipboardHandlerImpl {
    fn on_clipboard_change(&mut self) -> CallbackResult {
        let h = self.handler.clone();
        // RuntimeHandle 同样提供了 spawn 方法，用法一致
        self.rt.spawn(async move {
            h.process_change().await;
        });
        CallbackResult::Next
    }

    fn on_clipboard_error(&mut self, error: std::io::Error) -> CallbackResult {
        eprintln!("⚠️ [Clipboard] 监听错误: {}", error);
        CallbackResult::Next
    }
}

// pub fn start_listener(app: AppHandle) {
//     // 确保 GlobalClipboard 初始化
//     let handler_logic = Arc::new(GlobalClipboard::new(app.clone()));
    
//     // 🚀 获取 Tauri 维护的全局运行时句柄
//     let rt = tauri::async_runtime::handle().clone();

//     // 在独立线程中运行阻塞的剪贴板监听器
//     std::thread::spawn(move || {
//         let handler = ClipboardHandlerImpl { 
//             handler: handler_logic, 
//             rt 
//         };
        
//         match Master::new(handler) {
//             Ok(mut master) => {
//                 if let Err(e) = master.run() {
//                     eprintln!("❌ [Clipboard] 监听循环异常中断: {}", e);
//                 }
//             }
//             Err(e) => {
//                 eprintln!("❌ [Clipboard] 无法初始化 Master: {}", e);
//             }
//         }
//     });
// }

pub fn start_listener(app: AppHandle) {
    let handler_logic = Arc::new(GlobalClipboard::new(app.clone()));

    // 关键：把 app 克隆一份给闭包用
    let app_for_state = app.clone();

    tauri::async_runtime::spawn(async move {
        // 在闭包内部获取 state（现在 app_for_state 是 move 进来的，生命周期够长）
        let state = app_for_state.state::<crate::common::state::AppState>();

        let mut last_was_non_text = false;

        info!("🎧 [Clipboard] Polling 监听服务已启动 (间隔 600ms)");

        loop {
            match handler_logic.get_text() {
                Ok(text) => {
                    last_was_non_text = false;

                    let should_process = {
                        let last_global = state.last_content.lock();
                        !text.is_empty() && text != *last_global
                    };

                    if should_process {
                        {
                            let mut guard = state.last_content.lock();
                            *guard = text.clone();
                        }
                        info!("🔔 [Clipboard] 检测到变化: {} 字节", text.len());
                        handler_logic.process_change().await;
                    }
                }

                Err(arboard::Error::ContentNotAvailable) => {
                    if !last_was_non_text {
                        info!("📋 [Clipboard] 当前剪贴板内容为非文本格式 (已忽略)");
                        last_was_non_text = true;
                        let mut guard = state.last_content.lock();
                        guard.clear();
                    }
                }

                Err(e) => {
                    error!("⚠️ [Clipboard] 访问剪贴板失败: {}", e);
                    tokio::time::sleep(Duration::from_secs(1)).await;
                }
            }

            tokio::time::sleep(Duration::from_millis(600)).await;
        }
    });
}
