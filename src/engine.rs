use crate::config::Rule;
use regex::bytes::{Captures, Regex};
use std::borrow::Cow;
use aho_corasick::{AhoCorasick, MatchKind};

/// 规则层级：用于存储编译后的正则及其对应的偏移量
struct RegexLayer {
    re: Regex,
    masks: Vec<Vec<u8>>,
    offsets: Vec<usize>,
}

pub struct MaskEngine {
    // 层级 1: AC 引擎 (固定词，处理 O(1) 匹配)
    ac_engine: Option<AhoCorasick>,
    ac_masks: Vec<Vec<u8>>,

    // 层级 2 & 3: 优先级正则引擎
    high_layer: Option<RegexLayer>,
    low_layer: Option<RegexLayer>,
}

impl MaskEngine {
     /// 构造函数：执行规则分类、优先级排序、捕获组计算及引擎编译
    pub fn new(mut rules: Vec<Rule>) -> Self {
        
        // 1. 优先级降序排序：高优先级在左侧，符合正则引擎匹配偏好
        rules.sort_by(|a, b| b.priority.cmp(&a.priority).then_with(|| a.name.cmp(&b.name)));

        let mut ac_p = Vec::new(); let mut ac_m = Vec::new();
        let mut hr_p = Vec::new(); let mut hr_m = Vec::new();
        let mut lr_p = Vec::new(); let mut lr_m = Vec::new();
        
        for rule in rules {
            if is_literal(&rule.pattern) {
                ac_p.push(rule.pattern);
                ac_m.push(rule.mask.as_bytes().to_vec());
            } else if rule.priority > 5 {
                hr_p.push(format!("({})", rule.pattern));
                hr_m.push(rule.mask.as_bytes().to_vec());
            } else {
                lr_p.push(format!("({})", rule.pattern));
                lr_m.push(rule.mask.as_bytes().to_vec());
            }
        }
        
        Self {
            ac_engine: if ac_p.is_empty() { None } else {
                Some(AhoCorasick::builder()
                    .match_kind(MatchKind::LeftmostLongest)
                    .build(ac_p).expect("AC 引擎构建失败"))
            },
            ac_masks: ac_m,
            high_layer: Self::compile_layer(hr_p, hr_m),
            low_layer: Self::compile_layer(lr_p, lr_m),
        }
    }

     /// 编译正则层，并计算每个规则的捕获组起始索引
    fn compile_layer(patterns: Vec<String>, masks: Vec<Vec<u8>>) -> Option<RegexLayer> {
        if patterns.is_empty() { return None; }

        let mut offsets = Vec::new();
        let mut current_offset = 1; // 0 是整个匹配项，第一个规则从 1 开始

        for p in &patterns {
            let temp_re = Regex::new(p).expect("正则语法错误");
            offsets.push(current_offset);
            // temp_re 已经是 "(pattern)" 形式
            // 它在整体正则中占用的组数就是它的 captures_len() - 1
            // 但由于 joined 是 "(p1)|(p2)"，每一组其实只占用了原本的组空间
            current_offset += temp_re.captures_len() - 1; // 动态计算该规则占用的组数量
        }

        let combined_re = Regex::new(&patterns.join("|")).expect("超级正则聚合失败");

        Some(RegexLayer {
            re: combined_re,
            masks,
            offsets,
        })
    }

    /// 执行脱敏流程：AC -> High Priority Regex -> Low Priority Regex
    pub fn mask_line<'a>(&self, input: &'a [u8]) -> Cow<'a, [u8]> {
        // 第一阶段: AC 引擎
        let mut result = if let Some(ref ac) = self.ac_engine {
            Cow::Owned(ac.replace_all_bytes(input, &self.ac_masks))
        } else {
            Cow::Borrowed(input)
        };

        // 第二阶段: 高优先级正则
        if let Some(ref layer) = self.high_layer {
            let next = layer.re.replace_all(&result, |caps: &Captures| {
                self.dispatch(layer, caps)
            });
            // 关键：如果发生了替换（Owned），转移所有权；否则保持原样（维持 'a 生命周期）
            result = match next {
                Cow::Owned(v) => Cow::Owned(v),
                Cow::Borrowed(_) => result,
            };
        }

        // 第三阶段: 低优先级正则
        if let Some(ref layer) = self.low_layer {
            let next = layer.re.replace_all(&result, |caps: &Captures| {
                self.dispatch(layer, caps)
            });
            result = match next {
                Cow::Owned(v) => Cow::Owned(v),
                Cow::Borrowed(_) => result,
            };
        }
        result
    }

    fn dispatch(&self, layer: &RegexLayer, caps: &Captures) -> Vec<u8> {
        for (i, &offset) in layer.offsets.iter().enumerate() {
            if caps.get(offset).is_some() {
                return layer.masks[i].clone();
            }
        }
        b"<MASKED>".to_vec()
    }  
}

/// 简单的辅助函数：判断是否为纯文本（无正则特殊符号）
fn is_literal(pattern: &str) -> bool {
    let specials = [
        '.', '+', '*', '?', '(', ')', '[', ']', '{', '}', '|', '^', '$', '\\',
    ];
    !pattern.chars().any(|c| specials.contains(&c))
}