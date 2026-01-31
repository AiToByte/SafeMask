use std::sync::Arc;
use std::sync::atomic::AtomicBool;
// 🚀 确保只用 parking_lot，绝不出现 std::sync::RwLock
// use parking_lot::{Mutex, RwLock}; 
use crate::core::engine::MaskEngine;
use serde::{Serialize, Deserialize};

pub type SharedEngine = Arc<parking_lot::RwLock<Arc<MaskEngine>>>;

pub struct AppState {
    pub engine: SharedEngine,
    pub is_monitor_on: Arc<parking_lot::Mutex<bool>>,
    pub history: Arc<parking_lot::Mutex<Vec<crate::common::state::MaskHistoryItem>>>,
    pub is_internal_changing: Arc<AtomicBool>,
    pub last_content: Arc<parking_lot::Mutex<String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MaskHistoryItem {
    pub id: String,
    pub timestamp: String,
    pub original: String,
    pub masked: String,
}

impl AppState {
    // 🚀 必须确保有 pub 关键字
    pub fn add_history(&self, item: MaskHistoryItem) {
        let mut history = self.history.lock();
        history.insert(0, item);
        // 限制历史记录数量，防止内存无限增长
        if history.len() > 50 {
            history.pop();
        }
    }
}