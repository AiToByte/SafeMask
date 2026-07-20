//! 场景模式定义
//!
//! 定义 SafeMask 的三种工作模式：
//! - Shadow (影子模式): 复制不脱敏，粘贴时脱敏
//! - Sentry (哨兵模式): 复制即脱敏

use serde::{Deserialize, Serialize};

/// 场景模式
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SceneMode {
    /// 影子模式
    ///
    /// - 复制: 剪贴板保留原始文本
    /// - 粘贴: Alt+V 时脱敏粘贴
    /// - 适用: 日常开发、调试
    Shadow,

    /// 哨兵模式
    ///
    /// - 复制: 剪贴板立即脱敏
    /// - 适用: 远程会议、屏幕共享、公共场所
    Sentry,
}

impl SceneMode {
    /// 获取模式名称
    pub fn name(&self) -> &str {
        match self {
            Self::Shadow => "SHADOW",
            Self::Sentry => "SENTRY",
        }
    }

    /// 获取模式的中文名称
    pub fn display_name(&self) -> &str {
        match self {
            Self::Shadow => "影子模式",
            Self::Sentry => "哨兵模式",
        }
    }

    /// 获取模式的描述
    pub fn description(&self) -> &str {
        match self {
            Self::Shadow => "复制不脱敏，粘贴时脱敏",
            Self::Sentry => "复制即脱敏",
        }
    }

    /// 切换到另一个模式
    pub fn toggle(&self) -> Self {
        match self {
            Self::Shadow => Self::Sentry,
            Self::Sentry => Self::Shadow,
        }
    }

    /// 从字符串解析（宽容解析，未知值默认落到 `Shadow`）
    ///
    /// 语义与标准 `FromStr` 不同：不返回 `Result`，未知输入使用默认值。
    /// 因此显式抑制 `should_implement_trait` 警告。
    #[allow(clippy::should_implement_trait)]
    pub fn from_str(s: &str) -> Self {
        match s.to_uppercase().as_str() {
            "SENTRY" => Self::Sentry,
            _ => Self::Shadow,
        }
    }
}

impl Default for SceneMode {
    fn default() -> Self {
        Self::Shadow
    }
}

/// 场景配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SceneConfig {
    /// 是否启用 AI 引擎
    pub enable_ai: bool,
    /// 置信度阈值
    pub confidence_threshold: f32,
    /// 是否启用上下文增强
    pub enable_context_enhancement: bool,
    /// 是否启用校验位验证
    pub enable_checksum_validation: bool,
}

impl Default for SceneConfig {
    fn default() -> Self {
        Self {
            enable_ai: false,
            confidence_threshold: 0.5,
            enable_context_enhancement: true,
            enable_checksum_validation: true,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_scene_mode_toggle() {
        assert_eq!(SceneMode::Shadow.toggle(), SceneMode::Sentry);
        assert_eq!(SceneMode::Sentry.toggle(), SceneMode::Shadow);
    }

    #[test]
    fn test_scene_mode_from_str() {
        assert_eq!(SceneMode::from_str("SHADOW"), SceneMode::Shadow);
        assert_eq!(SceneMode::from_str("SENTRY"), SceneMode::Sentry);
        assert_eq!(SceneMode::from_str("shadow"), SceneMode::Shadow);
        assert_eq!(SceneMode::from_str("unknown"), SceneMode::Shadow);
    }

    #[test]
    fn test_scene_mode_display() {
        assert_eq!(SceneMode::Shadow.name(), "SHADOW");
        assert_eq!(SceneMode::Sentry.name(), "SENTRY");
        assert_eq!(SceneMode::Shadow.display_name(), "影子模式");
        assert_eq!(SceneMode::Sentry.display_name(), "哨兵模式");
    }
}
