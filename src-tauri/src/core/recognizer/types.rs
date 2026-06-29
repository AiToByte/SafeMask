//! 识别引擎核心类型定义
//!
//! 定义所有识别器共用的数据结构，包括实体类型、实体跨度、分析上下文等。
//! 这些类型是识别引擎层的"语言"，所有识别器都通过它们进行通信。

use serde::{Deserialize, Serialize};
use std::collections::HashMap;


// ─────────────────────────────────────────────────────────────────────────────
// 实体类型 (EntityType)
// ─────────────────────────────────────────────────────────────────────────────

/// 可识别的实体类型
///
/// 内置类型覆盖最常见的 PII 场景，`Custom` 变体支持任意扩展。
/// 实体类型是识别器之间的"契约"——识别器声明它能识别哪些类型，
/// 冲突解决层根据类型进行合并。
#[derive(Debug, Clone, Hash, Eq, PartialEq, Serialize, Deserialize)]
pub enum EntityType {
    // ── 身份类 ──
    /// 人名
    Person,
    /// 身份证号
    IdCard,
    /// 出生日期
    DateOfBirth,

    // ── 联系方式 ──
    /// 电子邮箱
    Email,
    /// 电话号码
    Phone,
    /// 物理地址
    Address,
    /// URL 链接
    Url,

    // ── 金融类 ──
    /// 银行卡号
    BankCard,

    // ── 凭证类 ──
    /// API Key / Token / Secret
    ApiKey,
    /// 密码
    Password,
    /// IP 地址
    IpAddress,

    // ── 扩展 ──
    /// 用户自定义实体类型
    Custom(String),
}

impl EntityType {
    /// 从字符串标签转换（用于 AI 模型输出解析）
    pub fn from_label(label: &str) -> Self {
        match label.to_lowercase().as_str() {
            "person" | "per" | "name" | "person_name" => Self::Person,
            "email" | "email_address" => Self::Email,
            "phone" | "phone_number" | "telephone" => Self::Phone,
            "address" | "location" | "street_address" => Self::Address,
            "id_card" | "idcard" | "national_id" | "ssn" => Self::IdCard,
            "bank_card" | "credit_card" | "debit_card" => Self::BankCard,
            "dob" | "date_of_birth" | "birthday" => Self::DateOfBirth,
            "url" | "website" | "link" => Self::Url,
            "api_key" | "apikey" | "token" | "secret" | "api_token" => Self::ApiKey,
            "password" | "passwd" | "pwd" => Self::Password,
            "ip" | "ip_address" | "ipv4" | "ipv6" => Self::IpAddress,
            other => Self::Custom(other.to_string()),
        }
    }

    /// 获取实体类型的显示标签（中文，用于脱敏标签显示）
    pub fn display_label(&self) -> &str {
        match self {
            Self::Person => "人名",
            Self::Email => "邮箱",
            Self::Phone => "手机",
            Self::Address => "地址",
            Self::IdCard => "身份证",
            Self::BankCard => "银行卡",
            Self::DateOfBirth => "生日",
            Self::Url => "网址",
            Self::ApiKey => "密钥",
            Self::Password => "密码",
            Self::IpAddress => "IP",
            Self::Custom(s) => s.as_str(),
        }
    }

    /// 获取实体类型的英文标签（用于 AI 模型交互）
    pub fn en_label(&self) -> &str {
        match self {
            Self::Person => "person",
            Self::Email => "email",
            Self::Phone => "phone",
            Self::Address => "address",
            Self::IdCard => "id_card",
            Self::BankCard => "bank_card",
            Self::DateOfBirth => "date_of_birth",
            Self::Url => "url",
            Self::ApiKey => "api_key",
            Self::Password => "password",
            Self::IpAddress => "ip_address",
            Self::Custom(s) => s.as_str(),
        }
    }
}

impl std::fmt::Display for EntityType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.display_label())
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// 识别器类型 (RecognizerType)
// ─────────────────────────────────────────────────────────────────────────────

/// 识别器的工作模式
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RecognizerType {
    /// 规则驱动（正则、字典等确定性匹配）
    Rule,
    /// AI 驱动（NER 模型等概率性匹配）
    Ai,
    /// 上下文增强（基于周围词汇调整置信度）
    Context,
    /// 用户自定义
    Custom,
}

// ─────────────────────────────────────────────────────────────────────────────
// 实体跨度 (EntitySpan)
// ─────────────────────────────────────────────────────────────────────────────

/// 识别结果：文本中的一个实体跨度
///
/// `start` 和 `end` 是字节偏移量（不是字符偏移量），
/// 这与 Rust 的 `&[u8]` 切片语义一致，避免 UTF-8 边界问题。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EntitySpan {
    /// 实体起始位置（字节偏移）
    pub start: usize,
    /// 实体结束位置（字节偏移，不含）
    pub end: usize,
    /// 实体类型
    pub entity_type: EntityType,
    /// 置信度 [0.0, 1.0]
    ///
    /// 规则引擎的匹配置信度为 1.0（确定性匹配），
    /// AI 引擎的置信度由模型输出决定。
    pub confidence: f32,
    /// 来源识别器名称
    pub source: String,
    /// 可选的上下文信息（如周围词汇）
    pub context: Option<String>,
    /// 替换掩码（如 `<EMAIL>`、`[人名]`）
    ///
    /// 如果为 `None`，脱敏策略层将使用实体类型的默认标签。
    pub mask: Option<String>,
}

impl EntitySpan {
    /// 创建一个新的实体跨度
    pub fn new(
        start: usize,
        end: usize,
        entity_type: EntityType,
        confidence: f32,
        source: impl Into<String>,
    ) -> Self {
        Self {
            start,
            end,
            entity_type,
            confidence,
            source: source.into(),
            context: None,
            mask: None,
        }
    }

    /// 创建带替换掩码的实体跨度
    pub fn with_mask(
        start: usize,
        end: usize,
        entity_type: EntityType,
        confidence: f32,
        source: impl Into<String>,
        mask: impl Into<String>,
    ) -> Self {
        Self {
            start,
            end,
            entity_type,
            confidence,
            source: source.into(),
            context: None,
            mask: Some(mask.into()),
        }
    }

    /// 获取实体在原文中的文本内容
    pub fn text<'a>(&self, source: &'a [u8]) -> &'a [u8] {
        &source[self.start..self.end]
    }

    /// 获取实体在原文中的文本内容（UTF-8 安全）
    pub fn text_str<'a>(&self, source: &'a str) -> &'a str {
        &source[self.start..self.end]
    }

    /// 是否与另一个跨度重叠
    pub fn overlaps_with(&self, other: &EntitySpan) -> bool {
        self.start < other.end && other.start < self.end
    }

    /// 计算与另一个跨度的重叠长度
    pub fn overlap_len(&self, other: &EntitySpan) -> usize {
        if self.overlaps_with(other) {
            self.end.min(other.end) - self.start.max(other.start)
        } else {
            0
        }
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// 文本编码 (TextEncoding)
// ─────────────────────────────────────────────────────────────────────────────

/// 文本编码类型
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TextEncoding {
    Utf8,
    Gbk,
    Latin1,
    // 未来扩展：更多编码
}

impl Default for TextEncoding {
    fn default() -> Self {
        Self::Utf8
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// 语言检测 (Language)
// ─────────────────────────────────────────────────────────────────────────────

/// 检测到的语言
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Language {
    Chinese,
    English,
    Japanese,
    Korean,
    Mixed,
    Unknown,
}

impl Default for Language {
    fn default() -> Self {
        Self::Unknown
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// 分析上下文 (AnalysisContext)
// ─────────────────────────────────────────────────────────────────────────────

/// 识别器的输入上下文
///
/// 封装了识别器完成工作所需的全部信息。
/// 使用 Builder 模式构造，便于扩展新字段而不破坏现有识别器。
pub struct AnalysisContext<'a> {
    /// 原始文本（字节流）
    pub text: &'a [u8],
    /// 文本编码
    pub encoding: TextEncoding,
    /// 检测到的语言（可选）
    pub language: Option<Language>,
    /// 文件类型（可选，文件处理场景使用）
    pub file_type: Option<String>,
    /// 前置识别器的结果（用于上下文增强）
    pub previous_spans: Vec<EntitySpan>,
    /// 附加元数据
    pub metadata: HashMap<String, String>,
}

impl<'a> AnalysisContext<'a> {
    /// 从 UTF-8 文本创建上下文
    pub fn from_text(text: &'a [u8]) -> Self {
        Self {
            text,
            encoding: TextEncoding::Utf8,
            language: None,
            file_type: None,
            previous_spans: Vec::new(),
            metadata: HashMap::new(),
        }
    }

    /// 获取文本的 UTF-8 表示（如果编码是 UTF-8）
    pub fn as_str(&self) -> Option<&str> {
        if self.encoding == TextEncoding::Utf8 {
            std::str::from_utf8(self.text).ok()
        } else {
            None
        }
    }

    /// 设置语言
    pub fn with_language(mut self, lang: Language) -> Self {
        self.language = Some(lang);
        self
    }

    /// 设置文件类型
    pub fn with_file_type(mut self, ft: impl Into<String>) -> Self {
        self.file_type = Some(ft.into());
        self
    }

    /// 注入前置识别器的结果
    pub fn with_previous_spans(mut self, spans: Vec<EntitySpan>) -> Self {
        self.previous_spans = spans;
        self
    }

    /// 添加元数据
    pub fn with_metadata(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.metadata.insert(key.into(), value.into());
        self
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// 分析结果 (AnalysisResult)
// ─────────────────────────────────────────────────────────────────────────────

/// 识别器的输出结果
#[derive(Debug, Clone)]
pub struct AnalysisResult {
    /// 识别到的实体跨度列表
    pub spans: Vec<EntitySpan>,
    /// 处理耗时（微秒）
    pub elapsed_us: u64,
    /// 识别器名称
    pub recognizer: String,
}

impl AnalysisResult {
    pub fn empty(recognizer: impl Into<String>) -> Self {
        Self {
            spans: Vec::new(),
            elapsed_us: 0,
            recognizer: recognizer.into(),
        }
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// 脱敏结果 (MaskResult)
// ─────────────────────────────────────────────────────────────────────────────

/// 完整的脱敏结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MaskResult {
    /// 原始文本
    pub original: String,
    /// 脱敏后的文本
    pub masked: String,
    /// 识别到的实体列表
    pub entities: Vec<EntitySpan>,
    /// 是否有变更
    pub has_changes: bool,
}

impl MaskResult {
    /// 创建无变更的结果
    pub fn unchanged(text: &str) -> Self {
        Self {
            original: text.to_string(),
            masked: text.to_string(),
            entities: Vec::new(),
            has_changes: false,
        }
    }
}
