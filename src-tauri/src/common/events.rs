/// SafeMask 全局事件名定义
/// 
/// 统一管理 Rust 后端发往前端的事件名称，防止硬编码带来的拼写错误。
pub struct AppEvents;

impl AppEvents {
    /// 文件处理进度事件
    /// Payload: { "percentage": f64 }
    #[allow(dead_code)]
    pub const FILE_PROGRESS: &'static str = "file-progress";

    /// 实时脱敏通知事件 (主要用于剪贴板自动脱敏后的提示)
    /// Payload: String (提示消息)
    pub const MASKED_EVENT: &'static str = "masked-event";

    /// 新的历史记录产生事件
    /// Payload: MaskHistoryItem (包含 id, timestamp, original, masked)
    pub const NEW_HISTORY: &'static str = "new-history";

    /// 窗口关闭请求事件
    /// 当用户点击关闭按钮时，后端拦截并发送此信号给前端弹出确认框
    /// Payload: "SIGNAL_CLOSE"
    #[allow(dead_code)]
    pub const REQUEST_CLOSE: &'static str = "request-close";
}

// 辅助：定义进度负载结构（如果需要强类型发送）
use serde::Serialize;

#[derive(Serialize, Clone)]
#[allow(dead_code)]
pub struct ProgressPayload {
    pub percentage: f64,
}

#[derive(Serialize, Clone)]
#[allow(dead_code)]
pub struct MaskNotificationPayload {
    pub message: String,
    pub content: Option<String>,
}