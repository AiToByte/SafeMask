use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum MaskWrapperStyle {
    Angle = 0,
    Square = 1,
}

impl MaskWrapperStyle {
    pub fn wrap(&self, content: &str) -> String {
        match self {
            Self::Angle => format!("<{}>", content),
            Self::Square => format!("[{}]", content),
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Angle => "angle",
            Self::Square => "square",
        }
    }

    pub fn from_str(s: &str) -> Self {
        match s {
            "square" => Self::Square,
            _ => Self::Angle,
        }
    }

    /// 解包：如果字符串是 <...> 或 [...]，返回 Some(裸内容)
    pub fn try_unwrap(s: &str) -> Option<&str> {
        if s.len() > 2 {
            let b = s.as_bytes();
            match (b[0], b[b.len() - 1]) {
                (b'<', b'>') | (b'[', b']') => Some(&s[1..s.len() - 1]),
                _ => None,
            }
        } else {
            None
        }
    }
}

fn default_model_urls() -> Vec<String> {
    vec![
        "https://obs.behource.com:9004/gxzh/2026/07/06/privacy-filter.zip"
            .to_string(),
        "https://950544b1401caf10f82ba1e82b03f89a.r2.cloudflarestorage.com/safemask-ai-model/privacy-filter/privacy-filter.zip"
            .to_string(),
        format!("https://github.com/AiToByte/SafeMask/releases/download/v{}/privacy-filter.zip", env!("CARGO_PKG_VERSION")),
    ]
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
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

    // --- AI 模型下载 ---
    /// 模型下载 URL 列表（优先级顺序，首个可用即使用）
    /// 仅跳过序列化（不写入 YAML）；反序列化时若字段不存在则使用 `default_model_urls`
    #[serde(skip_serializing, default = "default_model_urls")]
    pub model_download_urls: Vec<String>,

    // --- 记录写入器 ---
    /// 是否启用脱敏记录持久化（写入 .md 文件）
    pub record_writer_enabled: bool,

    // --- 脱敏标签格式 ---
    /// 全局脱敏标签包裹样式: "angle" (尖括号) 或 "square" (方括号)
    pub mask_wrapper_style: String,
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            magic_paste_shortcut: "Alt+V".to_string(),
            shadow_mode_enabled: true,
            paste_delay_ms: 150,
            enable_visual_feedback: true,
            enable_audio_feedback: true,
            model_download_urls: vec![
                "https://obs.behource.com:9004/gxzh/2026/07/06/privacy-filter.zip"
                    .to_string(),
                "https://950544b1401caf10f82ba1e82b03f89a.r2.cloudflarestorage.com/safemask-ai-model/privacy-filter/privacy-filter.zip"
                    .to_string(),
                format!("https://github.com/AiToByte/SafeMask/releases/download/v{}/privacy-filter.zip", env!("CARGO_PKG_VERSION")),
            ],
            record_writer_enabled: false,
            mask_wrapper_style: "angle".to_string(),
        }
    }
}