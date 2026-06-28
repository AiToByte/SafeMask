//! 脱敏引擎
//!
//! `MaskingEngine` 负责根据配置选择合适的脱敏策略，
//! 并将识别到的实体替换为脱敏后的文本。

use super::strategies::*;
use super::{MaskConfig, MaskStrategyType, MaskingStrategy};
use crate::core::recognizer::{EntitySpan, MaskResult};
use log::debug;
use std::collections::HashMap;

/// 脱敏引擎
///
/// 根据配置路由到合适的脱敏策略，执行文本脱敏。
pub struct MaskingEngine {
    /// 策略映射 (策略类型 → 策略实例)
    strategies: HashMap<MaskStrategyType, Box<dyn MaskingStrategy>>,
    /// 配置
    config: MaskConfig,
}

impl MaskingEngine {
    /// 创建脱敏引擎
    pub fn new(config: MaskConfig) -> Self {
        let mut strategies: HashMap<MaskStrategyType, Box<dyn MaskingStrategy>> = HashMap::new();

        // 注册所有内置策略
        strategies.insert(MaskStrategyType::Replace, Box::new(ReplaceStrategy));
        strategies.insert(MaskStrategyType::PartialMask, Box::new(PartialMaskStrategy));
        strategies.insert(MaskStrategyType::Hash, Box::new(HashStrategy));
        strategies.insert(MaskStrategyType::Redact, Box::new(RedactStrategy));
        strategies.insert(MaskStrategyType::Token, Box::new(TokenStrategy::new(config.token_counter_start)));
        strategies.insert(MaskStrategyType::Template, Box::new(TemplateStrategy));

        Self { strategies, config }
    }

    /// 使用默认配置创建
    pub fn default_config() -> Self {
        Self::new(MaskConfig::default())
    }

    /// 获取实体类型对应的策略
    fn strategy_for_entity(&self, entity_type: &crate::core::recognizer::EntityType) -> &dyn MaskingStrategy {
        let entity_key = entity_type.en_label();

        // 查找实体类型的特定策略
        if let Some(strategy_type) = self.config.entity_strategies.get(entity_key) {
            if let Some(strategy) = self.strategies.get(strategy_type) {
                return strategy.as_ref();
            }
        }

        // 使用默认策略
        self.strategies
            .get(&self.config.default_strategy)
            .map(|s| s.as_ref())
            .unwrap_or_else(|| {
                // 这不应该发生，因为默认策略总是存在
                self.strategies.get(&MaskStrategyType::Replace).unwrap().as_ref()
            })
    }

    /// 对单个实体执行脱敏
    pub fn mask_entity(&self, original: &str, span: &EntitySpan) -> String {
        let strategy = self.strategy_for_entity(&span.entity_type);
        strategy.mask(original, span, &self.config)
    }

    /// 对文本和识别结果执行完整脱敏
    pub fn apply(&self, text: &str, spans: &[EntitySpan]) -> MaskResult {
        if spans.is_empty() {
            return MaskResult::unchanged(text);
        }

        let text_bytes = text.as_bytes();
        let mut output = Vec::new();
        let mut last_pos = 0;

        // 按位置排序
        let mut sorted_spans: Vec<&EntitySpan> = spans.iter().collect();
        sorted_spans.sort_by_key(|s| s.start);

        for span in sorted_spans {
            if span.start < last_pos {
                continue; // 跳过重叠
            }

            // 添加未匹配部分
            if span.start > last_pos {
                output.extend_from_slice(&text_bytes[last_pos..span.start]);
            }

            // 获取原始文本片段
            let original = &text[span.start..span.end];

            // 执行脱敏
            let masked = self.mask_entity(original, span);
            output.extend_from_slice(masked.as_bytes());

            last_pos = span.end;
        }

        // 添加剩余部分
        if last_pos < text_bytes.len() {
            output.extend_from_slice(&text_bytes[last_pos..]);
        }

        let masked_text = String::from_utf8_lossy(&output).to_string();

        MaskResult {
            original: text.to_string(),
            masked: masked_text,
            entities: spans.to_vec(),
            has_changes: true,
        }
    }

    /// 更新配置
    pub fn update_config(&mut self, config: MaskConfig) {
        self.config = config;
    }

    /// 获取当前配置
    pub fn config(&self) -> &MaskConfig {
        &self.config
    }

    /// 获取可用策略列表
    pub fn available_strategies(&self) -> Vec<&str> {
        self.strategies.keys().map(|k| k.display_name()).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::recognizer::{EntityType, EntitySpan};

    fn make_span(entity_type: EntityType, start: usize, end: usize) -> EntitySpan {
        EntitySpan::new(start, end, entity_type, 1.0, "test")
    }

    #[test]
    fn test_mask_entity_replace() {
        let engine = MaskingEngine::default_config();
        let span = make_span(EntityType::Person, 0, 6);
        let result = engine.mask_entity("张三", &span);
        assert_eq!(result, "[人名]");
    }

    #[test]
    fn test_apply_multiple_entities() {
        let engine = MaskingEngine::default_config();
        let text = "姓名：张三，邮箱：test@example.com";
        let spans = vec![
            make_span(EntityType::Person, 9, 15),    // "张三" 的字节位置
            make_span(EntityType::Email, 21, 39),    // "test@example.com"
        ];

        let result = engine.apply(text, &spans);
        assert!(result.has_changes);
        assert!(result.masked.contains("[人名]"));
        assert!(result.masked.contains("[邮箱]"));
    }

    #[test]
    fn test_apply_no_spans() {
        let engine = MaskingEngine::default_config();
        let text = "没有敏感信息";
        let result = engine.apply(text, &[]);
        assert!(!result.has_changes);
        assert_eq!(result.original, result.masked);
    }

    #[test]
    fn test_custom_strategy_per_entity() {
        let mut config = MaskConfig::default();
        config.entity_strategies.insert("email".to_string(), MaskStrategyType::Redact);

        let engine = MaskingEngine::new(config);
        let span = make_span(EntityType::Email, 0, 20);
        let result = engine.mask_entity("test@example.com", &span);
        assert_eq!(result, "***");
    }
}
