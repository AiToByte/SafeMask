use crate::config::Rule;
use regex::{Regex, Captures};
use std::borrow::Cow;
use aho_corasick::{AhoCorasick, MatchKind};

pub struct MaskEngine {
   // 处理正则模式
    combined_regex: Option<Regex>,
    regex_masks: Vec<Vec<u8>>,
    
    ac_engine: Option<AhoCorasick>,
    ac_masks: Vec<Vec<u8>>, // 遮罩也存为字节流
}

impl MaskEngine {
    pub fn new(rules: Vec<Rule>) -> Self {
        let mut regex_patterns = Vec::new();
        let mut regex_masks = Vec::new();
        
        let mut ac_patterns = Vec::new();
        let mut ac_masks = Vec::new();

        for rule in rules {
            // 简单的启发式判断：如果正则不包含特殊字符，则视为固定词
            if is_literal(&rule.pattern) {
                ac_patterns.push(rule.pattern);
                ac_masks.push(rule.mask.into_bytes());
            } else {
                regex_patterns.push(format!("({})", rule.pattern));
                regex_masks.push(rule.mask.into_bytes());
            }
        }

        let combined_regex = if !regex_patterns.is_empty() {
            let pattern_str = regex_patterns.join("|");
            // 将 expect 改为更友好的处理或打印
            match Regex::new(&pattern_str) {
                Ok(re) => Some(re),
                Err(e) => {
                    eprintln!("❌ 正则编译错误: {}", e);
                    eprintln!("💡 提示: Rust regex 不支持环视断言 (?!) 或 (?<!)，请检查 rules 目录下的 YAML 规则。");
                    std::process::exit(1); // 优雅退出而不是 panic
                }
            }
        } else {
            None
        };

        let ac_engine = if !ac_patterns.is_empty() {
            Some(AhoCorasick::builder()
                .match_kind(MatchKind::LeftmostLongest) // 匹配最长路径，防止子串干扰
                .build(ac_patterns)
                .expect("AC 引擎初始化失败"))
        } else {
            None
        };

        Self {
            combined_regex,
            regex_masks,
            ac_engine,
            ac_masks,
        }
    }

    pub fn mask_line<'a>(&self, input: &'a [u8]) -> Cow<'a, [u8]> {
        // --- 第一阶段: AC 引擎处理 (固定词) ---
        // 如果 AC 引擎存在，处理后产生 Cow::Owned(String)；否则保持 Cow::Borrowed(&'a str)
        let ac_result = if let Some(ref ac) = self.ac_engine {
            // 注意：Aho-Corasick 的 replace_all 总是返回 String
            // 为了优化，你可以在此处先调用 find 判断是否有匹配，但通常直接处理即可
            Cow::Owned(ac.replace_all_bytes(input, &self.ac_masks))
        } else {
            Cow::Borrowed(input)
        };
        // --- 第二阶段: Regex 引擎处理 (模式匹配) ---
        let re_engine = match &self.combined_regex {
            Some(re) => re,
            None => return ac_result, // 如果没有正则规则，直接返回第一阶段结果
        };
       // 执行单次扫描替换
        // 这里的 re_result 生命周期受限于 ac_result
        let re_result = re_engine.replace_all(&ac_result, |caps: &Captures| {
            for i in 0..self.regex_masks.len() {
                if caps.get(i + 1).is_some() {
                    return self.regex_masks[i].as_slice();
                }
            }
            b"<MASKED>"
        });
            // --- 生命周期修复核心逻辑 ---
        match re_result {
            // 情况 1: 正则引擎修改了文本，产生了新的 String
            // 将其所有权通过 Cow::Owned 转移给调用者
            Cow::Owned(s) => Cow::Owned(s),

            // 情况 2: 正则引擎没动过文本（Borrowed）
            // 此时 re_result 指向的是 ac_result 的内存。
            // 为了避免生命周期报错，我们直接返回 ac_result。
            // 这样返回的生命周期就回到了 ac_result 拥有的所有权或 input 的借用。
            Cow::Borrowed(_) => ac_result,
        }
    }
}

/// 简单的辅助函数：判断是否为纯文本（无正则特殊符号）
fn is_literal(pattern: &str) -> bool {
    let specials = [
        '.', '+', '*', '?', '(', ')', '[', ']', '{', '}', '|', '^', '$', '\\',
    ];
    !pattern.chars().any(|c| specials.contains(&c))
}