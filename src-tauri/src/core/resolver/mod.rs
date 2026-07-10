//! 冲突解决层
//!
//! 当多个识别器返回重叠的实体跨度时，`ConflictResolver` 负责：
//! 1. 子区间雕刻（Carving）— 高优先级区间在低优先级区间中"凿"出位置，而非简单丢弃
//! 2. 置信度过滤 — 过滤低于阈值的结果
//!
//! # 雕刻算法
//!
//! 借鉴区间几何学：当低优先级大区间与高优先级小区间重叠时，不丢弃任何一个。
//! 大区间沿高优先级边界切开，分裂为左右两块，中间保留高优先级区间。
//!
//! ```text
//! 低优区间 (AI):      [─── ADDRESS ───────────────────]
//! 高优区间 (Regex):              [── IP ──]
//! 雕刻后:             [─ ADDRESS ─][── IP ──][─ ADDRESS ─]
//! ```

use crate::core::recognizer::{EntitySpan, EntityType};
use log::debug;

/// 冲突解决器
pub struct ConflictResolver {
    /// 置信度阈值
    confidence_threshold: f32,
}

/// 判断实体类型是否为容器类型（可吞没子项）
fn is_container_type(entity_type: &EntityType) -> bool {
    match entity_type {
        EntityType::Address => true,
        EntityType::Custom(s) => {
            let lower = s.to_lowercase();
            lower == "organization" || lower == "company"
        }
        _ => false,
    }
}

/// 判断容器能否吞没给定子项
/// 高风险类型（Phone, Email 等）永不吞没
fn can_swallow(container: &EntityType, child: &EntityType) -> bool {
    if !is_container_type(container) { return false; }
    !matches!(child,
        EntityType::Phone | EntityType::Email | EntityType::IdCard
        | EntityType::BankCard | EntityType::ApiKey | EntityType::Password
    )
}

/// 判断跨度是否为无意义碎片（空白/纯标点/单字符碎屑/非法 UTF-8）
fn is_useless_fragment(span: &EntitySpan, text: &[u8]) -> bool {
    if span.start >= span.end || span.end > text.len() { return true; }
    let fragment_bytes = &text[span.start..span.end];

    if let Ok(s) = std::str::from_utf8(fragment_bytes) {
        let trimmed = s.trim();
        if trimmed.is_empty() { return true; }

        if trimmed.chars().all(|c| {
            c.is_ascii_punctuation()
            || "，。：；！？、‘’“”【】（）—《》".contains(c)
        }) {
            return true;
        }

        let char_count = trimmed.chars().count();
        if char_count < 2
            && matches!(span.entity_type,
                EntityType::Address | EntityType::Person | EntityType::BankCard
            ) {
                return true;
        }
    } else {
        return true;
    }

    false
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

    /// 子区间雕刻合并（Sub-span Carving）
    ///
    /// # 处理流程
    ///
    /// 1. 过滤低置信度 span
    /// 2. 按优先级降序 → 置信度降序 → 宽度降序排序（高优先处理，成为"刀"）
    /// 3. 逐个将待处理区间与已接受的高优区间进行几何碰撞检测，执行雕刻
    /// 4. 最终按起始位置升序输出
    pub fn resolve(&self, spans: Vec<EntitySpan>, text: &[u8]) -> Vec<EntitySpan> {
        let input_count = spans.len();
        if spans.is_empty() {
            return spans;
        }

        // Step 1: 过滤低置信度
        let mut candidates: Vec<EntitySpan> = spans
            .into_iter()
            .filter(|s| s.confidence >= self.confidence_threshold)
            .collect();

        let filtered_count = candidates.len();
        if candidates.is_empty() {
            return candidates;
        }

        // Step 2: 排序 — 高优先级在前（成为 carving 的"刀"），同优先级宽区间优先
        candidates.sort_by(|a, b| {
            b.priority
                .cmp(&a.priority)
                .then(
                    b.confidence
                        .partial_cmp(&a.confidence)
                        .unwrap_or(std::cmp::Ordering::Equal),
                )
                .then((b.end - b.start).cmp(&(a.end - a.start)))
        });

        let mut accepted: Vec<EntitySpan> = Vec::new();

        // Step 3: 雕刻合并 + 容器吞没
        for candidate in candidates {
            // ── 容器吞没：低优容器吞没已接受的可吞子项 ──
            if is_container_type(&candidate.entity_type) {
                accepted.retain(|child| {
                    !(child.start >= candidate.start
                        && child.end <= candidate.end
                        && can_swallow(&candidate.entity_type, &child.entity_type))
                });
            }

            let mut fragments = vec![candidate];

            for accepted_span in &accepted {
                let mut next_fragments = Vec::new();

                for frag in fragments {
                    if !frag.overlaps_with(accepted_span) {
                        next_fragments.push(frag);
                        continue;
                    }

                    // ── 四种重叠几何情况 ──

                    // A: accepted 完全包含 frag → 丢弃 frag
                    if accepted_span.start <= frag.start && accepted_span.end >= frag.end {
                        continue;
                    }

                    // B: frag 完全包含 accepted → 分裂为左右两块
                    if accepted_span.start > frag.start && accepted_span.end < frag.end {
                        let mut left = frag.clone();
                        left.end = accepted_span.start;
                        let mut right = frag.clone();
                        right.start = accepted_span.end;
                        next_fragments.push(left);
                        next_fragments.push(right);
                        continue;
                    }

                    // C: 左重叠 (accepted 覆盖 frag 左半) → 保留右半
                    if accepted_span.start <= frag.start && accepted_span.end < frag.end {
                        let mut right = frag.clone();
                        right.start = accepted_span.end;
                        next_fragments.push(right);
                        continue;
                    }

                    // D: 右重叠 (accepted 覆盖 frag 右半) → 保留左半
                    if accepted_span.start > frag.start && accepted_span.end >= frag.end {
                        let mut left = frag.clone();
                        left.end = accepted_span.start;
                        next_fragments.push(left);
                        continue;
                    }
                }

                fragments = next_fragments;
            }

            accepted.extend(fragments);
        }

        // 过滤零长度碎片
        accepted.retain(|s| s.start < s.end);
        // 碎片清除：删除无意义的空白/标点/单字碎屑
        accepted.retain(|s| !is_useless_fragment(s, text));

        debug!(
            "conflict resolve done: input={} filtered={} output={}",
            input_count, filtered_count, accepted.len()
        );

        // Step 4: 按起始位置升序排列（便于脱敏引擎顺序替换）
        accepted.sort_by_key(|s| s.start);
        accepted
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

    fn make_span_pri(
        start: usize,
        end: usize,
        confidence: f32,
        priority: i32,
        source: &str,
    ) -> EntitySpan {
        EntitySpan::with_mask(
            start,
            end,
            EntityType::Email,
            confidence,
            source,
            "<EMAIL>",
        )
        .with_priority(priority)
    }

    // ── 基本无重叠 ──

    #[test]
    fn test_no_overlap() {
        let text = "abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ";
        let resolver = ConflictResolver::new(0.0);
        let spans = vec![make_span(0, 5, 1.0, "a"), make_span(10, 15, 1.0, "b")];
        let result = resolver.resolve(spans, text.as_bytes());
        assert_eq!(result.len(), 2);
    }

    // ── 雕刻：内部分裂（B） ──

    #[test]
    fn test_carving_inner_split() {
        let text = "abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ";
        // 低优宽区间包含高优窄区间 → 分裂低优为左右两块
        let resolver = ConflictResolver::new(0.0);
        let spans = vec![
            make_span_pri(0, 40, 0.7, 50, "ner"),    // 低优
            make_span_pri(15, 25, 0.9, 100, "regex"), // 高优
        ];
        let result = resolver.resolve(spans, text.as_bytes());
        // 期望: ner(0..15) + regex(15..25) + ner(25..40)
        assert_eq!(result.len(), 3);
        assert_eq!(result[0].source, "ner");
        assert_eq!(result[0].start, 0);
        assert_eq!(result[0].end, 15);
        assert_eq!(result[1].source, "regex");
        assert_eq!(result[1].start, 15);
        assert_eq!(result[1].end, 25);
        assert_eq!(result[2].source, "ner");
        assert_eq!(result[2].start, 25);
        assert_eq!(result[2].end, 40);
    }

    // ── 雕刻：左重叠（C） ──

    #[test]
    fn test_carving_left_overlap() {
        let text = "abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ";
        // 高优区间覆盖低优区间左半 → 保留右半
        let resolver = ConflictResolver::new(0.0);
        let spans = vec![
            make_span_pri(5, 30, 0.8, 50, "ner"),
            make_span_pri(0, 20, 0.9, 100, "regex"),
        ];
        let result = resolver.resolve(spans, text.as_bytes());
        // 期望: regex(0..20) + ner(20..30)
        assert_eq!(result.len(), 2);
        assert_eq!(result[0].source, "regex");
        assert_eq!(result[0].start, 0);
        assert_eq!(result[0].end, 20);
        assert_eq!(result[1].source, "ner");
        assert_eq!(result[1].start, 20);
        assert_eq!(result[1].end, 30);
    }

    // ── 雕刻：右重叠（D） ──

    #[test]
    fn test_carving_right_overlap() {
        let text = "abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ";
        // 高优区间覆盖低优区间右半 → 保留左半
        let resolver = ConflictResolver::new(0.0);
        let spans = vec![
            make_span_pri(10, 35, 0.8, 50, "ner"),
            make_span_pri(20, 40, 0.9, 100, "regex"),
        ];
        let result = resolver.resolve(spans, text.as_bytes());
        // 期望: ner(10..20) + regex(20..40)
        assert_eq!(result.len(), 2);
        assert_eq!(result[0].source, "ner");
        assert_eq!(result[0].start, 10);
        assert_eq!(result[0].end, 20);
        assert_eq!(result[1].source, "regex");
        assert_eq!(result[1].start, 20);
        assert_eq!(result[1].end, 40);
    }

    // ── 雕刻：完全包含（A） ──

    #[test]
    fn test_carving_full_containment() {
        let text = "abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ";
        // 高优完全包含低优 → 丢弃低优
        let resolver = ConflictResolver::new(0.0);
        let spans = vec![
            make_span_pri(10, 15, 0.6, 50, "ner"),
            make_span_pri(0, 30, 0.9, 100, "regex"),
        ];
        let result = resolver.resolve(spans, text.as_bytes());
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].source, "regex");
        assert_eq!(result[0].start, 0);
        assert_eq!(result[0].end, 30);
    }

    // ── 置信度过滤 ──

    #[test]
    fn test_confidence_filter() {
        let text = "abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ";
        let resolver = ConflictResolver::new(0.7);
        let spans = vec![make_span(0, 5, 0.5, "low"), make_span(10, 15, 0.9, "high")];
        let result = resolver.resolve(spans, text.as_bytes());
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].source, "high");
    }

    // ── 空输入 ──

    #[test]
    fn test_empty_input() {
        let text = "";
        let resolver = ConflictResolver::new(0.0);
        let result = resolver.resolve(vec![], text.as_bytes());
        assert!(result.is_empty());
    }

    // ── 三路连锁雕刻 ──

    #[test]
    fn test_carving_three_way() {
        let text = "abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ";
        // A(0..25, pri=50) 包含 B(10..15, pri=100) → A 分裂为 (0..10) + (15..25)
        // C(20..30, pri=80) 与 A(15..25) 重叠 → C 切割 A(15..25), A保留 (15..20)
        let resolver = ConflictResolver::new(0.0);
        let spans = vec![
            make_span_pri(0, 25, 0.6, 50, "A"),   // 最低优
            make_span_pri(10, 15, 0.9, 100, "B"), // 最高优
            make_span_pri(20, 30, 0.8, 80, "C"),  // 中优
        ];
        let result = resolver.resolve(spans, text.as_bytes());
        // B(10..15, pri=100) → accepted
        // C(20..30, pri=80) 与 B 不重叠 → accepted
        // A(0..25) 被 B 雕刻 → (0..10) + (15..25)
        //   (15..25) 再被 C 雕刻 → (15..20) 保留
        // 最终: A(0..10) + B(10..15) + A(15..20) + C(20..30)
        assert_eq!(result.len(), 4);
        assert_eq!(result[0].source, "A");
        assert_eq!(result[0].start, 0);
        assert_eq!(result[0].end, 10);
        assert_eq!(result[1].source, "B");
        assert_eq!(result[1].start, 10);
        assert_eq!(result[1].end, 15);
        assert_eq!(result[2].source, "A");
        assert_eq!(result[2].start, 15);
        assert_eq!(result[2].end, 20);
        assert_eq!(result[3].source, "C");
        assert_eq!(result[3].start, 20);
        assert_eq!(result[3].end, 30);
    }

    // ── 优先级优先于置信度 ──

    #[test]
    fn test_carving_priority_over_confidence() {
        let text = "abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ";
        // 低优先级高置信度 vs 高优先级低置信度 → 高优先级胜（雕刻低优先级）
        let resolver = ConflictResolver::new(0.0);
        let spans = vec![
            // 低优先级但高置信度
            make_span_pri(0, 20, 0.95, 50, "ner"),
            // 高优先级但低置信度
            make_span_pri(5, 15, 0.55, 100, "regex"),
        ];
        let result = resolver.resolve(spans, text.as_bytes());
        // regex(5..15, pri=100) 雕刻 ner(0..20, pri=50) → ner(0..5) + regex(5..15) + ner(15..20)
        assert_eq!(result.len(), 3);
        assert_eq!(result[1].source, "regex");
        assert_eq!(result[1].start, 5);
        assert_eq!(result[1].end, 15);
    }

    // ── 容器吞没：Address 吞没 Person ──

    #[test]
    fn test_container_swallow() {
        let text = "123 Main Street, New York, John ";
        let resolver = ConflictResolver::new(0.0);
        let spans = vec![
            EntitySpan::with_mask(0, 30, EntityType::Address, 0.7, "ner", "<ADDRESS>")
                .with_priority(50),
            EntitySpan::with_mask(21, 30, EntityType::Person, 0.9, "ner", "<PERSON>")
                .with_priority(100),
        ];
        let result = resolver.resolve(spans, text.as_bytes());
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].entity_type, EntityType::Address);
        assert_eq!(result[0].start, 0);
        assert_eq!(result[0].end, 30);
    }

    // ── 容器不吞没高风险类型（Phone） ──

    #[test]
    fn test_container_no_swallow_phone() {
        let text = "123 Main Street, New York 555-1234";
        let resolver = ConflictResolver::new(0.0);
        let spans = vec![
            EntitySpan::with_mask(0, 25, EntityType::Address, 0.7, "ner", "<ADDRESS>")
                .with_priority(50),
            EntitySpan::with_mask(17, 29, EntityType::Phone, 0.9, "regex", "<PHONE>")
                .with_priority(100),
        ];
        let result = resolver.resolve(spans, text.as_bytes());
        // Address(0..25) contains Phone(17..29) but should NOT swallow Phone
        // Phone carved Address → Address(0..17) + Phone(17..29)
        assert_eq!(result.len(), 2);
        assert_eq!(result[0].entity_type, EntityType::Address);
        assert_eq!(result[0].start, 0);
        assert_eq!(result[0].end, 17);
        assert_eq!(result[1].entity_type, EntityType::Phone);
        assert_eq!(result[1].start, 17);
        assert_eq!(result[1].end, 29);
    }

    // ── 碎片清除：纯标点 ──

    #[test]
    fn test_fragment_prune_punctuation() {
        let text = "Hello... world";
        let resolver = ConflictResolver::new(0.0);
        let spans = vec![
            make_span(5, 8, 0.8, "regex"), // "..."
        ];
        let result = resolver.resolve(spans, text.as_bytes());
        assert!(result.is_empty());
    }

    // ── 碎片清除：单字符 Person ──

    #[test]
    fn test_fragment_prune_single_char_person() {
        let text = "A person here";
        let resolver = ConflictResolver::new(0.0);
        let spans = vec![
            EntitySpan::with_mask(0, 1, EntityType::Person, 0.6, "ner", "<PERSON>"),
        ];
        let result = resolver.resolve(spans, text.as_bytes());
        assert!(result.is_empty());
    }
}
