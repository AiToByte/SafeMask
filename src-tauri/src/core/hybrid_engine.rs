//! 混合识别引擎
//!
//! `HybridEngine` 是 SafeMask 的核心引擎，组合了：
//! - `RecognizerRegistry` — 可插拔识别器管理
//! - `ConflictResolver` — 冲突解决
//!
//! 它是原有 `MaskEngine` 的升级版，保持向后兼容的同时，
//! 提供了可扩展的识别器架构。
//!
//! # 向后兼容
//!
//! `HybridEngine` 提供了与原有 `MaskEngine` 相同的 `mask_line` 方法，
//! 确保现有代码无需修改即可使用新引擎。

use crate::core::recognizer::{AnalysisContext, EntitySpan, MaskResult, RecognizerRegistry};
use crate::core::resolver::ConflictResolver;
use crate::core::rules::Rule;
use crate::core::masking::{MaskingEngine, MaskConfig};
use crate::infra::ai::ModelManager;
use log::info;
use std::borrow::Cow;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

/// 混合识别引擎
///
/// 组合多个识别器的结果，通过冲突解决层输出最终结果。
pub struct HybridEngine {
    /// 识别器注册表
    registry: RecognizerRegistry,
    /// 冲突解决器
    resolver: ConflictResolver,
    /// 脱敏引擎
    masking_engine: MaskingEngine,
    /// 模型管理器 (用于 AI 引擎)
    model_manager: Option<Arc<ModelManager>>,
    /// AI 引擎是否启用（原子标志，支持跨线程修改）
    ai_enabled: Arc<AtomicBool>,
}

impl HybridEngine {
    /// 从规则列表创建混合引擎
    ///
    /// 自动将规则分为字面量和正则两类，分别注册到对应的识别器。
    pub fn from_rules(rules: Vec<Rule>) -> Self {
        let config = crate::core::recognizer::registry::RegistryConfig::default();
        let mut registry = RecognizerRegistry::new(config);

        // 注册 Aho-Corasick 字典识别器（字面量模式）
        let ac_recognizer = crate::core::recognizer::AhoCorasickRecognizer::from_rules(
            &rules,
            Some("aho_corasick_engine"),
            Some(100), // 高优先级
        );
        info!("📚 字典识别器: {} 个条目", ac_recognizer.entry_count());
        registry.register(Box::new(ac_recognizer));

        // 注册正则识别器（正则模式）
        let regex_recognizer = crate::core::recognizer::RegexRecognizer::from_rules(
            &rules,
            Some("regex_engine"),
            Some(90), // 稍低优先级
        );
        info!("🔤 正则识别器: {} 条规则", regex_recognizer.rule_count());
        registry.register(Box::new(regex_recognizer));

        let resolver = ConflictResolver::new(0.5);

        info!(
            "⚙️ 混合引擎初始化完成: {} 个识别器已注册",
            registry.len()
        );

        Self {
            registry,
            resolver,
            masking_engine: MaskingEngine::default_config(),
            model_manager: None,
            ai_enabled: Arc::new(AtomicBool::new(false)),
        }
    }

    /// 启用 AI 引擎
    ///
    /// 如果模型目录存在可用模型，注册 NER 识别器。
    /// 可重复调用——第二次调用会重新扫描模型目录；
    /// 如果 AI 已启用且有模型，则跳过以避免重复注册。
    pub fn enable_ai_engine(&mut self, models_dir: impl AsRef<std::path::Path>) {
        // 如果 AI 已启用且已有模型管理器，跳过重复初始化
        if self.ai_enabled.load(Ordering::SeqCst) && self.model_manager.is_some() {
            info!("🤖 AI 引擎已启用，跳过重复初始化");
            return;
        }
        let model_manager = Arc::new(ModelManager::new(models_dir));

        if model_manager.has_models() {
            info!("🤖 发现 {} 个 AI 模型，注册 NER 识别器", model_manager.available_models().len());

            let ner_recognizer = crate::core::recognizer::NerRecognizer::new(
                model_manager.clone(),
                Some("ner_engine"),
                Some(50),   // 中等优先级
                Some(0.5),  // 默认置信度阈值
                Some(self.ai_enabled.clone()),
            );

            self.registry.register(Box::new(ner_recognizer));
            self.model_manager = Some(model_manager);
            self.ai_enabled.store(true, Ordering::SeqCst);

            info!("✅ AI 引擎已启用");
        } else {
            info!("ℹ️ 未发现 AI 模型，跳过 AI 引擎注册");
            self.model_manager = Some(model_manager);
        }
    }

    /// 获取模型管理器
    pub fn model_manager(&self) -> Option<&Arc<ModelManager>> {
        self.model_manager.as_ref()
    }

    /// 启用/停用 AI 引擎
    pub fn set_ai_enabled(&self, enabled: bool) -> bool {
        self.ai_enabled.store(enabled, Ordering::SeqCst);
        info!("🔄 AI 引擎已{}", if enabled { "启用" } else { "停用" });
        true
    }

    /// AI 引擎是否已启用
    pub fn is_ai_enabled(&self) -> bool {
        self.ai_enabled.load(Ordering::SeqCst)
    }

    /// 获取 AI 启用状态的 Arc 引用（用于跨线程共享）
    pub fn ai_enabled_arc(&self) -> Arc<AtomicBool> {
        self.ai_enabled.clone()
    }

    /// 获取 AI 引擎状态信息
    pub fn ai_status(&self) -> serde_json::Value {
        match &self.model_manager {
            Some(mm) => mm.status_info(),
            None => serde_json::json!({
                "state": "not_available",
                "error": "AI 引擎未初始化",
                "available_count": 0,
            }),
        }
    }

    /// 识别文本中的实体
    pub fn detect(&self, text: &[u8]) -> Vec<EntitySpan> {
        let context = AnalysisContext::from_text(text);
        let spans = self.registry.analyze(&context);
        self.resolver.resolve(spans, text)
    }

    /// 脱敏文本（向后兼容原有 `MaskEngine::mask_line` 接口）
    ///
    /// 对输入文本执行识别和脱敏，返回脱敏后的文本。
    /// 如果没有识别到任何实体，返回原始文本的借用（零拷贝）。
    pub fn mask_line<'a>(&self, input: &'a [u8]) -> Cow<'a, [u8]> {
        if input.is_empty() {
            return Cow::Borrowed(input);
        }

        let spans = self.detect(input);
        if spans.is_empty() {
            return Cow::Borrowed(input);
        }

        // 执行脱敏替换
        let result = self.apply_replacements(input, &spans);
        Cow::Owned(result)
    }

    /// 执行脱敏替换
    fn apply_replacements(&self, input: &[u8], spans: &[EntitySpan]) -> Vec<u8> {
        let mut output = Vec::with_capacity(input.len());
        let mut last_pos = 0;

        for span in spans {
            if span.start < last_pos {
                continue; // 跳过重叠
            }

            // 添加未匹配部分
            output.extend_from_slice(&input[last_pos..span.start]);

            // 使用规则定义的掩码，或使用实体类型的默认标签
            let mask = span.mask.clone()
                .unwrap_or_else(|| format!("[{}]", span.entity_type.display_label()));
            output.extend_from_slice(mask.as_bytes());

            last_pos = span.end;
        }

        // 添加剩余部分
        if last_pos < input.len() {
            output.extend_from_slice(&input[last_pos..]);
        }

        output
    }

    /// 完整的脱敏分析（返回结构化结果）
    ///
    /// 使用 MaskingEngine 执行脱敏，支持可配置的脱敏策略。
    pub fn analyze(&self, text: &str) -> MaskResult {
        let spans = self.detect(text.as_bytes());
        self.masking_engine.apply(text, &spans)
    }

    /// 更新脱敏配置
    pub fn update_masking_config(&mut self, config: MaskConfig) {
        self.masking_engine.update_config(config);
    }

    /// 获取脱敏引擎的引用
    pub fn masking_engine(&self) -> &MaskingEngine {
        &self.masking_engine
    }

    /// 获取脱敏引擎的可变引用
    pub fn masking_engine_mut(&mut self) -> &mut MaskingEngine {
        &mut self.masking_engine
    }

    /// 获取识别器注册表的引用
    pub fn registry(&self) -> &RecognizerRegistry {
        &self.registry
    }

    /// 获取识别器注册表的可变引用
    pub fn registry_mut(&mut self) -> &mut RecognizerRegistry {
        &mut self.registry
    }

    /// 获取冲突解决器的可变引用
    pub fn resolver_mut(&mut self) -> &mut ConflictResolver {
        &mut self.resolver
    }

    /// 获取规则数量（兼容原有接口）
    pub fn rule_count(&self) -> usize {
        self.registry.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::rules::Rule;

    fn make_rule(name: &str, pattern: &str, mask: &str, enabled: bool) -> Rule {
        Rule {
            name: name.to_string(),
            pattern: pattern.to_string(),
            mask: mask.to_string(),
            priority: 0,
            enabled,
            is_custom: false,
        }
    }

    #[test]
    fn test_hybrid_engine_basic() {
        let rules = vec![
            make_rule("email", r"[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}", "<EMAIL>", true),
            make_rule("phone", r"1[3-9]\d{9}", "<PHONE>", true),
        ];

        let engine = HybridEngine::from_rules(rules);

        let text = "联系我 test@example.com 或 13800138000";
        let result = engine.analyze(text);

        assert!(result.has_changes);
        assert!(result.masked.contains("<EMAIL>"));
        assert!(result.masked.contains("<PHONE>"));
    }

    #[test]
    fn test_hybrid_engine_no_match() {
        let rules = vec![
            make_rule("email", r"[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}", "<EMAIL>", true),
        ];

        let engine = HybridEngine::from_rules(rules);

        let text = "没有敏感信息的文本";
        let result = engine.analyze(text);

        assert!(!result.has_changes);
        assert_eq!(result.original, result.masked);
    }

    #[test]
    fn test_mask_line_zero_copy() {
        let rules = vec![
            make_rule("email", r"[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}", "<EMAIL>", true),
        ];

        let engine = HybridEngine::from_rules(rules);

        // 无匹配时应返回借用（零拷贝）
        let input = b"no sensitive data here";
        let result = engine.mask_line(input);
        match result {
            Cow::Borrowed(_) => {} // 期望的行为
            Cow::Owned(_) => panic!("Expected borrowed result for no-match case"),
        }
    }

    #[test]
    fn test_hybrid_engine_literal_match() {
        let rules = vec![
            make_rule("company", "SafeMask", "<COMPANY>", true),
        ];

        let engine = HybridEngine::from_rules(rules);

        let text = "SafeMask is a great tool";
        let result = engine.analyze(text);

        assert!(result.has_changes);
        assert!(result.masked.contains("<COMPANY>"));
    }
}
