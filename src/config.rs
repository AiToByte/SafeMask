use serde::Deserialize;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;
use anyhow::{Result, Context};

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
    #[serde(default)]
    pub group: String,
    pub rules: Vec<Rule>,
}

pub struct RuleManager;


impl RuleManager {
    /// 核心功能：自动加载内置规则目录和用户自定义目录
    pub fn load_all_rules() -> Vec<Rule> {
        let mut all_rules = Vec::new();
        
        // 规定两个加载路径
        let paths = vec![
            PathBuf::from("rules"),  // 内置目录
            PathBuf::from("custom"), // 用户自定义目录
        ];

        for path in paths {
            if path.exists() && path.is_dir() {
                all_rules.extend(Self::load_from_directory(path));
            }
        }
        all_rules
    }

    fn load_from_directory<P: AsRef<Path>>(dir: P) -> Vec<Rule> {
        let mut rules = Vec::new();
        for entry in WalkDir::new(dir).into_iter().filter_map(|e| e.ok()) {
            if entry.path().extension().map_or(false, |ext| ext == "yaml") {
                if let Ok(file_rules) = Self::parse_file(entry.path()) {
                    rules.extend(file_rules);
                }
            }
        }
        rules
    }

    fn parse_file(path: &Path) -> Result<Vec<Rule>> {
        let content = std::fs::read_to_string(path)?;
        let group: RuleGroup = serde_yaml::from_str(&content)
            .with_context(|| format!("解析规则文件失败: {:?}", path))?;
        Ok(group.rules)
    }
}