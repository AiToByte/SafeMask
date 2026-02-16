use std::sync::Arc;
use std::sync::atomic::AtomicBool;
use crate::core::engine::MaskEngine;
use serde::{Serialize, Deserialize};
use parking_lot::{Mutex, RwLock};
use crate::core::config::AppSettings;

pub type SharedEngine = Arc<RwLock<Arc<MaskEngine>>>;

pub struct AppState {
    /// 脱敏引擎 (支持热重载)
    pub engine: SharedEngine,

    /// 影子存储核心单例
    pub shadow_store: Arc<RwLock<ShadowClipboard>>,
    
    /// 持久化配置
    pub settings: Arc<RwLock<AppSettings>>,
    
    /// 正在执行魔术粘贴的标记 (原子操作，防止递归触发)
    pub is_magic_pasting: Arc<AtomicBool>,
    
    /// 哨兵模式开关 (实时监控)
    pub is_monitor_on: Arc<Mutex<bool>>,
    
    /// 历史记录列表 (环形缓冲区)
    pub history: Arc<Mutex<Vec<MaskHistoryItem>>>,
    
    /// 剪贴板轮询的去重缓存
    pub last_content: Arc<Mutex<String>>,
}

/// 影子剪贴板：存储当前的“影子宇宙”状态
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ShadowClipboard {
    pub original: String,      // 用户的原始明文
    pub masked: String,        // 引擎计算后的脱敏文本
    pub has_privacy: bool,     // 是否命中了敏感规则
    pub source_app: String,    // 来源应用名 (预留字段)
    pub timestamp: u64,        // 捕获时间戳
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MaskHistoryItem {
    pub id: String,
    pub timestamp: String,
    pub original: String,
    pub masked: String,
    pub mode: String, // 🚀 新增：值为 "SHADOW" 或 "SENTRY"
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

    /// 初始化影子存储并添加记录
    /// 返回值: 是否发现了隐私信息 (bool)
    pub fn update_shadow(&self, original: String, masked: String) -> bool {
        let has_changed = original != masked;
        
        // 1. 更新影子存储 (写锁)
        let mut shadow = self.shadow_store.write();
        *shadow = ShadowClipboard {
            original,
            masked,
            has_privacy: has_changed,
            source_app: "Unknown".to_string(), // 未来扩展：调用系统 API 获取活跃进程
            timestamp: chrono::Utc::now().timestamp() as u64,
        };
        
        has_changed
    }

    /// 获取影子副本的安全克隆 (读锁)
    pub fn get_shadow_snapshot(&self) -> ShadowClipboard {
        self.shadow_store.read().clone()
    }
}