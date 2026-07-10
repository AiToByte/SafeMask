//! 模型管理器
//!
//! 负责 AI 模型的生命周期管理：
//! - 模型发现（扫描模型目录）
//! - 懒加载（首次使用时才加载）
//! - 状态管理（未加载、加载中、就绪、错误）
//! - 模型元数据

use log::{info, warn, error};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use parking_lot::RwLock;
use serde::{Serialize, Deserialize};

/// 模型状态
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ModelState {
    /// 未加载
    NotLoaded,
    /// 加载中
    Loading,
    /// 就绪
    Ready,
    /// 错误
    Error(String),
}

/// 模型元数据
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelMetadata {
    /// 模型名称
    pub name: String,
    /// 模型版本
    pub version: String,
    /// 模型文件路径
    pub model_path: PathBuf,
    /// Tokenizer 文件路径
    pub tokenizer_path: PathBuf,
    /// 模型文件大小 (字节)
    pub model_size_bytes: u64,
    /// 支持的实体类型
    pub entity_types: Vec<String>,
    /// 模型描述
    pub description: String,
}

/// 模型管理器
///
/// 管理所有可用的 AI 模型。支持懒加载：
/// 模型只在第一次被请求时才加载到内存。
pub struct ModelManager {
    /// 模型目录
    models_dir: PathBuf,
    /// 已发现的模型元数据
    available_models: Vec<ModelMetadata>,
    /// 当前活跃模型的状态
    active_state: Arc<RwLock<ModelState>>,
    /// 当前活跃模型的元数据
    active_metadata: Arc<RwLock<Option<ModelMetadata>>>,
}

impl ModelManager {
    /// 创建模型管理器
    ///
    /// # 参数
    ///
    /// - `models_dir`: 模型存放目录
    pub fn new(models_dir: impl AsRef<Path>) -> Self {
        let models_dir = models_dir.as_ref().to_path_buf();

        info!("📦 模型管理器初始化，目录: {}", models_dir.display());

        let mut manager = Self {
            models_dir,
            available_models: Vec::new(),
            active_state: Arc::new(RwLock::new(ModelState::NotLoaded)),
            active_metadata: Arc::new(RwLock::new(None)),
        };

        // 扫描可用模型
        manager.discover_models();

        manager
    }

    /// 扫描模型目录，发现可用模型
    fn discover_models(&mut self) {
        // 调试：写入日志
        let debug_log = std::path::PathBuf::from("models_debug.log");
        let _ = std::fs::OpenOptions::new().create(true).append(true).open(&debug_log)
            .and_then(|mut f| {
                use std::io::Write;
                writeln!(f, "\n[ModelManager] 扫描目录: {} | 存在: {}", self.models_dir.display(), self.models_dir.exists())
            });

        if !self.models_dir.exists() {
            info!("📁 模型目录不存在，跳过扫描: {}", self.models_dir.display());
            return;
        }

        let mut models = Vec::new();

        // 优先检测根目录直接放置的模型文件（下载解压后的常见布局）
        let root_model_file = if self.models_dir.join("model_q4.onnx").exists() {
            Some(self.models_dir.join("model_q4.onnx"))
        } else if self.models_dir.join("model.onnx").exists() {
            Some(self.models_dir.join("model.onnx"))
        } else {
            None
        };
        if let Some(ref model_file) = root_model_file {
            let tokenizer_file = self.models_dir.join("tokenizer.json");
            if tokenizer_file.exists() {
                let model_size = std::fs::metadata(model_file)
                    .map(|m| m.len())
                    .unwrap_or(0);
                models.push(ModelMetadata {
                    name: "privacy-filter".to_string(),
                    version: "1.0".to_string(),
                    model_path: model_file.clone(),
                    tokenizer_path: tokenizer_file,
                    model_size_bytes: model_size,
                    entity_types: Self::default_entity_types(),
                    description: "AI NER 模型（根目录）".to_string(),
                });
                info!("🔍 发现根目录模型: privacy-filter ({:.1} MB)", model_size as f64 / 1024.0 / 1024.0);
            }
        }

        // 扫描子目录，每个子目录是一个模型
        if let Ok(entries) = std::fs::read_dir(&self.models_dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if !path.is_dir() {
                    continue;
                }

                // 优先查找 q4 量化版本
                let model_file = if path.join("model_q4.onnx").exists() {
                    path.join("model_q4.onnx")
                } else {
                    path.join("model.onnx")
                };
                let tokenizer_file = path.join("tokenizer.json");

                // 调试
                let debug_log = std::path::PathBuf::from("models_debug.log");
                let _ = std::fs::OpenOptions::new().create(true).append(true).open(&debug_log)
                    .and_then(|mut f| {
                        use std::io::Write;
                        writeln!(f, "[discover] 子目录: {} | model:{} | tokenizer:{}",
                            path.display(), model_file.file_name().unwrap_or_default().to_string_lossy(), tokenizer_file.exists())
                    });

                if model_file.exists() && tokenizer_file.exists() {
                    let name = path
                        .file_name()
                        .map(|n| n.to_string_lossy().to_string())
                        .unwrap_or_else(|| "unknown".to_string());

                    let model_size = std::fs::metadata(&model_file)
                        .map(|m| m.len())
                        .unwrap_or(0);

                    models.push(ModelMetadata {
                        name: name.clone(),
                        version: "1.0".to_string(),
                        model_path: model_file,
                        tokenizer_path: tokenizer_file,
                        model_size_bytes: model_size,
                        entity_types: Self::default_entity_types(),
                        description: format!("AI NER 模型: {}", name),
                    });

                    info!("🔍 发现模型: {} ({:.1} MB)", name, model_size as f64 / 1024.0 / 1024.0);
                }
            }
        }

        self.available_models = models;

        // 调试
        let debug_log = std::path::PathBuf::from("models_debug.log");
        let _ = std::fs::OpenOptions::new().create(true).append(true).open(&debug_log)
            .and_then(|mut f| {
                use std::io::Write;
                writeln!(f, "[discover] 发现模型数量: {}", self.available_models.len())
            });

        if self.available_models.is_empty() {
            info!("⚠️ 未发现任何模型。请将模型文件放置在: {}", self.models_dir.display());
        }
    }

    /// 获取默认支持的实体类型
    fn default_entity_types() -> Vec<String> {
        vec![
            "person".to_string(),
            "email".to_string(),
            "phone".to_string(),
            "address".to_string(),
            "account_number".to_string(),
            "date".to_string(),
            "url".to_string(),
            "secret".to_string(),
        ]
    }

    /// 获取可用模型列表
    pub fn available_models(&self) -> &[ModelMetadata] {
        &self.available_models
    }

    /// 是否有可用模型
    pub fn has_models(&self) -> bool {
        !self.available_models.is_empty()
    }

    /// 获取当前模型状态
    pub fn state(&self) -> ModelState {
        self.active_state.read().clone()
    }

    /// 获取当前模型元数据
    pub fn metadata(&self) -> Option<ModelMetadata> {
        self.active_metadata.read().clone()
    }

    /// 设置模型状态
    pub fn set_state(&self, state: ModelState) {
        *self.active_state.write() = state;
    }

    /// 设置活跃模型
    pub fn set_active_model(&self, metadata: ModelMetadata) {
        info!("🎯 设置活跃模型: {}", metadata.name);
        *self.active_metadata.write() = Some(metadata);
    }

    /// 获取第一个可用模型的路径
    pub fn first_model_paths(&self) -> Option<(PathBuf, PathBuf)> {
        self.available_models.first().map(|m| {
            (m.model_path.clone(), m.tokenizer_path.clone())
        })
    }

    /// 获取模型目录
    pub fn models_dir(&self) -> &Path {
        &self.models_dir
    }

    /// 获取模型状态信息（用于前端展示）
    pub fn status_info(&self) -> serde_json::Value {
        let state = self.state();
        let metadata = self.metadata()
            .or_else(|| self.available_models.first().cloned());

        serde_json::json!({
            "state": match &state {
                ModelState::NotLoaded => "not_loaded",
                ModelState::Loading => "loading",
                ModelState::Ready => "ready",
                ModelState::Error(_) => "error",
            },
            "error": match &state {
                ModelState::Error(e) => Some(e),
                _ => None,
            },
            "model": metadata.as_ref().map(|m| serde_json::json!({
                "name": m.name,
                "version": m.version,
                "size_mb": m.model_size_bytes as f64 / 1024.0 / 1024.0,
                "entity_types": m.entity_types,
            })),
            "available_count": self.available_models.len(),
            "models_dir": self.models_dir.display().to_string(),
        })
    }
}
