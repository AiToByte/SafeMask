/**
 * 核心算法逻辑
 */
use aho_corasick::AhoCorasick;
use regex::{Regex, RegexSet};
use std::borrow::Cow;
/// 预定义的模式, 实际项目中可移至rules.yaml
const STATIC_KEYWORDS: &[&str] = &["InternalSecret", "PrivateProject", "SuperAdmin"];
const REGEX_PATTERNS: &[&str] = &[
    r"sk-[a-zA-Z0-9]{32}",   // OpenAI
    r#"(jdbc:|postgres://|mongodb\+srv://)[^\s'"]+\s*"#, // 数据库连接
    r"\b(?:\d{1,3}\.){3}\d{1,3}\b",                    // IPv4
];

pub struct MaskEngine {
    ac: AhoCorasick,
    re_set: RegexSet,
    re_list: Vec<Regex>,
    mask: &'static str,
}

impl MaskEngine {
     pub fn new() -> Self {
        Self {
            ac: AhoCorasick::new(STATIC_KEYWORDS).expect("AC 引擎编译失败"),
            re_set: RegexSet::new(REGEX_PATTERNS).expect("RegexSet 编译失败"),
            re_list: REGEX_PATTERNS.iter().map(|p| Regex::new(p).unwrap()).collect(),
            mask: "<SAFE_MASK>",
        }
    }

    /// 核心脱敏逻辑，使用 Cow 避免不必要的字符串拷贝
    pub fn mask_line<'a>(&self, input: &'a str) -> Cow<'a, str> {

        
         // 第一阶段：Aho-Corasick 字典匹配（极速）
        // 为每一个模式提供一个替换标签（即创建一个包含 3 个 self.mask 的 Vec）
        let replacements = vec![self.mask; STATIC_KEYWORDS.len()]; 
        
        // 使用针对模式数量相等的替换逻辑
        let mut result = self.ac.replace_all(input, &replacements);

        // 第二阶段：RegexSet 探测是否有模式命中
        let matches: Vec<_> = self.re_set.matches(&result).into_iter().collect();
        
        if matches.is_empty() {
            // 如果既没命中字典也没命中正则，返回原始借用，零开销
            return Cow::Owned(result); 
        }

        // 第三阶段：针对命中的正则执行替换
        for index in matches {
            result = self.re_list[index]
                .replace_all(&result, self.mask)
                .into_owned();
        }
        Cow::Owned(result)
    }
}