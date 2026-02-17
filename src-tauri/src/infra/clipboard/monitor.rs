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

pub fn start_listener(app: AppHandle) {
    let handler_logic = Arc::new(GlobalClipboard::new(app.clone()));
    let app_clone = app.clone();

    tauri::async_runtime::spawn(async move {
        let state = app_clone.state::<crate::common::state::AppState>();
        info!("🎧 [Clipboard] 哨兵轮询服务已启动 (600ms)");

        loop {
            // 1. 获取当前剪贴板文本
            match handler_logic.get_text() {
                Ok(text) => {
                    let should_trigger = {
                        let last_global = state.last_content.lock();
                        // 🚀 仅做初步判定，不在这里更新 last_content
                        !text.is_empty() && text != *last_global
                    };

                    if should_trigger {
                        // 🚀 发现变化，交给 handler 处理（handler 会负责更新缓存和影子宇宙）
                        handler_logic.process_change().await;
                    }
                }
                Err(arboard::Error::ContentNotAvailable) => {
                    // 非文本内容，清空缓存以便下次能捕获文本
                    let mut guard = state.last_content.lock();
                    if !guard.is_empty() {
                        guard.clear();
                    }
                }
                Err(e) => {
                    error!("⚠️ [Clipboard] 访问失败: {}", e);
                    tokio::time::sleep(Duration::from_secs(1)).await;
                }
            }
            tokio::time::sleep(Duration::from_millis(600)).await;
        }
    });
}
