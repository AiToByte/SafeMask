//! 冲突解决层
//!
//! 当多个识别器返回重叠的实体跨度时，`ConflictResolver` 负责：
//! 1. 优先级仲裁 — 高优先级识别器的结果优先
//! 2. 重叠合并 — 重叠区域取更高置信度的结果
//! 3. 置信度过滤 — 过滤低于阈值的结果
//! 4. 结果去重 — 移除完全相同的跨度

use crate::core::recognizer::EntitySpan;
use log::debug;

/// 冲突解决器
pub struct ConflictResolver {
    /// 置信度阈值
    confidence_threshold: f32,
}

impl ConflictResolver {
    /// 创建冲突解决器
    pub fn new(confidence_threshold: f32) -> Self {
        Self {
            confidence_threshold: confidence_threshold.clamp(0.0, 1.0),
        }
    }

    /// 使用默认配置创建
    pub fn default_config() -> Self {
        Self::new(0.5)
    }

    /// 解决冲突，返回合并后的结果
    ///
    /// # 处理流程
    ///
    /// 1. 过滤低置信度结果
    /// 2. 按位置排序
    /// 3. 合并重叠跨度（取更高置信度）
    /// 4. 去重
    pub fn resolve(&self, spans: Vec<EntitySpan>) -> Vec<EntitySpan> {
        if spans.is_empty() {
            return spans;
        }

        // Step 1: 过滤低置信度
        let mut filtered: Vec<EntitySpan> = spans
            .into_iter()
            .filter(|s| s.confidence >= self.confidence_threshold)
            .collect();

        if filtered.is_empty() {
            return filtered;
        }

        // Step 2: 按位置排序（起始位置升序，相同位置按置信度降序）
        filtered.sort_by(|a, b| {
            a.start
                .cmp(&b.start)
                .then(b.confidence.partial_cmp(&a.confidence).unwrap_or(std::cmp::Ordering::Equal))
        });

        // Step 3: 合并重叠跨度
        let merged = self.merge_overlapping(filtered);

        debug!(
            "🔀 冲突解决完成: 输入 {} 个, 输出 {} 个",
            merged.len(),
            merged.len()
        );

        merged
    }

    /// 合并重叠的跨度
    ///
    /// 策略：
    /// - 完全包含：保留外层跨度（覆盖范围更大）
    /// - 部分重叠：保留置信度更高的跨度
    /// - 无重叠：都保留
    fn merge_overlapping(&self, spans: Vec<EntitySpan>) -> Vec<EntitySpan> {
        if spans.len() <= 1 {
            return spans;
        }

        let mut result: Vec<EntitySpan> = Vec::new();

        for span in spans {
            if result.is_empty() {
                result.push(span);
                continue;
            }

            let last = result.last_mut().unwrap();

            // 检查是否重叠
            if span.start < last.end {
                // 重叠：保留置信度更高的
                if span.confidence > last.confidence {
                    *last = span;
                }
                // 如果置信度相同，保留覆盖范围更大的（即 last，因为它先到）
            } else {
                // 无重叠：直接添加
                result.push(span);
            }
        }

        result
    }

    /// 更新置信度阈值
    pub fn set_threshold(&mut self, threshold: f32) {
        self.confidence_threshold = threshold.clamp(0.0, 1.0);
    }

    /// 获取当前置信度阈值
    pub fn threshold(&self) -> f32 {
        self.confidence_threshold
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::recognizer::EntityType;

    fn make_span(start: usize, end: usize, confidence: f32, source: &str) -> EntitySpan {
        let mut span = EntitySpan::new(start, end, EntityType::Email, confidence, source);
        span.mask = Some("<EMAIL>".to_string());
        span
    }

    #[test]
    fn test_no_overlap() {
        let resolver = ConflictResolver::new(0.0);
        let spans = vec![
            make_span(0, 5, 1.0, "a"),
            make_span(10, 15, 1.0, "b"),
        ];
        let result = resolver.resolve(spans);
        assert_eq!(result.len(), 2);
    }

    #[test]
    fn test_overlap_keeps_higher_confidence() {
        let resolver = ConflictResolver::new(0.0);
        let spans = vec![
            make_span(0, 10, 0.6, "low"),
            make_span(5, 15, 0.9, "high"),
        ];
        let result = resolver.resolve(spans);
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].source, "high");
    }

    #[test]
    fn test_confidence_filter() {
        let resolver = ConflictResolver::new(0.7);
        let spans = vec![
            make_span(0, 5, 0.5, "low"),
            make_span(10, 15, 0.9, "high"),
        ];
        let result = resolver.resolve(spans);
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].source, "high");
    }

    #[test]
    fn test_empty_input() {
        let resolver = ConflictResolver::new(0.0);
        let result = resolver.resolve(vec![]);
        assert!(result.is_empty());
    }
}
