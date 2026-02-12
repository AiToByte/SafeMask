use crate::core::rules::Rule;
use aho_corasick::{AhoCorasick, MatchKind};
use regex::bytes::{Regex};
use std::borrow::Cow;
use smallvec::SmallVec;
use log::{info};  // 添加导入

const LITERAL_PRIORITY: i32 = 999_000_000;  // 远高于普通规则

/// 内部结构：存储编译后的单个正则规则
struct CompiledRegex {
    re: Regex,
    mask: Vec<u8>,
    priority: i32,
}

/// 匹配片段记录：用于冲突合并
struct MatchSpan<'m> {
    start: usize,
    end: usize,
    priority: i32,
    mask: &'m [u8],
}

pub struct MaskEngine {
    /// 词典引擎：处理固定字符串
    ac_engine: Option<AhoCorasick>,
    ac_masks: Vec<Vec<u8>>,
    
    /// 正则引擎列表：按优先级排序
    regex_rules: Vec<CompiledRegex>,
}

impl MaskEngine {
    /// 构造引擎：自动分类规则并预编译
    pub fn new(mut rules: Vec<Rule>) -> Self {
        info!("⚙️ 引擎构建开始，规则数: {}", rules.len());
        // 1. 过滤未启用的规则并按优先级降序排列
        rules.retain(|r| r.enabled);
        rules.sort_by(|a, b| b.priority.cmp(&a.priority));

        let mut ac_patterns = Vec::new();
        let mut ac_masks = Vec::new();
        let mut regex_rules = Vec::new();

        for rule in rules {
            if is_literal(&rule.pattern) {
                // 固定词放入 AC 引擎
                ac_patterns.push(rule.pattern);
                ac_masks.push(rule.mask.as_bytes().to_vec());
            } else {
                // 正则表达式预编译
                if let Ok(re) = Regex::new(&rule.pattern) {
                    regex_rules.push(CompiledRegex {
                        re,
                        mask: rule.mask.as_bytes().to_vec(),
                        priority: rule.priority,
                    });
                } else {
                    eprintln!("⚠️ [Engine] 忽略无效正则: {}", rule.name);
                }
            }
        }

        let ac_engine = if ac_patterns.is_empty() {
            None
        } else {
            AhoCorasick::builder()
                .match_kind(MatchKind::LeftmostLongest)
                .build(ac_patterns)
                .ok()
        };

        Self {
            ac_engine,
            ac_masks,
            regex_rules,
        }
    }

    /// 核心方法：对单行字节流进行脱敏
    pub fn mask_line<'a>(&self, input: &'a [u8]) -> Cow<'a, [u8]> {
        if input.is_empty() {
            return Cow::Borrowed(input);
        }

        // 🚀 使用 SmallVec 优化：预留 16 个插槽在栈上，覆盖 99% 的单行匹配场景
        let mut matches: SmallVec<[MatchSpan; 16]> = SmallVec::new();

        // Stage 1: AC 自动机匹配 (固定词)
        if let Some(ref ac) = self.ac_engine {
            for mat in ac.find_iter(input) {
                matches.push(MatchSpan {
                    start: mat.start(),
                    end: mat.end(),
                    priority: LITERAL_PRIORITY, // 固定词通常拥有最高优先级
                    mask: &self.ac_masks[mat.pattern()],
                });
            }
        }

        // Stage 2: 正则匹配
        for rule in &self.regex_rules {
            for mat in rule.re.find_iter(input) {
                matches.push(MatchSpan {
                    start: mat.start(),
                    end: mat.end(),
                    priority: rule.priority,
                    mask: &rule.mask,
                });
            }
        }

        if matches.is_empty() {
            return Cow::Borrowed(input);
        }

        // Stage 3: 冲突解决与结果合并 (关联函数调用)
        self.apply_replacements(input, matches)
    }

    /// 解决覆盖冲突：采用贪婪合并策略
    fn apply_replacements<'a, 'm, I>(
        &self,
        input: &'a [u8],
        matches: I,
    ) -> Cow<'a, [u8]>
    where
        I: IntoIterator<Item = MatchSpan<'m>>,
    {
        // 如果你还需要排序和去重，就必须先收集
        let mut matches: Vec<MatchSpan<'m>> = matches.into_iter().collect();
        // 下面排序
        matches.sort_unstable_by(|a, b| {
            a.start.cmp(&b.start)
                .then(b.priority.cmp(&a.priority))
                .then((b.end - b.start).cmp(&(a.end - a.start)))
        });

        let mut output = Vec::with_capacity(input.len());
        let mut last_pos = 0;

        for m in matches {
            if m.start < last_pos {
                continue;
            }
            output.extend_from_slice(&input[last_pos..m.start]);
            output.extend_from_slice(m.mask);
            last_pos = m.end;
        }

        if last_pos < input.len() {
            output.extend_from_slice(&input[last_pos..]);
        }

        Cow::Owned(output)
    }
}

/// 辅助函数：检测是否为固定词（无正则特殊符号）
fn is_literal(pattern: &str) -> bool {
    let meta = ['.', '+', '*', '?', '(', ')', '|', '[', ']', '{', '}', '^', '$', '\\'];
    !pattern.chars().any(|c| meta.contains(&c))
}