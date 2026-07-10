//! AI 基础设施层
//!
//! 提供 AI 模型的加载、管理和推理能力。
//! 当前支持 ONNX 格式的 NER 模型。

pub mod model_manager;
pub mod ner_engine;

pub use model_manager::{ModelManager, ModelState};
pub use ner_engine::NerEngine;
