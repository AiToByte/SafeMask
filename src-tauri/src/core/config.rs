use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppSettings {
    // --- 快捷键配置 ---
    /// 影子模式安全粘贴快捷键 (例如: "Alt+V")
    pub magic_paste_shortcut: String,
    
    // --- 引擎行为 ---
    /// 是否开启影子模式（捕获但不自动修改剪贴板）
    pub shadow_mode_enabled: bool,
    /// 模拟粘贴后的还原延迟（毫秒），建议 100-300ms
    pub paste_delay_ms: u64,

    // --- 交互反馈 ---
    /// 是否开启粘贴后的视觉气泡提示
    pub enable_visual_feedback: bool,
    /// 是否开启粘贴后的听觉音效提示
    pub enable_audio_feedback: bool,
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            magic_paste_shortcut: "Alt+V".to_string(),
            shadow_mode_enabled: true,
            paste_delay_ms: 150,
            enable_visual_feedback: true,
            enable_audio_feedback: true,
        }
    }
}