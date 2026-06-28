//! NER 识别器
//!
//! 将 `NerEngine` 包装为 `Recognizer` trait 实现，
//! 使其可以无缝集成到 `RecognizerRegistry` 中。
//!
//! # 特点
//!
//! - 懒加载：模型只在第一次 `analyze()` 调用时加载
//! - 线程安全：`NerEngine` 内部使用 `Mutex` 保护 mutable 状态
//! - 可配置：置信度阈值、最大序列长度等

use super::types::*;
use super::Recognizer;
use crate::infra::ai::{NerEngine, ModelManager, ModelState};
use log::{info, warn, error};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use parking_lot::Mutex;

/// NER 识别器
///
/// 包装 `NerEngine`，实现 `Recognizer` trait。
/// 支持懒加载：模型只在第一次调用 `analyze()` 时加载。
pub struct NerRecognizer {
    /// NER 引擎 (懒加载)
    engine: Arc<Mutex<Option<NerEngine>>>,
    /// 模型管理器
    model_manager: Arc<ModelManager>,
    /// 识别器名称
    name: String,
    /// 识别器优先级
    priority: i32,
    /// 置信度阈值
    confidence_threshold: f32,
    /// 是否启用
    enabled: bool,
    /// AI 引擎是否启用（外部控制）
    ai_enabled: Arc<AtomicBool>,
}

impl NerRecognizer {
    /// 创建 NER 识别器
    ///
    /// # 参数
    ///
    /// - `model_manager`: 模型管理器，提供模型路径和状态管理
    /// - `name`: 识别器名称（默认 "ner_engine"）
    /// - `priority`: 识别器优先级（默认 50）
    /// - `confidence_threshold`: 置信度阈值（默认 0.5）
    /// - `ai_enabled`: AI 引擎启用标志（外部控制）
    pub fn new(
        model_manager: Arc<ModelManager>,
        name: Option<&str>,
        priority: Option<i32>,
        confidence_threshold: Option<f32>,
        ai_enabled: Option<Arc<AtomicBool>>,
    ) -> Self {
        Self {
            engine: Arc::new(Mutex::new(None)),
            model_manager,
            name: name.unwrap_or("ner_engine").to_string(),
            priority: priority.unwrap_or(50),
            confidence_threshold: confidence_threshold.unwrap_or(0.5),
            enabled: true,
            ai_enabled: ai_enabled.unwrap_or_else(|| Arc::new(AtomicBool::new(true))),
        }
    }

    /// 检查模型是否已加载（非阻塞）
    ///
    /// 如果模型未加载，触发后台加载并返回 false。
    /// 不会阻塞调用者。
    fn ensure_loaded(&self) -> bool {
        // 检查是否已加载
        {
            let engine = self.engine.lock();
            if engine.is_some() {
                return true;
            }
        }

        // 检查是否正在加载或已失败
        {
            let state = self.model_manager.state();
            match state {
                ModelState::Loading => return false,
                ModelState::Error(_) => return false,
                ModelState::Ready => return true,
                ModelState::NotLoaded => {}
            }
        }

        // 首次调用：标记为加载中，然后返回（不阻塞）
        // 模型会在后台异步加载
        self.model_manager.set_state(ModelState::Loading);

        let model_manager = self.model_manager.clone();
        let engine = self.engine.clone();
        let name = self.name.clone();

        std::thread::spawn(move || {
            // 设置加载超时（5 分钟）
            let timeout = std::time::Duration::from_secs(300);
            let start_time = std::time::Instant::now();
            let log_file = std::path::PathBuf::from("ai_model_load.log");
            let write_log = |msg: &str| {
                let _ = std::fs::OpenOptions::new()
                    .create(true).append(true).open(&log_file)
                    .and_then(|mut f| {
                        use std::io::Write;
                        writeln!(f, "[{}] {}", chrono::Local::now().format("%H:%M:%S"), msg)
                    });
            };

            write_log(&format!("🤖 [{}] 开始后台加载模型...", name));

            let model_paths = match model_manager.first_model_paths() {
                Some(paths) => {
                    write_log(&format!("✅ 找到模型路径: {:?}", paths));
                    paths
                }
                None => {
                    write_log(&format!("❌ [{}] 未找到可用模型", name));
                    model_manager.set_state(ModelState::Error("未找到模型文件".to_string()));
                    return;
                }
            };

            let model_dir = model_paths.0.parent()
                .map(|p| p.to_path_buf())
                .unwrap_or(model_paths.0.clone());

            write_log(&format!("📁 模型目录: {}", model_dir.display()));
            write_log(&format!("📦 model.onnx 大小: {:.1} MB",
                model_dir.join("model.onnx").metadata().map(|m| m.len()).unwrap_or(0) as f64 / 1024.0 / 1024.0));
            write_log(&format!("📦 model.onnx_data 大小: {:.1} MB",
                model_dir.join("model.onnx_data").metadata().map(|m| m.len()).unwrap_or(0) as f64 / 1024.0 / 1024.0));

            write_log("⏳ 正在加载 ONNX 模型...");
            write_log("   （大文件加载可能需要 1-3 分钟）");

            // 使用线程超时控制
            let (tx, rx) = std::sync::mpsc::channel();
            let model_dir_clone = model_dir.clone();

            std::thread::spawn(move || {
                let result = NerEngine::load(&model_dir_clone);
                let _ = tx.send(result);
            });

            // 等待加载完成或超时
            match rx.recv_timeout(timeout) {
                Ok(Ok(ner_engine)) => {
                    let elapsed = start_time.elapsed();
                    write_log(&format!("✅ [{}] 模型加载成功！总耗时: {:.1} 秒", name, elapsed.as_secs_f64()));
                    *engine.lock() = Some(ner_engine);

                    if let Some(metadata) = model_manager.available_models().first() {
                        model_manager.set_active_model(metadata.clone());
                    }
                    model_manager.set_state(ModelState::Ready);
                    write_log("✅ 模型状态已更新为 Ready");
                }
                Ok(Err(e)) => {
                    write_log(&format!("❌ [{}] 模型加载失败: {}", name, e));
                    model_manager.set_state(ModelState::Error(e.to_string()));
                }
                Err(_) => {
                    // 超时
                    let elapsed = start_time.elapsed();
                    write_log(&format!("❌ [{}] 模型加载超时！已用时: {:.1} 秒", name, elapsed.as_secs_f64()));
                    model_manager.set_state(ModelState::Error(format!("加载超时（超过 {} 秒）", timeout.as_secs())));
                }
            }
        });

        false // 首次调用返回 false，不阻塞
    }
}

impl Recognizer for NerRecognizer {
    fn name(&self) -> &str {
        &self.name
    }

    fn recognizer_type(&self) -> RecognizerType {
        RecognizerType::Ai
    }

    fn supported_entities(&self) -> Vec<EntityType> {
        vec![
            EntityType::Person,
            EntityType::Email,
            EntityType::Phone,
            EntityType::Address,
            EntityType::BankCard,
            EntityType::DateOfBirth,
            EntityType::Url,
            EntityType::ApiKey,
        ]
    }

    fn analyze(&self, context: &AnalysisContext) -> AnalysisResult {
        let start = std::time::Instant::now();

        // 检查 AI 引擎是否被外部禁用
        if !self.ai_enabled.load(Ordering::SeqCst) {
            return AnalysisResult::empty(&self.name);
        }

        // 确保模型已加载
        if !self.ensure_loaded() {
            return AnalysisResult::empty(&self.name);
        }

        // 获取文本
        let text = match context.as_str() {
            Some(s) => s,
            None => {
                warn!("⚠️ [{}] 非 UTF-8 文本，跳过分析", self.name);
                return AnalysisResult::empty(&self.name);
            }
        };

        // 执行推理
        let mut engine_guard = self.engine.lock();
        let engine = engine_guard.as_mut().unwrap();

        match engine.infer(text) {
            Ok(mut spans) => {
                // 过滤低置信度结果
                spans.retain(|s| s.confidence >= self.confidence_threshold);

                let elapsed = start.elapsed();
                AnalysisResult {
                    spans,
                    elapsed_us: elapsed.as_micros() as u64,
                    recognizer: self.name.clone(),
                }
            }
            Err(e) => {
                error!("❌ [{}] 推理失败: {}", self.name, e);
                AnalysisResult::empty(&self.name)
            }
        }
    }

    fn priority(&self) -> i32 {
        self.priority
    }

    fn is_enabled(&self) -> bool {
        self.enabled
    }
}
