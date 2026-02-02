use crate::infra::clipboard::handler::GlobalClipboard;
use clipboard_master::{CallbackResult, ClipboardHandler};
use std::sync::Arc;
use tauri::{AppHandle};
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
    
    // 使用 Tauri 的 async runtime，直接 spawn polling loop（无需独立线程）
    tauri::async_runtime::spawn(async move {
        let mut last_content = String::new();  // 缓存上次内容，避免重复处理
        
        info!("🎧 [Clipboard] Polling 监听服务已启动 (间隔 500ms)");
        
        loop {
            // 安全读取剪贴板
            let current = match handler_logic.get_text() {
                Ok(text) => text,
                Err(e) => {
                    error!("⚠️ [Clipboard] 读取失败: {}", e);
                    String::new()
                }
            };
            
            // 如果变化，处理
            if !current.is_empty() && current != last_content {
                info!("🔔 [Clipboard] 检测到变化: {} 字节", current.len());
                last_content = current.clone();
                handler_logic.process_change().await;
            }
            
            // 等待下次 poll（可调 300-1000ms）
            tokio::time::sleep(Duration::from_millis(500)).await;
        }
    });
}