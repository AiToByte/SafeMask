use crate::common::state::AppState;
use arboard::Clipboard;
use enigo::{Enigo, Key, KeyboardControllable};
use std::sync::atomic::Ordering;
use std::time::Duration;
use tauri::{AppHandle, Manager, Emitter};
use log::{info};

pub struct MagicPaster;

impl MagicPaster {
    pub async fn execute(app: &AppHandle) {
        let state = app.state::<AppState>();

        // 🚀 核心修复：如果前端正在录制快捷键，后端闭嘴，不准模拟按键
        if state.is_recording_mode.load(Ordering::SeqCst) { 
            info!("[MagicPaste] 检测到录制模式，取消模拟执行");
            return;
        }

        let settings = state.settings.read().clone();
        let shadow = state.shadow_store.read().clone();

        let (target_text, feedback_type) = if settings.shadow_mode_enabled {
            if shadow.has_privacy { (shadow.masked.clone(), "PASTE_MASKED") }
            else { (shadow.original.clone(), "NORMAL") }
        } else {
            if shadow.has_privacy { (shadow.original.clone(), "PASTE_ORIGINAL") } 
            else { (shadow.original.clone(), "NORMAL") }
        };

        if target_text.is_empty() { return; }

        state.is_magic_pasting.store(true, Ordering::SeqCst);

        // 🚀 核心改动：增加执行序列的健壮性
        if let Ok(_) = Self::perform_swap_sequence(target_text, settings.paste_delay_ms).await {
             let _ = app.emit("magic-feedback", serde_json::json!({ "type": feedback_type }));
        }

        tokio::time::sleep(Duration::from_millis(50)).await;
        state.is_magic_pasting.store(false, Ordering::SeqCst);
    }

    async fn perform_swap_sequence(target_text: String, delay: u64) -> Result<(), Box<dyn std::error::Error>> {
        let mut cb = Clipboard::new()?;
        let backup = cb.get_text().unwrap_or_default();

        // 1. 覆盖剪贴板
        cb.set_text(target_text)?;

        // 2. 模拟按键（优化版）
        Self::simulate_paste_keys();

        // 3. 🚀 关键：给目标应用留出足够的读取时间
        // 如果 150ms 还是不行，建议在 UI 设置中调大到 300ms
        tokio::time::sleep(Duration::from_millis(delay)).await;

        // 4. 还原剪贴板
        cb.set_text(backup)?;
        
        Ok(())
    }

    fn simulate_paste_keys() {
        let mut enigo = Enigo::new();

        // 🚀 核心修复逻辑：
        // 因为用户触发快捷键时按住了物理 ALT 键，
        // 我们必须先在软件层面发送一个 ALT 松开的信号，
        // 否则系统会认为用户在按 Ctrl + Alt + V。

        #[cfg(target_os = "windows")]
        {
            enigo.key_up(Key::Alt); // 强制松开物理 Alt
            let _ = tokio::time::sleep(Duration::from_millis(20)); // 微小停顿
        }

        #[cfg(target_os = "macos")]
        let modifier = Key::Meta;
        #[cfg(not(target_os = "macos"))]
        let modifier = Key::Control;

        // 执行粘贴组合键
        enigo.key_down(modifier);
        let _ = tokio::time::sleep(Duration::from_millis(20)); // 模拟真人按下的间隔
        enigo.key_click(Key::Layout('v'));
        let _ = tokio::time::sleep(Duration::from_millis(20));
        enigo.key_up(modifier);
        
        // 如果是 Windows，粘贴完后再把 Alt 补回来（可选，避免影响用户继续操作）
        // #[cfg(target_os = "windows")]xiaosheng
        // enigo.key_down(Key::Alt); 
    }
}