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
use std::path::{Path, PathBuf};
use ort::session::Session;
use ort::value::Tensor;

/// 查找系统已安装的 ONNX Runtime DLL
fn find_onnxruntime_dll() -> Option<PathBuf> {
    // 1. 检查环境变量
    if let Ok(path) = std::env::var("ORT_DYLIB_PATH") {
        let p = PathBuf::from(path);
        if p.exists() {
            return Some(p);
        }
    }

    // 2. 检查系统目录
    let system_paths = vec![
        PathBuf::from("C:\\Windows\\System32\\onnxruntime.dll"),
        PathBuf::from("C:\\Windows\\SysWOW64\\onnxruntime.dll"),
    ];

    for path in system_paths {
        if path.exists() {
            return Some(path);
        }
    }

    // 3. 检查 PATH 环境变量中的目录
    if let Ok(path_env) = std::env::var("PATH") {
        for dir in path_env.split(';') {
            let dll_path = PathBuf::from(dir).join("onnxruntime.dll");
            if dll_path.exists() {
                return Some(dll_path);
            }
        }
    }

    None
}

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

        // 写入日志文件
        let log_file = std::path::PathBuf::from("ai_model_load.log");
        let write_log = |msg: &str| {
            let _ = std::fs::OpenOptions::new()
                .create(true).append(true).open(&log_file)
                .and_then(|mut f| {
                    use std::io::Write;
                    writeln!(f, "[NerEngine] {}", msg)
                });
        };

        write_log(&format!("开始加载模型: {}", model_path.display()));
        write_log(&format!("模型文件大小: {:.1} MB", model_path.metadata().map(|m| m.len()).unwrap_or(0) as f64 / 1024.0 / 1024.0));

        // 设置环境变量，跳过 ONNX Runtime 下载，使用系统已安装的版本
        // SAFETY: 在单线程初始化阶段设置环境变量是安全的
        unsafe { std::env::set_var("ORT_SKIP_DOWNLOAD", "1"); }
        write_log("已设置 ORT_SKIP_DOWNLOAD=1，使用系统 ONNX Runtime");

        // 1. 初始化 ONNX Runtime 环境
        write_log("步骤 1/3: 初始化 ONNX Runtime 环境...");

        let start_time = std::time::Instant::now();

        // 跳过环境初始化，直接创建 Session
        // ort 会在需要时自动创建环境
        write_log("  跳过显式环境初始化，直接创建 Session...");

        // 2. 创建 Session 并加载模型
        write_log("步骤 2/4: 创建 ONNX Session 并加载模型...");
        write_log("  调用 Session::builder()...");
        let mut builder = Session::builder()
            .map_err(|e| {
                write_log(&format!("❌ 创建 Session Builder 失败: {}", e));
                anyhow::anyhow!("创建 ONNX Session 失败: {}", e)
            })?;
        write_log("  Session::builder() 成功");

        write_log("  设置优化级别...");
        builder = builder
            .with_optimization_level(ort::session::builder::GraphOptimizationLevel::Level3)
            .map_err(|e| {
                write_log(&format!("❌ 设置优化级别失败: {}", e));
                anyhow::anyhow!("设置优化级别失败: {}", e)
            })?;
        write_log("  优化级别设置成功");

        // 🚀 限制 ONNX Runtime 线程数，降低资源占用
        let ort_threads = std::env::var("ORT_NUM_THREADS")
            .ok()
            .and_then(|s| s.parse::<usize>().ok())
            .unwrap_or(2);
        write_log(&format!("  设置 ONNX 线程数: intra={}, inter={}", ort_threads, ort_threads));
        builder = builder
            .with_intra_threads(ort_threads)
            .map_err(|e| {
                write_log(&format!("⚠️ 设置 intra_threads 失败: {}", e));
                anyhow::anyhow!("设置 intra_threads 失败: {}", e)
            })?;
        builder = builder
            .with_inter_threads(ort_threads)
            .map_err(|e| {
                write_log(&format!("⚠️ 设置 inter_threads 失败: {}", e));
                anyhow::anyhow!("设置 inter_threads 失败: {}", e)
            })?;
        write_log("  线程数设置成功");

        write_log(&format!("  从文件加载模型: {}", model_path.display()));
        write_log("  调用 commit_from_file()...");
        write_log("  （这可能需要几分钟，请耐心等待...）");

        let commit_start = std::time::Instant::now();
        write_log(&format!("  开始时间: {:?}", commit_start));

        let session = match builder.commit_from_file(&model_path) {
            Ok(s) => {
                let commit_elapsed = commit_start.elapsed();
                write_log(&format!("  commit_from_file 完成！耗时: {:.1} 秒", commit_elapsed.as_secs_f64()));
                s
            }
            Err(e) => {
                let commit_elapsed = commit_start.elapsed();
                write_log(&format!("❌ 加载模型文件失败！耗时: {:.1} 秒", commit_elapsed.as_secs_f64()));
                write_log(&format!("  错误详情: {}", e));
                return Err(anyhow::anyhow!("加载模型文件失败 {}: {}", model_path.display(), e));
            }
        };

        let elapsed = start_time.elapsed();
        write_log(&format!("✅ ONNX 模型加载成功！总耗时: {:.1} 秒", elapsed.as_secs_f64()));

        // 3. 加载 Tokenizer
        write_log("步骤 3/4: 加载 Tokenizer...");
        let tokenizer = tokenizers::Tokenizer::from_file(&tokenizer_path)
            .map_err(|e| {
                write_log(&format!("❌ 加载 Tokenizer 失败: {}", e));
                anyhow::anyhow!("加载 Tokenizer 失败: {}", e)
            })?;

        write_log("✅ Tokenizer 加载成功");

        // 4. 加载标签列表
        write_log("步骤 4/4: 加载标签列表...");
        let labels = Self::load_labels(model_dir).unwrap_or_else(|| {
            write_log("⚠️ 未找到标签配置，使用默认 BIOES 标签");
            Self::default_labels()
        });

        write_log(&format!("✅ 标签列表加载成功: {} 个标签", labels.len()));
        write_log("🎉 模型加载完成！");

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

        if let Some(id2label) = config.get("id2label") {
            if let Some(map) = id2label.as_object() {
                let mut labels = vec![String::new(); map.len()];
                for (id_str, label) in map {
                    if let (Ok(idx), Some(label_str)) = (id_str.parse::<usize>(), label.as_str()) {
                        if idx < labels.len() {
                            labels[idx] = label_str.to_string();
                        }
                    }
                }
                if labels.iter().all(|l| !l.is_empty()) {
                    return Some(labels);
                }
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

        // 1. Tokenize (提取所有需要的数据到 owned 变量)
        let encoding = self.tokenizer.encode(text, false)
            .map_err(|e| anyhow::anyhow!("Tokenize 失败: {}", e))?;

        let token_ids: Vec<u32> = encoding.get_ids().to_vec();
        let attention_mask: Vec<u32> = encoding.get_attention_mask().to_vec();
        let offsets: Vec<(usize, usize)> = encoding.get_offsets().to_vec();
        let seq_len = token_ids.len().min(self.max_length);

        let input_ids_i64: Vec<i64> = token_ids[..seq_len].iter().map(|&x| x as i64).collect();
        let attention_mask_i64: Vec<i64> = attention_mask[..seq_len].iter().map(|&x| x as i64).collect();

        // 2. 创建输入张量 (ort 2.0 API)
        let input_ids_tensor = Tensor::from_array(([1, seq_len], input_ids_i64))
            .map_err(|e| anyhow::anyhow!("创建 input_ids 张量失败: {}", e))?;
        let attention_mask_tensor = Tensor::from_array(([1, seq_len], attention_mask_i64))
            .map_err(|e| anyhow::anyhow!("创建 attention_mask 张量失败: {}", e))?;

        // 3. 执行推理并提取 logits (在同一个作用域内完成，释放 outputs 的 borrow)
        let (seq_len_actual, num_labels, logits_owned) = {
            let outputs = self.session.run(ort::inputs![
                "input_ids" => input_ids_tensor,
                "attention_mask" => attention_mask_tensor,
            ])
            .map_err(|e| anyhow::anyhow!("ONNX 推理失败: {}", e))?;

            let logits_value = outputs.get("logits").context("模型输出中未找到 logits")?;
            let (shape, logits_slice) = logits_value.try_extract_tensor::<f32>()
                .map_err(|e| anyhow::anyhow!("提取 logits 失败: {}", e))?;

            let seq = shape[1] as usize;
            let labels = shape[2] as usize;
            let data: Vec<f32> = logits_slice.to_vec();

            // 诊断日志：记录模型对每个 token 的预测
            let log_file = std::path::PathBuf::from("ai_model_load.log");
            let _ = std::fs::OpenOptions::new().create(true).append(true).open(&log_file)
                .and_then(|mut f| {
                    use std::io::Write;
                    writeln!(f, "\n=== 推理诊断 (seq_len={}) ===", seq).ok();
                    for i in 0..seq.min(30) {
                        let offset = offsets.get(i).copied().unwrap_or((0, 0));
                        let token_text = if offset.0 < text.len() && offset.1 <= text.len() {
                            &text[offset.0..offset.1]
                        } else { "" };
                        let start_idx = i * labels;
                        let end_idx = start_idx + labels;
                        let token_logits = &data[start_idx..end_idx.min(data.len())];
                        let (best_label, best_score) = if !token_logits.is_empty() {
                            let mut max_idx = 0usize;
                            let mut max_val = f32::NEG_INFINITY;
                            for (li, &v) in token_logits.iter().enumerate() {
                                if v > max_val { max_val = v; max_idx = li; }
                            }
                            let label_str = self.labels.get(max_idx).map(|s| s.as_str()).unwrap_or("O");
                            let score = ((1.0_f64) / token_logits.iter()
                                .map(|&x| ((x - max_val) as f64).exp())
                                .sum::<f64>()) as f32;
                            (label_str, score)
                        } else { ("O", 0.0) };
                        if best_label != "O" {
                            write!(f, "  token[{}] \"{}\" [{:?}..{:?}]: {} ({:.3})", i, token_text, offset.0, offset.1, best_label, best_score).ok();
                        }
                    }
                    writeln!(f, "=== 诊断结束 ===").ok();
                    Ok::<_, std::io::Error>(())
                });

            (seq, labels, data)
            // outputs 在这里被 drop，释放 mutable borrow
        };

        // 4. 重建 ndarray 视图
        let logits_view = ndarray::ArrayView3::from_shape(
            (1, seq_len_actual, num_labels),
            &logits_owned,
        ).context("重建 logits 张量失败")?;

        // 5. BIOES 后处理
        let spans = self.decode_bioes(&logits_view, &offsets, text, seq_len_actual);

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
