use crate::common::state::{AppState, MaskHistoryItem};
use crate::common::events::AppEvents;
use arboard::Clipboard;
use std::sync::Arc;
use std::sync::atomic::Ordering;
use tauri::{AppHandle, Manager, Emitter};
use chrono::Local;
use uuid::Uuid;
use log::{info, warn, error};

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
        if state.is_magic_pasting.load(Ordering::Acquire) { 
            info!("检测到内部模拟粘贴引起的剪贴板变更，已自动忽略");
            return; 
        }

        // 2. 环境感知
        let settings = state.settings.read().clone();
        if !*state.is_monitor_on.lock() {
            info!("剪贴板监控当前处于关闭状态，已自动忽略变更");
            return; 
        }

        // 3. 读取内容
        let text = match self.get_text() {
            Ok(t) => t,
            Err(e) => {
                warn!("[Clipboard] 无法读取内容: {}", e);
                return;
            }
        };

        if text.is_empty() { return; }

        // 4. 🚀 关键：去重并更新缓存
        {
            let mut last = state.last_content.lock();
            if text == *last { return; }
            *last = text.clone(); // 在这里完成更新，确保 monitor 不再重复触发
        }
        info!("🔔 [Handler] 开始处理新内容 ({} bytes)", text.len());


        // 5. 执行脱敏计算
        let masked_text = {
            let engine = state.engine.read();
            let result = engine.mask_line(text.as_bytes());
            String::from_utf8_lossy(&result).to_string()
        };

        let has_privacy = text != masked_text;
        info!("[Monitor] 脱敏计算完成，命中隐私: {}", has_privacy);

        // 6. 🚀 关键修复：无论是否命中隐私，都更新影子存储
        // 这确保了 Alt+V 永远有内容可以粘贴
        state.update_shadow(text.clone(), masked_text.clone());
        info!("[Handler] 影子宇宙同步完成。是否有隐私: {}", has_privacy);

        // 7. 🚀 核心改动：只要发现隐私，无论什么模式，立即记录历史
        if has_privacy {
            info!("[Handler] 检测到隐私内容，已记录历史");
            self.record_privacy_history(text.clone(), masked_text.clone()).await;
        }

         // 8. 模式分流
        if !settings.shadow_mode_enabled && has_privacy {
            // 哨兵模式下且有隐私：自动执行写回
            info!("[Handler] 哨兵宇宙：自动洗白剪贴板...");
            self.execute_sentry_intercept(text, masked_text).await;
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
        info!("[Handler] 审计记录已入库");
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

        tokio::time::sleep(std::time::Duration::from_millis(100)).await;
        state.is_magic_pasting.store(false, Ordering::Release);
    }
}