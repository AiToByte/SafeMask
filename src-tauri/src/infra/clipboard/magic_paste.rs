use crate::common::state::AppState;
use arboard::Clipboard;
use enigo::{Enigo, Key, KeyboardControllable};
use std::sync::atomic::Ordering;
use std::time::Duration;
use tauri::{AppHandle, Manager, Emitter};

pub struct MagicPaster;

impl MagicPaster {
    pub async fn execute(app: &AppHandle) {
        let state = app.state::<AppState>();
        let settings = state.settings.read().clone();
        let shadow = state.shadow_store.read().clone();

        // 模式判定
        let target_text = if settings.shadow_mode_enabled {
            if !shadow.has_privacy { return; }
            shadow.masked.clone()
        } else {
            if !shadow.has_privacy { return; }
            shadow.original.clone() 
        };

        state.is_magic_pasting.store(true, Ordering::SeqCst);

        // 🚀 执行交换序列
        if let Ok(_) = Self::perform_swap_sequence(target_text, settings.paste_delay_ms).await {
             let feedback_type = if settings.shadow_mode_enabled { "PASTE_MASKED" } else { "PASTE_ORIGINAL" };
             let _ = app.emit("magic-feedback", serde_json::json!({ "type": feedback_type }));
        }

        tokio::time::sleep(Duration::from_millis(100)).await;
        state.is_magic_pasting.store(false, Ordering::SeqCst);
    }

    // 🚀 核心修复：添加缺失的交换序列方法
    async fn perform_swap_sequence(target_text: String, delay: u64) -> Result<(), Box<dyn std::error::Error>> {
        let mut cb = Clipboard::new()?;
        
        // 1. 备份当前内容
        let backup = cb.get_text().unwrap_or_default();
        
        // 2. 写入目标内容
        cb.set_text(target_text)?;
        
        // 3. 模拟按键
        Self::simulate_paste_keys();
        
        // 4. 等待应用读取
        tokio::time::sleep(Duration::from_millis(delay)).await;
        
        // 5. 还原内容
        cb.set_text(backup)?;
        
        Ok(())
    }

    // 🚀 核心修复：添加按键模拟方法
    fn simulate_paste_keys() {
        let mut enigo = Enigo::new();
        #[cfg(target_os = "macos")]
        let modifier = Key::Meta;
        #[cfg(not(target_os = "macos"))]
        let modifier = Key::Control;

        enigo.key_down(modifier);
        enigo.key_click(Key::Layout('v'));
        enigo.key_up(modifier);
    }
}