//! NER 推理引擎
//!
//! 基于 ONNX Runtime 的 NER (Named Entity Recognition) 推理引擎。
//! 加载 `openai/privacy-filter` 等 token 分类模型，执行 PII 检测。
//!
//! # 推理流程
//!
//! 1. Tokenize 输入文本
//! 2. 准备输入张量 (input_ids, attention_mask)
//! 3. 执行 ONNX 推理
//! 4. BIOES 后处理 → 实体跨度
//!
//! # 模型格式
//!
//! 期望的模型目录结构：
//! ```text
//! models/
//! └── privacy-filter/
//!     ├── model.onnx        # ONNX 模型文件
//!     ├── tokenizer.json    # HuggingFace tokenizer
//!     └── config.json       # 模型配置（可选）
//! ```

use crate::core::recognizer::{EntitySpan, EntityType};
use anyhow::{Context, Result};
use log::info;
use std::path::Path;
use ort::session::Session;
use ort::value::Tensor;

/// BIOES 标签
#[derive(Debug, Clone, PartialEq)]
enum BioesLabel {
    /// Outside (非实体)
    O,
    /// Beginning (实体开始)
    B(String),
    /// Inside (实体内部)
    I(String),
    /// End (实体结束)
    E(String),
    /// Single (单 token 实体)
    S(String),
}

impl BioesLabel {
    /// 从字符串解析 BIOES 标签
    fn parse(label: &str) -> Self {
        if label == "O" {
            return Self::O;
        }

        if let Some(rest) = label.strip_prefix("B-") {
            Self::B(rest.to_string())
        } else if let Some(rest) = label.strip_prefix("I-") {
            Self::I(rest.to_string())
        } else if let Some(rest) = label.strip_prefix("E-") {
            Self::E(rest.to_string())
        } else if let Some(rest) = label.strip_prefix("S-") {
            Self::S(rest.to_string())
        } else {
            Self::O
        }
    }
}

/// NER 推理引擎
pub struct NerEngine {
    /// ONNX Runtime 会话
    session: Session,
    /// Tokenizer
    tokenizer: tokenizers::Tokenizer,
    /// 标签列表 (索引 → 标签)
    labels: Vec<String>,
    /// 最大序列长度
    max_length: usize,
}

impl NerEngine {
    /// 从模型目录加载 NER 引擎
    pub fn load(model_dir: impl AsRef<Path>) -> Result<Self> {
        let model_dir = model_dir.as_ref();

        // 优先查找 q4 量化版本，然后是标准版本
        let model_path = if model_dir.join("model_q4.onnx").exists() {
            model_dir.join("model_q4.onnx")
        } else {
            model_dir.join("model.onnx")
        };
        let tokenizer_path = model_dir.join("tokenizer.json");

        // 注意：ORT_SKIP_DOWNLOAD 是 ort-sys 编译期的 build env var，
        // 运行时设置无效，已在 Cargo.toml 中通过 features = ["copy-dylibs"] 处理

        info!("NER: loading model from {}", model_path.display());

        let mut builder = Session::builder()
            .map_err(|e| anyhow::anyhow!("创建 ONNX Session 失败: {}", e))?;

        builder = builder
            .with_optimization_level(ort::session::builder::GraphOptimizationLevel::Level3)
            .map_err(|e| anyhow::anyhow!("设置优化级别失败: {}", e))?;

        let ort_threads = std::env::var("ORT_NUM_THREADS")
            .ok()
            .and_then(|s| s.parse::<usize>().ok())
            .unwrap_or(2);
        builder = builder
            .with_intra_threads(ort_threads)
            .map_err(|e| anyhow::anyhow!("设置 intra_threads 失败: {}", e))?;
        builder = builder
            .with_inter_threads(ort_threads)
            .map_err(|e| anyhow::anyhow!("设置 inter_threads 失败: {}", e))?;

        let session = builder.commit_from_file(&model_path)
            .map_err(|e| anyhow::anyhow!("加载模型文件失败 {}: {}", model_path.display(), e))?;

        info!("NER: ONNX session created");

        let tokenizer = tokenizers::Tokenizer::from_file(&tokenizer_path)
            .map_err(|e| anyhow::anyhow!("加载 Tokenizer 失败: {}", e))?;

        info!("NER: tokenizer loaded");

        let labels = Self::load_labels(model_dir).unwrap_or_else(Self::default_labels);

        info!("NER: model ready, {} labels", labels.len());

        Ok(Self {
            session,
            tokenizer,
            labels,
            max_length: 512,
        })
    }

    /// 从配置文件加载标签列表
    fn load_labels(model_dir: &Path) -> Option<Vec<String>> {
        let config_path = model_dir.join("config.json");
        if !config_path.exists() {
            return None;
        }

        let content = std::fs::read_to_string(config_path).ok()?;
        let config: serde_json::Value = serde_json::from_str(&content).ok()?;

        if let Some(id2label) = config.get("id2label")
            && let Some(map) = id2label.as_object() {
                let mut labels = vec![String::new(); map.len()];
                for (id_str, label) in map {
                    if let (Ok(idx), Some(label_str)) = (id_str.parse::<usize>(), label.as_str())
                        && idx < labels.len() {
                            labels[idx] = label_str.to_string();
                        }
                }
                if labels.iter().all(|l| !l.is_empty()) {
                    return Some(labels);
                }
            }

        None
    }

    /// 默认 BIOES 标签列表 (openai/privacy-filter 兼容)
    fn default_labels() -> Vec<String> {
        vec![
            "O".to_string(),
            "B-account_number".to_string(),
            "I-account_number".to_string(),
            "B-private_address".to_string(),
            "I-private_address".to_string(),
            "B-private_date".to_string(),
            "I-private_date".to_string(),
            "B-private_email".to_string(),
            "I-private_email".to_string(),
            "B-private_person".to_string(),
            "I-private_person".to_string(),
            "B-private_phone".to_string(),
            "I-private_phone".to_string(),
            "B-private_url".to_string(),
            "I-private_url".to_string(),
            "B-secret".to_string(),
            "I-secret".to_string(),
            "E-account_number".to_string(),
            "E-private_address".to_string(),
            "E-private_date".to_string(),
            "E-private_email".to_string(),
            "E-private_person".to_string(),
            "E-private_phone".to_string(),
            "E-private_url".to_string(),
            "E-secret".to_string(),
            "S-account_number".to_string(),
            "S-private_address".to_string(),
            "S-private_date".to_string(),
            "S-private_email".to_string(),
            "S-private_person".to_string(),
            "S-private_phone".to_string(),
            "S-private_url".to_string(),
            "S-secret".to_string(),
        ]
    }

    /// 执行 NER 推理
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
            let input_ids_tensor = Tensor::from_array(([1, seq_len], input_ids))
                .map_err(|e| anyhow::anyhow!("创建 input_ids 张量失败: {}", e))?;
            let attention_mask_tensor = Tensor::from_array(([1, seq_len], attention_mask))
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

    /// BIOES 后处理
    fn decode_bioes(
        &self,
        logits: &ndarray::ArrayView3<f32>,
        offsets: &[(usize, usize)],
        original_text: &str,
        seq_len: usize,
    ) -> Vec<EntitySpan> {
        let mut spans = Vec::new();
        let mut current_entity: Option<(usize, String, f32)> = None;

        for i in 0..seq_len {
            let token_logits = logits.slice(ndarray::s![0, i, ..]);
            let (label_idx, score) = softmax_argmax(&token_logits);

            let label_str = self.labels.get(label_idx)
                .map(|s| s.as_str())
                .unwrap_or("O");
            let label = BioesLabel::parse(label_str);

            let (tok_start, tok_end) = offsets.get(i).copied().unwrap_or((0, 0));

            match label {
                BioesLabel::O => {
                    if let Some((start, entity_name, conf)) = current_entity.take() {
                        spans.push(make_span(start, tok_start, &entity_name, conf));
                    }
                }
                BioesLabel::S(entity_name) => {
                    if let Some((start, prev_name, conf)) = current_entity.take() {
                        spans.push(make_span(start, tok_start, &prev_name, conf));
                    }
                    spans.push(make_span(tok_start, tok_end, &entity_name, score));
                }
                BioesLabel::B(entity_name) => {
                    if let Some((start, prev_name, conf)) = current_entity.take() {
                        spans.push(make_span(start, tok_start, &prev_name, conf));
                    }
                    current_entity = Some((tok_start, entity_name, score));
                }
                BioesLabel::I(entity_name) => {
                    if let Some((_, ref cur_name, _)) = current_entity {
                        if *cur_name != entity_name {
                            if let Some((start, prev_name, conf)) = current_entity.take() {
                                spans.push(make_span(start, tok_start, &prev_name, conf));
                            }
                            current_entity = Some((tok_start, entity_name, score));
                        }
                    } else {
                        current_entity = Some((tok_start, entity_name, score));
                    }
                }
                BioesLabel::E(entity_name) => {
                    if let Some((start, cur_name, conf)) = current_entity.take() {
                        if cur_name == entity_name {
                            spans.push(make_span(start, tok_end, &entity_name, conf.min(score)));
                        } else {
                            spans.push(make_span(start, tok_start, &cur_name, conf));
                            spans.push(make_span(tok_start, tok_end, &entity_name, score));
                        }
                    } else {
                        spans.push(make_span(tok_start, tok_end, &entity_name, score));
                    }
                }
            }
        }

        // 结束最后一个实体
        if let Some((start, entity_name, conf)) = current_entity.take() {
            let text_len = original_text.len();
            spans.push(make_span(start, text_len, &entity_name, conf));
        }

        // 修剪前导/后缀空白（模型常将前导空格纳入实体，导致残余碎片）
        for span in &mut spans {
            let span_bytes = &original_text.as_bytes()[span.start..span.end];
            let leading = span_bytes.iter().take_while(|&&b| b == b' ' || b == b'\t' || b == b'\n' || b == b'\r').count();
            let trailing = span_bytes.iter().rev().take_while(|&&b| b == b' ' || b == b'\t' || b == b'\n' || b == b'\r').count();
            span.start += leading;
            span.end = span.end.saturating_sub(trailing);
        }

        // 修复 offset
        fix_offsets(&mut spans, original_text);

        spans
    }
}

/// 创建实体跨度
fn make_span(start: usize, end: usize, entity_name: &str, confidence: f32) -> EntitySpan {
    EntitySpan::new(
        start,
        end,
        map_entity_type(entity_name),
        confidence,
        "ner_engine",
    )
}

/// Softmax + Argmax
fn softmax_argmax(logits: &ndarray::ArrayView1<f32>) -> (usize, f32) {
    let mut max_idx = 0;
    let mut max_val = f32::NEG_INFINITY;

    for (i, &val) in logits.iter().enumerate() {
        if val > max_val {
            max_val = val;
            max_idx = i;
        }
    }

    let exp_sum: f64 = logits.iter()
        .map(|&x| (x as f64 - max_val as f64).exp())
        .sum();
    let prob = (1.0 / exp_sum) as f32;

    (max_idx, prob)
}

/// 修复 byte offset — 确保 span 边界落在有效的 UTF-8 字符边界上
///
/// start 向**前**回退到最近的字符边界，end 向**后**推进到最近的字符边界。
/// 这样可以防止 span 切分多字节 UTF-8 字符。
fn fix_offsets(spans: &mut Vec<EntitySpan>, text: &str) {
    let text_len = text.len();

    for span in spans.iter_mut() {
        span.start = span.start.min(text_len);
        span.end = span.end.min(text_len);

        if span.start > span.end {
            std::mem::swap(&mut span.start, &mut span.end);
        }

        // start 向后退到字符边界
        while span.start > 0 && !text.is_char_boundary(span.start) {
            span.start -= 1;
        }
        // end 向前推到字符边界
        while span.end < text_len && !text.is_char_boundary(span.end) {
            span.end += 1;
        }
    }

    spans.retain(|s| s.start < s.end);
}

/// 将模型实体名称映射到 EntityType
fn map_entity_type(name: &str) -> EntityType {
    match name.to_lowercase().as_str() {
        "person" | "person_name" | "private_person" => EntityType::Person,
        "email" | "email_address" | "private_email" => EntityType::Email,
        "phone" | "phone_number" | "private_phone" => EntityType::Phone,
        "address" | "street_address" | "private_address" => EntityType::Address,
        "account_number" => EntityType::BankCard,
        "date" | "private_date" | "dob" => EntityType::DateOfBirth,
        "url" | "website" | "private_url" => EntityType::Url,
        "secret" | "api_key" | "token" => EntityType::ApiKey,
        other => EntityType::Custom(other.to_string()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bioes_label_parse() {
        assert_eq!(BioesLabel::parse("O"), BioesLabel::O);
        assert_eq!(BioesLabel::parse("B-person"), BioesLabel::B("person".to_string()));
        assert_eq!(BioesLabel::parse("I-person"), BioesLabel::I("person".to_string()));
        assert_eq!(BioesLabel::parse("E-person"), BioesLabel::E("person".to_string()));
        assert_eq!(BioesLabel::parse("S-person"), BioesLabel::S("person".to_string()));
    }

    #[test]
    fn test_map_entity_type() {
        assert_eq!(map_entity_type("person"), EntityType::Person);
        assert_eq!(map_entity_type("private_email"), EntityType::Email);
        assert_eq!(map_entity_type("phone"), EntityType::Phone);
        assert_eq!(map_entity_type("secret"), EntityType::ApiKey);
    }

}
