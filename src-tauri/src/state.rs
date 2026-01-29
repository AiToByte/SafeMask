use std::sync::{Arc, Mutex, RwLock};
use std::sync::atomic::AtomicBool; // 导入原子类型
use crate::engine::MaskEngine;
use tokio::sync::watch;
use serde::{Serialize, Deserialize}; // 🚀 必须显式导入这两个宏

// 常量配置抽取
pub const MACRO_CHUNK_SIZE: usize = 16 * 1024 * 1024; 
pub const BUFFER_SIZE: usize = 8 * 1024 * 1024;    

/// 历史记录项结构
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MaskHistoryItem {
    pub id: String,
    pub timestamp: String,
    pub original: String,
    pub masked: String,
}

/// 应用全局状态结构体
pub struct AppState {
     // 引擎现在作为 State 的一部分，支持读写锁热重载
    pub engine: Arc<RwLock<MaskEngine>>,
    pub is_monitor_on: Arc<Mutex<bool>>,
    #[allow(dead_code)]
    pub last_content: Arc<Mutex<String>>,
    // 🚀 新增：最近 50 条脱敏历史记录
    pub history: Arc<Mutex<Vec<MaskHistoryItem>>>,// 新增：用于通知监听线程优雅停止的通道
     // 🚀 新增：内部写回标记，防止脱敏后的写回操作触发“监听风暴”
    pub is_internal_changing: Arc<AtomicBool>, 
    pub shutdown_tx: watch::Sender<()>,
    #[allow(dead_code)]
    pub shutdown_rx: watch::Receiver<()>,
}

/// 进度负载结构（用于跨模块序列化）
#[derive(serde::Serialize, Clone)]
pub struct ProgressPayload {
    pub percentage: f32,
    pub processed_mb: f64,
}