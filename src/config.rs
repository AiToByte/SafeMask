use serde::Deserialize;
use std::path::Path;
use walkdir::WalkDir;

#[derive(Debug, Deserialize, Clone)]
pub struct Rule {
    #[allow(dead_code)]
    pub name: String,
    pub pattern: String,
    pub mask: String,
    #[serde(default = "default_priority")] // 如果YAML没写优先级，则默认为0
    pub priority: i32,
}

fn default_priority() -> i32 { 0 }

#[derive(Debug, Deserialize, Clone)]
pub struct RuleGroup {
    #[allow(dead_code)]
    pub group: String,
    pub rules: Vec<Rule>,
}

/// 递归扫描目录加载所有 YAML 规则
pub fn load_all_rules<P: AsRef<Path>>(dir: P) -> Vec<Rule> {
    let mut all_rules = Vec::new();
    for entry in WalkDir::new(dir).into_iter().filter_map(|e| e.ok()) {
        let path = entry.path();
        if path.is_file() && path.extension().map_or(false, |ext| ext == "yaml") {
            let content = std::fs::read_to_string(path).expect("无法读取规则文件");
            match serde_yaml::from_str::<RuleGroup>(&content) {
                Ok(group) => all_rules.extend(group.rules),
                Err(e) => eprintln!("⚠️ 规则文件解析失败 {:?}: {}", path, e),
            }
        }
    }
    all_rules
}