# Plan: SafeMask 2.0 — 下一代智能隐私保护引擎

**Vision**: 成为超级国民级隐私保护软件
**Philosophy**: 第一性原理 — 用户只关心"我的数据安全了吗"，不关心技术实现
**Complexity**: Large (分 6 个阶段，渐进式交付)

---

## 一、第一性原理分析

### 用户的本质需求

```
用户不关心：
  ✗ 用了什么模型
  ✗ 正则还是 AI
  ✗ BIOES 是什么
  ✗ ONNX 还是 TensorRT

用户只关心：
  ✓ 我的隐私数据被保护了吗？（完整性）
  ✓ 有没有漏掉的？（召回率）
  ✓ 有没有误伤的？（精确率）
  ✓ 速度快不快？（体验）
  ✓ 操作简单吗？（易用性）
```

### 设计原则

1. **零配置即可用** — 开箱即用，不需要用户理解技术细节
2. **渐进式增强** — 基础功能永远可用，高级功能按需开启
3. **引擎可插拔** — 识别引擎是可替换的组件，不是硬编码的逻辑
4. **数据主权** — 一切计算在本地，用户完全控制
5. **性能即特性** — 快不是优化目标，是核心特性

---

## 二、行业前沿技术分析

### 2.1 识别技术光谱

```
确定性 ◄──────────────────────────────────────────────► 概率性

正则表达式    字典匹配    NER 模型    LLM 推理    多模态
 │            │           │           │           │
 ▼            ▼           ▼           ▼           ▼
手机号规则    人名词典    BERT-NER    GPT-4 PII   OCR+NER
身份证规则    公司名册    DeBERTa     Claude      图片脱敏
邮箱规则      敏感词库    GLiNER      本地 LLM    音频转写
```

### 2.2 业界标杆架构

**Microsoft Presidio** (最成熟的开源 PII 框架):
- 可插拔识别器架构 (Recognizer Registry)
- NLP 引擎 + 正则引擎 + 自定义识别器
- 上下文感知增强 (周围词汇提升置信度)
- 独立的匿名化引擎 (替换/遮盖/哈希/加密)

**GLiNER** (零样本 NER):
- 双编码器架构 (DeBERTa-v3)
- 实体类型作为文本输入，真正的零样本泛化
- 可识别训练时未见过的实体类型
- 多语言支持

**openai/privacy-filter** (token 分类):
- BIOES tagging scheme
- 8 类 PII 检测
- q4 量化，~50MB
- WebGPU/WASM 加速

### 2.3 SafeMask 应该借鉴什么

| 来源 | 借鉴内容 | 应用位置 |
|------|----------|----------|
| Presidio | 可插拔识别器架构 | 识别引擎层 |
| Presidio | 上下文感知增强 | 置信度计算 |
| Presidio | 独立匿名化引擎 | 脱敏策略层 |
| GLiNER | 零样本实体识别 | AI 引擎 |
| privacy-filter | BIOES 后处理 | AI 结果解析 |
| privacy-filter | 量化模型加载 | 模型管理 |

---

## 三、目标架构 — 六层分离

### 3.1 架构全景

```
┌─────────────────────────────────────────────────────────────────┐
│                        Layer 6: 用户界面层                       │
│  Vue 3 + Pinia │ 一键脱敏 │ 实时预览 │ 历史记录 │ 设置面板       │
└────────────────────────────┬────────────────────────────────────┘
                             │ IPC (Tauri Commands)
┌────────────────────────────▼────────────────────────────────────┐
│                        Layer 5: 业务编排层                       │
│  剪贴板监听 │ 文件处理 │ 批量任务 │ 定时任务 │ 场景模式           │
└────────────────────────────┬────────────────────────────────────┘
                             │
┌────────────────────────────▼────────────────────────────────────┐
│                        Layer 4: 脱敏策略层                       │
│  替换策略 │ 遮盖策略 │ 哈希策略 │ 加密策略 │ 自定义模板           │
└────────────────────────────┬────────────────────────────────────┘
                             │
┌────────────────────────────▼────────────────────────────────────┐
│                        Layer 3: 冲突解决层                       │
│  优先级仲裁 │ 重叠合并 │ 置信度过滤 │ 上下文增强 │ 结果验证       │
└────────────────────────────┬────────────────────────────────────┘
                             │
┌────────────────────────────▼────────────────────────────────────┐
│                     Layer 2: 识别引擎层 (可插拔)                  │
│                                                                  │
│  ┌─────────┐ ┌─────────┐ ┌─────────┐ ┌─────────┐ ┌─────────┐  │
│  │ 规则引擎 │ │ AI 引擎  │ │ 字典引擎 │ │ 上下文  │ │ 自定义  │  │
│  │ Regex   │ │ ONNX    │ │ Aho-C   │ │ 引擎    │ │ 识别器  │  │
│  │ Engine  │ │ Engine  │ │ Engine  │ │ Context │ │ Plugin  │  │
│  └────┬────┘ └────┬────┘ └────┬────┘ └────┬────┘ └────┬────┘  │
│       │           │           │           │           │         │
│  ┌────▼───────────▼───────────▼───────────▼───────────▼──────┐  │
│  │              Recognizer Registry (识别器注册表)             │  │
│  │         统一接口 │ 生命周期管理 │ 配置热重载                │  │
│  └───────────────────────────────────────────────────────────┘  │
└────────────────────────────┬────────────────────────────────────┘
                             │
┌────────────────────────────▼────────────────────────────────────┐
│                     Layer 1: 基础设施层                          │
│  文件系统 │ 剪贴板 │ 模型管理 │ 配置管理 │ 日志 │ 性能监控        │
└─────────────────────────────────────────────────────────────────┘
```

### 3.2 各层职责详解

#### Layer 1: 基础设施层 (Infrastructure)

```
infra/
├── fs/                    # 文件系统操作
│   ├── mmap.rs           # 内存映射文件
│   ├── watcher.rs        # 文件监控
│   └── temp.rs           # 临时文件管理
├── clipboard/            # 剪贴板操作
│   ├── platform/         # 平台特定实现
│   │   ├── windows.rs
│   │   ├── macos.rs
│   │   └── linux.rs
│   ├── handler.rs        # 剪贴板读写
│   └── monitor.rs        # 剪贴板监听
├── model/                # AI 模型管理
│   ├── registry.rs       # 模型注册表
│   ├── loader.rs         # 模型加载器
│   ├── cache.rs          # 模型缓存
│   └── quantizer.rs      # 模型量化
├── config/               # 配置管理
│   ├── store.rs          # 配置存储
│   ├── schema.rs         # 配置 Schema
│   └── migration.rs      # 配置迁移
├── logging/              # 日志系统
│   ├── structured.rs     # 结构化日志
│   └── rotation.rs       # 日志轮转
└── metrics/              # 性能监控
    ├── collector.rs      # 指标收集
    └── reporter.rs       # 指标报告
```

**设计原则**: 每个模块都是独立的，不依赖上层逻辑。

#### Layer 2: 识别引擎层 (Detection Engine)

这是架构的**核心创新点** — 可插拔识别器架构。

```rust
// 统一识别器接口
pub trait Recognizer: Send + Sync {
    /// 识别器名称
    fn name(&self) -> &str;

    /// 识别器类型
    fn recognizer_type(&self) -> RecognizerType;

    /// 支持的实体类型
    fn supported_entities(&self) -> Vec<EntityType>;

    /// 识别文本中的实体
    fn analyze(&self, context: &AnalysisContext) -> Vec<EntitySpan>;

    /// 优先级 (越高越优先)
    fn priority(&self) -> i32 { 0 }

    /// 是否启用
    fn is_enabled(&self) -> bool { true }
}

// 识别器类型
pub enum RecognizerType {
    Rule,       // 规则驱动 (正则、字典)
    Ai,         // AI 驱动 (NER 模型)
    Context,    // 上下文增强
    Custom,     // 用户自定义
}

// 实体类型 (可扩展)
#[derive(Debug, Clone, Hash, Eq, PartialEq)]
pub enum EntityType {
    // 内置类型
    Person,
    Email,
    Phone,
    Address,
    IdCard,
    BankCard,
    DateOfBirth,
    Url,
    ApiKey,
    Password,
    IpAddress,
    // 扩展类型
    Custom(String),
}

// 分析上下文
pub struct AnalysisContext {
    pub text: Vec<u8>,              // 原始文本 (字节流)
    pub encoding: TextEncoding,     // 文本编码
    pub language: Option<Language>,  // 检测到的语言
    pub file_type: Option<FileType>, // 文件类型
    pub previous_spans: Vec<EntitySpan>, // 前置引擎的结果
}

// 实体跨度
#[derive(Debug, Clone)]
pub struct EntitySpan {
    pub start: usize,
    pub end: usize,
    pub entity_type: EntityType,
    pub confidence: f32,            // 置信度 [0.0, 1.0]
    pub source: String,             // 来源识别器
    pub context: Option<String>,    // 上下文信息
}
```

**内置识别器**:

| 识别器 | 类型 | 实体类型 | 说明 |
|--------|------|----------|------|
| `RegexRecognizer` | Rule | 所有模式化实体 | 正则表达式匹配 |
| `AhoCorasickRecognizer` | Rule | 固定词 | 字典/敏感词匹配 |
| `NerRecognizer` | Ai | Person, Address, etc. | ONNX NER 模型 |
| `ContextEnhancer` | Context | 所有 | 上下文置信度增强 |
| `ChecksumRecognizer` | Rule | IdCard, BankCard | 校验位验证 |
| `CustomRecognizer` | Custom | 用户定义 | 用户自定义规则 |

**识别器注册表**:

```rust
pub struct RecognizerRegistry {
    recognizers: Vec<Box<dyn Recognizer>>,
    config: RegistryConfig,
}

impl RecognizerRegistry {
    /// 注册识别器
    pub fn register(&mut self, recognizer: Box<dyn Recognizer>) {
        self.recognizers.push(recognizer);
        self.recognizers.sort_by_key(|r| std::cmp::Reverse(r.priority()));
    }

    /// 执行所有识别器
    pub fn analyze(&self, context: &AnalysisContext) -> Vec<EntitySpan> {
        let mut all_spans = Vec::new();

        for recognizer in &self.recognizers {
            if !recognizer.is_enabled() {
                continue;
            }
            let spans = recognizer.analyze(context);
            all_spans.extend(spans);
        }

        all_spans
    }

    /// 热重载配置
    pub fn reload_config(&mut self, config: RegistryConfig) {
        self.config = config;
        // 重新排序、启用/禁用识别器
    }
}
```

#### Layer 3: 冲突解决层 (Resolution)

多个识别器可能识别出重叠的实体，需要智能合并。

```rust
pub struct ConflictResolver {
    strategies: Vec<Box<dyn ResolutionStrategy>>,
}

pub trait ResolutionStrategy: Send + Sync {
    fn resolve(&self, spans: Vec<EntitySpan>) -> Vec<EntitySpan>;
}

// 策略实现
pub struct PriorityStrategy;        // 优先级仲裁
pub struct OverlapMergeStrategy;    // 重叠合并
pub struct ConfidenceFilterStrategy; // 置信度过滤
pub struct ContextBoostStrategy;    // 上下文增强

impl ConflictResolver {
    pub fn resolve(&self, spans: Vec<EntitySpan>) -> Vec<EntitySpan> {
        let mut result = spans;
        for strategy in &self.strategies {
            result = strategy.resolve(result);
        }
        result
    }
}
```

**冲突解决规则**:
1. **优先级仲裁**: 高优先级识别器的结果优先
2. **重叠合并**: 重叠区域取更高置信度的结果
3. **置信度过滤**: 过滤低于阈值的结果
4. **上下文增强**: 周围词汇可提升/降低置信度
5. **校验位验证**: 对有校验位的实体（身份证、银行卡）进行验证

#### Layer 4: 脱敏策略层 (Masking Strategy)

识别结果通过脱敏策略转换为最终输出。

```rust
pub trait MaskingStrategy: Send + Sync {
    fn mask(&self, text: &[u8], span: &EntitySpan, config: &MaskConfig) -> Vec<u8>;
}

// 策略实现
pub struct ReplaceStrategy;      // 替换: 张三 → [PERSON]
pub struct PartialMaskStrategy;  // 部分遮盖: 138****5678
pub struct HashStrategy;         // 哈希: 张三 → a1b2c3
pub struct RedactStrategy;       // 删除: 张三 → ***
pub struct TokenStrategy;        // Token: 张三 → <PERSON_001>
pub struct TemplateStrategy;     // 模板: 自定义替换规则

pub struct MaskingEngine {
    strategies: HashMap<EntityType, Box<dyn MaskingStrategy>>,
    default_strategy: Box<dyn MaskingStrategy>,
    templates: MaskTemplates,
}
```

**脱敏模式**:

| 模式 | 输入 | 输出 | 场景 |
|------|------|------|------|
| 替换 | 张三 | [人名] | 通用脱敏 |
| 部分遮盖 | 13812345678 | 138****5678 | 可读性要求高 |
| 哈希 | 张三 | 8f14e45f | 不可逆脱敏 |
| 删除 | 张三 | *** | 最高安全级 |
| Token | 张三 | <PERSON_001> | 可逆脱敏 |
| 模板 | 张三 | 某某某 | 自定义规则 |

#### Layer 5: 业务编排层 (Orchestration)

将底层能力组合为用户可理解的业务流程。

```rust
// 场景模式
pub enum SceneMode {
    Shadow,     // 影子模式: 复制不脱敏，粘贴时脱敏
    Sentry,     // 哨兵模式: 复制即脱敏
    Batch,      // 批量模式: 文件批量处理
    Realtime,   // 实时模式: 输入即脱敏
}

// 业务流程编排
pub struct Orchestrator {
    engine: Arc<HybridEngine>,
    masking: Arc<MaskingEngine>,
    clipboard: Arc<ClipboardManager>,
    file_processor: Arc<FileProcessor>,
}

impl Orchestrator {
    /// 一键脱敏文本
    pub async fn mask_text(&self, text: &str) -> MaskResult {
        let context = AnalysisContext::from_text(text);
        let spans = self.engine.analyze(&context);
        let resolved = self.conflict_resolver.resolve(spans);
        let masked = self.masking.apply(text, &resolved);
        MaskResult { original: text.to_string(), masked, spans: resolved }
    }

    /// 文件批量脱敏
    pub async fn mask_file(&self, path: &Path) -> MaskResult {
        // mmap → 分块 → 并行识别 → 合并 → 写入
    }

    /// 剪贴板监听
    pub async fn start_clipboard_monitor(&self) {
        // 监听 → 识别 → 脱敏 → 更新影子存储
    }
}
```

#### Layer 6: 用户界面层 (Presentation)

```
src/
├── App.vue                    # 主布局
├── components/
│   ├── dashboard/             # 仪表盘
│   │   ├── Overview.vue       # 概览卡片
│   │   ├── Stats.vue          # 统计图表
│   │   └── QuickActions.vue   # 快捷操作
│   ├── detection/             # 检测相关
│   │   ├── TextInput.vue      # 文本输入
│   │   ├── ResultView.vue     # 结果展示
│   │   ├── EntityHighlight.vue # 实体高亮
│   │   └── ConfidenceBar.vue  # 置信度条
│   ├── rules/                 # 规则管理
│   │   ├── RuleList.vue       # 规则列表
│   │   ├── RuleEditor.vue     # 规则编辑器
│   │   ├── RuleSandbox.vue    # 规则沙盒
│   │   └── RuleImport.vue     # 规则导入导出
│   ├── files/                 # 文件处理
│   │   ├── FileDrop.vue       # 文件拖放
│   │   ├── BatchProcess.vue   # 批量处理
│   │   └── ProgressView.vue   # 进度展示
│   ├── settings/              # 设置
│   │   ├── General.vue        # 通用设置
│   │   ├── Engine.vue         # 引擎配置
│   │   ├── Masking.vue        # 脱敏策略
│   │   ├── Shortcuts.vue      # 快捷键
│   │   └── About.vue          # 关于
│   └── ui/                    # 通用组件
│       ├── Button.vue
│       ├── Card.vue
│       ├── Modal.vue
│       └── Toast.vue
├── stores/
│   ├── useAppStore.ts         # 应用状态
│   ├── useDetectionStore.ts   # 检测状态
│   ├── useRuleStore.ts        # 规则状态
│   └── useSettingsStore.ts    # 设置状态
└── services/
    ├── api.ts                 # Tauri IPC 封装
    └── bridge.ts              # 前后端桥接
```

---

## 四、核心技术实现

### 4.1 AI 引擎实现

```rust
use ort::{GraphOptimizationLevel, Session, Value};
use tokenizers::Tokenizer;

pub struct NerEngine {
    session: Session,
    tokenizer: Tokenizer,
    config: NerConfig,
    entity_labels: Vec<String>,
}

impl NerEngine {
    pub fn new(model_dir: &Path) -> Result<Self> {
        let model_path = model_dir.join("model.onnx");
        let tokenizer_path = model_dir.join("tokenizer.json");

        let session = Session::builder()?
            .with_optimization_level(GraphOptimizationLevel::Level3)?
            .with_intra_threads(num_cpus::get())?
            .commit_from_file(&model_path)?;

        let tokenizer = Tokenizer::from_file(&tokenizer_path)
            .map_err(|e| anyhow::anyhow!("Tokenizer load failed: {}", e))?;

        Ok(Self {
            session,
            tokenizer,
            config: NerConfig::default(),
            entity_labels: Self::load_entity_labels(model_dir)?,
        })
    }

    pub fn analyze(&self, text: &str) -> Result<Vec<EntitySpan>> {
        // 1. Tokenize
        let encoding = self.tokenizer.encode(text, false)?;
        let input_ids = encoding.get_ids();
        let attention_mask = encoding.get_attention_mask();

        // 2. 准备输入张量
        let input_ids_tensor = Value::from_array(vec![1, input_ids.len() as i64],
            input_ids.iter().map(|&x| x as i64).collect::<Vec<_>>())?;
        let attention_mask_tensor = Value::from_array(vec![1, attention_mask.len() as i64],
            attention_mask.iter().map(|&x| x as i64).collect::<Vec<_>>())?;

        // 3. 推理
        let outputs = self.session.run(vec![
            ("input_ids", input_ids_tensor),
            ("attention_mask", attention_mask_tensor),
        ])?;

        // 4. BIOES 后处理
        let logits = outputs[0].try_extract_tensor::<f32>()?;
        let spans = self.decode_bioes(&logits, &encoding, text)?;

        // 5. 置信度过滤
        let filtered = spans.into_iter()
            .filter(|s| s.confidence >= self.config.confidence_threshold)
            .collect();

        Ok(filtered)
    }

    fn decode_bioes(
        &self,
        logits: &ndarray::ArrayView3<f32>,
        encoding: &tokenizers::Encoding,
        original_text: &str,
    ) -> Result<Vec<EntitySpan>> {
        let mut spans = Vec::new();
        let mut current_span: Option<(usize, usize, String, f32)> = None;

        for (i, &token_id) in encoding.get_ids().iter().enumerate() {
            let token_logits = logits.slice(ndarray::s![0, i, ..]);
            let (label, score) = self.get_best_label(&token_logits)?;

            match label.as_str() {
                l if l.starts_with("B-") => {
                    // 保存之前的 span
                    if let Some((start, end, entity, conf)) = current_span.take() {
                        spans.push(EntitySpan {
                            start,
                            end,
                            entity_type: EntityType::from_str(&entity),
                            confidence: conf,
                            source: "ner".to_string(),
                            context: None,
                        });
                    }
                    let offset = encoding.get_offsets(i).unwrap_or((0, 0));
                    current_span = Some((offset.0, offset.1, l[2..].to_string(), score));
                }
                l if l.starts_with("I-") => {
                    if let Some(ref mut span) = current_span {
                        let offset = encoding.get_offsets(i).unwrap_or((0, 0));
                        span.1 = offset.1;
                        span.3 = span.3.min(score);
                    }
                }
                l if l.starts_with("E-") => {
                    if let Some((start, _, entity, conf)) = current_span.take() {
                        let offset = encoding.get_offsets(i).unwrap_or((0, 0));
                        spans.push(EntitySpan {
                            start,
                            end: offset.1,
                            entity_type: EntityType::from_str(&entity),
                            confidence: conf.min(score),
                            source: "ner".to_string(),
                            context: None,
                        });
                    }
                }
                l if l.starts_with("S-") => {
                    let offset = encoding.get_offsets(i).unwrap_or((0, 0));
                    spans.push(EntitySpan {
                        start: offset.0,
                        end: offset.1,
                        entity_type: EntityType::from_str(&l[2..]),
                        confidence: score,
                        source: "ner".to_string(),
                        context: None,
                    });
                }
                _ => {} // O tag
            }
        }

        Ok(spans)
    }
}
```

### 4.2 混合引擎实现

```rust
pub struct HybridEngine {
    registry: RecognizerRegistry,
    conflict_resolver: ConflictResolver,
    masking_engine: MaskingEngine,
}

impl HybridEngine {
    pub fn new(config: EngineConfig) -> Result<Self> {
        let mut registry = RecognizerRegistry::new(config.registry);

        // 注册内置识别器
        registry.register(Box::new(RegexRecognizer::from_rules(&config.rules)?));
        registry.register(Box::new(AhoCorasickRecognizer::from_dict(&config.dictionary)?));
        registry.register(Box::new(ChecksumRecognizer::new()));

        // 注册 AI 引擎 (如果启用)
        if config.ai.enabled {
            let ner_engine = NerEngine::new(&config.ai.model_dir)?;
            registry.register(Box::new(NerRecognizer::new(ner_engine)));
        }

        // 注册上下文增强器
        registry.register(Box::new(ContextEnhancer::new()));

        let conflict_resolver = ConflictResolver::new(config.resolution);
        let masking_engine = MaskingEngine::new(config.masking);

        Ok(Self { registry, conflict_resolver, masking_engine })
    }

    pub fn mask_text(&self, text: &str) -> MaskResult {
        // 1. 分析
        let context = AnalysisContext::from_text(text);
        let spans = self.registry.analyze(&context);

        // 2. 冲突解决
        let resolved = self.conflict_resolver.resolve(spans);

        // 3. 脱敏
        let masked = self.masking_engine.apply(text.as_bytes(), &resolved);
        let masked_text = String::from_utf8_lossy(&masked).to_string();

        MaskResult {
            original: text.to_string(),
            masked: masked_text,
            entities: resolved,
            has_changes: text != masked_text,
        }
    }
}
```

### 4.3 配置热重载

```rust
pub struct ConfigWatcher {
    watcher: RecommendedWatcher,
    config_path: PathBuf,
    engine: Arc<RwLock<HybridEngine>>,
}

impl ConfigWatcher {
    pub fn start(&mut self) -> Result<()> {
        self.watcher.watch(&self.config_path, RecursiveMode::NonRecursive)?;

        loop {
            match self.rx.recv() {
                Ok(event) => {
                    if self.is_config_change(&event) {
                        self.reload_config();
                    }
                }
                Err(e) => log::error!("Config watch error: {}", e),
            }
        }
    }

    fn reload_config(&self) {
        match Config::load(&self.config_path) {
            Ok(config) => {
                let mut engine = self.engine.write();
                if let Err(e) = engine.reload(config) {
                    log::error!("Engine reload failed: {}", e);
                } else {
                    log::info!("Engine config reloaded successfully");
                }
            }
            Err(e) => log::error!("Config load failed: {}", e),
        }
    }
}
```

---

## 五、实现路线图

### Phase 1: 基础重构 — 可插拔架构 (3-4 天)

**目标**: 将现有硬编码的规则引擎重构为可插拔架构

1. 定义 `Recognizer` trait 和相关类型
2. 实现 `RecognizerRegistry`
3. 将现有 `MaskEngine` 重构为 `RegexRecognizer` 和 `AhoCorasickRecognizer`
4. 实现 `ConflictResolver` 基础版本
5. 更新 `AppState` 使用新的 `HybridEngine`

**验证**: 现有功能完全兼容，所有测试通过。

### Phase 2: AI 引擎集成 (3-4 天)

**目标**: 集成 ONNX NER 模型

1. 添加 `ort` 和 `tokenizers` 依赖
2. 实现 `NerEngine` — 模型加载、推理、BIOES 后处理
3. 实现 `NerRecognizer` — 包装为 `Recognizer` 接口
4. 实现模型懒加载和缓存
5. 添加模型下载和更新机制

**验证**: AI 引擎可独立识别 8 类 PII。

### Phase 3: 脱敏策略引擎 (2-3 天)

**目标**: 实现可配置的脱敏策略

1. 定义 `MaskingStrategy` trait
2. 实现 6 种脱敏策略
3. 实现 `MaskingEngine` — 策略路由和配置
4. 添加脱敏模板系统
5. 前端策略选择 UI

**验证**: 用户可选择不同脱敏策略，实时预览效果。

### Phase 4: 上下文增强与高级冲突解决 (2-3 天)

**目标**: 提升识别准确率

1. 实现 `ContextEnhancer` — 周围词汇分析
2. 实现 `ChecksumRecognizer` — 身份证/银行卡校验
3. 增强 `ConflictResolver` — 多策略组合
4. 实现置信度阈值可配置
5. 添加误报反馈机制

**验证**: 误报率降低 30%+，漏报率降低 20%+。

### Phase 5: 业务编排与场景模式 (2-3 天)

**目标**: 打造用户友好的业务流程

1. 实现 `Orchestrator` — 业务流程编排
2. 重构剪贴板监听为 `SceneMode` 驱动
3. 实现批量文件处理管线
4. 添加定时任务支持
5. 实现处理历史和统计

**验证**: 用户可一键切换场景模式，批量处理文件。

### Phase 6: 前端现代化 (3-4 天)

**目标**: 打造现代化、直觉化的用户界面

1. 重新设计仪表盘 — 概览卡片、统计图表
2. 实现实体高亮结果展示
3. 添加置信度可视化
4. 实现规则沙盒增强版
5. 添加引擎状态监控面板
6. 优化移动端适配

**验证**: 用户可在 3 秒内理解当前状态，5 秒内完成一次脱敏操作。

---

## 六、技术依赖

### Rust Crates

```toml
[dependencies]
# 现有依赖保持不变

# AI 引擎
ort = "2.0"                    # ONNX Runtime
tokenizers = "0.19"            # HuggingFace tokenizers
ndarray = "0.15"               # 数组处理
num_cpus = "1.16"              # CPU 核心数检测

# 配置管理
notify = "6.1"                 # 文件监控 (配置热重载)
toml = "0.8"                   # TOML 配置格式

# 性能监控
tracing = "0.1"                # 结构化日志
tracing-subscriber = "0.3"     # 日志订阅者

# 可选: 更多文件格式支持
csv = "1.3"                    # CSV 文件支持
```

### AI 模型

| 模型 | 大小 | 用途 | 来源 |
|------|------|------|------|
| `openai/privacy-filter` | ~50MB (q4) | 通用 PII 检测 | HuggingFace |
| `dslim/bert-base-NER` | ~400MB | 英文 NER | HuggingFace (可选) |
| `uer/roberta-base-finetuned-cluener2020-chinese` | ~400MB | 中文 NER | HuggingFace (可选) |

---

## 七、架构优势

### 对比现有架构

| 维度 | 现有架构 | 目标架构 |
|------|----------|----------|
| 扩展性 | 添加规则需改代码 | 插件化，热重载 |
| 识别能力 | 仅正则+字典 | 正则+字典+AI+上下文 |
| 脱敏策略 | 单一替换 | 6种策略可选 |
| 冲突处理 | 简单优先级 | 多策略智能合并 |
| 用户体验 | 功能导向 | 场景导向 |
| 可测试性 | 难以单元测试 | 每层独立测试 |

### 扩展性保障

1. **新识别器**: 实现 `Recognizer` trait 即可注册
2. **新实体类型**: `EntityType::Custom(String)` 支持任意扩展
3. **新脱敏策略**: 实现 `MaskingStrategy` trait 即可使用
4. **新场景模式**: 在 `Orchestrator` 中添加新模式
5. **新文件格式**: 在 `FileProcessor` 中添加新处理器

---

## 八、验收标准

### Phase 1 验收
- [ ] 现有功能 100% 兼容
- [ ] 所有现有测试通过
- [ ] `RecognizerRegistry` 可动态注册/注销识别器

### Phase 2 验收
- [ ] AI 引擎可识别 8 类 PII (person, email, phone, address, account_number, date, url, secret)
- [ ] 模型懒加载不影响启动速度
- [ ] 推理延迟 < 100ms (单句)

### Phase 3 验收
- [ ] 6 种脱敏策略可用
- [ ] 策略切换实时生效
- [ ] 脱敏模板可自定义

### Phase 4 验收
- [ ] 上下文增强提升置信度准确性
- [ ] 身份证/银行卡校验位验证通过
- [ ] 误报率 < 5%

### Phase 5 验收
- [ ] 批量文件处理支持
- [ ] 场景模式切换流畅
- [ ] 处理历史可查询

### Phase 6 验收
- [ ] 仪表盘加载 < 1s
- [ ] 结果高亮准确
- [ ] 移动端适配完成

---

## 九、下一步

确认方案后，从 **Phase 1** 开始 — 将现有架构重构为可插拔模式。这是后续所有功能的基础。
