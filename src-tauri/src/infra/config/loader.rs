use crate::common::errors::{AppError, AppResult};
use crate::core::rules::{Rule, RuleGroup};
use std::fs;
use std::path::{PathBuf};
use tauri::{AppHandle, Manager};
use std::env;

pub struct ConfigLoader;

/// 默认规则配置（当配置文件不存在时写入）
#[allow(dead_code)]
const DEFAULT_RULES_YAML: &str = r#"
- name: "手机号"
  pattern: "(?P<prefix>1[3-9]\\d{1})(?P<mask>\\d{4})(?P<suffix>\\d{4})"
  mask: "<CHINA_PHONE>"
  priority: 10
  enabled: true

- name: "身份证号"
  pattern: "(?P<prefix>\\d{6})(?P<mask>\\d{8})(?P<suffix>\\d{3}[0-9Xx])"
  mask: "<ID_CARD>"
  priority: 20
  enabled: true

- name: "Email"
  pattern: "(?P<prefix>[\\w\\.-]+)@(?P<suffix>[\\w\\.-]+\\.\\w+)"
  mask: "<EMAIL>"
  priority: 5
  enabled: true
"#;

impl ConfigLoader {
    /// 加载所有规则（内置 + 自定义）
    pub fn load_all_rules(app: &AppHandle) -> AppResult<Vec<Rule>> {
        let mut all_rules = Vec::new();
        
        // 根据模式（dev/prod）获取基目录
        let base_dir = if cfg!(debug_assertions) {
            // dev 模式：从 Cargo.toml 所在目录（src-tauri）加载源代码路径
            PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        } else {
            // prod 模式：从打包资源目录加载
            app.path().resource_dir().map_err(|e| AppError::Config(e.to_string()))?
        };

        // 内置规则：base_dir/rules
        let rules_dir = base_dir.join("rules");
        if rules_dir.exists() {
            all_rules.extend(Self::load_from_dir(rules_dir, false)?);
        } else {
            eprintln!("⚠️ [Config] 内置规则目录不存在: {:?}", rules_dir);
        }

        // 自定义规则：base_dir/custom
        let custom_dir = base_dir.join("custom");
        if custom_dir.exists() {
            all_rules.extend(Self::load_from_dir(custom_dir, true)?);
        } else {
            eprintln!("⚠️ [Config] 自定义规则目录不存在: {:?}", custom_dir);
        }

        Ok(all_rules)
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
                .map_err(|e| AppError::Config(e.to_string()))?;
            group.rules
        } else {
            vec![]
        };

        // 更新现有规则或添加新规则
        if let Some(pos) = rules.iter().position(|r| r.name == rule.name) {
            rules[pos] = rule;
        } else {
            let mut new_rule = rule;
            new_rule.is_custom = true; // 强制标记为自定义
            rules.push(new_rule);
        }

        let yaml = serde_yaml::to_string(&RuleGroup {
            group: "CUSTOM".into(),
            rules,
        }).map_err(|e| AppError::Config(e.to_string()))?;

        fs::write(file_path, yaml)?;
        Ok(())
    }

    /// 删除自定义规则
    pub fn delete_custom_rule(name: &str) -> AppResult<()> {
        let file_path = PathBuf::from("custom/user_rules.yaml");
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

    fn load_from_dir(dir: PathBuf, is_custom: bool) -> AppResult<Vec<Rule>> {
        let mut rules = Vec::new();
        let entries = fs::read_dir(dir)?;

        for entry in entries {
            let path = entry?.path();
            if path.extension().and_then(|s| s.to_str()) == Some("yaml") {
                let content = fs::read_to_string(&path)?;
                if let Ok(group) = serde_yaml::from_str::<RuleGroup>(&content) {
                    let mut file_rules = group.rules;
                    for r in &mut file_rules { r.is_custom = is_custom; }
                    rules.extend(file_rules);
                }
            }
        }
        Ok(rules)
    }
}