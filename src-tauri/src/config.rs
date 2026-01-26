use serde::{Deserialize, Serialize}; // 🚀 增加 Serialize
use std::path::{Path, PathBuf};
use walkdir::WalkDir;
use anyhow::{Result, Context};

#[derive(Debug, Deserialize, Serialize, Clone)] // 🚀 增加 Serialize
pub struct Rule {
    #[allow(dead_code)]
    pub name: String,
    pub pattern: String,
    pub mask: String,
    #[serde(default = "default_priority")] // 如果YAML没写优先级，则默认为0
    pub priority: i32,
     // 🚀 新增：标记是否为用户自定义
    #[serde(default)]
    pub is_custom: bool,
}

fn default_priority() -> i32 { 0 }

#[derive(Debug, Deserialize, Serialize, Clone)] // 🚀 增加 Serialize
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
                let is_custom = path.ends_with("custom");
                all_rules.extend(Self::load_from_directory(path, is_custom));
            }
        }
        all_rules
    }

    /// 🚀 新增：保存自定义规则到 custom/user_rules.yaml
    pub fn save_custom_rule(rule: Rule) -> Result<()> {
        let custom_dir = PathBuf::from("custom");
        if !custom_dir.exists() {
            std::fs::create_dir_all(&custom_dir)?;
        }

        let file_path = custom_dir.join("user_rules.yaml");
        
        // 1. 读取现有规则
        let mut current_rules = if file_path.exists() {
            let content = std::fs::read_to_string(&file_path)?;
            let group: RuleGroup = serde_yaml::from_str(&content).unwrap_or(RuleGroup {
                group: "CUSTOM".into(),
                rules: vec![],
            });
            group.rules
        } else {
            vec![]
        };

        // 2. 更新或追加 (通过 name 判定)
        if let Some(pos) = current_rules.iter().position(|r| r.name == rule.name) {
            current_rules[pos] = rule;
        } else {
            current_rules.push(rule);
        }

        // 3. 写回文件
        let new_group = RuleGroup {
            group: "CUSTOM".into(),
            rules: current_rules,
        };
        let yaml_content = serde_yaml::to_string(&new_group)?;
        std::fs::write(file_path, yaml_content)?;

        Ok(())
    }

    // 修改此内部方法，增加 is_custom 参数
    fn load_from_directory<P: AsRef<Path>>(dir: P, is_custom: bool) -> Vec<Rule> {
        let mut rules = Vec::new();
        for entry in WalkDir::new(dir).into_iter().filter_map(|e| e.ok()) {
            if entry.path().extension().map_or(false, |ext| ext == "yaml") {
                if let Ok(mut file_rules) = Self::parse_file(entry.path()) {
                    // 🚀 为该目录下加载的所有规则打上标记
                    for rule in &mut file_rules {
                        rule.is_custom = is_custom;
                    }
                    rules.extend(file_rules);
                }
            }
        }
        rules
    }

    /// 🚀 新增：删除自定义规则
    pub fn delete_custom_rule(name: String) -> Result<()> {
        let file_path = PathBuf::from("custom/user_rules.yaml");
        if !file_path.exists() { return Ok(()); }

        let content = std::fs::read_to_string(&file_path)?;
        let mut group: RuleGroup = serde_yaml::from_str(&content)?;
        
        // 过滤掉匹配名称的规则
        group.rules.retain(|r| r.name != name);

        let yaml_content = serde_yaml::to_string(&group)?;
        std::fs::write(file_path, yaml_content)?;
        Ok(())
    }

    fn parse_file(path: &Path) -> Result<Vec<Rule>> {
        let content = std::fs::read_to_string(path)?;
        let group: RuleGroup = serde_yaml::from_str(&content)
            .with_context(|| format!("解析规则文件失败: {:?}", path))?;
        Ok(group.rules)
    }
}