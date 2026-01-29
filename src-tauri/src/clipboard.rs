use clipboard_master::{ClipboardHandler, CallbackResult};
use arboard::{Clipboard, Error as ArboardError}; // 重命名以防冲突
use std::sync::atomic::Ordering;
// 🚀 导入 AppState 和 MaskHistoryItem
use crate::state::{AppState, MaskHistoryItem}; 
// 🚀 必须导入 Manager 才能使用 .state() 方法
use tauri::{AppHandle, Emitter, Manager}; 

// 🚀 必须导入这两个 Trait/Struct
use chrono::Local;
use uuid::Uuid;

/// 剪贴板处理器上下文
pub struct GlobalClipboardHandler {
    pub app_handle: AppHandle,
    // 🚀 核心优化：长连接上下文
    // 我们将 Clipboard 实例保存在结构体中。由于监听线程是持久的，
    // 这样避免了每秒钟数十次创建实例带来的 COM 初始化开销。
    clipboard: Option<Clipboard>,
}

impl GlobalClipboardHandler {
    pub fn new(app_handle: AppHandle) -> Self {
        Self {
            app_handle,
            // 初始时延迟加载，或者直接在这里初始化
            clipboard: Clipboard::new().ok(),
        }
    }

    /// 内部安全读取函数，带重试机制
    fn get_text_safe(&mut self) -> Option<String> {
        let cb = self.clipboard.as_mut()?;
        
        // 🚀 优化：如果剪贴板忙（被其他程序占用），进行 3 轮快速重试
        for i in 0..3 {
            match cb.get_text() {
                Ok(t) => return Some(t),
                // 如果剪贴板被占用，等待一会再试
                Err(ArboardError::ClipboardOccupied) | Err(ArboardError::ConversionFailure) => {
                    std::thread::sleep(std::time::Duration::from_millis(50 * (i + 1)));
                }
                _ => break,
            }
        }
        None
    }
}


impl ClipboardHandler for GlobalClipboardHandler {
    /// 当系统剪贴板内容发生变化时，OS 会回调此方法
    fn on_clipboard_change(&mut self) -> CallbackResult {
         // 🚀 优化 1: 使用局部作用域提前释放对 self 的不可变借用
        // 这样在调用 self.get_text_safe() 时就不会有借用冲突
        let (is_changing, is_monitor_enabled) = {
            let state = self.app_handle.state::<AppState>();
            (
                state.is_internal_changing.clone(), 
                state.is_monitor_on.clone()
            )
        };

         // 判定是否需要拦截
        if is_changing.load(Ordering::SeqCst) || !*is_monitor_enabled.lock().unwrap() {
            return CallbackResult::Next;
        }

        // 2. 确保剪贴板实例健康
        if self.clipboard.is_none() {
            self.clipboard = Clipboard::new().ok();
        }

        // 🚀 优化 2: 调用可变借用方法读取内容
        if let Some(current_text) = self.get_text_safe() {
            if current_text.trim().is_empty() || current_text.len() > 1024 * 1024 {
                return CallbackResult::Next;
            }

            // 检查内容缓存，防止处理重复内容
            let is_duplicate = {
                let state = self.app_handle.state::<AppState>();
                let last = state.last_content.lock().unwrap();
                current_text == *last
            };
            
            if is_duplicate { return CallbackResult::Next; }

            // 执行脱敏逻辑
            let masked_text = {
                let state = self.app_handle.state::<AppState>();
                let engine = state.engine.read().unwrap();
                let masked_bytes = engine.mask_line(current_text.as_bytes());
                String::from_utf8_lossy(&masked_bytes).into_owned()
            };

            // 如果内容发生了改变
            if masked_text != current_text {
                // 开启内部写回锁
                is_changing.store(true, Ordering::SeqCst);
                
                // 更新最后一次内容缓存
                {
                    let state = self.app_handle.state::<AppState>();
                    let mut last = state.last_content.lock().unwrap();
                    *last = masked_text.clone();
                }

                // 写入剪贴板
                if let Some(cb) = self.clipboard.as_mut() {
                    if let Ok(_) = cb.set_text(masked_text.clone()) {
                        // 构建历史记录
                        let history_item = MaskHistoryItem {
                            id: Uuid::new_v4().to_string(),
                            timestamp: Local::now().format("%H:%M:%S").to_string(),
                            original: current_text,
                            masked: masked_text,
                        };
                        
                        // 存入 State 并发射事件给前端
                        let state = self.app_handle.state::<AppState>();
                        let mut history = state.history.lock().unwrap();
                        history.insert(0, history_item.clone());
                        if history.len() > 50 { history.pop(); }
                        let _ = self.app_handle.emit("new-history", history_item);
                        let _ = self.app_handle.emit("masked-event", "🛡️ 隐私内容已自动脱敏");
                    }
                }

                // 🚀 优化 3: 解决线程逃逸问题
                // 不要克隆 State，而是克隆 AppHandle，线程内部重新获取状态
                let handle_clone = self.app_handle.clone();
                std::thread::spawn(move || {
                    std::thread::sleep(std::time::Duration::from_millis(300));
                    // 线程内部安全获取状态，AppHandle 是 'static 的
                    let state = handle_clone.state::<AppState>();
                    state.is_internal_changing.store(false, Ordering::SeqCst);
                });
            }
        }

        CallbackResult::Next
    }

    fn on_clipboard_error(&mut self, _error: std::io::Error) -> CallbackResult {
        // 遇到严重错误（如远程桌面断开导致的 COM 失效）时重置上下文
        self.clipboard = None;
        CallbackResult::Next
    }
}