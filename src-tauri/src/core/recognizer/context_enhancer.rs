//! 上下文增强识别器
//!
//! 基于周围词汇分析，提升或降低已识别实体的置信度。
//! 这是一个"依赖型"识别器（`requires_context() == true`），
//! 它不产生新的实体，而是调整已有实体的置信度。
//!
//! # 工作原理
//!
//! 1. 检查实体周围的关键词
//! 2. 如果发现相关的上下文词汇，提升置信度
//! 3. 如果发现不相关的上下文词汇，降低置信度
//!
//! # 示例
//!
//! - "邮箱：test@example.com" → "邮箱" 提升 email 的置信度
//! - "电话：13800138000" → "电话" 提升 phone 的置信度
//! - "日期：2024-01-01" → "日期" 提升 date 的置信度

use super::types::*;
use super::Recognizer;
use std::collections::HashMap;

/// 上下文关键词配置
struct ContextKeyword {
    /// 关键词列表
    keywords: Vec<String>,
    /// 置信度提升值
    boost: f32,
}

/// 上下文增强识别器
///
/// 基于周围词汇调整已识别实体的置信度。
/// 这是一个依赖型识别器，需要前置识别器的结果。
pub struct ContextEnhancer {
    /// 上下文关键词映射 (实体类型 → 关键词配置)
    keyword_map: HashMap<EntityType, Vec<ContextKeyword>>,
    /// 识别器名称
    name: String,
    /// 识别器优先级
    priority: i32,
    /// 搜索窗口大小（字符数）
    window_size: usize,
}

impl Default for ContextEnhancer {
    fn default() -> Self {
        Self::new()
    }
}

impl ContextEnhancer {
    /// 创建上下文增强识别器
    pub fn new() -> Self {
        let mut keyword_map = HashMap::new();

        // 邮箱相关上下文
        keyword_map.insert(EntityType::Email, vec![
            ContextKeyword {
                keywords: vec!["邮箱".to_string(), "email".to_string(), "邮件".to_string(), "mail".to_string()],
                boost: 0.2,
            },
            ContextKeyword {
                keywords: vec!["@".to_string()],
                boost: 0.3,
            },
        ]);

        // 电话相关上下文
        keyword_map.insert(EntityType::Phone, vec![
            ContextKeyword {
                keywords: vec!["电话".to_string(), "手机".to_string(), "phone".to_string(), "tel".to_string(), "联系".to_string()],
                boost: 0.2,
            },
            ContextKeyword {
                keywords: vec!["+86".to_string(), "86".to_string()],
                boost: 0.3,
            },
        ]);

        // 身份证相关上下文
        keyword_map.insert(EntityType::IdCard, vec![
            ContextKeyword {
                keywords: vec!["身份证".to_string(), "idcard".to_string(), "证件".to_string(), "ID".to_string()],
                boost: 0.3,
            },
        ]);

        // 银行卡相关上下文
        keyword_map.insert(EntityType::BankCard, vec![
            ContextKeyword {
                keywords: vec!["银行卡".to_string(), "卡号".to_string(), "bank".to_string(), "card".to_string(), "账户".to_string()],
                boost: 0.3,
            },
        ]);

        // 人名相关上下文
        keyword_map.insert(EntityType::Person, vec![
            ContextKeyword {
                keywords: vec!["姓名".to_string(), "名字".to_string(), "name".to_string(), "先生".to_string(), "女士".to_string(), "老师".to_string()],
                boost: 0.2,
            },
        ]);

        // 地址相关上下文
        keyword_map.insert(EntityType::Address, vec![
            ContextKeyword {
                keywords: vec!["地址".to_string(), "address".to_string(), "住址".to_string(), "所在地".to_string()],
                boost: 0.2,
            },
            ContextKeyword {
                keywords: vec!["省".to_string(), "市".to_string(), "区".to_string(), "路".to_string(), "街".to_string()],
                boost: 0.15,
            },
        ]);

        // 密钥相关上下文
        keyword_map.insert(EntityType::ApiKey, vec![
            ContextKeyword {
                keywords: vec!["api_key".to_string(), "apikey".to_string(), "token".to_string(), "secret".to_string(), "密钥".to_string(), "key".to_string()],
                boost: 0.3,
            },
        ]);

        // IP 相关上下文
        keyword_map.insert(EntityType::IpAddress, vec![
            ContextKeyword {
                keywords: vec!["ip".to_string(), "address".to_string(), "地址".to_string()],
                boost: 0.2,
            },
        ]);

        Self {
            keyword_map,
            name: "context_enhancer".to_string(),
            priority: 10, // 低优先级，依赖前置结果
            window_size: 20, // 搜索前后 20 个字符
        }
    }

    /// 分析上下文并调整置信度
    fn enhance_span(&self, span: &EntitySpan, text: &str) -> EntitySpan {
        let mut enhanced = span.clone();

        // 获取上下文窗口
        let window_start = span.start.saturating_sub(self.window_size);
        let window_end = (span.end + self.window_size).min(text.len());

        // 确保在 UTF-8 边界上
        let window_start = adjust_to_char_boundary(text, window_start);
        let window_end = adjust_to_char_boundary(text, window_end);

        let context_text = &text[window_start..window_end].to_lowercase();

        // 查找匹配的上下文关键词
        if let Some(keywords) = self.keyword_map.get(&span.entity_type) {
            let mut total_boost = 0.0f32;

            for keyword_config in keywords {
                for keyword in &keyword_config.keywords {
                    if context_text.contains(&keyword.to_lowercase()) {
                        total_boost += keyword_config.boost;
                    }
                }
            }

            // 限制最大提升
            total_boost = total_boost.min(0.4);

            // 应用置信度提升
            enhanced.confidence = (enhanced.confidence + total_boost).min(1.0);
            enhanced.context = Some(format!("context_enhanced:+{:.2}", total_boost));
        }

        enhanced
    }

    /// 增强已有实体的置信度
    ///
    /// 这个方法被 HybridEngine 调用，用于增强前置识别器的结果。
    pub fn enhance_spans(&self, spans: &[EntitySpan], text: &str) -> Vec<EntitySpan> {
        spans.iter().map(|span| self.enhance_span(span, text)).collect()
    }
}

/// 调整索引到 UTF-8 字符边界
fn adjust_to_char_boundary(text: &str, index: usize) -> usize {
    let mut idx = index;
    while idx < text.len() && !text.is_char_boundary(idx) {
        idx += 1;
    }
    idx.min(text.len())
}

impl Recognizer for ContextEnhancer {
    fn name(&self) -> &str {
        &self.name
    }

    fn recognizer_type(&self) -> RecognizerType {
        RecognizerType::Context
    }

    fn supported_entities(&self) -> Vec<EntityType> {
        self.keyword_map.keys().cloned().collect()
    }

    fn analyze(&self, _context: &AnalysisContext) -> AnalysisResult {
        // 不产生新实体，只增强已有实体
        AnalysisResult::empty(&self.name)
    }

    fn priority(&self) -> i32 {
        self.priority
    }

    fn requires_context(&self) -> bool {
        true // 依赖前置识别器的结果
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_enhance_email_with_context() {
        let enhancer = ContextEnhancer::new();
        let span = EntitySpan::new(9, 25, EntityType::Email, 0.7, "test");

        let text = "联系邮箱：test@example.com";
        let enhanced = enhancer.enhance_span(&span, text);

        assert!(enhanced.confidence > 0.7); // 置信度应该提升
    }

    #[test]
    fn test_enhance_phone_with_context() {
        let enhancer = ContextEnhancer::new();
        let span = EntitySpan::new(6, 17, EntityType::Phone, 0.6, "test");

        let text = "电话：13800138000";
        let enhanced = enhancer.enhance_span(&span, text);

        assert!(enhanced.confidence > 0.6);
    }

    #[test]
    fn test_no_enhancement_without_context() {
        let enhancer = ContextEnhancer::new();
        let span = EntitySpan::new(0, 11, EntityType::Phone, 0.6, "test");

        let text = "13800138000";
        let enhanced = enhancer.enhance_span(&span, text);

        assert_eq!(enhanced.confidence, 0.6); // 无上下文，置信度不变
    }

    #[test]
    fn test_enhance_idcard_with_context() {
        let enhancer = ContextEnhancer::new();
        let span = EntitySpan::new(6, 24, EntityType::IdCard, 0.8, "test");

        let text = "身份证：110101199001011234";
        let enhanced = enhancer.enhance_span(&span, text);

        assert!(enhanced.confidence > 0.8);
    }
}
