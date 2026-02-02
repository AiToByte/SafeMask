use crate::common::state::{AppState, MaskHistoryItem};
use crate::common::events::AppEvents;
use arboard::Clipboard;
use std::sync::Arc;
use std::sync::atomic::Ordering;
use tauri::{AppHandle, Manager, Emitter};
use chrono::Local;
use uuid::Uuid;
use log::{info, error};

pub struct GlobalClipboard {
    app: AppHandle,
    backend: Arc<parking_lot::Mutex<Clipboard>>,
}

impl GlobalClipboard {
    pub fn new(app: AppHandle) -> Self {
        let cb = Clipboard::new().expect("无法初始化剪贴板后端");
        Self {
            app,
            backend: Arc::new(parking_lot::Mutex::new(cb)),
        }
    }

    // 公开方法：获取当前剪贴板文本
    pub fn get_text(&self) -> Result<String, arboard::Error> {
        self.backend.lock().get_text()
    }

    // 公开方法：设置剪贴板文本
    pub fn set_text(&self, text: String) -> Result<(), arboard::Error> {
        self.backend.lock().set_text(text)
    }

    pub async fn process_change(&self) {
        let state = self.app.state::<AppState>();

        // 1. 检查开关和内部标记
        if !*state.is_monitor_on.lock() || state.is_internal_changing.load(Ordering::Acquire) {
            return;
        }

        // 2. 读取内容（使用公开方法）
        let text = match self.get_text() {
            Ok(t) => {
                info!("[Clipboard] 读取成功，长度: {}", t.len());
                t
            }
            Err(e) => {
                error!("[Clipboard] 读取失败: {}", e);
                return;
            }
        };

        if text.trim().is_empty() || text.len() > 2 * 1024 * 1024 { return; } // 忽略过大内容

        // 3. 执行脱敏
        let (masked_text, has_changed) = {
            // 🚀 直接调用 .read() 即可，不需要 .expect()
            let engine_guard = state.engine.read(); 
            
            // engine_guard 此时是 Arc<MaskEngine> 的守卫
            let result = engine_guard.mask_line(text.as_bytes());
            
            let masked = String::from_utf8_lossy(&result).to_string();
            let changed = masked != text;
            (masked, changed)
        };

        if !has_changed {
            info!("[Clipboard] 内容无敏感信息，无需替换");
            return;
        }

        // 4. 写回并记录历史
        state.is_internal_changing.store(true, Ordering::Release);
        
        if let Err(e) = self.set_text(masked_text.clone()) {
            error!("[Clipboard] 写回失败: {}", e);
            state.is_internal_changing.store(false, Ordering::Release);
            return;
        }

        let history_item = MaskHistoryItem {
            id: Uuid::new_v4().to_string(),
            timestamp: Local::now().format("%H:%M:%S").to_string(),
            original: text,
            masked: masked_text.clone(),
        };

        state.add_history(history_item.clone());
        let _ = self.app.emit(AppEvents::NEW_HISTORY, history_item);
        let _ = self.app.emit(AppEvents::MASKED_EVENT, "🛡️ 隐私信息已自动脱敏");

        // 延迟重置标志，防止循环触发
        let is_changing = state.is_internal_changing.clone();
        tauri::async_runtime::spawn(async move {
            tokio::time::sleep(std::time::Duration::from_millis(300)).await;
            is_changing.store(false, Ordering::Release);
        });

        info!("[Clipboard] 脱敏完成，已写回");
    }
}