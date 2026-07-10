//! Aho-Corasick 字典识别器
//!
//! 基于 Aho-Corasick 自动机的字典匹配识别器。
//! 适用于固定词匹配（如敏感词库、人名词典、公司名册等），
//! 时间复杂度为 O(n)，远优于逐个正则匹配。
//!
//! # 性能特点
//!
//! - 一次扫描匹配所有模式，时间复杂度 O(n + m + z)
//!   - n: 文本长度, m: 模式总长度, z: 匹配数
//! - 使用 `LeftmostLongest` 匹配策略，避免重叠匹配
//! - 适合大量固定词的场景（数千到数万条）

use super::types::*;
use super::Recognizer;
use aho_corasick::{AhoCorasick, MatchKind};

/// 字典条目
struct DictEntry {
    /// 关联的实体类型
    entity_type: EntityType,
    /// 来源标识（如词典名称）
    source: String,
    /// 替换掩码
    mask: String,
}

/// Aho-Corasick 字典识别器
pub struct AhoCorasickRecognizer {
    /// Aho-Corasick 自动机
    engine: Option<AhoCorasick>,
    /// 字典条目列表（与自动机模式索引对应）
    entries: Vec<DictEntry>,
    /// 识别器名称
    name: String,
    /// 是否启用
    enabled: bool,
    /// 识别器优先级
    priority: i32,
}

impl AhoCorasickRecognizer {
    /// 从规则列表创建字典识别器
    ///
    /// 自动筛选出字面量模式（无正则特殊字符的规则）。
    pub fn from_rules(
        rules: &[crate::core::rules::Rule],
        name: Option<&str>,
        priority: Option<i32>,
    ) -> Self {
        let mut patterns = Vec::new();
        let mut entries = Vec::new();

        for rule in rules {
            if !rule.enabled {
                continue;
            }

            // 只处理字面量模式
            if !is_literal(&rule.pattern) {
                continue;
            }

            patterns.push(rule.pattern.clone());
            entries.push(DictEntry {
                entity_type: guess_entity_type(&rule.name),
                source: format!("rule:{}", rule.name),
                mask: rule.mask.clone(),
            });
        }

        let engine = if patterns.is_empty() {
            None
        } else {
            AhoCorasick::builder()
                .match_kind(MatchKind::LeftmostLongest)
                .build(&patterns)
                .ok()
        };

        Self {
            engine,
            entries,
            name: name.unwrap_or("aho_corasick_engine").to_string(),
            enabled: true,
            priority: priority.unwrap_or(90),
        }
    }

    /// 从字典文件创建识别器（预留接口）
    #[allow(dead_code)]
    pub fn from_dictionary(
        dict_path: &str,
        entity_type: EntityType,
        name: Option<&str>,
    ) -> Result<Self, String> {
        let content = std::fs::read_to_string(dict_path)
            .map_err(|e| format!("读取字典文件失败: {}", e))?;

        let mut patterns = Vec::new();
        let mut entries = Vec::new();

        for line in content.lines() {
            let word = line.trim();
            if word.is_empty() || word.starts_with('#') {
                continue;
            }
            patterns.push(word.to_string());
            entries.push(DictEntry {
                entity_type: entity_type.clone(),
                source: format!("dict:{}", dict_path),
                mask: format!("[{}]", entity_type.display_label()),
            });
        }

        let engine = if patterns.is_empty() {
            None
        } else {
            AhoCorasick::builder()
                .match_kind(MatchKind::LeftmostLongest)
                .build(&patterns)
                .ok()
        };

        Ok(Self {
            engine,
            entries,
            name: name.unwrap_or("dict_engine").to_string(),
            enabled: true,
            priority: 90,
        })
    }

    /// 获取字典条目数量
    pub fn entry_count(&self) -> usize {
        self.entries.len()
    }
}

impl Recognizer for AhoCorasickRecognizer {
    fn name(&self) -> &str {
        &self.name
    }

    fn recognizer_type(&self) -> RecognizerType {
        RecognizerType::Rule
    }

    fn supported_entities(&self) -> Vec<EntityType> {
        // 字典识别器是通用的
        Vec::new()
    }

    fn analyze(&self, context: &AnalysisContext) -> AnalysisResult {
        let input = context.text;
        let mut spans = Vec::new();

        if let Some(ref ac) = self.engine {
            for mat in ac.find_iter(input) {
                let pattern_idx = mat.pattern().as_usize();
                if let Some(entry) = self.entries.get(pattern_idx) {
                    spans.push(EntitySpan {
                        start: mat.start(),
                        end: mat.end(),
                        entity_type: entry.entity_type.clone(),
                        confidence: 1.0, // 字典匹配是确定性的
                        source: self.name.clone(),
                        context: Some(entry.source.clone()),
                        mask: Some(entry.mask.clone()),
                        priority: 0,
                    });
                }
            }
        }

        AnalysisResult {
            spans,
            elapsed_us: 0,
            recognizer: self.name.clone(),
        }
    }

    fn priority(&self) -> i32 {
        self.priority
    }

    fn is_enabled(&self) -> bool {
        self.enabled
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// 辅助函数
// ─────────────────────────────────────────────────────────────────────────────

/// 检测模式是否为字面量
fn is_literal(pattern: &str) -> bool {
    let meta = ['.', '+', '*', '?', '(', ')', '|', '[', ']', '{', '}', '^', '$', '\\'];
    !pattern.chars().any(|c| meta.contains(&c))
}

/// 根据规则名称猜测实体类型
fn guess_entity_type(name: &str) -> EntityType {
    let name_lower = name.to_lowercase();

    if name_lower.contains("email") || name_lower.contains("邮箱") {
        EntityType::Email
    } else if name_lower.contains("phone") || name_lower.contains("手机") || name_lower.contains("电话") {
        EntityType::Phone
    } else if name_lower.contains("idcard") || name_lower.contains("身份证") {
        EntityType::IdCard
    } else if name_lower.contains("bank") || name_lower.contains("银行卡") {
        EntityType::BankCard
    } else if name_lower.contains("key") || name_lower.contains("token") || name_lower.contains("密钥") {
        EntityType::ApiKey
    } else if name_lower.contains("password") || name_lower.contains("密码") {
        EntityType::Password
    } else if name_lower.contains("person") || name_lower.contains("姓名") || name_lower.contains("人名") {
        EntityType::Person
    } else if name_lower.contains("address") || name_lower.contains("地址") {
        EntityType::Address
    } else {
        EntityType::Custom(name.to_string())
    }
}
