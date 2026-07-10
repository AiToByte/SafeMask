//! 可插拔识别器架构
//!
//! 本模块定义了 SafeMask 识别引擎的核心抽象：`Recognizer` trait。
//! 所有识别器（规则引擎、AI 引擎、字典引擎等）都通过这个 trait 统一接口。
//!
//! # 设计原则
//!
//! - **接口隔离**: 识别器只需实现 `Recognizer` trait，不依赖上层逻辑
//! **开闭原则**: 新识别器通过实现 trait 扩展，无需修改现有代码
//! - **依赖倒置**: 上层依赖抽象（trait），不依赖具体实现
//!
//! # 使用示例
//!
//! ```rust
//! use safemask_lib::core::recognizer::*;
//!
//! // 创建分析上下文
//! let context = AnalysisContext::from_text(b"my email is test@example.com");
//!
//! // 识别器通过 Registry 统一管理
//! let mut registry = RecognizerRegistry::default_config();
//! let spans = registry.analyze(&context);
//! for span in &spans {
//!     println!("Found {:?} at {}..{}", span.entity_type, span.start, span.end);
//! }
//! ```

pub mod types;
pub mod registry;
pub mod regex_recognizer;
pub mod aho_corasick_recognizer;
pub mod ner_recognizer;
pub mod context_enhancer;
pub mod checksum_recognizer;

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// 重新导出核心类型
pub use types::*;
pub use registry::RecognizerRegistry;
pub use regex_recognizer::RegexRecognizer;
pub use aho_corasick_recognizer::AhoCorasickRecognizer;
pub use ner_recognizer::NerRecognizer;


// ─────────────────────────────────────────────────────────────────────────────
// Recognizer Trait
// ─────────────────────────────────────────────────────────────────────────────

/// 识别器统一接口
///
/// 所有识别器都必须实现此 trait。识别器是无状态的（或内部状态通过
/// `&self` 安全访问），可以安全地在多线程环境中使用。
///
/// # 实现指南
///
/// 1. `name()` 返回唯一的识别器名称，用于日志和调试
/// 2. `supported_entities()` 声明此识别器能识别的实体类型
/// 3. `analyze()` 执行实际的识别逻辑
/// 4. `priority()` 返回优先级，数值越大越先执行
/// 5. `is_enabled()` 控制识别器是否参与识别
///
/// # 性能要求
///
/// - `analyze()` 应尽可能高效，因为它可能被频繁调用
/// - 对于大文本，识别器应自行处理分块逻辑
/// - 避免在 `analyze()` 中进行不必要的内存分配
pub trait Recognizer: Send + Sync {
    /// 识别器名称（唯一标识）
    ///
    /// 用于日志、调试和冲突解决中的来源标记。
    /// 建议使用 snake_case 格式，如 "regex_engine"、"ner_model"。
    fn name(&self) -> &str;

    /// 识别器类型
    fn recognizer_type(&self) -> RecognizerType;

    /// 此识别器支持的实体类型列表
    ///
    /// 返回空列表表示此识别器是通用的（可识别任何类型）。
    fn supported_entities(&self) -> Vec<EntityType>;

    /// 执行识别分析
    ///
    /// 输入一个 `AnalysisContext`，返回识别到的实体跨度列表。
    /// 返回的 `AnalysisResult` 包含识别结果和性能指标。
    fn analyze(&self, context: &AnalysisContext) -> AnalysisResult;

    /// 识别器优先级（数值越大越先执行）
    ///
    /// 默认优先级为 0。建议：
    /// - 规则引擎: 100 (高优先级，确定性匹配)
    /// - AI 引擎: 50 (中优先级，概率性匹配)
    /// - 上下文增强: 10 (低优先级，依赖前置结果)
    fn priority(&self) -> i32 {
        0
    }

    /// 是否启用此识别器
    ///
    /// 默认启用。可以在运行时动态控制。
    fn is_enabled(&self) -> bool {
        true
    }

    /// 此识别器是否依赖前置识别器的结果
    ///
    /// 如果返回 `true`，注册表会确保在此识别器执行前，
    /// 所有非依赖识别器已经执行完毕。
    fn requires_context(&self) -> bool {
        false
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// 识别器工厂 (RecognizerFactory)
// ─────────────────────────────────────────────────────────────────────────────

/// 识别器工厂 trait
///
/// 用于动态创建识别器实例。在配置热重载等场景中，
/// 工厂可以根据新配置创建新的识别器实例。
pub trait RecognizerFactory: Send + Sync {
    /// 工厂名称
    fn name(&self) -> &str;

    /// 创建识别器实例
    fn create(&self, config: &RecognizerConfig) -> Result<Box<dyn Recognizer>, String>;
}

/// 识别器配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecognizerConfig {
    /// 识别器名称
    pub name: String,
    /// 是否启用
    pub enabled: bool,
    /// 优先级
    pub priority: i32,
    /// 额外配置参数
    pub params: HashMap<String, String>,
}

impl Default for RecognizerConfig {
    fn default() -> Self {
        Self {
            name: String::new(),
            enabled: true,
            priority: 0,
            params: HashMap::new(),
        }
    }
}
