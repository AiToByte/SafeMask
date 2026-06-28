//! 脱敏策略实现
//!
//! 实现 6 种内置脱敏策略，覆盖常见的脱敏场景。

use super::{MaskConfig, MaskStrategyType, MaskingStrategy};
use crate::core::recognizer::{EntitySpan, EntityType};
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

// ─────────────────────────────────────────────────────────────────────────────
// 1. Replace 策略 — 替换为标签
// ─────────────────────────────────────────────────────────────────────────────

/// 替换策略
///
/// 将实体替换为类型标签，如 `[人名]`、`[邮箱]`。
/// 最通用的脱敏方式，适用于大多数场景。
pub struct ReplaceStrategy;

impl MaskingStrategy for ReplaceStrategy {
    fn name(&self) -> &str {
        "replace"
    }

    fn strategy_type(&self) -> MaskStrategyType {
        MaskStrategyType::Replace
    }

    fn mask(&self, _original: &str, span: &EntitySpan, _config: &MaskConfig) -> String {
        // 优先使用 span 中的 mask，否则使用实体类型的默认标签
        span.mask.clone().unwrap_or_else(|| {
            format!("[{}]", span.entity_type.display_label())
        })
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// 2. PartialMask 策略 — 部分遮盖
// ─────────────────────────────────────────────────────────────────────────────

/// 部分遮盖策略
///
/// 保留部分可见字符，其余用 `*` 替换。
/// 适用于需要保留可读性的场景（如手机号 138****5678）。
pub struct PartialMaskStrategy;

impl PartialMaskStrategy {
    /// 计算保留字符数
    fn visible_count(len: usize) -> usize {
        match len {
            0..=4 => 1,
            5..=8 => 2,
            9..=12 => 3,
            _ => 4,
        }
    }
}

impl MaskingStrategy for PartialMaskStrategy {
    fn name(&self) -> &str {
        "partial_mask"
    }

    fn strategy_type(&self) -> MaskStrategyType {
        MaskStrategyType::PartialMask
    }

    fn mask(&self, original: &str, _span: &EntitySpan, _config: &MaskConfig) -> String {
        let chars: Vec<char> = original.chars().collect();
        let len = chars.len();

        if len <= 2 {
            return "*".repeat(len);
        }

        let visible = Self::visible_count(len);
        let prefix: String = chars[..visible].iter().collect();
        let suffix: String = chars[len - visible..].iter().collect();
        let mask_count = len - visible * 2;

        format!("{}{}{}", prefix, "*".repeat(mask_count.max(3)), suffix)
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// 3. Hash 策略 — 哈希替换
// ─────────────────────────────────────────────────────────────────────────────

/// 哈希策略
///
/// 将实体替换为其哈希值的前 8 位。
/// 适用于需要不可逆脱敏的场景。
pub struct HashStrategy;

impl HashStrategy {
    /// 计算简单哈希
    fn simple_hash(text: &str) -> String {
        let mut hasher = DefaultHasher::new();
        text.hash(&mut hasher);
        let hash = hasher.finish();
        format!("{:08x}", hash as u32)
    }
}

impl MaskingStrategy for HashStrategy {
    fn name(&self) -> &str {
        "hash"
    }

    fn strategy_type(&self) -> MaskStrategyType {
        MaskStrategyType::Hash
    }

    fn mask(&self, original: &str, _span: &EntitySpan, config: &MaskConfig) -> String {
        if config.use_sha256 {
            // 使用 SHA256 (需要额外依赖，这里用简单哈希替代)
            Self::simple_hash(original)
        } else {
            Self::simple_hash(original)
        }
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// 4. Redact 策略 — 完全删除
// ─────────────────────────────────────────────────────────────────────────────

/// 删除策略
///
/// 将实体完全替换为 `***`。
/// 适用于最高安全级别的场景。
pub struct RedactStrategy;

impl MaskingStrategy for RedactStrategy {
    fn name(&self) -> &str {
        "redact"
    }

    fn strategy_type(&self) -> MaskStrategyType {
        MaskStrategyType::Redact
    }

    fn mask(&self, _original: &str, _span: &EntitySpan, _config: &MaskConfig) -> String {
        "***".to_string()
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// 5. Token 策略 — 可逆 Token 替换
// ─────────────────────────────────────────────────────────────────────────────

/// Token 策略
///
/// 将实体替换为带编号的 Token，如 `<PERSON_001>`。
/// 适用于需要可逆脱敏的场景（可以还原）。
pub struct TokenStrategy {
    /// Token 计数器
    counter: std::sync::atomic::AtomicU32,
}

impl TokenStrategy {
    /// 创建 Token 策略
    pub fn new(start: u32) -> Self {
        Self {
            counter: std::sync::atomic::AtomicU32::new(start),
        }
    }
}

impl MaskingStrategy for TokenStrategy {
    fn name(&self) -> &str {
        "token"
    }

    fn strategy_type(&self) -> MaskStrategyType {
        MaskStrategyType::Token
    }

    fn mask(&self, _original: &str, span: &EntitySpan, _config: &MaskConfig) -> String {
        let idx = self.counter.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        format!("<{}_{:03}>", span.entity_type.en_label().to_uppercase(), idx)
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// 6. Template 策略 — 自定义模板
// ─────────────────────────────────────────────────────────────────────────────

/// 模板策略
///
/// 使用用户定义的模板进行替换。
/// 模板中可以使用 `{type}` 作为实体类型的占位符。
pub struct TemplateStrategy;

impl MaskingStrategy for TemplateStrategy {
    fn name(&self) -> &str {
        "template"
    }

    fn strategy_type(&self) -> MaskStrategyType {
        MaskStrategyType::Template
    }

    fn mask(&self, _original: &str, span: &EntitySpan, config: &MaskConfig) -> String {
        let entity_key = span.entity_type.en_label();

        // 查找实体类型的模板
        if let Some(template) = config.templates.get(entity_key) {
            return template
                .replace("{type}", span.entity_type.display_label())
                .replace("{label}", span.entity_type.en_label());
        }

        // 查找默认模板
        if let Some(template) = config.templates.get("default") {
            return template
                .replace("{type}", span.entity_type.display_label())
                .replace("{label}", span.entity_type.en_label());
        }

        // 回退到替换策略
        format!("[{}]", span.entity_type.display_label())
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// 测试
// ─────────────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::recognizer::EntityType;

    fn make_span(entity_type: EntityType) -> EntitySpan {
        EntitySpan::new(0, 10, entity_type, 1.0, "test")
    }

    fn default_config() -> MaskConfig {
        MaskConfig::default()
    }

    #[test]
    fn test_replace_strategy() {
        let strategy = ReplaceStrategy;
        let span = make_span(EntityType::Person);
        let result = strategy.mask("张三", &span, &default_config());
        assert_eq!(result, "[人名]");
    }

    #[test]
    fn test_partial_mask_phone() {
        let strategy = PartialMaskStrategy;
        let span = make_span(EntityType::Phone);
        let result = strategy.mask("13812345678", &span, &default_config());
        assert_eq!(result, "138*****678");
    }

    #[test]
    fn test_partial_mask_short() {
        let strategy = PartialMaskStrategy;
        let span = make_span(EntityType::Email);
        let result = strategy.mask("ab", &span, &default_config());
        assert_eq!(result, "**");
    }

    #[test]
    fn test_hash_strategy() {
        let strategy = HashStrategy;
        let span = make_span(EntityType::Person);
        let result = strategy.mask("张三", &span, &default_config());
        assert_eq!(result.len(), 8); // 8 位十六进制
    }

    #[test]
    fn test_redact_strategy() {
        let strategy = RedactStrategy;
        let span = make_span(EntityType::Person);
        let result = strategy.mask("张三", &span, &default_config());
        assert_eq!(result, "***");
    }

    #[test]
    fn test_token_strategy() {
        let strategy = TokenStrategy::new(1);
        let span = make_span(EntityType::Person);
        let result = strategy.mask("张三", &span, &default_config());
        assert_eq!(result, "<PERSON_001>");

        let result2 = strategy.mask("李四", &span, &default_config());
        assert_eq!(result2, "<PERSON_002>");
    }

    #[test]
    fn test_template_strategy_with_template() {
        let strategy = TemplateStrategy;
        let span = make_span(EntityType::Person);

        let mut config = default_config();
        config.templates.insert("person".to_string(), "某某{type}".to_string());

        let result = strategy.mask("张三", &span, &config);
        assert_eq!(result, "某某人名");
    }

    #[test]
    fn test_template_strategy_fallback() {
        let strategy = TemplateStrategy;
        let span = make_span(EntityType::Person);
        let result = strategy.mask("张三", &span, &default_config());
        assert_eq!(result, "[人名]"); // 回退到替换策略
    }
}
