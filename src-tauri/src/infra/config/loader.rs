use crate::common::errors::{AppError, AppResult};
use crate::core::rules::{Rule, RuleGroup};
use crate::core::config::AppSettings;
use std::fs;
use std::path::{Path, PathBuf}; // 🚀 修复：导入 Path
use tauri::{AppHandle, Manager};
use walkdir::WalkDir; 
// 🚀 核心修复：引入 anyhow 的 Context Trait 以使用 with_context
use anyhow::Context; 
use log::{info, error};

pub struct ConfigLoader;

impl ConfigLoader {

    /// 核心功能：自动加载内置规则目录和用户自定义目录
    pub fn load_all_rules(app_handle: &AppHandle) -> Vec<Rule> {
        let mut all_rules = Vec::new();

        // 1. 加载内置规则 (只读资源)
        // 在 NSIS 便携版中，此目录位于临时文件夹；在安装版中位于 Program Files
        if let Ok(resource_dir) = app_handle.path().resource_dir() {
            let built_in_path = resource_dir.join("rules");
            if built_in_path.exists() {
                info!("📁 加载系统内置规则: {:?}", built_in_path);
                all_rules.extend(Self::load_from_directory(&built_in_path, false));
            } else {
                 // 记录日志而不是直接崩溃
                error!("⚠️ 资源目录存在但找不到 rules 文件夹: {:?}", built_in_path);
            }
        } else {
            error!("⚠️ 无法获取资源目录资源，可能运行环境不规范");
        }

        // 2. 加载用户自定义规则 (持久化配置)
        let custom_dir = Self::get_custom_storage_path(app_handle);
        if custom_dir.exists() {
            info!("📁 加载用户自定义规则: {:?}", custom_dir);
            all_rules.extend(Self::load_from_directory(&custom_dir, true));
        } else {
            // 首次运行，尝试创建目录
            let _ = fs::create_dir_all(&custom_dir);
        }

        all_rules
    }


    /// 递归扫描目录下的所有 YAML 文件
    #[allow(dead_code)]
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
    pub fn save_custom_rule(app_handle: &AppHandle, rule: Rule) -> AppResult<()> {
        let custom_dir = Self::get_custom_storage_path(app_handle);
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
    pub fn delete_custom_rule(app_handle: &AppHandle, name: &str) -> AppResult<()> {
        let file_path = Self::get_custom_storage_path(app_handle).join("user_rules.yaml");
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
    /// 🚀 智能路径判定：适配安装版、ZIP便携版、NSIS单文件版
    fn get_custom_storage_path(app_handle: &AppHandle) -> PathBuf {
        // 获取当前 EXE 所在目录
        let exe_path = std::env::current_exe().unwrap_or_default();
        let exe_dir = exe_path.parent().unwrap_or(Path::new(""));

        // 判定 1：是否在临时文件夹运行 (NSIS Portable 特征)
        // Windows 临时文件夹通常包含 "Temp" 字符串
        let is_in_temp = exe_dir.to_string_lossy().to_lowercase().contains("temp");

        if is_in_temp {
            // 【针对 NSIS 单文件版】：临时目录不可靠，使用系统的 AppData 目录实现真正持久化
            app_handle.path().app_local_data_dir()
                .unwrap_or_else(|_| PathBuf::from("./data")) // 兜底
                .join("custom")
        } else {
            // 【针对 ZIP 绿色版 / 开发模式】：直接在 EXE 同级目录创建 custom 文件夹
            // 这样用户把整个文件夹考走，规则也会跟着走，实现真正的绿色便携
            exe_dir.join("custom")
        }
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

    /// 加载应用设置：优先从磁盘读取，失败则返回默认值
    pub fn load_settings(app_handle: &AppHandle) -> AppSettings {
        let settings_path = Self::get_custom_storage_path(app_handle).join("settings.yaml");
        
        if settings_path.exists() {
            match fs::read_to_string(&settings_path) {
                Ok(content) => {
                    serde_yaml::from_str(&content).unwrap_or_else(|e| {
                        error!("解析 settings.yaml 失败，使用默认配置: {}", e);
                        AppSettings::default()
                    })
                }
                Err(e) => {
                    error!("读取 settings.yaml 失败: {}", e);
                    AppSettings::default()
                }
            }
        } else {
            // 首次运行，保存一份默认配置到磁盘
            let default_settings = AppSettings::default();
            let _ = Self::save_settings(app_handle, &default_settings);
            default_settings
        }
    }

    /// 将设置持久化到磁盘
    pub fn save_settings(app_handle: &AppHandle, settings: &AppSettings) -> AppResult<()> {
        let custom_dir = Self::get_custom_storage_path(app_handle);
        if !custom_dir.exists() {
            fs::create_dir_all(&custom_dir)?;
        }

        let file_path = custom_dir.join("settings.yaml");
        let yaml = serde_yaml::to_string(settings)
            .map_err(|e| AppError::Config(format!("配置序列化失败: {}", e)))?;

        fs::write(file_path, yaml)
            .map_err(|e| AppError::Io(e))?;
            
        info!("✅ 应用设置已成功持久化到磁盘");
        Ok(())
    }
}