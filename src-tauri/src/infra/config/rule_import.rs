//! 自定义规则批量导入 / 导出。
//!
//! 解析与校验为纯函数，便于单元测试；落盘由 `ConfigLoader` 原子写入。

use crate::core::rules::{Rule, RuleGroup};
use regex::Regex;
use serde::{Deserialize, Serialize};

/// 单文件最大体积（字节）
pub const MAX_FILE_BYTES: u64 = 256 * 1024;
/// 单次最多文件数
pub const MAX_FILES: usize = 20;
/// 单次最多规则条数
pub const MAX_RULES: usize = 500;
/// name / mask / pattern 长度上限
pub const MAX_NAME_LEN: usize = 128;
pub const MAX_MASK_LEN: usize = 128;
pub const MAX_PATTERN_LEN: usize = 4096;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConflictPolicy {
    /// 覆盖同名自定义规则；与内置规则同名则跳过
    OverwriteCustom,
}

impl ConflictPolicy {
    pub fn parse(s: &str) -> Self {
        // 目前仅支持 overwrite_custom；其它字符串回退到该策略
        match s.trim().to_ascii_lowercase().as_str() {
            "overwrite_custom" | "overwrite" | "" => Self::OverwriteCustom,
            _ => Self::OverwriteCustom,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ImportItemStatus {
    Imported,
    Overwritten,
    SkippedBuiltin,
    SkippedInvalid,
    Failed,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImportRuleItemResult {
    pub name: String,
    pub status: ImportItemStatus,
    pub message: String,
    pub source_file: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImportRulesReport {
    pub total_parsed: u32,
    pub imported: u32,
    pub overwritten: u32,
    pub skipped: u32,
    pub failed: u32,
    pub items: Vec<ImportRuleItemResult>,
    /// 合并后的完整自定义规则列表（仅成功路径由调用方使用）
    #[serde(skip)]
    pub merged_custom_rules: Vec<Rule>,
}

#[derive(Debug, Clone)]
pub struct ParsedImportFile {
    pub source: String,
    pub rules: Vec<Rule>,
}

/// 解析 YAML 内容：兼容 RuleGroup 与 Vec&lt;Rule&gt;。
pub fn parse_rules_yaml(content: &str, source: &str) -> Result<ParsedImportFile, String> {
    let content = content.strip_prefix('\u{feff}').unwrap_or(content).trim();
    if content.is_empty() {
        return Err(format!("{}: 文件为空", source));
    }

    if let Ok(group) = serde_yaml::from_str::<RuleGroup>(content) {
        return Ok(ParsedImportFile {
            source: source.to_string(),
            rules: group.rules,
        });
    }

    match serde_yaml::from_str::<Vec<Rule>>(content) {
        Ok(rules) => Ok(ParsedImportFile {
            source: source.to_string(),
            rules,
        }),
        Err(e) => Err(format!("{}: YAML 解析失败: {}", source, e)),
    }
}

/// 规范化 mask：已有尖/方括号则保留；裸标签按 wrapper_style 包裹。
pub fn normalize_mask(mask: &str, wrapper_style: &str) -> String {
    let m = mask.trim();
    if m.is_empty() {
        return String::new();
    }
    let already_wrapped = (m.starts_with('<') && m.ends_with('>'))
        || (m.starts_with('[') && m.ends_with(']'));
    if already_wrapped {
        return m.to_string();
    }
    let bare = m.trim_matches(|c| c == '<' || c == '>' || c == '[' || c == ']');
    if wrapper_style == "square" {
        format!("[{}]", bare)
    } else {
        format!("<{}>", bare)
    }
}

/// 单条规则校验；成功返回规范化后的 Rule（is_custom=true）。
pub fn validate_and_normalize_rule(
    mut rule: Rule,
    wrapper_style: &str,
    source: &str,
) -> Result<Rule, String> {
    rule.name = rule.name.trim().to_string();
    rule.pattern = rule.pattern.trim().to_string();
    rule.mask = rule.mask.trim().to_string();

    if rule.name.is_empty() {
        return Err(format!("{}: 规则 name 不能为空", source));
    }
    if rule.name.len() > MAX_NAME_LEN {
        return Err(format!(
            "{}: 规则 name 过长（>{}）: {}",
            source, MAX_NAME_LEN, rule.name
        ));
    }
    if rule.name.contains(['/', '\\', '\0', '\n', '\r']) {
        return Err(format!("{}: 规则 name 含非法字符: {}", source, rule.name));
    }
    if rule.pattern.is_empty() {
        return Err(format!("{}: 规则 pattern 不能为空 ({})", source, rule.name));
    }
    if rule.pattern.len() > MAX_PATTERN_LEN {
        return Err(format!(
            "{}: 规则 pattern 过长 ({})",
            source, rule.name
        ));
    }
    if let Err(e) = Regex::new(&rule.pattern) {
        return Err(format!(
            "{}: 正则语法错误 ({}): {}",
            source, rule.name, e
        ));
    }

    rule.mask = normalize_mask(&rule.mask, wrapper_style);
    if rule.mask.is_empty() {
        return Err(format!("{}: 规则 mask 不能为空 ({})", source, rule.name));
    }
    if rule.mask.len() > MAX_MASK_LEN {
        return Err(format!("{}: 规则 mask 过长 ({})", source, rule.name));
    }

    // 钳制 priority
    if rule.priority > 1000 {
        rule.priority = 1000;
    } else if rule.priority < -1000 {
        rule.priority = -1000;
    }

    rule.is_custom = true;
    Ok(rule)
}

/// 将多文件解析结果合并进现有自定义规则列表。
///
/// - `existing_custom`: 当前 user_rules.yaml 中的规则
/// - `builtin_names`: 内置规则名称集合（不可覆盖）
/// - 导入集内同名：后出现的覆盖先出现的
pub fn merge_import(
    existing_custom: Vec<Rule>,
    builtin_names: &std::collections::HashSet<String>,
    files: Vec<ParsedImportFile>,
    wrapper_style: &str,
    _policy: ConflictPolicy,
) -> ImportRulesReport {
    let mut items = Vec::new();
    let mut total_parsed = 0u32;
    let mut imported = 0u32;
    let mut overwritten = 0u32;
    let mut skipped = 0u32;
    let mut failed = 0u32;

    // 先在导入集内规范化并按出现顺序折叠同名
    let mut staged: Vec<(String, Rule, String)> = Vec::new(); // name, rule, source
    let mut staged_index: std::collections::HashMap<String, usize> =
        std::collections::HashMap::new();

    for file in files {
        for raw in file.rules {
            total_parsed += 1;
            let name_hint = raw.name.clone();
            match validate_and_normalize_rule(raw, wrapper_style, &file.source) {
                Ok(rule) => {
                    let name = rule.name.clone();
                    if let Some(&idx) = staged_index.get(&name) {
                        staged[idx] = (name, rule, file.source.clone());
                    } else {
                        staged_index.insert(name.clone(), staged.len());
                        staged.push((name, rule, file.source.clone()));
                    }
                }
                Err(msg) => {
                    failed += 1;
                    skipped += 1;
                    items.push(ImportRuleItemResult {
                        name: name_hint,
                        status: ImportItemStatus::SkippedInvalid,
                        message: msg,
                        source_file: Some(file.source.clone()),
                    });
                }
            }
        }
    }

    // 再与现有自定义 / 内置合并
    let mut custom_map: std::collections::HashMap<String, Rule> = existing_custom
        .into_iter()
        .map(|r| (r.name.clone(), r))
        .collect();

    for (name, rule, source) in staged {
        if builtin_names.contains(&name) {
            skipped += 1;
            items.push(ImportRuleItemResult {
                name: name.clone(),
                status: ImportItemStatus::SkippedBuiltin,
                message: format!("与内置规则同名，已跳过（不可覆盖内置）"),
                source_file: Some(source),
            });
            continue;
        }

        if custom_map.contains_key(&name) {
            custom_map.insert(name.clone(), rule);
            overwritten += 1;
            items.push(ImportRuleItemResult {
                name,
                status: ImportItemStatus::Overwritten,
                message: "已覆盖同名自定义规则".into(),
                source_file: Some(source),
            });
        } else {
            custom_map.insert(name.clone(), rule);
            imported += 1;
            items.push(ImportRuleItemResult {
                name,
                status: ImportItemStatus::Imported,
                message: "已导入".into(),
                source_file: Some(source),
            });
        }
    }

    // 保持稳定顺序：按 name 排序便于 diff/测试
    let mut merged: Vec<Rule> = custom_map.into_values().collect();
    merged.sort_by(|a, b| a.name.cmp(&b.name));

    ImportRulesReport {
        total_parsed,
        imported,
        overwritten,
        skipped,
        failed,
        items,
        merged_custom_rules: merged,
    }
}

/// 导出模板 YAML（固定字符串）。
pub fn rules_import_template_yaml() -> &'static str {
    r#"# SafeMask Rules Import Template v1
# 兼容 RuleGroup 对象格式；也可使用纯 rules 数组。
# name 全局唯一；与内置规则同名将被跳过；与自定义同名将被覆盖。
# mask 可写 EMAIL 或 <EMAIL> / [EMAIL]；裸标签会按应用当前包裹样式自动补齐。

group: "CUSTOM"
rules:
  - name: "Example_Internal_ID"
    pattern: '\bID-[0-9]{8}\b'
    mask: "<INTERNAL_ID>"
    priority: 20
    enabled: true

  - name: "Example_Keyword"
    pattern: "机密项目代号"
    mask: "<SECRET_CODE>"
    priority: 10
    enabled: true
"#
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

    #[test]
    fn parse_rule_group_and_array() {
        let group = r#"
group: CUSTOM
rules:
  - name: A
    pattern: 'foo'
    mask: "<A>"
"#;
        let g = parse_rules_yaml(group, "g.yaml").unwrap();
        assert_eq!(g.rules.len(), 1);

        let arr = r#"
- name: B
  pattern: 'bar'
  mask: "<B>"
"#;
        let a = parse_rules_yaml(arr, "a.yaml").unwrap();
        assert_eq!(a.rules.len(), 1);
    }

    #[test]
    fn normalize_mask_preserves_wrappers() {
        assert_eq!(normalize_mask("<EMAIL>", "square"), "<EMAIL>");
        assert_eq!(normalize_mask("[EMAIL]", "angle"), "[EMAIL]");
        assert_eq!(normalize_mask("EMAIL", "angle"), "<EMAIL>");
        assert_eq!(normalize_mask("EMAIL", "square"), "[EMAIL]");
    }

    #[test]
    fn reject_bad_regex() {
        let r = Rule {
            name: "bad".into(),
            pattern: "(".into(),
            mask: "<X>".into(),
            priority: 1,
            enabled: true,
            is_custom: false,
        };
        assert!(validate_and_normalize_rule(r, "angle", "t").is_err());
    }

    #[test]
    fn overwrite_custom_skip_builtin() {
        let existing = vec![Rule {
            name: "CustomOld".into(),
            pattern: "old".into(),
            mask: "<OLD>".into(),
            priority: 1,
            enabled: true,
            is_custom: true,
        }];
        let mut builtin = HashSet::new();
        builtin.insert("Builtin".into());

        let yaml = r#"
group: CUSTOM
rules:
  - name: CustomOld
    pattern: 'new'
    mask: "<NEW>"
  - name: Builtin
    pattern: 'x'
    mask: "<B>"
  - name: Fresh
    pattern: 'y'
    mask: "FRESH"
"#;
        let file = parse_rules_yaml(yaml, "imp.yaml").unwrap();
        let report = merge_import(
            existing,
            &builtin,
            vec![file],
            "angle",
            ConflictPolicy::OverwriteCustom,
        );

        assert_eq!(report.overwritten, 1);
        assert_eq!(report.imported, 1);
        assert_eq!(report.skipped, 1);
        assert!(
            report
                .merged_custom_rules
                .iter()
                .any(|r| r.name == "CustomOld" && r.pattern == "new")
        );
        assert!(
            report
                .merged_custom_rules
                .iter()
                .any(|r| r.name == "Fresh" && r.mask == "<FRESH>")
        );
        assert!(!report.merged_custom_rules.iter().any(|r| r.name == "Builtin"));
    }

    #[test]
    fn empty_file_fails() {
        assert!(parse_rules_yaml("   ", "e.yaml").is_err());
    }
}
