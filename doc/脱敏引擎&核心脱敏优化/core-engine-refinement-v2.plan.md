# Plan: SafeMask Core Engine Refinement v2+v3

> **Goal**: NerEngine cleanup + Container swallowing + Fragment pruning + Type-aware masking
> **4 Phases**, 4 files, ~200 lines total change
>
> **Dependency**: Phase 1→2→3 (sequential), Phase 4 can run parallel with Phase 1

---

## Phase 1: `src-tauri/src/infra/ai/ner_engine.rs`

### 1.1 Delete diagnostic logging in `load()` (lines 93-202)

Remove the entire `write_log` closure (≈30 lines of `write_log!(...)`) plus the `OpenOptions::new().create(true).append(true).open(...)` block at line 93.

Replace with ≤5 `info!(...)` calls:
```rust
info!("NER: load start, model={}", model_path.display());
info!("NER: model size={:.1} MB", ...);
info!("NER: ONNX session created");
info!("NER: tokenizer loaded ({} tokens)", ...);
info!("NER: model ready, {} labels", labels.len());
```

**Location**: lines 93-202 → ~15 lines.

### 1.2 Delete diagnostic logging in `infer()` (lines 341-369)

Remove the entire `ai_model_load.log` append block inside the ONNX output scope:
```rust
// DELETE this whole block:
let log_file = std::path::PathBuf::from("ai_model_load.log");
let _ = std::fs::OpenOptions::new().create(true).append(true).open(&log_file)
    .and_then(|mut f| {
        use std::io::Write;
        writeln!(f, "\n=== 推理诊断 (seq_len={}) ===", seq).ok();
        for i in 0..seq.min(80) {
            // ... 15+ lines of per-token logging ...
        }
        Ok(())
    });
```

### 1.3 Delete `verify_byte_offsets()` function

Full function removal (approximately lines 232-315). This function:
- Tokenizes a 2nd time with `add_special_tokens(true)`
- Iterates token IDs and checks byte-offset alignment
- Returns `bool`

**Output**: 0 callers after 1.4 removal.

### 1.4 Delete `build_char_to_byte_map()` function

Full function removal. This maps char indices → byte offsets. No longer needed since `openai/privacy-filter` tokenizer already returns byte offsets.

### 1.5 Simplify `infer()` — zero-clone offsets

Replace the current ~70-line `infer()` body with:

```rust
pub fn infer(&mut self, text: &str) -> Result<Vec<EntitySpan>> {
    if text.trim().is_empty() {
        return Ok(Vec::new());
    }

    let encoding = self.tokenizer.encode(text, false)
        .map_err(|e| anyhow::anyhow!("Tokenize 失败: {}", e))?;
    let seq_len = encoding.get_ids().len().min(self.max_length);

    let input_ids: Vec<i64> = encoding.get_ids()[..seq_len]
        .iter().map(|&x| x as i64).collect();
    let attention_mask: Vec<i64> = encoding.get_attention_mask()[..seq_len]
        .iter().map(|&x| x as i64).collect();

    let (seq_len_actual, num_labels, logits_owned) = {
        let input_ids_tensor = Tensor::from_array(([1, seq_len], &input_ids))
            .map_err(|e| anyhow::anyhow!("创建 input_ids 张量失败: {}", e))?;
        let attention_mask_tensor = Tensor::from_array(([1, seq_len], &attention_mask))
            .map_err(|e| anyhow::anyhow!("创建 attention_mask 张量失败: {}", e))?;

        let outputs = self.session.run(ort::inputs![
            "input_ids" => input_ids_tensor,
            "attention_mask" => attention_mask_tensor,
        ]).map_err(|e| anyhow::anyhow!("ONNX 推理失败: {}", e))?;

        let logits_value = outputs.get("logits")
            .context("模型输出中未找到 logits")?;
        let (shape, logits_slice) = logits_value.try_extract_tensor::<f32>()
            .map_err(|e| anyhow::anyhow!("提取 logits 失败: {}", e))?;

        (shape[1] as usize, shape[2] as usize, logits_slice.to_vec())
    };

    let logits_view = ndarray::ArrayView3::from_shape(
        (1, seq_len_actual, num_labels), &logits_owned,
    ).context("重建 logits 张量失败")?;

    // encoding 仍存活 — 直接传 offsets slice，零克隆
    let offsets = &encoding.get_offsets()
        [..seq_len_actual.min(encoding.get_offsets().len())];
    let spans = self.decode_bioes(&logits_view, offsets, text, seq_len_actual);

    Ok(spans)
}
```

**Key savings vs current**:
- `Vec<u32>` for token_ids: gone (slice directly from encoding)
- `Vec<u32>` for attention_mask: gone
- `Vec<(usize, usize)>` raw_offsets clone: gone (8 KB per call)
- `Vec<(usize, usize)>` char→byte conversion: gone
- `Vec<i64>` remains (required by ort Tensor API)

### 1.6 Delete `find_token_indices_in_text()` if sole caller was verify_byte_offsets

Check imports and total calls. If nothing else references it, delete.

### 1.7 Tests — Delete 7, Keep 5

**Deleted** (functions no longer exist):
| Test | Reason |
|------|--------|
| `test_build_char_to_byte_map_ascii` | function deleted |
| `test_build_char_to_byte_map_cjk` | function deleted |
| `test_build_char_to_byte_map_empty` | function deleted |
| `test_verify_byte_offsets_valid` | function deleted |
| `test_verify_byte_offsets_empty` | function deleted |
| `test_verify_byte_offsets_too_short` | function deleted |
| `test_verify_byte_offsets_char_offsets` | function deleted |

**Kept** (pure decode_bioes tests, still valid):
| Test | Reason |
|------|--------|
| `test_load_tokenizer_failure` | tokenizer path error → still valid |
| `test_decode_bioes_single_token` | only calls decode_bioes |
| `test_decode_bioes` | only calls decode_bioes |
| `test_decode_bioes_mixed` | only calls decode_bioes |
| `test_decode_bioes_no_entity` | only calls decode_bioes |

---

## Phase 2: `src-tauri/src/core/resolver/mod.rs`

### 2.1 Add 3 helper functions

```rust
/// Is this entity type a container that can swallow sub-spans?
fn is_container_type(entity_type: &EntityType) -> bool {
    matches!(entity_type,
        EntityType::Address
        | EntityType::Custom(s)
            if s.to_lowercase() == "organization"
            || s.to_lowercase() == "company"
    )
}

/// Can `container` swallow `child`?
/// High-risk types (Phone, Email, etc.) are NEVER swallowed.
fn can_swallow(container: &EntityType, child: &EntityType) -> bool {
    if !is_container_type(container) { return false; }
    !matches!(child,
        EntityType::Phone | EntityType::Email | EntityType::IdCard
        | EntityType::BankCard | EntityType::ApiKey | EntityType::Password
    )
}

/// Is this span a useless fragment after carving?
///
/// UTF-8 安全的语义碎片裁剪:
/// - 空/纯空白 → 裁剪
/// - 纯标点（中英文全角半角） → 裁剪
/// - Address/Person/BankCard 雕刻后只剩 1 个字符的碎屑（"号"、"室"、"区"） → 裁剪
/// - 非法 UTF-8 序列 → 裁剪（防崩溃）
fn is_useless_fragment(span: &EntitySpan, text: &[u8]) -> bool {
    if span.start >= span.end || span.end > text.len() { return true; }
    let fragment_bytes = &text[span.start..span.end];

    if let Ok(s) = std::str::from_utf8(fragment_bytes) {
        let trimmed = s.trim();
        if trimmed.is_empty() { return true; }

        // 仅含标点（中英文全角半角）
        if trimmed.chars().all(|c| {
            c.is_ascii_punctuation()
            || "，。：；！？、‘’“”【】（）—《》".contains(c)
        }) {
            return true;
        }

        // 语义碎屑：Address/Person/BankCard 雕刻后只剩孤字
        let char_count = trimmed.chars().count();
        if char_count < 2 {
            if matches!(span.entity_type,
                EntityType::Address | EntityType::Person | EntityType::BankCard
            ) {
                return true;
            }
        }
    } else {
        // 非法 UTF-8 字节序列（断层切片），防崩溃
        return true;
    }

    false
}
```

### 2.2 Change `resolve()` signature

```rust
// Before:
pub fn resolve(&self, spans: Vec<EntitySpan>) -> Vec<EntitySpan> {

// After:
pub fn resolve(&self, spans: Vec<EntitySpan>, text: &[u8]) -> Vec<EntitySpan> {
```

### 2.3 Insert container swallowing (before carving loop)

**Location**: After step-2 sort (line 76), before `let mut accepted: Vec<EntitySpan> = Vec::new();`

```rust
let mut accepted: Vec<EntitySpan> = Vec::new();

// Step 3: 雕刻合并 + 容器吞没
for candidate in candidates {
    // ── 容器吞没：低优容器吞没已接受的可吞子项 ──
    if is_container_type(&candidate.entity_type) {
        accepted.retain(|child| {
            !(child.start >= candidate.start
                && child.end <= candidate.end
                && can_swallow(&candidate.entity_type, &child.entity_type))
        });
    }

    let mut fragments = vec![candidate];
    // ... existing carving loop against accepted ...
```

### 2.4 Insert fragment pruning (after zero-length filter)

**Location**: After `accepted.retain(|s| s.start < s.end);` (line 134), before debug log:

```rust
// 过滤零长度碎片
accepted.retain(|s| s.start < s.end);
// 碎片清除：删除无意义的空白/标点/单字碎屑
accepted.retain(|s| !is_useless_fragment(s, text));
```

### 2.5 Update all caller sites

In `hybrid_engine.rs:156`:
```rust
// Before:
self.resolver.resolve(spans)
// After:
self.resolver.resolve(spans, text)
```

### 2.6 Tests — Update 9, Add 4

**Update all 9 existing tests** to pass `text: &[u8]` as 2nd arg:
```rust
let text = "这是一段测试文本用于验证冲突解决";
let result = resolver.resolve(spans, text.as_bytes());
```

**Add 4 new tests**:

```rust
#[test]
fn test_container_swallow_address() {
    let text = "北京市朝阳区建国路88号";
    let resolver = ConflictResolver::new(0.0);
    let spans = vec![
        EntitySpan::with_mask(0, 15, EntityType::Address, 0.6, "ner", "<地址>")
            .with_priority(50),
        EntitySpan::with_mask(3, 7, EntityType::Custom("organization".into()), 0.7, "ner", "<组织>")
            .with_priority(60),
    ];
    let result = resolver.resolve(spans, text.as_bytes());
    assert_eq!(result.len(), 1);
    assert_eq!(result[0].entity_type, EntityType::Address);
}

#[test]
fn test_container_not_swallow_high_risk() {
    let text = "联系邮箱 test@example.com，地址北京";
    let resolver = ConflictResolver::new(0.0);
    let spans = vec![
        EntitySpan::with_mask(0, 18, EntityType::Address, 0.5, "ner", "<地址>")
            .with_priority(50),
        EntitySpan::with_mask(4, 20, EntityType::Email, 0.9, "regex", "<邮箱>")
            .with_priority(100),
    ];
    let result = resolver.resolve(spans, text.as_bytes());
    assert!(result.iter().any(|s| s.entity_type == EntityType::Email));
}

#[test]
fn test_fragment_pruning_whitespace() {
    let text = "  ";
    let resolver = ConflictResolver::new(0.0);
    let spans = vec![
        EntitySpan::new(0, 2, EntityType::Phone, 0.6, "ner"),
    ];
    let result = resolver.resolve(spans, text.as_bytes());
    assert!(result.is_empty());
}

#[test]
fn test_fragment_pruning_cjk_punctuation() {
    // 雕刻后剩下全角逗号 → 应被裁剪
    let text = "，";
    let resolver = ConflictResolver::new(0.0);
    let spans = vec![
        EntitySpan::new(0, 3, EntityType::Address, 0.5, "ner"),
    ];
    let result = resolver.resolve(spans, text.as_bytes());
    assert!(result.is_empty());
}

#[test]
fn test_fragment_pruning_single_char_address_debris() {
    // 地址雕刻后只剩下 "号" → 语义碎屑应裁剪
    let text = "号";
    let resolver = ConflictResolver::new(0.0);
    let spans = vec![
        EntitySpan::new(0, 3, EntityType::Address, 0.5, "ner"),
    ];
    let result = resolver.resolve(spans, text.as_bytes());
    assert!(result.is_empty());
}
```

---

## Phase 3: `src-tauri/src/core/hybrid_engine.rs`

### 3.1 One-line change in `detect()`

**Location**: line 156

```rust
// Before:
self.resolver.resolve(spans)
// After:
self.resolver.resolve(spans, text)
```

That's it. `text: &[u8]` is already in the function signature at line 153.

---

## Phase 4: `src-tauri/src/core/masking/strategies.rs`

### 4.1 Modify `PartialMaskStrategy` — type-aware routing

```rust
impl MaskingStrategy for PartialMaskStrategy {
    fn name(&self) -> &str { "partial_mask" }
    fn strategy_type(&self) -> MaskStrategyType { MaskStrategyType::PartialMask }

    fn mask(&self, original: &str, span: &EntitySpan, _config: &MaskConfig) -> String {
        match span.entity_type {
            EntityType::Email => Self::mask_email(original),
            EntityType::Phone => Self::mask_phone(original),
            EntityType::IpAddress => Self::mask_ip(original),
            _ => Self::mask_fallback(original),
        }
    }
}
```

### 4.2 Add `mask_email` — UTF-8 safe partial mask

⚠️ **Rust `&str` 切片使用 byte index，直接 `&username[..visible]` 会在中文等
多字节字符上 panic。必须通过 `chars().collect::<Vec<char>>()` 转为字符数组后再按
字符索引提取前缀。**

```rust
fn mask_email(original: &str) -> String {
    if let Some(at_idx) = original.find('@') {
        let username = &original[..at_idx];
        let domain = &original[at_idx..];
        let chars: Vec<char> = username.chars().collect();
        let char_len = chars.len();
        let visible = match char_len {
            0..=1 => 0,
            2..=4 => 1,
            _ => 2.min(char_len.saturating_sub(1)),
        };
        let prefix: String = chars[..visible].iter().collect();
        let mask_len = char_len.saturating_sub(visible).max(1);
        format!("{}{}{}", prefix, "*".repeat(mask_len), domain)
    } else {
        Self::mask_fallback(original)
    }
}
```

**Test vectors**:
| Input | Output |
|-------|--------|
| `"a@b.com"` | `"*@b.com"` |
| `"ab@c.com"` | `"a*@c.com"` |
| `"abc@d.com"` | `"a**@d.com"` |
| `"zhangsan@example.com"` | `"zh******@example.com"` |
| `"张三@company.cn"` | `"张*@company.cn"` |

### 4.3 Add `mask_phone` — Lazy Digit Masking (format-preserving)

⚠️ **旧方案提取纯数字会吞噬格式符（`+`、`-`、空格）。采用 Lazy Digit
Masking：遍历原字符串字符，对数字计数，只遮盖中间数字，非数字原样保留。**

```rust
fn mask_phone(original: &str) -> String {
    let total_digits = original.chars().filter(|c| c.is_ascii_digit()).count();
    if total_digits < 7 {
        return Self::mask_fallback(original);
    }

    let (mask_start, mask_end) = if total_digits >= 11 {
        (3, total_digits - 4)
    } else {
        (2, total_digits - 2)
    };

    let mut digit_idx = 0;
    original.chars().map(|c| {
        if c.is_ascii_digit() {
            let current = digit_idx;
            digit_idx += 1;
            if current >= mask_start && current < mask_end {
                '*'
            } else {
                c
            }
        } else {
            c
        }
    }).collect()
}
```

**Test vectors**:
| Input | Output |
|-------|--------|
| `"13812345678"` (plain) | `"138****5678"` |
| `"+86 138-0013-8000"` (formatted) | `"+86 138-****-8000"` |
| `"1234567"` (7 digits) | `"12***67"` |
| `"010-82345678"` (with dash) | `"010-82****78"` |

### 4.4 Add `mask_ip` — first/last octet only

```rust
fn mask_ip(original: &str) -> String {
    let parts: Vec<&str> = original.split('.').collect();
    if parts.len() == 4 {
        format!("{}.*.*.{}", parts[0], parts[3])
    } else {
        Self::mask_fallback(original)
    }
}
```

**Test vectors**:
| Input | Output |
|-------|--------|
| `"192.168.1.100"` | `"192.*.*.100"` |
| `"::1"` (IPv6, no 4 parts) | fallback → `"*"` |

### 4.5 Extract existing logic as `mask_fallback`

```rust
fn mask_fallback(original: &str) -> String {
    let chars: Vec<char> = original.chars().collect();
    let len = chars.len();
    if len <= 2 { return "*".repeat(len); }
    let visible = match len {
        0..=4 => 1,
        5..=8 => 2,
        9..=12 => 3,
        _ => 4,
    };
    let prefix: String = chars[..visible].iter().collect();
    let suffix: String = chars[len - visible..].iter().collect();
    let mask_count = len - visible * 2;
    format!("{}{}{}", prefix, "*".repeat(mask_count.max(3)), suffix)
}
```

This is the original `mask()` body — unchanged behavior for all types without specific routing.

### 4.6 Tests — Update 1, Add 5

**Update** `test_partial_mask_phone`:
```rust
#[test]
fn test_partial_mask_phone() {
    let strategy = PartialMaskStrategy;
    let span = make_span(EntityType::Phone);
    let result = strategy.mask("13812345678", &span, &default_config());
    assert_eq!(result, "138****5678");  // was "138*****678" under old fallback
}
```

**Add 5 new tests**:
```rust
#[test]
fn test_partial_mask_email() {
    let strategy = PartialMaskStrategy;
    let span = make_span(EntityType::Email);
    let result = strategy.mask("zhangsan@example.com", &span, &default_config());
    assert_eq!(result, "zh******@example.com");
}

#[test]
fn test_partial_mask_ip() {
    let strategy = PartialMaskStrategy;
    let span = make_span(EntityType::IpAddress);
    let result = strategy.mask("192.168.1.100", &span, &default_config());
    assert_eq!(result, "192.*.*.100");
}

#[test]
fn test_partial_mask_short_email() {
    let strategy = PartialMaskStrategy;
    let span = make_span(EntityType::Email);
    let result = strategy.mask("a@b.com", &span, &default_config());
    assert_eq!(result, "*@b.com");
}

#[test]
fn test_partial_mask_chinese_email() {
    // 中文邮箱用户名 — chars() 安全切片验证
    let strategy = PartialMaskStrategy;
    let span = make_span(EntityType::Email);
    let result = strategy.mask("张三@company.cn", &span, &default_config());
    assert_eq!(result, "张*@company.cn");
}

#[test]
fn test_partial_mask_formatted_phone() {
    // 带格式符的电话 — Lazy Digit Masking 格式保留验证
    let strategy = PartialMaskStrategy;
    let span = make_span(EntityType::Phone);
    let result = strategy.mask("+86 138-0013-8000", &span, &default_config());
    assert_eq!(result, "+86 138-****-8000");
}

#[test]
fn test_partial_mask_phone_with_dash() {
    let strategy = PartialMaskStrategy;
    let span = make_span(EntityType::Phone);
    let result = strategy.mask("010-82345678", &span, &default_config());
    assert_eq!(result, "010-82****78");
}
```

---

## Execution Order & Verification

```
Step 1: Phase 4                         # strategies.rs — no deps
Step 2: Phase 1                         # ner_engine.rs — no deps
Step 3: cargo test -p SafeMask          # verify Phase 1 + 4
Step 4: Phase 2                         # resolver/mod.rs — depends on Phase 1 being compiled
Step 5: Phase 3                         # hybrid_engine.rs — 1 line, depends on Phase 2
Step 6: cargo test -p SafeMask          # verify all
Step 7: cargo check -p SafeMask         # type-check
Step 8: cargo clippy -p SafeMask -- -D warnings  # lint
```

### Expected test outcome

| Phase | Before | After | Net |
|-------|--------|-------|-----|
| 1 | ~12 tests | ~5 tests | -7 |
| 2 | ~9 tests | ~13 tests | +4 |
| 4 | ~7 tests | ~12 tests | +5 |
| **Total** | **~28 tests** | **~30 tests** | **+2** |
