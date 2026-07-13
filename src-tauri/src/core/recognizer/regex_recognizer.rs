//! 正则识别器
//!
//! 将原有的 `MaskEngine` 中的正则匹配逻辑提取为独立的 `Recognizer` 实现。
//! 这是 SafeMask 最核心的确定性识别器，用于匹配模式化的 PII（如邮箱、手机号、IP 等）。
//!
//! # 设计要点
//!
//! - 操作在字节流 (`&[u8]`) 上，跳过 UTF-8 验证，性能更高
//! - 支持优先级排序，高优先级规则先匹配
//! - 使用 `SmallVec` 优化栈上小数组，减少堆分配

use super::types::*;
use super::Recognizer;
use regex::bytes::Regex;
use smallvec::SmallVec;
use log::{debug, warn};

/// 编译后的正则规则
struct CompiledRule {
    /// 规则名称
    name: String,
    /// 编译后的正则表达式
    re: Regex,
    /// 替换掩码文本
    mask: String,
    /// 优先级
    priority: i32,
    /// 关联的实体类型
    entity_type: EntityType,
}

/// 正则识别器
///
/// 基于正则表达式的确定性 PII 识别器。
/// 支持多条规则，按优先级排序执行。
pub struct RegexRecognizer {
    /// 编译后的规则列表（按优先级降序排列）
    rules: Vec<CompiledRule>,
    /// 识别器名称
    name: String,
    /// 是否启用
    enabled: bool,
    /// 识别器优先级
    priority: i32,
}

impl RegexRecognizer {
    /// 从规则列表创建正则识别器
    ///
    /// # 参数
    ///
    /// - `rules`: 规则定义列表（来自 YAML 配置）
    /// - `name`: 识别器名称（默认 "regex_engine"）
    /// - `priority`: 识别器优先级（默认 100）
    pub fn from_rules(
        rules: &[crate::core::rules::Rule],
        name: Option<&str>,
        priority: Option<i32>,
    ) -> Self {
        let mut compiled = Vec::new();

        for rule in rules {
            if !rule.enabled {
                continue;
            }

            // 跳过字面量模式（由 AhoCorasickRecognizer 处理）
            if is_literal(&rule.pattern) {
                continue;
            }

            match regex::bytes::RegexBuilder::new(&rule.pattern)
                .unicode(false)
                .build()
            {
                Ok(re) => {
                    let entity_type = guess_entity_type(&rule.name, &rule.pattern);
                    compiled.push(CompiledRule {
                        name: rule.name.clone(),
                        re,
                        mask: rule.mask.clone(),
                        priority: rule.priority,
                        entity_type,
                    });
                }
                Err(_) => {
                    // 如果 unicode(false) 编译失败（如正则含 Unicode 字符类），尝试 unicode(true)
                    match regex::bytes::RegexBuilder::new(&rule.pattern)
                        .unicode(true)
                        .build()
                    {
                        Ok(re) => {
                            let entity_type = guess_entity_type(&rule.name, &rule.pattern);
                            compiled.push(CompiledRule {
                                name: rule.name.clone(),
                                re,
                                mask: rule.mask.clone(),
                                priority: rule.priority,
                                entity_type,
                            });
                            debug!("  → 回退 unicode(true) 编译成功 '{}'", rule.name);
                        }
                        Err(e) => {
                            warn!("⚠️ [RegexRecognizer] 忽略无效正则 '{}': {}", rule.name, e);
                        }
                    }
                }
            }
        }

        // 按优先级降序排列
        compiled.sort_by(|a, b| b.priority.cmp(&a.priority));

        Self {
            rules: compiled,
            name: name.unwrap_or("regex_engine").to_string(),
            enabled: true,
            priority: priority.unwrap_or(100),
        }
    }

    /// 获取规则数量
    pub fn rule_count(&self) -> usize {
        self.rules.len()
    }

    /// 获取替换掩码（供脱敏策略层使用）
    pub fn get_mask(&self, rule_name: &str) -> Option<&str> {
        self.rules
            .iter()
            .find(|r| r.name == rule_name)
            .map(|r| r.mask.as_str())
    }
}

impl Recognizer for RegexRecognizer {
    fn name(&self) -> &str {
        &self.name
    }

    fn recognizer_type(&self) -> RecognizerType {
        RecognizerType::Rule
    }

    fn supported_entities(&self) -> Vec<EntityType> {
        // 正则识别器是通用的，可识别任何类型
        Vec::new()
    }

    fn analyze(&self, context: &AnalysisContext) -> AnalysisResult {
        let input = context.text;
        let mut spans: SmallVec<[EntitySpan; 16]> = SmallVec::new();

        for rule in &self.rules {
            for mat in rule.re.find_iter(input) {
                spans.push(EntitySpan {
                    start: mat.start(),
                    end: mat.end(),
                    entity_type: rule.entity_type.clone(),
                    confidence: 1.0, // 正则匹配是确定性的
                    source: self.name.clone(),
                    context: Some(format!("rule:{}", rule.name)),
                    mask: Some(rule.mask.clone()),
                    priority: 0,
                });
            }
        }

        AnalysisResult {
            spans: spans.into_vec(),
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

/// 检测模式是否为字面量（无正则特殊字符）
///
/// 字面量模式应由 AhoCorasickRecognizer 处理，性能更高。
fn is_literal(pattern: &str) -> bool {
    let meta = ['.', '+', '*', '?', '(', ')', '|', '[', ']', '{', '}', '^', '$', '\\'];
    !pattern.chars().any(|c| meta.contains(&c))
}

/// 根据规则名称和模式猜测实体类型
fn guess_entity_type(name: &str, pattern: &str) -> EntityType {
    let name_lower = name.to_lowercase();
    let pattern_lower = pattern.to_lowercase();

    if name_lower.contains("email") || name_lower.contains("邮箱") || pattern_lower.contains("@") {
        EntityType::Email
    } else if name_lower.contains("phone") || name_lower.contains("手机") || name_lower.contains("电话") {
        EntityType::Phone
    } else if name_lower.contains("idcard") || name_lower.contains("身份证") {
        EntityType::IdCard
    } else if name_lower.contains("bank") || name_lower.contains("银行卡") || name_lower.contains("信用卡") {
        EntityType::BankCard
    } else if name_lower.contains("ip") || name_lower.contains("地址") && pattern_lower.contains("\\d") {
        EntityType::IpAddress
    } else if name_lower.contains("url") || name_lower.contains("链接") || name_lower.contains("网址") {
        EntityType::Url
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
