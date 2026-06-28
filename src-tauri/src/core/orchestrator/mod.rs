//! 业务编排层
//!
//! 将底层识别和脱敏能力组合为用户可理解的业务流程。
//!
//! # 场景模式
//!
//! | 模式 | 说明 | 行为 |
//! |------|------|------|
//! | Shadow | 影子模式 | 复制不脱敏，粘贴时脱敏 |
//! | Sentry | 哨兵模式 | 复制即脱敏 |
//! | Batch | 批量模式 | 文件批量处理 |
//!
//! # 设计原则
//!
//! - 用户不需要理解技术细节，只需选择场景模式
//! - 系统自动编排底层能力
//! - 所有操作都是可审计的

pub mod scene;

pub use scene::{SceneMode, SceneConfig};

use crate::core::hybrid_engine::HybridEngine;
use crate::core::recognizer::MaskResult;
use crate::core::masking::MaskConfig;
use log::info;
use std::sync::Arc;
use parking_lot::RwLock;

/// 业务编排器
///
/// 组合识别引擎和脱敏策略，提供高级业务接口。
pub struct Orchestrator {
    /// 混合识别引擎
    engine: Arc<RwLock<HybridEngine>>,
    /// 当前场景模式
    scene_mode: SceneMode,
    /// 场景配置
    scene_config: SceneConfig,
}

impl Orchestrator {
    /// 创建业务编排器
    pub fn new(engine: Arc<RwLock<HybridEngine>>) -> Self {
        Self {
            engine,
            scene_mode: SceneMode::Shadow, // 默认影子模式
            scene_config: SceneConfig::default(),
        }
    }

    /// 切换场景模式
    pub fn set_scene_mode(&mut self, mode: SceneMode) {
        info!("🔄 切换场景模式: {} → {}", self.scene_mode.name(), mode.name());
        self.scene_mode = mode;
    }

    /// 获取当前场景模式
    pub fn scene_mode(&self) -> SceneMode {
        self.scene_mode
    }

    /// 更新场景配置
    pub fn update_scene_config(&mut self, config: SceneConfig) {
        self.scene_config = config;
    }

    /// 一键脱敏文本
    ///
    /// 这是最常用的业务接口，用户只需要传入文本，
    /// 系统自动完成识别和脱敏。
    pub fn mask_text(&self, text: &str) -> MaskResult {
        let engine = self.engine.read();
        engine.analyze(text)
    }

    /// 仅检测（不脱敏）
    ///
    /// 返回识别到的实体列表，用于预览或统计。
    pub fn detect_only(&self, text: &str) -> Vec<crate::core::recognizer::EntitySpan> {
        let engine = self.engine.read();
        engine.detect(text.as_bytes())
    }

    /// 处理剪贴板内容（根据场景模式）
    ///
    /// 根据当前场景模式决定是否脱敏：
    /// - Shadow 模式: 返回原始文本（不脱敏）
    /// - Sentry 模式: 返回脱敏后的文本
    pub fn process_clipboard(&self, text: &str) -> ClipboardProcessResult {
        match self.scene_mode {
            SceneMode::Shadow => {
                // 影子模式：不脱敏，但记录检测结果
                let spans = self.detect_only(text);
                ClipboardProcessResult {
                    original: text.to_string(),
                    processed: text.to_string(),
                    has_privacy: !spans.is_empty(),
                    entities: spans,
                    mode: SceneMode::Shadow,
                }
            }
            SceneMode::Sentry => {
                // 哨兵模式：立即脱敏
                let result = self.mask_text(text);
                ClipboardProcessResult {
                    original: result.original,
                    processed: result.masked,
                    has_privacy: result.has_changes,
                    entities: result.entities,
                    mode: SceneMode::Sentry,
                }
            }
        }
    }

    /// 获取引擎状态信息
    pub fn engine_status(&self) -> serde_json::Value {
        let engine = self.engine.read();
        serde_json::json!({
            "scene_mode": self.scene_mode.name(),
            "rule_count": engine.rule_count(),
            "ai_status": engine.ai_status(),
            "recognizers": engine.registry().recognizer_names(),
        })
    }

    /// 更新脱敏配置
    pub fn update_masking_config(&mut self, config: MaskConfig) {
        let mut engine = self.engine.write();
        engine.update_masking_config(config);
    }

    /// 获取引擎的引用（用于高级操作）
    pub fn engine(&self) -> &Arc<RwLock<HybridEngine>> {
        &self.engine
    }
}

/// 剪贴板处理结果
#[derive(Debug, Clone)]
pub struct ClipboardProcessResult {
    /// 原始文本
    pub original: String,
    /// 处理后的文本
    pub processed: String,
    /// 是否包含隐私信息
    pub has_privacy: bool,
    /// 识别到的实体
    pub entities: Vec<crate::core::recognizer::EntitySpan>,
    /// 使用的场景模式
    pub mode: SceneMode,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::rules::Rule;

    fn create_test_engine() -> Arc<RwLock<HybridEngine>> {
        let rules = vec![
            Rule {
                name: "email".to_string(),
                pattern: r"[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}".to_string(),
                mask: "<EMAIL>".to_string(),
                priority: 0,
                enabled: true,
                is_custom: false,
            },
        ];
        Arc::new(RwLock::new(HybridEngine::from_rules(rules)))
    }

    #[test]
    fn test_orchestrator_shadow_mode() {
        let engine = create_test_engine();
        let mut orchestrator = Orchestrator::new(engine);
        orchestrator.set_scene_mode(SceneMode::Shadow);

        let result = orchestrator.process_clipboard("test@example.com");
        assert_eq!(result.mode, SceneMode::Shadow);
        assert!(result.has_privacy); // 检测到隐私
        assert_eq!(result.original, result.processed); // 但不脱敏
    }

    #[test]
    fn test_orchestrator_sentry_mode() {
        let engine = create_test_engine();
        let mut orchestrator = Orchestrator::new(engine);
        orchestrator.set_scene_mode(SceneMode::Sentry);

        let result = orchestrator.process_clipboard("test@example.com");
        assert_eq!(result.mode, SceneMode::Sentry);
        assert!(result.has_privacy);
        assert!(result.processed.contains("<EMAIL>")); // 脱敏了
    }

    #[test]
    fn test_mask_text() {
        let engine = create_test_engine();
        let orchestrator = Orchestrator::new(engine);

        let result = orchestrator.mask_text("联系我 test@example.com");
        assert!(result.has_changes);
        assert!(result.masked.contains("<EMAIL>"));
    }

    #[test]
    fn test_detect_only() {
        let engine = create_test_engine();
        let orchestrator = Orchestrator::new(engine);

        let spans = orchestrator.detect_only("联系我 test@example.com");
        assert_eq!(spans.len(), 1);
        assert_eq!(spans[0].entity_type, crate::core::recognizer::EntityType::Email);
    }
}
