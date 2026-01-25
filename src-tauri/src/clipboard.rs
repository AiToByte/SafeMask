use clipboard_master::{ClipboardHandler, CallbackResult};
use arboard::{Clipboard, Error as ArboardError}; // 重命名以防冲突
use std::sync::{Arc, Mutex};
use crate::engine::MaskEngine;
use tauri::AppHandle;
use tauri::Emitter; // Tauri v2 

pub struct GlobalClipboardHandler {
    pub app_handle: AppHandle,
    pub engine: Arc<MaskEngine>,
    pub last_content: Arc<Mutex<String>>,
    pub is_enabled: Arc<Mutex<bool>>,
}

impl ClipboardHandler for GlobalClipboardHandler {
    /// 当系统剪贴板内容发生变化时，OS 会回调此方法
    fn on_clipboard_change(&mut self) -> CallbackResult {
        // 1. 检查自动监控开关是否开启
        if !*self.is_enabled.lock().unwrap() {
            return CallbackResult::Next;
        }

        let mut ctx = match Clipboard::new() {
            Ok(c) => c,
            Err(_) => return CallbackResult::Next,
        };
        
        // 2. 尝试获取文本。如果当前剪贴板是图片、文件或空，get_text() 会返回 Err
        match ctx.get_text() {
            Ok(current_text) => {
                // 3. 性能优化：如果文本超级巨大（例如超过 2MB），建议跳过自动脱敏，防止 UI 卡死
                // 这种大数据建议引导用户使用“文件模式”
                if current_text.len() > 2 * 1024 * 1024 {
                    return CallbackResult::Next;
                }

                let mut last = self.last_content.lock().unwrap();

                // 4. 关键：防震荡机制（防止脱敏写回操作再次触发变动事件）
                if current_text != *last && !current_text.is_empty() {
                    let masked_bytes = self.engine.mask_line(current_text.as_bytes());
                    let masked_text = String::from_utf8_lossy(&masked_bytes).into_owned();
                    // 5. 只有内容真正发生脱敏替换时才执行操作
                    if masked_text != current_text {
                        *last = masked_text.clone();
                        // 尝试写回脱敏后的文本
                        if let Ok(_) = ctx.set_text(masked_text) {
                            // 🚀 生成历史记录
                            let history_item = MaskHistoryItem {
                                id: Uuid::new_v4().to_string(),
                                timestamp: Local::now().format("%H:%M:%S").to_string(),
                                original: current_text.clone(),
                                masked: masked_text,
                            };

                            // 更新状态中的历史记录
                            let state = self.app_handle.state::<AppState>();
                            let mut history = state.history.lock().unwrap();
                            history.insert(0, history_item.clone());
                            if history.len() > 50 { history.pop(); } // 保持容量

                            // 通知前端有新历史和 Toast
                            let _ = self.app_handle.emit("new-history", history_item);
                            let _ = self.app_handle.emit("masked-event", "🛡️ 隐私内容已自动脱敏");
                        }
                    }
                }
            },
            // 修复：使用正确的 Arboard 错误类型
            Err(ArboardError::ContentNotAvailable) => {},
            Err(e) => eprintln!("剪贴板处理异常: {:?}", e),
        }
        CallbackResult::Next
    }

    fn on_clipboard_error(&mut self, error: std::io::Error) -> CallbackResult {
        eprintln!("剪贴板监听流异常: {}", error);
        CallbackResult::Next
    }
}