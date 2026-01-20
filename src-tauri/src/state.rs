use std::sync::{Arc, Mutex};
use once_cell::sync::Lazy;
use crate::engine::MaskEngine;
use crate::config::RuleManager;

// 常量配置抽取
pub const MACRO_CHUNK_SIZE: usize = 4 * 1024 * 1024; 
pub const BUFFER_SIZE: usize = 8 * 1024 * 1024;    

/// 全局静态引擎单例
pub static ENGINE: Lazy<MaskEngine> = Lazy::new(|| {
    let rules = RuleManager::load_all_rules();
    MaskEngine::new(rules)
});

/// 应用全局状态结构体
pub struct AppState {
    #[allow(dead_code)]
    pub engine: Arc<MaskEngine>,
    pub is_monitor_on: Arc<Mutex<bool>>,
    #[allow(dead_code)]
    pub last_content: Arc<Mutex<String>>,
}

/// 进度负载结构（用于跨模块序列化）
#[derive(serde::Serialize, Clone)]
pub struct ProgressPayload {
    pub percentage: f32,
    pub processed_mb: f64,
}