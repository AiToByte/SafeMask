use crate::common::errors::{AppError, AppResult};
use crate::core::rules::{Rule, RuleGroup};
use std::fs;
use std::path::{Path, PathBuf}; // 🚀 修复：导入 Path
use tauri::{AppHandle, Manager};
use walkdir::WalkDir; 
// 🚀 核心修复：引入 anyhow 的 Context Trait 以使用 with_context
use anyhow::Context; 
use log::{info, error};  // 添加导入

pub struct ConfigLoader;

impl ConfigLoader {
    /// 核心功能：自动加载内置规则目录和用户自定义目录
    pub fn load_all_rules(app_handle: &AppHandle) -> Vec<Rule> {
        let mut all_rules = Vec::new();
        info!("📁 获取资源目录...");
         // 🚀 动态获取打包后的资源目录
        let resource_dir = app_handle.path().resource_dir().expect("无法获取资源目录");
        info!("📁 资源目录: {:?}", resource_dir);
        // 规定两个加载路径
        let paths = vec![
            resource_dir.join("rules"),  // 内置目录
            resource_dir.join("custom"), // 用户自定义目录
        ];

        for path in paths {
            if path.exists() && path.is_dir() {
                let is_custom = path.ends_with("custom");
                all_rules.extend(Self::load_from_directory(path, is_custom));
            }
        }
        all_rules
    }


    /// 递归扫描目录下的所有 YAML 文件
    fn scan_directory<P: AsRef<Path>>(path: P, is_custom: bool) -> Vec<Rule> {
        let mut rules = Vec::new();
        for entry in WalkDir::new(path).into_iter().filter_map(|e| e.ok()) {
            if entry.path().extension().and_then(|s| s.to_str()) == Some("yaml") {
                info!("✅ 解析文件成功: {:?}", entry.path());
                if let Ok(content) = fs::read_to_string(entry.path()) {
                    // 兼容两种 YAML 格式：RuleGroup 对象 或 Rule 数组
                    let parsed_rules = if let Ok(group) = serde_yaml::from_str::<RuleGroup>(&content) {
                        group.rules
                    } else if let Ok(list) = serde_yaml::from_str::<Vec<Rule>>(&content) {
                        list
                    } else {
                        eprintln!("⚠️ [Config] 无法解析 YAML 文件（格式不符）: {:?}", entry.path());
                        continue;
                    };

                    for mut r in parsed_rules {
                        r.is_custom = is_custom;
                        rules.push(r);
                    }
                }
            }
        }
        rules
    }

    /// 保存单个自定义规则到 custom/user_rules.yaml
    pub fn save_custom_rule(rule: Rule) -> AppResult<()> {
        let custom_dir = PathBuf::from("custom");
        if !custom_dir.exists() {
            fs::create_dir_all(&custom_dir)?;
        }

        let file_path = custom_dir.join("user_rules.yaml");
        let mut rules = if file_path.exists() {
            let content = fs::read_to_string(&file_path)?;
            let group: RuleGroup = serde_yaml::from_str(&content)
                .unwrap_or(RuleGroup { group: "CUSTOM".into(), rules: vec![] });
            group.rules
        } else {
            vec![]
        };

        // 更新现有规则或添加新规则 (通过名称判定)
        if let Some(pos) = rules.iter().position(|r| r.name == rule.name) {
            rules[pos] = rule;
        } else {
            let mut new_rule = rule;
            new_rule.is_custom = true;
            rules.push(new_rule);
        }

        let yaml = serde_yaml::to_string(&RuleGroup {
            group: "CUSTOM".into(),
            rules,
        }).map_err(|e| AppError::Config(format!("YAML 序列化失败: {}", e)))?;

        fs::write(file_path, yaml)?;
        Ok(())
    }

    /// 从 custom/user_rules.yaml 中删除规则
    pub fn delete_custom_rule(name: &str) -> AppResult<()> {
        let file_path = PathBuf::from("custom").join("user_rules.yaml");
        if !file_path.exists() { return Ok(()); }

        let content = fs::read_to_string(&file_path)?;
        let mut group: RuleGroup = serde_yaml::from_str(&content)
            .map_err(|e| AppError::Config(e.to_string()))?;
        
        group.rules.retain(|r| r.name != name);

        let yaml = serde_yaml::to_string(&group)
            .map_err(|e| AppError::Config(e.to_string()))?;
        
        fs::write(file_path, yaml)?;
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

      /// 解析单个 YAML 文件：支持 RuleGroup 对象格式和 Vec<Rule> 数组格式
    fn parse_file(path: &Path) -> anyhow::Result<Vec<Rule>> {
        let content = fs::read_to_string(path)
            .with_context(|| format!("读取规则文件失败: {:?}", path))?;

        // 尝试解析为 RuleGroup { group: String, rules: Vec<Rule> }
        if let Ok(group) = serde_yaml::from_str::<RuleGroup>(&content) {
            return Ok(group.rules);
        }

        // 尝试解析为纯数组 [Rule, Rule, ...]
        let rules: Vec<Rule> = serde_yaml::from_str(&content)
            .with_context(|| format!("YAML 格式解析失败: {:?}", path))?;
            
        Ok(rules)
    }
}