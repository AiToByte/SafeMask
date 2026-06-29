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

// ─────────────────────────────────────────────────────────────────────────────
// Tests
// ─────────────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::rules::Rule;

    fn make_rule(name: &str, pattern: &str, mask: &str, priority: i32) -> Rule {
        Rule {
            name: name.to_string(),
            pattern: pattern.to_string(),
            mask: mask.to_string(),
            priority,
            enabled: true,
            is_custom: false,
        }
    }

    // =====================================================================
    // (a) AI key in English text
    // =====================================================================

    #[test]
    fn test_ai_key_in_english_text() {
        let rules = vec![
            make_rule("OpenAI", r"\bsk-[a-zA-Z0-9\-]{32,}\b", "<OPENAI_KEY>", 10),
        ];
        let engine = MaskEngine::new(rules);

        let input = b"My key is sk-proj-abcDEF1234567890abcdef1234567890xyz here";
        let result = engine.mask_line(input);
        let output = String::from_utf8_lossy(&result);

        println!("English test:");
        println!("  Input:  {}", std::str::from_utf8(input).unwrap());
        println!("  Output: {}", output);
        println!("  Bytes:  {:?}", &result[..]);

        assert_eq!(output, "My key is <OPENAI_KEY> here");

        // Verify byte offsets are correct:
        //   "My key is " = 10 bytes, mask = 12 bytes, " here" = 5 bytes
        assert_eq!(result.len(), 27);
        assert_eq!(&result[..10], b"My key is ");
        assert_eq!(&result[10..22], b"<OPENAI_KEY>");
        assert_eq!(&result[22..], b" here");
    }

    // =====================================================================
    // (b) AI key embedded in Chinese text
    // =====================================================================

    #[test]
    fn test_ai_key_in_chinese_text() {
        let rules = vec![
            make_rule("OpenAI", r"\bsk-[a-zA-Z0-9\-]{32,}\b", "<OPENAI_KEY>", 10),
        ];
        let engine = MaskEngine::new(rules);

        let input_str = "我的密钥是sk-proj-abcDEF1234567890abcdef1234567890xyz在这里";
        let input = input_str.as_bytes();

        let result = engine.mask_line(input);
        let output = String::from_utf8_lossy(&result);

        println!("Chinese test:");
        println!("  Input:  {}", input_str);
        println!("  Output: {}", output);
        println!("  Input bytes: {:?}", input);
        println!("  Output bytes: {:?}", &result[..]);

        // Byte layout of input:
        //   我的密钥是  = 5 × 3 = 15 bytes (bytes 0..15)
        //   sk-proj-...xyz = 46 bytes          (bytes 15..61)
        //   在这里  = 3 × 3 = 9 bytes           (bytes 61..70)
        assert_eq!(input.len(), 70);
        assert_eq!(&input[..15], "我的密钥是".as_bytes());
        assert_eq!(&input[15..61], b"sk-proj-abcDEF1234567890abcdef1234567890xyz");
        assert_eq!(&input[61..], "在这里".as_bytes());

        // The mask must appear at the correct byte position (right after the 15-byte prefix)
        assert_eq!(output, "我的密钥是<OPENAI_KEY>在这里");
        assert_eq!(&result[..15], "我的密钥是".as_bytes());
        assert_eq!(&result[15..27], b"<OPENAI_KEY>");
        assert_eq!(&result[27..], "在这里".as_bytes());
    }

    // =====================================================================
    // (c) Overlapping OpenAI + DeepSeek patterns
    // =====================================================================

    #[test]
    fn test_overlapping_openai_deepseek() {
        // This OpenAI key also happens to start with 32 lowercase+digits,
        // so the DeepSeek pattern also matches the same prefix.
        let rules = vec![
            make_rule("OpenAI",   r"\bsk-[a-zA-Z0-9\-]{32,}\b", "<OPENAI_KEY>",   10),
            make_rule("DeepSeek", r"\bsk-[a-z0-9]{32}\b",       "<DEEPSEEK_KEY>", 10),
        ];
        let engine = MaskEngine::new(rules);

        let input = b"sk-proj-abcDEF1234567890abcdef1234567890xyz";
        let result = engine.mask_line(input);
        let output = String::from_utf8_lossy(&result);

        println!("Overlap test:");
        println!("  Input:  {}", std::str::from_utf8(input).unwrap());
        println!("  Output: {}", output);

        // Both match at start=0. OpenAI span is 0..46, DeepSeek is 0..35.
        // Sort by (start ASC, priority DESC, length DESC) => OpenAI first.
        // OpenAI accepted, DeepSeek skipped (overlaps).
        assert_eq!(output, "<OPENAI_KEY>");
        assert_eq!(result.len(), 12);
    }

    #[test]
    fn test_overlapping_pure_deepseek_key() {
        // A pure DeepSeek key (all lowercase, exactly 32 chars) matches BOTH patterns:
        //   OpenAI:   sk-[a-zA-Z0-9\-]{32,}  =>  matches entire 35-byte key
        //   DeepSeek: sk-[a-z0-9]{32}        =>  matches entire 35-byte key
        let rules = vec![
            make_rule("OpenAI",   r"\bsk-[a-zA-Z0-9\-]{32,}\b", "<OPENAI_KEY>",   10),
            make_rule("DeepSeek", r"\bsk-[a-z0-9]{32}\b",       "<DEEPSEEK_KEY>", 10),
        ];
        let engine = MaskEngine::new(rules);

        let input = b"key=sk-abcdef1234567890abcdef1234567890ab end";
        let result = engine.mask_line(input);
        let output = String::from_utf8_lossy(&result);

        println!("Pure DeepSeek overlap test:");
        println!("  Input:  {}", std::str::from_utf8(input).unwrap());
        println!("  Output: {}", output);

        // Both match at same position with same length (35 bytes).
        // Either mask is acceptable; the key point is only ONE replacement occurs.
        let is_openai   = output == "key=<OPENAI_KEY> end";
        let is_deepseek = output == "key=<DEEPSEEK_KEY> end";
        assert!(is_openai || is_deepseek, "Unexpected output: {}", output);
    }

    // =====================================================================
    // (d) Multiple matches in one line (email + phone + IP)
    // =====================================================================

    #[test]
    fn test_multiple_matches_email_phone_ip() {
        let rules = vec![
            make_rule("email", r"[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}", "<EMAIL>", 5),
            make_rule("phone", r"1[3-9]\d{9}",                                     "<PHONE>", 5),
            make_rule("ip",    r"\b\d{1,3}\.\d{1,3}\.\d{1,3}\.\d{1,3}\b",         "<IP>",    5),
        ];
        let engine = MaskEngine::new(rules);

        let input = b"Email test@example.com phone 13800138000 ip 192.168.1.1 end";
        let result = engine.mask_line(input);
        let output = String::from_utf8_lossy(&result);

        println!("Multiple test:");
        println!("  Input:  {}", std::str::from_utf8(input).unwrap());
        println!("  Output: {}", output);

        assert!(output.contains("<EMAIL>"), "Missing EMAIL in: {}", output);
        assert!(output.contains("<PHONE>"), "Missing PHONE in: {}", output);
        assert!(output.contains("<IP>"),    "Missing IP in: {}", output);

        // Verify exact output
        assert_eq!(output, "Email <EMAIL> phone <PHONE> ip <IP> end");
    }

    // =====================================================================
    // (e) Empty input, input with no matches
    // =====================================================================

    #[test]
    fn test_empty_input() {
        let rules = vec![
            make_rule("OpenAI", r"\bsk-[a-zA-Z0-9\-]{32,}\b", "<OPENAI_KEY>", 10),
        ];
        let engine = MaskEngine::new(rules);

        let result = engine.mask_line(b"");
        assert_eq!(result.len(), 0);
        assert!(matches!(result, Cow::Borrowed(_)), "Empty input should return Borrowed");
    }

    #[test]
    fn test_no_match() {
        let rules = vec![
            make_rule("OpenAI", r"\bsk-[a-zA-Z0-9\-]{32,}\b", "<OPENAI_KEY>", 10),
        ];
        let engine = MaskEngine::new(rules);

        let input = b"no sensitive data here";
        let result = engine.mask_line(input);
        assert_eq!(&*result, input);
        assert!(matches!(result, Cow::Borrowed(_)), "No-match should return Borrowed");
    }

    // =====================================================================
    // (f) UTF-8 boundary cases: Chinese chars adjacent to ASCII matches
    // =====================================================================

    #[test]
    fn test_chinese_before_key_no_space() {
        // Chinese character directly adjacent to key (no space).
        // \b in regex::bytes: last byte of 是 = 0xAF (non-word), s = 0x73 (word) => boundary matches.
        let rules = vec![
            make_rule("OpenAI", r"\bsk-[a-zA-Z0-9\-]{32,}\b", "<OPENAI_KEY>", 10),
        ];
        let engine = MaskEngine::new(rules);

        let input_str = "是sk-proj-abcDEF1234567890abcdef1234567890xyz";
        let input = input_str.as_bytes();
        let result = engine.mask_line(input);
        let output = String::from_utf8_lossy(&result);

        println!("Chinese-adjacent test:");
        println!("  Input:  {}", input_str);
        println!("  Output: {}", output);
        println!("  Input bytes:  {:?}", input);
        println!("  Output bytes: {:?}", &result[..]);

        // 是 = 3 bytes (E6 98 AF), key = 46 bytes
        assert_eq!(input.len(), 49);
        assert_eq!(output, "是<OPENAI_KEY>");
        assert_eq!(&result[..3], "是".as_bytes());
        assert_eq!(&result[3..15], b"<OPENAI_KEY>");
        assert_eq!(result.len(), 15);
    }

    #[test]
    fn test_chinese_after_key_no_space() {
        // Key directly followed by Chinese character (no space).
        // \b in regex::bytes: z = 0x7A (word), first byte of 在 = 0xE5 (non-word) => boundary matches.
        let rules = vec![
            make_rule("OpenAI", r"\bsk-[a-zA-Z0-9\-]{32,}\b", "<OPENAI_KEY>", 10),
        ];
        let engine = MaskEngine::new(rules);

        let input_str = "sk-proj-abcDEF1234567890abcdef1234567890xyz在";
        let input = input_str.as_bytes();
        let result = engine.mask_line(input);
        let output = String::from_utf8_lossy(&result);

        println!("Chinese-after test:");
        println!("  Input:  {}", input_str);
        println!("  Output: {}", output);

        // key = 46 bytes, 在 = 3 bytes
        assert_eq!(input.len(), 49);
        assert_eq!(output, "<OPENAI_KEY>在");
        assert_eq!(&result[..12], b"<OPENAI_KEY>");
        assert_eq!(&result[12..], "在".as_bytes());
    }

    #[test]
    fn test_chinese_both_sides_no_space() {
        // Chinese chars on BOTH sides of the key, no spaces.
        let rules = vec![
            make_rule("OpenAI", r"\bsk-[a-zA-Z0-9\-]{32,}\b", "<OPENAI_KEY>", 10),
        ];
        let engine = MaskEngine::new(rules);

        let input_str = "密钥sk-proj-abcDEF1234567890abcdef1234567890xyz在";
        let input = input_str.as_bytes();
        let result = engine.mask_line(input);
        let output = String::from_utf8_lossy(&result);

        println!("Chinese-both-sides test:");
        println!("  Input:  {}", input_str);
        println!("  Output: {}", output);

        // 密钥 = 6 bytes, key = 46 bytes, 在 = 3 bytes  =>  total 55
        assert_eq!(input.len(), 55);
        assert_eq!(output, "密钥<OPENAI_KEY>在");
        assert_eq!(&result[..6],  "密钥".as_bytes());
        assert_eq!(&result[6..18], b"<OPENAI_KEY>");
        assert_eq!(&result[18..], "在".as_bytes());
    }

    #[test]
    fn test_key_between_two_chinese_chars() {
        // Minimal case: one Chinese char on each side.
        let rules = vec![
            make_rule("OpenAI", r"\bsk-[a-zA-Z0-9\-]{32,}\b", "<OPENAI_KEY>", 10),
        ];
        let engine = MaskEngine::new(rules);

        let input_str = "是sk-proj-abcDEF1234567890abcdef1234567890xyz在";
        let input = input_str.as_bytes();
        let result = engine.mask_line(input);
        let output = String::from_utf8_lossy(&result);

        println!("Minimal Chinese surround test:");
        println!("  Input:  {}", input_str);
        println!("  Output: {}", output);

        assert_eq!(output, "是<OPENAI_KEY>在");
        // 是(3) + <OPENAI_KEY>(12) + 在(3) = 18 bytes
        assert_eq!(result.len(), 18);
    }

    // =====================================================================
    // (g) Byte offset correctness — verify masks at RIGHT positions
    // =====================================================================

    #[test]
    fn test_byte_offset_correctness_single_match() {
        let rules = vec![
            make_rule("OpenAI", r"\bsk-[a-zA-Z0-9\-]{32,}\b", "<OPENAI_KEY>", 10),
        ];
        let engine = MaskEngine::new(rules);

        let input_str = "ABCDsk-proj-abcDEF1234567890abcdef1234567890xyzEFGH";
        let input = input_str.as_bytes();
        let result = engine.mask_line(input);
        let output = String::from_utf8_lossy(&result);

        println!("Byte offset test:");
        println!("  Input:  {}", input_str);
        println!("  Output: {}", output);

        // ABCD = 4 bytes, key = 46 bytes, EFGH = 4 bytes  =>  total 54
        assert_eq!(input.len(), 54);
        // Expected: "ABCD<OPENAI_KEY>EFGH"  =>  4 + 12 + 4 = 20 bytes
        assert_eq!(output, "ABCD<OPENAI_KEY>EFGH");
        assert_eq!(result.len(), 20);

        // Verify each region byte-by-byte
        assert_eq!(&result[..4],   b"ABCD");
        assert_eq!(&result[4..16], b"<OPENAI_KEY>");
        assert_eq!(&result[16..],  b"EFGH");
    }

    #[test]
    fn test_byte_offset_correctness_two_matches() {
        let rules = vec![
            make_rule("OpenAI", r"\bsk-[a-zA-Z0-9\-]{32,}\b", "<KEY>",   10),
            make_rule("email", r"[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}", "<EMAIL>", 5),
        ];
        let engine = MaskEngine::new(rules);

        let input = b"AA test@example.com BB sk-proj-abcDEF1234567890abcdef1234567890xyz CC";
        let result = engine.mask_line(input);
        let output = String::from_utf8_lossy(&result);

        println!("Two-match offset test:");
        println!("  Input:  {}", std::str::from_utf8(input).unwrap());
        println!("  Output: {}", output);

        // Expected layout:
        //   "AA " (3) + "<EMAIL>" (7) + " BB " (4) + "<KEY>" (5) + " CC" (3) = 22 bytes
        assert_eq!(output, "AA <EMAIL> BB <KEY> CC");
        assert_eq!(result.len(), 22);
        assert_eq!(&result[..3],   b"AA ");
        assert_eq!(&result[3..10], b"<EMAIL>");
        assert_eq!(&result[10..14], b" BB ");
        assert_eq!(&result[14..19], b"<KEY>");
        assert_eq!(&result[19..],   b" CC");
    }

    #[test]
    fn test_byte_offset_with_chinese_and_multiple_matches() {
        // Chinese text + two different masked entities.
        // This tests that byte offsets remain correct even with multi-byte chars in the mix.
        let rules = vec![
            make_rule("OpenAI", r"\bsk-[a-zA-Z0-9\-]{32,}\b", "<KEY>",   10),
            make_rule("email", r"[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}", "<EMAIL>", 5),
        ];
        let engine = MaskEngine::new(rules);

        let input_str = "你好test@example.com你好sk-proj-abcDEF1234567890abcdef1234567890xyz你好";
        let input = input_str.as_bytes();
        let result = engine.mask_line(input);
        let output = String::from_utf8_lossy(&result);

        println!("Chinese + multi-match offset test:");
        println!("  Input:  {}", input_str);
        println!("  Output: {}", output);

        // Input byte layout:
        //   你好 = 6 bytes (0..6)
        //   test@example.com = 16 bytes (6..22)
        //   你好 = 6 bytes (22..28)
        //   sk-proj-...xyz = 46 bytes (28..74)
        //   你好 = 6 bytes (74..80)
        assert_eq!(input.len(), 80);

        // Expected output: "你好<EMAIL>你好<KEY>你好"
        //   你好(6) + <EMAIL>(7) + 你好(6) + <KEY>(5) + 你好(6) = 30 bytes
        assert_eq!(output, "你好<EMAIL>你好<KEY>你好");
        assert_eq!(result.len(), 30);

        assert_eq!(&result[..6],   "你好".as_bytes());
        assert_eq!(&result[6..13], b"<EMAIL>");
        assert_eq!(&result[13..19], "你好".as_bytes());
        assert_eq!(&result[19..24], b"<KEY>");
        assert_eq!(&result[24..30], "你好".as_bytes());
    }

    // =====================================================================
    // Additional edge cases discovered during analysis
    // =====================================================================

    #[test]
    fn test_word_boundary_prevents_match_after_underscore() {
        // \b in regex::bytes treats _ as a word character.
        // When the key is preceded by _, there's no word boundary between _ and s,
        // so the regex does NOT match. This is expected \b behavior.
        let rules = vec![
            make_rule("OpenAI", r"\bsk-[a-zA-Z0-9\-]{32,}\b", "<OPENAI_KEY>", 10),
        ];
        let engine = MaskEngine::new(rules);

        let input = b"API_key=sk-proj-abcDEF1234567890abcdef1234567890xyz";
        let result = engine.mask_line(input);
        let output = String::from_utf8_lossy(&result);

        println!("Word boundary _ test:");
        println!("  Input:  {}", std::str::from_utf8(input).unwrap());
        println!("  Output: {}", output);

        // '_' is a word char, 's' is a word char => no \b => no match.
        // The key is NOT masked.
        assert_eq!(output, std::str::from_utf8(input).unwrap());
    }

    #[test]
    fn test_word_boundary_allows_match_after_equals() {
        // '=' is NOT a word character, so \b matches between = and s.
        let rules = vec![
            make_rule("OpenAI", r"\bsk-[a-zA-Z0-9\-]{32,}\b", "<OPENAI_KEY>", 10),
        ];
        let engine = MaskEngine::new(rules);

        let input = b"key=sk-proj-abcDEF1234567890abcdef1234567890xyz";
        let result = engine.mask_line(input);
        let output = String::from_utf8_lossy(&result);

        println!("Word boundary = test:");
        println!("  Input:  {}", std::str::from_utf8(input).unwrap());
        println!("  Output: {}", output);

        assert_eq!(output, "key=<OPENAI_KEY>");
    }

    #[test]
    fn test_no_utf8_split_in_output() {
        // Stress test: ensure the output is always valid UTF-8 when the input is valid UTF-8.
        // Test with many Chinese chars surrounding a key.
        let rules = vec![
            make_rule("OpenAI", r"\bsk-[a-zA-Z0-9\-]{32,}\b", "<OPENAI_KEY>", 10),
        ];
        let engine = MaskEngine::new(rules);

        let input_str = "中文中文中文中文sk-proj-abcDEF1234567890abcdef1234567890xyz中文中文中文中文";
        let input = input_str.as_bytes();
        let result = engine.mask_line(input);
        let output = String::from_utf8_lossy(&result);

        // The output must be valid UTF-8 (no replacement chars)
        assert!(!output.contains('\u{FFFD}'), "Output contains replacement char: {}", output);
        assert_eq!(output, "中文中文中文中文<OPENAI_KEY>中文中文中文中文");

        // Verify round-trip: output bytes must be valid UTF-8
        let output_str = std::str::from_utf8(&result);
        assert!(output_str.is_ok(), "Output is not valid UTF-8: {:?}", &result[..]);
    }

    #[test]
    fn test_disabled_rule_is_ignored() {
        let rules = vec![
            Rule {
                name: "OpenAI".to_string(),
                pattern: r"\bsk-[a-zA-Z0-9\-]{32,}\b".to_string(),
                mask: "<OPENAI_KEY>".to_string(),
                priority: 10,
                enabled: false,  // disabled!
                is_custom: false,
            },
        ];
        let engine = MaskEngine::new(rules);

        let input = b"sk-proj-abcDEF1234567890abcdef1234567890xyz";
        let result = engine.mask_line(input);
        let output = String::from_utf8_lossy(&result);

        // Disabled rule should not match
        assert_eq!(output, std::str::from_utf8(input).unwrap());
    }

    #[test]
    fn test_higher_priority_rule_wins_on_overlap() {
        // Two rules match the same region; higher priority should win.
        let rules = vec![
            make_rule("low",  r"sk-[a-zA-Z0-9\-]{10,}", "<LOW>",  1),
            make_rule("high", r"sk-[a-zA-Z0-9\-]{20,}", "<HIGH>", 100),
        ];
        let engine = MaskEngine::new(rules);

        let input = b"test sk-proj-abcDEF1234567890abcdef1234567890xyz end";
        let result = engine.mask_line(input);
        let output = String::from_utf8_lossy(&result);

        println!("Priority test:");
        println!("  Input:  {}", std::str::from_utf8(input).unwrap());
        println!("  Output: {}", output);

        // Both match at same position. high priority (100) > low priority (1).
        assert_eq!(output, "test <HIGH> end");
    }

    #[test]
    fn test_literal_pattern_via_ac_engine() {
        // A literal pattern (no regex meta chars) goes through Aho-Corasick.
        let rules = vec![
            make_rule("company", "SafeMask", "<COMPANY>", 0),
        ];
        let engine = MaskEngine::new(rules);

        let input = b"Use SafeMask for privacy";
        let result = engine.mask_line(input);
        let output = String::from_utf8_lossy(&result);

        assert_eq!(output, "Use <COMPANY> for privacy");
    }

    #[test]
    fn test_literal_and_regex_together() {
        // Literal + regex patterns should work together without offset issues.
        let rules = vec![
            make_rule("company", "SafeMask", "<COMPANY>", 0),
            make_rule("email",   r"[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}", "<EMAIL>", 5),
        ];
        let engine = MaskEngine::new(rules);

        let input = b"SafeMask contact: test@example.com";
        let result = engine.mask_line(input);
        let output = String::from_utf8_lossy(&result);

        println!("Literal + regex test:");
        println!("  Input:  {}", std::str::from_utf8(input).unwrap());
        println!("  Output: {}", output);

        assert_eq!(output, "<COMPANY> contact: <EMAIL>");
    }

    #[test]
    fn test_literal_and_regex_with_chinese() {
        // Literal + regex + Chinese text — verifies AC and regex byte offsets are compatible.
        let rules = vec![
            make_rule("company", "SafeMask", "<COMPANY>", 0),
            make_rule("email",   r"[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}", "<EMAIL>", 5),
        ];
        let engine = MaskEngine::new(rules);

        let input_str = "工具SafeMask邮箱test@example.com好的";
        let input = input_str.as_bytes();
        let result = engine.mask_line(input);
        let output = String::from_utf8_lossy(&result);

        println!("Literal + regex + Chinese test:");
        println!("  Input:  {}", input_str);
        println!("  Output: {}", output);

        // Byte layout:
        //   工具 = 6 bytes (0..6)
        //   SafeMask = 8 bytes (6..14)
        //   邮箱 = 6 bytes (14..20)
        //   test@example.com = 16 bytes (20..36)
        //   好的 = 6 bytes (36..42)
        assert_eq!(input.len(), 42);

        assert_eq!(output, "工具<COMPANY>邮箱<EMAIL>好的");
        // Verify byte-level correctness
        assert_eq!(&result[..6],   "工具".as_bytes());
        assert_eq!(&result[6..14], b"<COMPANY>");
        assert_eq!(&result[14..20], "邮箱".as_bytes());
        assert_eq!(&result[20..27], b"<EMAIL>");
        assert_eq!(&result[27..33], "好的".as_bytes());
    }
}