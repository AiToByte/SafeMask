use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Rule {
    /// 规则名称 (唯一标识)
    pub name: String,
    
    /// 正则表达式或关键字
    pub pattern: String,
    
    /// 替换掩码 (例如: <EMAIL>)
    pub mask: String,
    
    /// 优先级 (数字越大越先处理)
    #[serde(default)]
    pub priority: i32,
    
    /// 是否启用
    #[serde(default = "default_true")]
    pub enabled: bool,

    /// 是否为用户自定义规则 (影响 UI 显示和存储位置)
    #[serde(default)]
    pub is_custom: bool,
}

fn default_true() -> bool {
    true
}

/// 用于 YAML 存储的包装结构
#[derive(Debug, Serialize, Deserialize)]
pub struct RuleGroup {
    pub group: String,
    pub rules: Vec<Rule>,
}