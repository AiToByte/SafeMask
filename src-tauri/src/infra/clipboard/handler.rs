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

        // 1. 卫语句：如果是内部模拟粘贴引起的变更，直接跳过
        if state.is_magic_pasting.load(Ordering::Acquire) { return; }

        // 2. 环境感知
        let settings = state.settings.read().clone();
        if !*state.is_monitor_on.lock() { return; }

        // 3. 读取内容
        let text = match self.get_text() {
            Ok(t) => t,
            Err(_) => return,
        };

        // 4. 去重校验
        {
            let mut last = state.last_content.lock();
            if text == *last { return; }
            *last = text.clone();
        }

        // 5. 执行脱敏计算
        let masked_text = {
            let engine = state.engine.read();
            String::from_utf8_lossy(&engine.mask_line(text.as_bytes())).to_string()
        };

        let has_privacy = text != masked_text;

        // 🚀 核心改动：只要发现隐私，无论什么模式，立即记录历史
        if has_privacy {
            self.record_privacy_history(text.clone(), masked_text.clone()).await;
        }

        // 6. 模式分流
        if settings.shadow_mode_enabled {
            // --- 影子模式 ---
            state.update_shadow(text, masked_text);
            if has_privacy {
                // 通知前端：影子就绪
                let _ = self.app.emit("shadow-status", "READY");
            }
        } else {
            // --- 哨兵模式 ---
            if has_privacy {
                // 执行拦截写回
                self.execute_sentry_intercept(text, masked_text).await;
            }
        }
    }

    /// [新增] 统一的历史记录写入方法
    async fn record_privacy_history(&self, original: String, masked: String) {
        let state = self.app.state::<AppState>();
        
        // 获取当前模式快照
        let mode = if state.settings.read().shadow_mode_enabled {
            "SHADOW".to_string()
        } else {
            "SENTRY".to_string()
        };

        let history_item = MaskHistoryItem {
            id: Uuid::new_v4().to_string(),
            timestamp: Local::now().format("%H:%M:%S").to_string(),
            original,
            masked,
            mode, // 🚀 记录该条记录产生时的模式
        };

        state.add_history(history_item.clone());
        let _ = self.app.emit("new-history", history_item);
    }

    /// 哨兵拦截执行器 (保持精简，不再负责历史记录)
    async fn execute_sentry_intercept(&self, _original: String, masked: String) {
        let state = self.app.state::<AppState>();
        state.is_magic_pasting.store(true, Ordering::Release);
        
        if let Ok(_) = self.set_text(masked.clone()) {
            {
                let mut last = state.last_content.lock();
                *last = masked;
            }
            let _ = self.app.emit("magic-feedback", serde_json::json!({ "type": "AUTO_MASK_SUCCESS" }));
        }

        tokio::time::sleep(std::time::Duration::from_millis(50)).await;
        state.is_magic_pasting.store(false, Ordering::Release);
    }
}