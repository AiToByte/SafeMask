//! 脱敏策略层
//!
//! 定义脱敏策略的统一接口和多种实现。
//! 脱敏策略负责将识别到的实体转换为脱敏后的文本。
//!
//! # 设计原则
//!
//! - **策略模式**: 每种脱敏方式都是独立的策略实现
//! - **可配置**: 用户可以为不同实体类型选择不同的脱敏策略
//! - **可扩展**: 新增脱敏策略只需实现 `MaskingStrategy` trait
//!
//! # 内置策略
//!
//! | 策略 | 输入 | 输出 | 场景 |
//! |------|------|------|------|
//! | Replace | 张三 | [人名] | 通用脱敏 |
//! | PartialMask | 13812345678 | 138****5678 | 可读性要求高 |
//! | Hash | 张三 | 8f14e45f | 不可逆脱敏 |
//! | Redact | 张三 | *** | 最高安全级 |
//! | Token | 张三 | <PERSON_001> | 可逆脱敏 |
//! | Template | 张三 | 某某某 | 自定义规则 |

pub mod strategies;
pub mod engine;

// 重新导出核心类型
pub use strategies::*;
pub use engine::MaskingEngine;

use crate::core::recognizer::{EntitySpan, EntityType};
use serde::{Deserialize, Serialize};

/// 脱敏配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MaskConfig {
    /// 默认脱敏策略
    pub default_strategy: MaskStrategyType,
    /// 按实体类型指定的策略
    pub entity_strategies: std::collections::HashMap<String, MaskStrategyType>,
    /// 自定义模板 (用于 Template 策略)
    pub templates: std::collections::HashMap<String, String>,
    /// Token 策略的计数器起始值
    pub token_counter_start: u32,
    /// Hash 策略是否使用 SHA256 (否则使用简单哈希)
    pub use_sha256: bool,
}

impl Default for MaskConfig {
    fn default() -> Self {
        Self {
            default_strategy: MaskStrategyType::Replace,
            entity_strategies: std::collections::HashMap::new(),
            templates: std::collections::HashMap::new(),
            token_counter_start: 1,
            use_sha256: false,
        }
    }
}

/// 脱敏策略类型
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum MaskStrategyType {
    /// 替换: 张三 → [人名]
    Replace,
    /// 部分遮盖: 138****5678
    PartialMask,
    /// 哈希: 张三 → 8f14e45f
    Hash,
    /// 删除: 张三 → ***
    Redact,
    /// Token: 张三 → <PERSON_001>
    Token,
    /// 模板: 自定义替换规则
    Template,
}

impl MaskStrategyType {
    /// 从字符串解析策略类型
    pub fn from_str(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "replace" => Self::Replace,
            "partial_mask" | "partialmask" | "partial" => Self::PartialMask,
            "hash" => Self::Hash,
            "redact" => Self::Redact,
            "token" => Self::Token,
            "template" => Self::Template,
            _ => Self::Replace,
        }
    }

    /// 获取策略的显示名称
    pub fn display_name(&self) -> &str {
        match self {
            Self::Replace => "替换",
            Self::PartialMask => "部分遮盖",
            Self::Hash => "哈希",
            Self::Redact => "删除",
            Self::Token => "Token",
            Self::Template => "模板",
        }
    }
}

/// 脱敏策略 trait
///
/// 所有脱敏策略都必须实现此 trait。
pub trait MaskingStrategy: Send + Sync {
    /// 策略名称
    fn name(&self) -> &str;

    /// 策略类型
    fn strategy_type(&self) -> MaskStrategyType;

    /// 执行脱敏
    ///
    /// # 参数
    ///
    /// - `original`: 原始文本片段
    /// - `span`: 实体跨度信息
    /// - `config`: 脱敏配置
    ///
    /// # 返回
    ///
    /// 脱敏后的文本片段
    fn mask(&self, original: &str, span: &EntitySpan, config: &MaskConfig) -> String;
}
