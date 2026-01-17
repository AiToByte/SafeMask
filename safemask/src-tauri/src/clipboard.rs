use clipboard_master::{ClipboardHandler, CallbackResult};
use arboard::Clipboard;
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

                // 4. 防循环校验
                if current_text != *last && !current_text.is_empty() {
                    let masked_bytes = self.engine.mask_line(current_text.as_bytes());
                    let masked_text = String::from_utf8_lossy(&masked_bytes).into_owned();

                    if masked_text != current_text {
                        *last = masked_text.clone();
                        // 尝试写回脱敏后的文本
                        if let Ok(_) = ctx.set_text(masked_text) {
                            let _ = self.app_handle.emit("masked-event", "🛡️ 隐私内容已自动脱敏");
                        }
                    }
                }
            },
            // 5. 重点：处理非文本错误（图片、文件列表等）
            Err(e) => {
                match e {
                    ClipboardError::ContentNotAvailable => {
                        // 这种情况通常是用户复制了图片、文件或二进制数据
                        // 我们保持沉默，不做任何处理，直接跳过
                    },
                    _ => {
                        // 记录其他可能的系统级错误
                        eprintln!("剪贴板访问异常: {:?}", e);
                    }
                }
            }
        }
        CallbackResult::Next
    }

    fn on_clipboard_error(&mut self, error: std::io::Error) -> CallbackResult {
        // 这里的错误通常是底层的系统信号异常
        eprintln!("OS 剪贴板事件流中断: {}", error);
        CallbackResult::Next
    }
}