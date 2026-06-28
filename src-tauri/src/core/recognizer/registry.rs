//! 识别器注册表
//!
//! `RecognizerRegistry` 是识别引擎层的核心组件，负责：
//! - 管理所有已注册的识别器
//! - 按优先级排序执行
//! - 支持动态注册/注销
//! - 分离依赖型和非依赖型识别器的执行顺序

use super::types::*;
use super::Recognizer;
use log::{debug, info, warn};
use std::sync::Arc;

/// 识别器注册表
///
/// 管理所有识别器的生命周期和执行顺序。
/// 识别器按优先级降序排列，优先级高的先执行。
///
/// # 线程安全
///
/// `RecognizerRegistry` 本身是 `Send + Sync` 的，
/// 因为所有识别器都要求 `Send + Sync`。
pub struct RecognizerRegistry {
    /// 已注册的识别器（按优先级降序排列）
    recognizers: Vec<RegisteredRecognizer>,
    /// 注册表配置
    config: RegistryConfig,
}

/// 已注册的识别器包装
struct RegisteredRecognizer {
    /// 识别器实例
    recognizer: Box<dyn Recognizer>,
    /// 是否启用
    enabled: bool,
    /// 执行顺序（由优先级决定）
    order: usize,
}

/// 注册表配置
#[derive(Debug, Clone)]
pub struct RegistryConfig {
    /// 全局置信度阈值（低于此值的结果将被过滤）
    pub global_confidence_threshold: f32,
    /// 是否启用性能追踪
    pub enable_tracing: bool,
    /// 最大并行识别器数（预留，未来使用）
    pub max_parallel: usize,
}

impl Default for RegistryConfig {
    fn default() -> Self {
        Self {
            global_confidence_threshold: 0.5,
            enable_tracing: true,
            max_parallel: 4,
        }
    }
}

impl RecognizerRegistry {
    /// 创建空的注册表
    pub fn new(config: RegistryConfig) -> Self {
        Self {
            recognizers: Vec::new(),
            config,
        }
    }

    /// 使用默认配置创建注册表
    pub fn default_config() -> Self {
        Self::new(RegistryConfig::default())
    }

    /// 注册一个识别器
    ///
    /// 识别器将根据其 `priority()` 值自动排序。
    /// 如果已存在同名识别器，将发出警告但仍会注册。
    pub fn register(&mut self, recognizer: Box<dyn Recognizer>) {
        let name = recognizer.name().to_string();
        let priority = recognizer.priority();
        let enabled = recognizer.is_enabled();

        info!(
            "📋 注册识别器: {} (优先级: {}, 类型: {:?}, 启用: {})",
            name,
            priority,
            recognizer.recognizer_type(),
            enabled
        );

        self.recognizers.push(RegisteredRecognizer {
            recognizer,
            enabled,
            order: 0, // 稍后重新排序
        });

        // 重新排序（优先级降序）
        self.resort();
    }

    /// 批量注册识别器
    pub fn register_all(&mut self, recognizers: Vec<Box<dyn Recognizer>>) {
        for r in recognizers {
            self.register(r);
        }
    }

    /// 注销指定名称的识别器
    ///
    /// 返回是否成功注销。
    pub fn unregister(&mut self, name: &str) -> bool {
        let before = self.recognizers.len();
        self.recognizers.retain(|r| r.recognizer.name() != name);
        let removed = before - self.recognizers.len();

        if removed > 0 {
            info!("🗑️ 注销识别器: {} (移除 {} 个)", name, removed);
            self.resort();
            true
        } else {
            warn!("⚠️ 尝试注销不存在的识别器: {}", name);
            false
        }
    }

    /// 启用/禁用指定识别器
    pub fn set_enabled(&mut self, name: &str, enabled: bool) -> bool {
        for r in &mut self.recognizers {
            if r.recognizer.name() == name {
                r.enabled = enabled;
                info!(
                    "🔄 识别器 {} 已{}",
                    name,
                    if enabled { "启用" } else { "禁用" }
                );
                return true;
            }
        }
        false
    }

    /// 获取已注册识别器的数量
    pub fn len(&self) -> usize {
        self.recognizers.len()
    }

    /// 是否为空
    pub fn is_empty(&self) -> bool {
        self.recognizers.is_empty()
    }

    /// 获取所有已注册识别器的名称
    pub fn recognizer_names(&self) -> Vec<&str> {
        self.recognizers
            .iter()
            .map(|r| r.recognizer.name())
            .collect()
    }

    /// 执行所有识别器，返回合并后的结果
    ///
    /// 执行顺序：
    /// 1. 先执行所有非依赖型识别器（`requires_context() == false`）
    /// 2. 将非依赖型结果注入 `AnalysisContext`
    /// 3. 再执行所有依赖型识别器（`requires_context() == true`）
    pub fn analyze(&self, context: &AnalysisContext) -> Vec<EntitySpan> {
        let mut all_spans = Vec::new();

        // Phase 1: 非依赖型识别器
        for registered in &self.recognizers {
            if !registered.enabled {
                continue;
            }
            if registered.recognizer.requires_context() {
                continue; // 跳过依赖型，下一轮处理
            }

            let result = self.run_recognizer(registered, context);
            all_spans.extend(result.spans);
        }

        // Phase 2: 依赖型识别器（携带前置结果）
        let mut context_with_spans = AnalysisContext::from_text(context.text)
            .with_previous_spans(all_spans.clone());

        if let Some(lang) = context.language {
            context_with_spans = context_with_spans.with_language(lang);
        }
        if let Some(ft) = &context.file_type {
            context_with_spans = context_with_spans.with_file_type(ft);
        }

        for registered in &self.recognizers {
            if !registered.enabled {
                continue;
            }
            if !registered.recognizer.requires_context() {
                continue; // 跳过非依赖型，已在上一轮处理
            }

            let result = self.run_recognizer(registered, &context_with_spans);
            all_spans.extend(result.spans);
        }

        // 过滤低置信度结果
        all_spans.retain(|s| s.confidence >= self.config.global_confidence_threshold);

        debug!(
            "📊 识别完成: 共 {} 个实体跨度",
            all_spans.len()
        );

        all_spans
    }

    /// 执行单个识别器并追踪性能
    fn run_recognizer(
        &self,
        registered: &RegisteredRecognizer,
        context: &AnalysisContext,
    ) -> AnalysisResult {
        let start = std::time::Instant::now();
        let name = registered.recognizer.name().to_string();

        let mut result = registered.recognizer.analyze(context);

        if self.config.enable_tracing {
            let elapsed = start.elapsed();
            result.elapsed_us = elapsed.as_micros() as u64;
            result.recognizer = name.clone();

            if elapsed.as_millis() > 100 {
                warn!(
                    "⏱️ 识别器 {} 耗时较长: {:.2}ms",
                    name,
                    elapsed.as_secs_f64() * 1000.0
                );
            } else {
                debug!(
                    "⏱️ 识别器 {} 耗时: {:.2}ms, 结果: {} 个实体",
                    name,
                    elapsed.as_secs_f64() * 1000.0,
                    result.spans.len()
                );
            }
        }

        result
    }

    /// 重新排序识别器（优先级降序）
    fn resort(&mut self) {
        self.recognizers
            .sort_by(|a, b| b.recognizer.priority().cmp(&a.recognizer.priority()));

        // 更新执行顺序
        for (i, r) in self.recognizers.iter_mut().enumerate() {
            r.order = i;
        }
    }

    /// 获取注册表配置的可变引用
    pub fn config_mut(&mut self) -> &mut RegistryConfig {
        &mut self.config
    }

    /// 更新全局置信度阈值
    pub fn set_confidence_threshold(&mut self, threshold: f32) {
        self.config.global_confidence_threshold = threshold.clamp(0.0, 1.0);
        info!(
            "🎯 全局置信度阈值更新为: {:.2}",
            self.config.global_confidence_threshold
        );
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// 测试
// ─────────────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    /// 测试用的简单识别器
    struct TestRecognizer {
        name: String,
        priority: i32,
        entity_type: EntityType,
        pattern: String,
    }

    impl TestRecognizer {
        fn new(name: &str, priority: i32, entity_type: EntityType, pattern: &str) -> Self {
            Self {
                name: name.to_string(),
                priority,
                entity_type,
                pattern: pattern.to_string(),
            }
        }
    }

    impl Recognizer for TestRecognizer {
        fn name(&self) -> &str {
            &self.name
        }

        fn recognizer_type(&self) -> RecognizerType {
            RecognizerType::Rule
        }

        fn supported_entities(&self) -> Vec<EntityType> {
            vec![self.entity_type.clone()]
        }

        fn analyze(&self, context: &AnalysisContext) -> AnalysisResult {
            let text = std::str::from_utf8(context.text).unwrap_or("");
            let mut spans = Vec::new();

            if let Some(pos) = text.find(&self.pattern) {
                spans.push(EntitySpan::new(
                    pos,
                    pos + self.pattern.len(),
                    self.entity_type.clone(),
                    1.0,
                    &self.name,
                ));
            }

            AnalysisResult {
                spans,
                elapsed_us: 0,
                recognizer: self.name.clone(),
            }
        }

        fn priority(&self) -> i32 {
            self.priority
        }
    }

    #[test]
    fn test_registry_register_and_analyze() {
        let mut registry = RecognizerRegistry::default_config();

        registry.register(Box::new(TestRecognizer::new(
            "test_email",
            100,
            EntityType::Email,
            "test@example.com",
        )));

        registry.register(Box::new(TestRecognizer::new(
            "test_phone",
            90,
            EntityType::Phone,
            "13800138000",
        )));

        assert_eq!(registry.len(), 2);

        let context = AnalysisContext::from_text(b"email: test@example.com, phone: 13800138000");
        let spans = registry.analyze(&context);

        assert_eq!(spans.len(), 2);
        assert!(spans.iter().any(|s| s.entity_type == EntityType::Email));
        assert!(spans.iter().any(|s| s.entity_type == EntityType::Phone));
    }

    #[test]
    fn test_registry_priority_order() {
        let mut registry = RecognizerRegistry::default_config();

        // 低优先级先注册
        registry.register(Box::new(TestRecognizer::new(
            "low_priority",
            10,
            EntityType::Custom("low".to_string()),
            "test",
        )));

        // 高优先级后注册
        registry.register(Box::new(TestRecognizer::new(
            "high_priority",
            100,
            EntityType::Custom("high".to_string()),
            "test",
        )));

        let names = registry.recognizer_names();
        assert_eq!(names[0], "high_priority");
        assert_eq!(names[1], "low_priority");
    }

    #[test]
    fn test_registry_unregister() {
        let mut registry = RecognizerRegistry::default_config();

        registry.register(Box::new(TestRecognizer::new(
            "test",
            100,
            EntityType::Email,
            "test",
        )));

        assert_eq!(registry.len(), 1);
        assert!(registry.unregister("test"));
        assert_eq!(registry.len(), 0);
        assert!(!registry.unregister("nonexistent"));
    }

    #[test]
    fn test_registry_confidence_filter() {
        let mut registry = RecognizerRegistry::default_config();
        registry.set_confidence_threshold(0.8);

        // 创建一个低置信度的识别器
        struct LowConfidenceRecognizer;
        impl Recognizer for LowConfidenceRecognizer {
            fn name(&self) -> &str { "low_conf" }
            fn recognizer_type(&self) -> RecognizerType { RecognizerType::Ai }
            fn supported_entities(&self) -> Vec<EntityType> { vec![EntityType::Person] }
            fn analyze(&self, _context: &AnalysisContext) -> AnalysisResult {
                AnalysisResult {
                    spans: vec![EntitySpan::new(0, 4, EntityType::Person, 0.3, "low_conf")],
                    elapsed_us: 0,
                    recognizer: "low_conf".to_string(),
                }
            }
        }

        registry.register(Box::new(LowConfidenceRecognizer));

        let context = AnalysisContext::from_text(b"John");
        let spans = registry.analyze(&context);

        // 低置信度结果应被过滤
        assert_eq!(spans.len(), 0);
    }
}
