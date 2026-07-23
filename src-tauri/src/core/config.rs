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

    // --- 外观主题 ---
    /// UI 主题标识符，例如 "default" | "claude"。
    ///
    /// 后端不校验合法性 —— 前端 `normalizeThemeId` 会将非法值统一回退到 "default"。
    /// 保持宽松是为了向前兼容：如果未来新增了主题但用户降级到旧版，
    /// 配置文件里的新主题字符串会被前端安全忽略，而不会导致启动失败。
    #[serde(default = "default_theme")]
    pub theme: String,
}

fn default_theme() -> String {
    "default".to_string()
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
            theme: default_theme(),
        }
    }
}

// ── Tests ────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    /// 默认 AppSettings 的 theme 字段应为 "default"，与前端 DEFAULT_THEME_ID 保持一致。
    #[test]
    fn default_settings_use_default_theme() {
        let settings = AppSettings::default();
        assert_eq!(settings.theme, "default");
    }

    /// YAML 中缺失 `theme` 字段时，反序列化应回退到默认值 "default"。
    /// 保证从旧版本升级的用户不会因为字段缺失而启动失败。
    #[test]
    fn missing_theme_field_falls_back_to_default() {
        let yaml = r#"
magic_paste_shortcut: "Alt+V"
shadow_mode_enabled: true
paste_delay_ms: 150
enable_visual_feedback: true
enable_audio_feedback: true
record_writer_enabled: false
mask_wrapper_style: "angle"
"#;
        let settings: AppSettings = serde_yaml::from_str(yaml).unwrap();
        assert_eq!(settings.theme, "default");
    }

    /// YAML 中显式设置的 theme 值应无损保留（不做后端合法性校验）。
    /// 前端 normalizeThemeId 负责在渲染前把非法值收敛到默认主题。
    #[test]
    fn explicit_theme_field_is_preserved() {
        let yaml = r#"
magic_paste_shortcut: "Alt+V"
shadow_mode_enabled: true
paste_delay_ms: 150
enable_visual_feedback: true
enable_audio_feedback: true
record_writer_enabled: false
mask_wrapper_style: "angle"
theme: "claude"
"#;
        let settings: AppSettings = serde_yaml::from_str(yaml).unwrap();
        assert_eq!(settings.theme, "claude");
    }

    /// 序列化 → 反序列化应保持 theme 字段无损往返。
    #[test]
    fn theme_field_survives_serde_roundtrip() {
        let mut settings = AppSettings::default();
        settings.theme = "claude".to_string();
        let yaml = serde_yaml::to_string(&settings).unwrap();
        let restored: AppSettings = serde_yaml::from_str(&yaml).unwrap();
        assert_eq!(restored.theme, "claude");
    }
}