use thiserror::Error;
use serde::Serialize;

#[derive(Error, Debug)]
pub enum AppError {
    #[error("IO 错误: {0}")]
    Io(#[from] std::io::Error),

    #[error("正则引擎错误: {0}")]
    Regex(#[from] regex::Error),

    #[error("配置错误: {0}")]
    Config(String),

    #[error("剪贴板访问失败: {0}")]
    Clipboard(String),

    #[error("任务处理中止: {0}")]
    #[allow(dead_code)]
    Aborted(String),
    
    #[error("内部系统错误: {0}")]
    Internal(String),
}

// 序列化实现，以便前端 catch 到错误字符串
impl Serialize for AppError {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}

pub type AppResult<T> = Result<T, AppError>;