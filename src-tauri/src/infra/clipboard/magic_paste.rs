use crate::common::state::AppState;
use arboard::{Clipboard, ImageData};
use enigo::{Enigo, Key, KeyboardControllable};
use log::{info, warn};
use std::cell::RefCell;
use std::sync::atomic::Ordering;
use std::time::Duration;
use tauri::{AppHandle, Emitter, Manager};

thread_local! {
    static ENIGO: RefCell<Enigo> = RefCell::new(Enigo::new());
}

/// 剪贴板备份内容：文本 / 图片 / 无法恢复
enum ClipboardBackup {
    Text(String),
    Image(ImageData<'static>),
    /// 剪贴板既不是文本也不是图片（如文件列表、私有格式），无法安全备份。
    /// 还原阶段将跳过写入，避免用空字符串覆盖用户原有内容。
    Unrecoverable,
}

pub struct MagicPaster;

impl MagicPaster {
    pub async fn execute(app: &AppHandle) {
        let state = app.state::<AppState>();

        if state.is_recording_mode.load(Ordering::SeqCst) {
            info!("[MagicPaste] 检测到录制模式，取消模拟执行");
            return;
        }

        let settings = state.settings.read().clone();
        let shadow = state.shadow_store.read().clone();

        let (target_text, feedback_type) = if settings.shadow_mode_enabled {
            if shadow.has_privacy {
                (shadow.masked.clone(), "PASTE_MASKED")
            } else {
                (shadow.original.clone(), "NORMAL")
            }
        } else if shadow.has_privacy {
            (shadow.original.clone(), "PASTE_ORIGINAL")
        } else {
            (shadow.original.clone(), "NORMAL")
        };

        if target_text.is_empty() {
            return;
        }

        state.is_magic_pasting.store(true, Ordering::SeqCst);

        if Self::perform_swap_sequence(target_text, settings.paste_delay_ms)
            .await
            .is_ok()
        {
            let _ = app.emit("magic-feedback", serde_json::json!({ "type": feedback_type }));
        }

        tokio::time::sleep(Duration::from_millis(50)).await;
        state.is_magic_pasting.store(false, Ordering::SeqCst);
    }

    async fn perform_swap_sequence(
        target_text: String,
        delay: u64,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let mut cb = Clipboard::new()?;

        // 1. 备份原剪贴板（探测文本/图片，都失败则标记不可恢复而非覆盖成空字符串）
        let backup = Self::snapshot_clipboard(&mut cb);

        // 2. 写入脱敏/原文文本
        cb.set_text(target_text)?;

        // 3. 模拟按键触发目标应用粘贴
        Self::simulate_paste_keys();

        // 4. 留出目标应用读取时间
        tokio::time::sleep(Duration::from_millis(delay)).await;

        // 5. 按备份类型还原
        match backup {
            ClipboardBackup::Text(text) => {
                cb.set_text(text)?;
            }
            ClipboardBackup::Image(image) => {
                if let Err(e) = cb.set_image(image) {
                    warn!("[MagicPaste] 还原图片剪贴板失败: {}", e);
                }
            }
            ClipboardBackup::Unrecoverable => {
                // 原剪贴板不是文本也不是图片（可能是文件/HTML/自定义格式）。
                // 不做还原：脱敏文本会作为剪贴板残留，但不会用空字符串销毁用户原始内容。
                warn!("[MagicPaste] 原剪贴板内容格式不支持备份，跳过还原");
            }
        }

        Ok(())
    }

    /// 探测并快照当前剪贴板：优先文本，其次图片
    fn snapshot_clipboard(cb: &mut Clipboard) -> ClipboardBackup {
        match cb.get_text() {
            Ok(text) => ClipboardBackup::Text(text),
            Err(_) => match cb.get_image() {
                Ok(image) => ClipboardBackup::Image(image),
                Err(_) => ClipboardBackup::Unrecoverable,
            },
        }
    }

    fn simulate_paste_keys() {
        ENIGO.with(|cell| {
            let mut enigo = cell.borrow_mut();

            #[cfg(target_os = "windows")]
            {
                // 用户触发快捷键时物理 Alt 仍按下；先在软件层松开
                // 避免系统识别为 Ctrl+Alt+V。
                enigo.key_up(Key::Alt);
                std::thread::sleep(Duration::from_millis(20));
            }

            #[cfg(target_os = "macos")]
            let modifier = Key::Meta;
            #[cfg(not(target_os = "macos"))]
            let modifier = Key::Control;

            enigo.key_down(modifier);
            std::thread::sleep(Duration::from_millis(20));
            enigo.key_click(Key::Layout('v'));
            std::thread::sleep(Duration::from_millis(20));
            enigo.key_up(modifier);
        });
    }
}
