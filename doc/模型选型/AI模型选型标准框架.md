# AI 模型选型标准框架

> 版本: 1.0 | 适用: SafeMask v1.2.4+ | 最后更新: 2026-07-13

---

## 目录

1. [背景与目标](#1-背景与目标)
2. [选型标准总览](#2-选型标准总览)
3. [第一层：硬性过滤条件](#3-第一层硬性过滤条件)
4. [第二层：评分维度](#4-第二层评分维度)
5. [第三层：兼容性评估](#5-第三层兼容性评估)
6. [第四层：搜索与筛选流程](#6-第四层搜索与筛选流程)
7. [多模型并行架构方案](#7-多模型并行架构方案)
8. [附录](#8-附录)

---

## 1. 背景与目标

### 1.1 当前模型现状

SafeMask 当前使用 **OpenAI Privacy Filter**（`openai/privacy-filter`）作为 AI NER 引擎：

| 属性 | 值 |
|------|-----|
| 架构 | 自定义 MoE（128 experts, top-4 激活） |
| 参数量 | 1.5B 总参数 / 50M 活跃参数 |
| 精度 | Q4 量化（`model_q4.onnx`） |
| 标签方案 | BIOES（33 类 = O + 8 实体类别 × 4 边界标签） |
| 上下文长度 | 128K tokens |
| 部署位置 | `src-tauri/models/privacy-filter/` |
| 推理引擎 | `ort` crate + `tokenizers` crate（`ner_engine.rs`） |

支持的 8 类 PII：

```
private_person  private_email  private_phone  private_address
private_url     private_date   account_number  secret
```

### 1.2 框架适用范围

本框架适用于在 HuggingFace 上搜索、评估、选择**任何可用于替换或补充当前 AI 模型的 token 分类模型**。框架聚焦于：

- **直接兼容**：不改或少改代码即可集成
- **系统评估**：有量化标准，避免主观选择
- **可复用**：每次评估新模型时使用相同模板

### 1.3 核心约束

以下模型**不在此框架评估范围内**（因架构差异过大需重写推理引擎）：

| 排除类型 | 原因 | 示例 |
|---------|------|------|
| GLiNER / span-based 模型 | 非 token 分类架构，对齐方式不同 | `nvidia/gliner-pii` |
| LLM-based 检测 | 需自回归推理，延迟太高 | GPT-4o, Qwen 等 |
| 纯正则/规则系统 | 非 ML 模型 | Presidio（但其 ML 组件可评估） |
| spaCy / Stanza 模型 | 非 ONNX 格式，需 Python 运行时 | `en_core_web_lg` |

---

## 2. 选型标准总览

### 2.1 评估体系总图

```
候选模型
    │
    ▼
┌─────────────────────────────────────┐
│  第一层：硬性过滤（6 项，一票否决）       │
│  pipeline_tag / ONNX / 接口 /     │
│  tokenizer / 许可证 / 动态轴        │
└─────────────────────────────────────┘
    │ 通过
    ▼
┌─────────────────────────────────────┐
│  第二层：评分维度（7 项，加权总分）       │
│  标签方案 25% + PII覆盖 20% +        │
│  性能F1 20% + 体积延迟 15% +         │
│  上下文 10% + 多语言 5% + 维护 5%    │
└─────────────────────────────────────┘
    │
    ▼
┌─────────────────────────────────────┐
│  第三层：兼容性评估（代码改动级）         │
│  按改动量分 A/B/C/D/E 五级           │
│  精确到 ner_engine.rs 行号           │
└─────────────────────────────────────┘
    │
    ▼
┌─────────────────────────────────────┐
│  第四层：决策输出                      │
│  ├─ 单一推荐：最高分 + 最低改动级       │
│  ├─ 并行集：主+辅 / 通用+专用 / 场景切换 │
│  └─ 不推荐                           │
└─────────────────────────────────────┘
```

### 2.2 改动级别定义

| 级别 | 描述 | 代码改动范围 | 预估工时 |
|------|------|------------|---------|
| **A** | 零改动，复制模型文件即可 | 无 | 5 分钟 |
| **B** | 标签映射 + 默认标签列表更新 | `ner_engine.rs:default_labels()`, `map_entity_type()` | 30 分钟 |
| **C** | B + 输入适配 | 同上 + `infer()` 增加 `token_type_ids` | 1 小时 |
| **D** | C + 解码器修改 | 同上 + `decode_bioes()` 适配 BIO/BILOU | 2-4 小时 |
| **E** | 架构不兼容，需重写推理引擎 | 大规模重构 | 数天 |

### 2.3 评分汇总表（7 维度 × 5 分制）

| 维度 | 权重 | 1分 | 2分 | 3分 | 4分 | 5分 |
|------|------|-----|-----|-----|-----|-----|
| 标签方案匹配度 | 25% | 自定义方案 | IO 方案 | BIO/BILOU | BIOES 不同类 | BIOES 同类 |
| PII 覆盖度 | 20% | 非 PII | 3-4 类 | 5-7 类 | 8 类 | 8 类 + 更多 |
| 性能指标 F1 | 20% | < 0.88 | ≥ 0.88 | ≥ 0.92 | ≥ 0.95 | ≥ 0.97 |
| 延迟与体积 | 15% | > 3B 参数 | 1-3B | 500M-1B | < 500M 有量化 | < 200M 有量化 |
| 上下文长度 | 10% | < 256 | 256-384 | 512 | 2048-4096 | ≥ 8192 |
| 多语言支持 | 5% | 单非英文 | 仅英文 | 英文+少量 | 3-5 语言 | 10+ 语言 |
| 维护状态 | 5% | 无数据 | 2022 前 | 2023 | 2024 | 2025-2026 |

**总分 = ∑(维度分 × 权重)**

| 总分范围 | 结论 |
|---------|------|
| ≥ 4.0 | 强推荐，可直接替换 |
| 3.0 - 3.9 | 推荐，可作为并行模型 |
| 2.0 - 2.9 | 可考虑，需较大改动或仅用于特定场景 |
| < 2.0 | 不推荐 |

---

## 3. 第一层：硬性过滤条件

### 3.1 Pipeline 任务

```
必须满足: pipeline_tag = "token-classification"
```

**为什么**：PII 检测以 NER 方式实现，即对每个 token 预测 BIO/BIOES 标签。这是标准化的 token 分类任务，HuggingFace 使用 `pipeline_tag` 作为规范标签。

**检查方法**：

| 方式 | 操作 |
|------|------|
| HuggingFace 页面 | 模型卡片顶部显示 "Token Classification" 标签 |
| API 字段 | `GET /api/models/{org}/{model}` 返回的 `pipeline_tag` |
| model card YAML | 文件头部 `pipeline_tag: token-classification` |

**反例**（会被排除）：
- `text-generation`（LLM）
- `zero-shot-classification`（GLiNER 类型）
- `feature-extraction`（embedding 模型）
- `fill-mask`（MLM 模型）

### 3.2 ONNX 可用性

```
必须满足: 已提供 ONNX 文件 或 可通过 optimum 导出
```

**为什么**：SafeMask 使用 `ort` crate（ONNX Runtime）进行推理，不支持 PyTorch 直接加载。

**两种通过方式**：

**方式 A：已提供 ONNX 文件**
模型仓库根目录或 `onnx/` 子目录下包含 `.onnx` 文件。

检查方法：
```
https://huggingface.co/{org}/{model}/tree/main
# 查看是否有 model.onnx / model_q4.onnx / onnx/model.onnx
```

**方式 B：可通过 optimum 导出**

模型使用 HuggingFace `transformers` 库，且为标准 encoder 架构（BERT / RoBERTa / XLM-RoBERTa / DeBERTa）。

验证命令（在 Python 环境中）：
```bash
pip install optimum onnx
optimum-cli export onnx --task token-classification -m {org}/{model} ./onnx_output/
```

会失败的模型特征：
- 使用非标准 `modeling_*.py` 自定义实现
- 依赖于 HuggingFace 不支持的特殊算子
- MoE 架构（如当前 Privacy Filter，虽然它有 ONNX 版本）

### 3.3 输入/输出接口规范

```
inputs:  input_ids (i64, [batch, seq_len])        → 必须存在
         attention_mask (i64, [batch, seq_len])    → 必须存在
         token_type_ids (i64, [batch, seq_len])    → 可选（见第 5 章处理方案）

output:  logits (f32, [batch, seq_len, num_labels]) → 必须存在，名称为 "logits"
```

**为什么**：`ner_engine.rs:220-237` 中硬编码了这些张量名称和形状：

```rust
// ner_engine.rs:220-224 — 输入张量
let input_ids_tensor = Tensor::from_array(([1, seq_len], input_ids))?;
let attention_mask_tensor = Tensor::from_array(([1, seq_len], attention_mask))?;

// ner_engine.rs:226-229 — 推理调用，硬编码输入名
let outputs = self.session.run(ort::inputs![
    "input_ids" => input_ids_tensor,
    "attention_mask" => attention_mask_tensor,
])?;

// ner_engine.rs:231-232 — 硬编码输出名
let logits_value = outputs.get("logits")
    .context("模型输出中未找到 logits")?;
```

**检查方法**：使用 Python ONNX Runtime 检查模型接口：

```python
import onnxruntime as ort
session = ort.InferenceSession("model.onnx")
for inp in session.get_inputs():
    print(f"Input: {inp.name}, shape: {inp.shape}, type: {inp.type}")
for out in session.get_outputs():
    print(f"Output: {out.name}, shape: {out.shape}, type: {out.type}")
```

**通过条件**：
- 输入名必须包含 `"input_ids"` 和 `"attention_mask"`
- 输出名必须包含 `"logits"`
- 允许存在额外输入（如 `token_type_ids`），代码中有处处理方案

### 3.4 Tokenizer 格式

```
必须满足: tokenizer.json 文件存在于模型仓库中
```

**为什么**：`ner_engine.rs:120` 使用 `tokenizers::Tokenizer::from_file(&path)` 加载 `tokenizer.json`。

**检查方法**：

```
https://huggingface.co/{org}/{model}/tree/main
# 列表中应有 tokenizer.json 文件
```

**不通过的格式**（需要额外转换）：
- 仅提供 `tokenizer.model`（tiktoken Open AI 格式）
- 仅提供 `sentencepiece.bpe.model`
- 仅提供 `tokenizer_config.json`（需要配合 Python 生成 fast tokenizer）

**注意**：XLM-RoBERTa 系列的官方仓库通常同时提供 `sentencepiece.bpe.model` 和 `tokenizer.json`。取 `tokenizer.json` 即可。

### 3.5 许可证

```
必须满足: license IN ("apache-2.0", "mit")
```

仅 Apache 2.0 和 MIT 允许不受限的商业使用、修改和再分发。

**检查方法**：

| 位置 | 操作 |
|------|------|
| HuggingFace 页面右侧 | "License" 字段 |
| 模型卡片 YAML 头部 | `license: apache-2.0` |
| API 字段 | `GET .../api/models/{org}/{model}` → `cardData.license` |

**许可证兼容性速查**：

| 许可证 | 商业使用 | 修改 | 再分发 | 是否通过 |
|--------|---------|------|--------|---------|
| Apache 2.0 | ✅ | ✅ | ✅ | ✅ |
| MIT | ✅ | ✅ | ✅ | ✅ |
| BSD 3-Clause | ✅ | ✅ | ✅ | ⚠️ 需逐项确认条款 |
| CC-BY-NC | ❌ | ✅ | ❌ | ❌ |
| GPL 3.0 | ✅ | ✅ | ⚠️ 需开源衍生 | ⚠️ 不适合闭源分发 |
| AGPL 3.0 | ✅ | ✅ | ❌ 网络使用也受限 | ❌ |
| 无声明 | ⚠️ | ⚠️ | ⚠️ | ❌ 法律风险 |

### 3.6 动态轴支持

```
必须满足: ONNX 模型的 batch 和 sequence 维度为动态（-1）
```

**为什么**：推理时每次的序列长度不同，静态轴模型只能处理固定长度输入。

**检查方法**：

```python
import onnxruntime as ort
session = ort.InferenceSession("model.onnx")
for inp in session.get_inputs():
    shape = inp.shape  # 应包含 -1 或 "batch"/"sequence" 符号名
    print(f"{inp.name}: {shape}")
```

**通过条件**：输入张量的第 0 维（batch）和第 1 维（sequence）为 `-1` 或 `"batch"` / `"sequence"`。

**注意**：`optimum` 的默认导出行为就是动态轴。绝大多数 HuggingFace ONNX 模型已满足此条件。

---

## 4. 第二层：评分维度

### 4.1 标签方案匹配度（权重 25%）

**为什么权重最高**：标签方案直接决定了 `ner_engine.rs` 中 `decode_bioes()` 解码器的改动量，是整个集成中最核心的兼容性因素。

**评分标准**：

| 分数 | 条件 | 示例 |
|------|------|------|
| 5 | BIOES + 实体类别与当前 8 类完全一致 | `openai/privacy-filter`, `yasserrmd/privacy-filter-ONNX` |
| 4 | BIOES 但实体类别不同（需更新标签映射） | `gheim-ch-560m`（BIOES + 同 8 类） |
| 3 | BIO（IOB2）方案 | `dslim/bert-base-NER`, `multilang-pii-ner` |
| 2 | BILOU（等价 BIOES 但标签名不同） | spaCy 原生格式 |
| 1 | 自定义/IO 方案或其他非标准标签 | 非常见模型 |

**如何识别标签方案**：

```python
import json
with open("config.json") as f:
    config = json.load(f)

labels = list(config["id2label"].values())
# 检查标签前缀
bioes = any(l.startswith("E-") or l.startswith("S-") for l in labels if l != "O")
bilou = any(l.startswith("L-") or l.startswith("U-") for l in labels if l != "O")
bio = any(l.startswith("B-") for l in labels if l != "O") and not bioes and not bilou
```

**BIO → BIOES 适配说明**（改动级 D）：

BIO 方案只有 `B-`（开始）和 `I-`（内部），缺少 `E-`（结束）和 `S-`（单体）。当前 `decode_bioes()` 依赖 `E-` 确定实体结束边界。适配方案：

```rust
// 在 decode_bioes() 中，对 BIO 模型做后处理：
// 1. B-X → 视为 B-X（新实体开始）
// 2. I-X → 视为 I-X（接续）
// 3. B-X 之后遇到 O 或 B-Y → 推断前一个实体结束
// 4. I-X 之后遇到 O 或 B-Y → 推断前一个实体结束
// 不需要生成 E/S 标签，当前解码器在 O/B 处自动截断
```

### 4.2 PII 覆盖度（权重 20%）

**评分标准**：

| 分数 | 条件 |
|------|------|
| 5 | 完全覆盖 8 类 PII + 额外有用类型（如 CREDIT_CARD, SSN, AGE, CITY） |
| 4 | 完全覆盖 8 类 PII（当前需求 100% 满足） |
| 3 | 覆盖 5-7 类（有 1-3 类缺失，需规则引擎补充） |
| 2 | 覆盖 3-4 类（仅覆盖核心类型：person/email/phone） |
| 1 | 非 PII 专用（通用 NER：PER/ORG/LOC/MISC） |

**SafeMask 的 8 类 PII 与通用 NER 类型映射**：

| SafeMask 类型 | 通用 NER 等效类型 | 常见变体名 |
|--------------|------------------|-----------|
| `Person` | PER | person, name, given_name, surname |
| `Email` | — | email, email_address |
| `Phone` | — | phone, phone_number, telephone |
| `Address` | LOC (部分) | address, street, location |
| `Url` | — | url, website, web |
| `DateOfBirth` | DATE (部分) | date, dob, birth_date |
| `BankCard` | — | credit_card, account, iban |
| `ApiKey` | — | secret, key, password, token |

**如何评估**：阅读模型卡片 `README.md` 中的实体列表章节，或检查 `config.json` 的 `id2label`。

### 4.3 性能指标 F1（权重 20%）

**评分标准**：

| 分数 | 条件 |
|------|------|
| 5 | F1 ≥ 0.97 |
| 4 | F1 ≥ 0.95 |
| 3 | F1 ≥ 0.92 |
| 2 | F1 ≥ 0.88 |
| 1 | F1 < 0.88 或数据缺失 |

**如何查找**：

检查模型卡片上的以下位置（按优先级）：

1. **`model-index`**（YAML 头部结构化指标）：
   ```yaml
   model-index:
     - results:
         - metrics:
             - type: f1
               value: 0.97
   ```

2. **Evaluation Results 章节**（README.md 中的表格）

3. **论文/技术报告**：模型卡片引用的论文

**注意**：优先关注 **PII 实体级别的 F1**，而非整体微平均。常见 PII 模型对 `person` 和 `email` 的 F1 通常很高（0.98+），但对 `secret`、`account_number` 可能较低。

### 4.4 延迟与体积（权重 15%）

**评分标准**：

| 分数 | 条件 | 说明 |
|------|------|------|
| 5 | < 200M 参数 + 有 INT8/Q4 量化版 | 极轻量，CPU 实时推理 |
| 4 | < 500M 参数 + 有量化版 | 轻量，CPU 流畅运行 |
| 3 | 500M - 1B 参数 或 仅 FP16 | 可接受，CPU 稍慢 |
| 2 | 1B - 3B 参数（当前 MoE 水平） | 当前水平，活跃参数量小 |
| 1 | > 3B 参数 | 太重量级，不适合边缘部署 |

**重要**：对 MoE 架构的模型，关注**活跃参数量**而非总参数量。例如 OpenAI Privacy Filter（1.5B 总参数 / 50M 活跃参数）的实际推理量相当于一个很小的 dense 模型。

**ONNX 文件大小作为参考**：

| 量化级别 | 每个参数 | 100M 参数模型 | 500M 参数模型 |
|---------|---------|-------------|-------------|
| FP32 | 4 bytes | 400 MB | 2 GB |
| FP16 | 2 bytes | 200 MB | 1 GB |
| INT8/Q8 | 1 byte | 100 MB | 500 MB |
| INT4/Q4 | 0.5 bytes | 50 MB | 250 MB |

### 4.5 上下文长度（权重 10%）

**评分标准**：

| 分数 | 条件 |
|------|------|
| 5 | ≥ 8192 tokens |
| 4 | 2048 - 4096 tokens |
| 3 | 512 tokens（当前 `ner_engine.rs:max_length = 512`） |
| 2 | 256 - 384 tokens |
| 1 | < 256 tokens |

**检查方法**：`config.json` 中的 `max_position_embeddings` 字段。

**SafeMask 当前限制**：
```rust
// ner_engine.rs:133
max_length: 512,  // 硬编码为 512
```

这意味着即使模型支持更长上下文，当前代码也会截断到 512。如果需要利用更长上下文，需：
```rust
// 改为从模型配置读取
max_length: model_config.max_position_embeddings.min(2048), // 或用户可配置
```

### 4.6 多语言支持（权重 5%）

**评分标准**：

| 分数 | 条件 |
|------|------|
| 5 | 10+ 语言支持 |
| 4 | 3-5 语言（含中日韩或主要欧洲语言） |
| 3 | 英文 + 少量其他语言 |
| 2 | 仅英文 |
| 1 | 仅单一非英文语言 |

**检查方法**：

模型卡片 YAML 头部：
```yaml
language:
  - "en"
  - "de"
  - "fr"
  - "it"
  - "zh"
```

或查看训练数据描述中的语言覆盖范围。

### 4.7 维护状态（权重 5%）

**评分标准**：

| 分数 | 条件 |
|------|------|
| 5 | 2025-2026 年更新，月下载量 > 1000 |
| 4 | 2024 年更新，稳定下载量 |
| 3 | 2023 年更新 |
| 2 | 2022 年或更早 |
| 1 | 无更新、无下载数据 |

**检查方法**：

```
# API 查看最后更新时间和下载量
GET /api/models/{org}/{model}
# → lastModified, downloads 字段

# HuggingFace 页面
# 页面右侧显示 "Last Updated" 和 "Downloads last month"
```

---

## 5. 第三层：兼容性评估

### 5.1 代码改动映射表

以下按 `ner_engine.rs` 的行号列出每个代码位置对模型兼容性的影响：

| 行号 | 代码片段 | 依赖的外部因素 | 改模型时需调整？ |
|------|---------|--------------|----------------|
| 85-89 | `model_path` 检测逻辑 | ONNX 文件名 (`model_q4.onnx` / `model.onnx`) | 否（通用） |
| 120-123 | `Tokenizer::from_file(&tokenizer_path)` | `tokenizer.json` 格式 | 否（通用） |
| 125 | `Self::load_labels(model_dir)` | `config.json` 的 `id2label` 结构 | **是** |
| 133 | `max_length: 512` | 模型的 `max_position_embeddings` | **建议调整** |
| 167-202 | `default_labels()` | 模型的标签集 | **是**（如果 `config.json` 缺失） |
| 215-218 | `input_ids` / `attention_mask` 构建 | 模型的输入张量名 | **是**（如果模型需要 `token_type_ids`） |
| 226-229 | `session.run(ort::inputs!["input_ids", "attention_mask"])` | 模型的输入张量名 | **是**（如果输入名不同） |
| 231-232 | `outputs.get("logits")` | 模型的输出张量名 | **是**（如果输出名不同） |
| 252-337 | `decode_bioes()` | BIOES / BIO / BILOU 标签方案 | **是**（如果不是 BIOES） |
| 400-412 | `map_entity_type()` | 模型的实体标签名 | **是** |
| 65 | `BioesLabel` 枚举定义 | BIOES 标签方案 | **是**（如果不是 BIOES） |

### 5.2 输入差异处理决策树

```
模型需要哪些输入张量？
│
├── 仅 input_ids + attention_mask（推荐）
│   └→ 零改动，直接使用
│
├── input_ids + attention_mask + token_type_ids
│   └→ 在 infer() 中添加零张量：
│      let token_type_ids = vec![0i64; seq_len];
│      let tti_tensor = Tensor::from_array(([1, seq_len], token_type_ids))?;
│      session.run(ort::inputs![
│          "input_ids" => ...,
│          "attention_mask" => ...,
│          "token_type_ids" => tti_tensor,
│      ])?;
│
├── 仅有 input_ids（无 attention_mask）
│   └→ 需要生成 dummy attention_mask = vec![1; seq_len]
│      （大多数模型有 attention_mask，极少见情况）
│
└── 输入名不是标准名（如 "inputs" 而非 "input_ids"）
    └→ 需要修改 infer() 中的字符串常量
```

**常见架构的输入需求速查**：

| 架构 | input_ids | attention_mask | token_type_ids |
|------|-----------|---------------|----------------|
| BERT | ✅ | ✅ | ✅（可填 0）|
| RoBERTa | ✅ | ✅ | ❌ |
| XLM-RoBERTa | ✅ | ✅ | ❌ |
| DistilBERT | ✅ | ✅ | ❌ |
| DeBERTa-v3 | ✅ | ✅ | ❌ |
| ModernBERT | ✅ | ✅ | ❌ |
| GPT-2 | ✅ | ✅ | ❌ |

### 5.3 标签映射迁移指南

当新模型使用不同的实体标签名时，需要在 `map_entity_type()` 中添加映射：

```rust
// ner_engine.rs:399-412 — 扩展示例
fn map_entity_type(name: &str) -> EntityType {
    match name.to_lowercase().as_str() {
        // === 当前映射（OpenAI Privacy Filter）===
        "person" | "person_name" | "private_person"  => EntityType::Person,
        "email" | "email_address" | "private_email"   => EntityType::Email,
        "phone" | "phone_number" | "private_phone"    => EntityType::Phone,
        "address" | "street_address" | "private_address" => EntityType::Address,
        "account_number"                               => EntityType::BankCard,
        "date" | "private_date" | "dob"               => EntityType::DateOfBirth,
        "url" | "website" | "private_url"             => EntityType::Url,
        "secret" | "api_key" | "token"                => EntityType::ApiKey,

        // === 新模型特有标签 ===
        // multilang-pii-ner / bert-small-pii-detection 等
        "city"            => EntityType::Custom("city".into()),
        "street"          => EntityType::Address,          // 归入 Address
        "zipcode"         => EntityType::Custom("zipcode".into()),
        "buildingnum"     => EntityType::Custom("building_num".into()),
        "credit_card"     => EntityType::BankCard,         // 归入 BankCard
        "ssn"             => EntityType::Custom("ssn".into()),
        "passport"        => EntityType::Custom("passport".into()),
        "driver_license"  => EntityType::Custom("driver_license".into()),
        "age"             => EntityType::Custom("age".into()),
        "gender"          => EntityType::Custom("gender".into()),
        "company"         => EntityType::Custom("company".into()),
        "organization"    => EntityType::Custom("org".into()),
        "location"        => EntityType::Address,          // 归入 Address

        // 未映射的标签：作为 Custom 保留原始名
        other => EntityType::Custom(other.to_string()),
    }
}
```

### 5.4 已知候选模型兼容性矩阵

| # | 模型 | 架构 | 标签方案 | 输入接口 | 输出接口 | 改动级 | F1 | 体积 | 备注 |
|---|------|------|---------|---------|---------|--------|-----|------|------|
| 1 | `openai/privacy-filter` | Custom MoE | BIOES 33类 | ids+mask | logits | A | — | 50M active | 当前在用 |
| 2 | `yasserrmd/privacy-filter-ONNX` | 同上（FP16） | BIOES 33类 | ids+mask | logits | A | same | ~2.6GB | 精度更高 |
| 3 | `onnx-community/multilang-pii-ner-ONNX` | XLM-RoBERTa | BIO 多类 | ids+mask | logits | D | 0.954 | ~500M | 30+ PII 类型 |
| 4 | `rtrigoso/bert-small-pii-detection-ONNX` | BERT-Small | BIO 24类 | ids+mask+tti | logits | C | - | ~70M | 23 PII 类 |
| 5 | `protectai/bert-base-NER-onnx` | BERT-Base | BIO 4类 | ids+mask+tti | logits | C | - | ~110M | 通用 NER |
| 6 | `dslim/bert-base-NER` | BERT-Base | BIO 4类 | ids+mask+tti | logits | C | - | ~110M | 需导出 ONNX |
| 7 | `gheim-ch-560m` | XLM-RoBERTa | BIOES 33类 | ids+mask | logits | B | - | 560M | 多语言 |
| 8 | `Byrne-Anon` | Custom | BIOES 33类 | ids+mask | logits | B | - | 85M | 轻量 |
| 9 | `bigcode/starpii` | BigCode Encoder | BIO 6类 | ids+mask+tti | logits | C | - | ~350M | 代码专用，gated |
| 10 | `gravitee-io/bert-small-pii-detection` | BERT-Small | BIO 24类 | ids+mask+tti | logits | C | 详见卡片 | ~70M | 需导出 ONNX |

---

## 6. 第四层：搜索与筛选流程

### 6.1 标准搜索流程

```
Step 1: HuggingFace API 多路搜索
│
├── 主搜索：pii + token-classification（按下载量排序）
├── 补充：privacy + token-classification
├── 补充：ner + token-classification + onnx
├── 补充：deidentification + token-classification
│
Step 2: 第一层过滤（6 项硬性条件）
│   遍历搜索结果，排除不满足的模型
│
Step 3: 下载候选模型，本地验证
│
├── 下载模型文件（snapshot_download）
├── 检查 tokenizer.json 可用性
├── Python 检查 ONNX 输入/输出张量
├── 检查 config.json 标签方案
│
Step 4: 第二层评分
│
├── 逐项打分（1-5）
├── 计算加权总分
├── 记录到评分工作表中
│
Step 5: 第三层兼容性评估
│
├── 确定改动级（A/B/C/D/E）
├── 估计集成工时
├── 标注需要注意的代码位置
│
Step 6: 决策输出
│
├── 高分（≥4.0）+ A/B 级 → 直接推荐替换主模型
├── 高分 + C/D 级 → 推荐作为辅助模型
├── 中分（3.0-3.9） → 推荐作为并行模型
└── 低分 → 排除
```

### 6.2 HuggingFace API 命令汇总

**REST API**：

```bash
# 主搜索：PII 检测模型（按下载量）
curl "https://huggingface.co/api/models?search=pii&filter=token-classification&sort=downloads&direction=-1&limit=50"

# 搜索 ONNX + token-classification（已有 ONNX 的模型）
curl "https://huggingface.co/api/models?search=pii&filter=token-classification&filter=onnx&sort=downloads&direction=-1&limit=50"

# 搜索 privacy/匿名化模型
curl "https://huggingface.co/api/models?search=privacy+detection&filter=token-classification&sort=downloads&direction=-1&limit=50"

# 搜索多语言 PII NER
curl "https://huggingface.co/api/models?search=multilingual+pii+ner&filter=token-classification&sort=downloads&direction=-1&limit=50"

# 搜索 deidentification 模型
curl "https://huggingface.co/api/models?search=deidentification&filter=token-classification&sort=downloads"

# 按作者搜索（特定组织）
curl "https://huggingface.co/api/models?author=openai&pipeline_tag=token-classification&sort=downloads"
curl "https://huggingface.co/api/models?author=nvidia&pipeline_tag=token-classification&sort=downloads"
curl "https://huggingface.co/api/models?author=bigcode&pipeline_tag=token-classification&sort=downloads"
```

**Python SDK**：

```python
from huggingface_hub import HfApi
api = HfApi()

# 搜索 PII + token-classification
models = api.list_models(
    search="pii",
    filter="token-classification",
    sort="downloads",
    direction=-1,
    limit=50,
    full=True,  # 返回完整元数据
)

# 搜索 ONNX 版本
onnx_models = api.list_models(
    search="pii",
    filter=["token-classification", "onnx"],
    sort="downloads",
    direction=-1,
)

# 筛选出有 tokenizer.json 的模型
for model in models:
    # 检查模型文件列表是否包含 tokenizer.json
    files = api.list_repo_files(model.modelId)
    if "tokenizer.json" in files:
        print(f"✅ {model.modelId} — {model.downloads} downloads")
```

**HuggingFace CLI**：

```bash
# 搜索模型
huggingface-cli search pii --pipeline-tag token-classification --sort downloads

# 查看模型详情
huggingface-cli model-info {org}/{model}

# 查看模型文件列表
huggingface-cli file-list {org}/{model}
```

### 6.3 模型评分模板

每次评估新模型时，使用以下模板记录：

```markdown
## 模型评估记录

### 基本信息

| 字段 | 值 |
|------|-----|
| 模型名称 | `{org}/{model}` |
| 评估日期 | YYYY-MM-DD |
| 架构 | {arch} |
| 许可证 | {license} |
| 月下载量 | {downloads} |
| 最后更新 | {last_updated} |

### 第一层过滤结果

| 标准 | 通过？ | 备注 |
|------|--------|------|
| pipeline_tag = token-classification | ✅ / ❌ | |
| ONNX 可用 | ✅ / ❌ | 已提供 / 可导出 / 不可用 |
| 接口规范 | ✅ / ❌ | inputs: {names}, outputs: {names} |
| tokenizer.json 存在 | ✅ / ❌ | |
| 许可证兼容 | ✅ / ❌ | |
| 动态轴支持 | ✅ / ❌ | |

### 第二层评分

| 维度 | 分数 (1-5) | 权重 | 加权分 | 依据 |
|------|-----------|------|--------|------|
| 标签方案匹配度 | | 25% | | |
| PII 覆盖度 | | 20% | | |
| 性能指标 F1 | | 20% | | |
| 延迟与体积 | | 15% | | |
| 上下文长度 | | 10% | | |
| 多语言支持 | | 5% | | |
| 维护状态 | | 5% | | |
| **总分** | | **100%** | **{score}** | |

### 第三层兼容性

| 检查项 | 状态 | 备注 |
|--------|------|------|
| 输入名匹配 | ✅ / ⚠️ / ❌ | |
| 需要 token_type_ids | ✅ / ❌ | |
| 输出名 "logits" | ✅ / ❌ | |
| 标签方案 | BIOES / BIO / BILOU / 其他 | |
| 标签数匹配 | {num_labels} | |
| 实体类型列表 | {entity_types} | |

**改动级**: {A/B/C/D/E}
**预估工时**: {hours}

### 决策

- [ ] 强推荐（替换主模型）
- [ ] 推荐（作为并行模型）
- [ ] 可考虑（特定场景）
- [ ] 不推荐

### 备注

{additional_notes}
```

### 6.4 决策流程图

```
         ┌──────────────┐
         │  候选模型通过   │
         │  第一层过滤？   │
         └──────┬───────┘
                │
        ┌───────┴───────┐
        │ 是             │ 否
        ▼               ▼
   ┌──────────┐    ┌──────────┐
   │ 第二层评分 │    │   排除    │
   └────┬─────┘    └──────────┘
        │
   ┌────┴─────┐
   │ 总分 ≥ 4.0 │
   └────┬─────┘
        │
   ┌────┴───────────┐
   │ 是              │ 否
   ▼                 ▼
┌──────────┐   ┌──────────────┐
│ 第三层    │   │ 总分 ≥ 3.0？ │
│ 兼容性评估 │   └──────┬───────┘
└────┬─────┘          │
     │          ┌─────┴─────┐
     ▼          │ 是         │ 否
┌────────┐     ▼           ▼
│ 改动级  │  ┌────────┐  ┌────────┐
│ A/B/C/D │  │ 第三层  │  │  排除   │
└───┬─────┘  │ 兼容性  │  └────────┘
    │       └───┬─────┘
    ▼           │
┌───────┐      ▼
│ A/B?  │  ┌───────────┐
└───┬───┘  │ 改动级 C/D?│
    │      └─────┬─────┘
┌───┴───┐       │
│ 是     │ 否    │
▼       ▼       │
┌────┐ ┌────┐   │
│替换 │ │并行│   │
│主模 │ │模型│   │
│型   │ │    │   │
└────┘ └────┘   │
          ┌─────┴─────┐
          │ 是         │ 否
          ▼           ▼
       ┌────────┐  ┌────────┐
       │ 辅助模型 │  │ 暂不采纳 │
       └────────┘  └────────┘
```

---

## 7. 多模型并行架构方案

### 7.1 三种并行策略

#### 策略一：主模型 + 辅助模型（推荐）

一个主模型覆盖核心 8 类 PII，一个或多个辅助模型覆盖额外类型。

```
                            ┌──────────────────┐
                            │    RegistryAnalyzer  │
                            │  (结果合并 + 去重)   │
                            └────────┬─────────┘
                                     │
                    ┌────────────────┼────────────────┐
                    ▼                ▼                 ▼
            ┌──────────────┐ ┌──────────────┐ ┌──────────────┐
            │   主模型      │ │   辅模型 A    │ │   辅模型 B    │
            │ Privacy Filter│ │multilang-pii │ │bert-small-pii│
            │ 8 类 PII     │ │ 30+ 类型     │ │ 23 类 PII    │
            │ 50M active   │ │ ~500M       │ │ ~70M         │
            └──────────────┘ └──────────────┘ └──────────────┘
```

**合并规则**：
1. 所有模型的 EntitySpan 结果合并到同一列表
2. 按优先级（主模型 > 辅模型）排序
3. 重叠 span 处理：同类型取置信度高者，不同类型取主模型优先
4. 去重：相同 start/end/type 的 span 合并

#### 策略二：按场景切换

前端分析输入文本特征，自动选择最佳模型：

```
用户输入文本
    │
    ▼
┌─────────────────────────┐
│  场景检测                │
│  ├─ 文本语言检测          │
│  ├─ 是否包含代码片段？     │
│  └─ 文本类型分类          │
└──────────┬──────────────┘
           │
     ┌─────┴─────┬──────────┬──────────┐
     ▼           ▼          ▼          ▼
┌──────────┐ ┌────────┐ ┌────────┐ ┌──────────┐
│ 英文文档  │ │ 中文文档 │ │ 代码   │ │ 多语言文档 │
│ Privacy  │ │Privacy │ │starpii │ │multilang │
│ Filter   │ │Filter  │ │        │ │-pii-ner  │
└──────────┘ └────────┘ └────────┘ └──────────┘
```

#### 策略三：级联推理（先快后慢）

先跑轻量模型，置信度低的片段交给重量级模型复审：

```
输入文本
    │
    ▼
┌──────────────────────┐
│  快速模型（轻量）       │
│  bert-small-pii       │
│  低延迟 CPU 推理       │
└──────────┬───────────┘
           │
    ┌──────┴──────┐
    │ 置信度 ≥ 0.9 │
    │ 且不重叠？    │
    └──────┬──────┘
     ┌─────┴─────┐
     │ 是         │ 否
     ▼           ▼
  ┌──────┐ ┌──────────────┐
  │ 直接  │ │ 重量级模型复审 │
  │ 采纳  │ │ Privacy Filter│
  └──────┘ └──────────────┘
```

### 7.2 并行推理管道设计

```rust
// 概念设计：多模型推理管理器
pub struct MultiModelEngine {
    models: Vec<(String, NerEngine)>,
    strategy: FusionStrategy,
}

enum FusionStrategy {
    /// 主模型 + 辅助模型（主模型优先）
    MasterAux { master: String, aux: Vec<String> },
    /// 所有模型投票
    Vote { min_votes: usize },
    /// 级联（先快后慢）
    Cascade { fast: String, slow: String },
}

impl MultiModelEngine {
    pub fn analyze(&mut self, text: &str) -> Vec<EntitySpan> {
        match &self.strategy {
            FusionStrategy::MasterAux { master, aux } => {
                let mut all_spans = Vec::new();
                // 主模型推理
                if let Some(engine) = self.get_mut(master) {
                    all_spans.extend(engine.infer(text).unwrap_or_default());
                }
                // 辅助模型推理
                for name in aux {
                    if let Some(engine) = self.get_mut(name) {
                        all_spans.extend(engine.infer(text).unwrap_or_default());
                    }
                }
                // 合并 + 去重 + 冲突解决
                self.fuse_spans(all_spans)
            }
            // ... 其他策略
        }
    }
}
```

### 7.3 结果合并与冲突解决

当多个模型产生重叠实体时，使用扩展后的冲突解决规则：

```
重叠情况处理：
│
├── 两模型预测相同类型 + 显著重叠
│   → 取并集范围 (min start, max end)
│   → 取高置信度
│
├── 两模型预测不同类型 + 完全包含
│   → 主模型优先（按配置的模型优先级）
│
├── 两模型预测不同类型 + 部分重叠
│   → 保留两个实体（冲突解决器按优先级处理）
│
├── 仅一端检测到实体
│   → 保留
│
└── 两端检测到相同 span
    → 去重
```

### 7.4 前端模型选择器设计建议

可在 `SettingsPage` 中添加模型管理区域：

```
┌─────────────────────────────────────────┐
│  AI 识别模型                             │
│                                         │
│  [当前模型: openai/privacy-filter v1.0] │
│                                         │
│  ┌─ 可用模型 ──────────────────────────┐│
│  │  ○ privacy-filter (当前) 8类 PII   ││
│  │  ○ multilang-pii-ner   32类 PII    ││
│  │  ○ bert-small-pii      23类 PII    ││
│  └─────────────────────────────────────┘│
│                                         │
│  ┌─ 并行策略 ──────────────────────────┐│
│  │  [主模型 + 辅助模型 ▼]              ││
│  │  主模型: [privacy-filter ▼]        ││
│  │  辅助模型: [multilang-pii-ner ▼]   ││
│  │  [+ 添加辅助模型]                   ││
│  └─────────────────────────────────────┘│
│                                         │
│  模型状态: ● 就绪 (3/3 模型可用)        │
│  上次推理: 15ms (单模型)                │
└─────────────────────────────────────────┘
```

### 7.5 性能预算估算

同一文本依次或并行运行多个模型的延迟预算：

| 场景 | 模型组合 | 预估总延迟（CPU） | 内存占用 |
|------|---------|-----------------|---------|
| 单模型 | Privacy Filter | ~10-20ms | ~300MB |
| 主+辅（2 模型） | Privacy + multilang | ~30-50ms | ~800MB |
| 通用+专用（3 模型） | Privacy + multilang + bert-small | ~40-70ms | ~900MB |
| 级联（先快后慢） | bert-small → Privacy | ~15-35ms | ~400MB |

**优化方案**：
- 使用线程池并行运行多个模型（`std::sync::mpsc` 或 Rayon）
- 小文本只跑主模型，长文本才启用辅助模型
- 辅助模型的结果缓存（内容哈希键）

---

## 8. 附录

### 附录 A：模型评分工作表示例

以下是对当前已知候选模型的完整评分：

#### A.1 openai/privacy-filter（当前模型）

| 维度 | 分数 | 权重 | 加权分 | 依据 |
|------|------|------|--------|------|
| 标签方案匹配度 | 5 | 25% | 1.25 | BIOES 33 类，完全匹配 |
| PII 覆盖度 | 4 | 20% | 0.80 | 8 类核心 PII |
| 性能指标 F1 | — | 20% | — | F1 数据未公开 |
| 延迟与体积 | 3 | 15% | 0.45 | 1.5B 总/50M 活跃，有 Q4 量化 |
| 上下文长度 | 5 | 10% | 0.50 | 128K tokens |
| 多语言支持 | 3 | 5% | 0.15 | 多语言（o200k tokenizer） |
| 维护状态 | 5 | 5% | 0.25 | 2026 年发布，活跃 |
| **总分** | | **100%** | **3.40** | 基准线 |

#### A.2 onnx-community/multilang-pii-ner-ONNX

| 维度 | 分数 | 权重 | 加权分 | 依据 |
|------|------|------|--------|------|
| 标签方案匹配度 | 3 | 25% | 0.75 | BIO 方案，非 BIOES（改动级 D） |
| PII 覆盖度 | 5 | 20% | 1.00 | 30+ 精细 PII 类型 |
| 性能指标 F1 | 5 | 20% | 1.00 | F1=0.954（接近 5 分线 0.97） |
| 延迟与体积 | 4 | 15% | 0.60 | XLM-RoBERTa Base ~500M，已提供 ONNX |
| 上下文长度 | 3 | 10% | 0.30 | 512（XLM-RoBERTa Base 默认） |
| 多语言支持 | 5 | 5% | 0.25 | EN/DE/IT/FR（基于 XLM-RoBERTa） |
| 维护状态 | 5 | 5% | 0.25 | 2025-2026 活跃 |
| **总分** | | **100%** | **4.15** | **强推荐** |

#### A.3 rtrigoso/bert-small-pii-detection-ONNX

| 维度 | 分数 | 权重 | 加权分 | 依据 |
|------|------|------|--------|------|
| 标签方案匹配度 | 3 | 25% | 0.75 | BIO 方案，24 类标签 |
| PII 覆盖度 | 4 | 20% | 0.80 | 23 类 PII（含 CREDIT_CARD, SSN 等） |
| 性能指标 F1 | 4 | 20% | 0.80 | 详见模型卡片，多数实体 >0.90 |
| 延迟与体积 | 5 | 15% | 0.75 | BERT-Small ~70M，极轻量 |
| 上下文长度 | 3 | 10% | 0.30 | 512 |
| 多语言支持 | 2 | 5% | 0.10 | 仅英文 |
| 维护状态 | 4 | 5% | 0.20 | 2024 更新 |
| **总分** | | **100%** | **3.70** | **推荐（并行/辅助）** |

### 附录 B：常用 HuggingFace 搜索命令速查

#### REST API 大全

```bash
# ============ 基础搜索 ============

# 按关键词搜索
GET /api/models?search=pii

# 按 pipeline_tag 搜索
GET /api/models?pipeline_tag=token-classification

# 多条件组合
GET /api/models?search=pii&pipeline_tag=token-classification&sort=downloads

# ============ 排序 ============

# 按下载量降序
GET /api/models?search=pii&sort=downloads&direction=-1

# 按点赞数降序
GET /api/models?search=pii&sort=likes&direction=-1

# 按最后更新
GET /api/models?search=pii&sort=lastModified&direction=-1

# ============ 过滤 ============

# 按库过滤
GET /api/models?search=pii&filter=onnx

# 按许可证过滤
GET /api/models?search=pii&filter=license:apache-2.0

# 按作者过滤
GET /api/models?author=openai

# ============ 翻页 ============

GET /api/models?search=pii&limit=50
# 返回 headers 中的 Link 字段包含下一页 URL

# ============ 完整信息 ============

GET /api/models/{org}/{model}
# 返回完整元数据，含 cardData, siblings (文件列表), config 等

GET /api/models/{org}/{model}?full=true
# 更完整的信息

# ============ 查看模型文件列表 ============

GET /api/models/{org}/{model}/tree/main
# 列出仓库中的文件和目录
```

#### Python SDK

```python
from huggingface_hub import HfApi, ModelFilter, ModelSearchArguments
api = HfApi()

# 搜索 PII 模型
models = api.list_models(
    task="token-classification",
    search="pii",
    sort="downloads",
    direction=-1,
    limit=100,
)

# 过滤出有 ONNX 的
for m in models:
    siblings = [s.rfilename for s in api.get_model_siblings(m.modelId)]
    has_onnx = any(".onnx" in s for s in siblings)
    has_tok = "tokenizer.json" in siblings
    print(f"{m.modelId}: onnx={has_onnx}, tok={has_tok}")

# 查看模型配置
config = api.get_model_config(m.modelId)
if config:
    print(f"model_type: {config.get('model_type')}")
    print(f"architectures: {config.get('architectures')}")
    print(f"num_labels: {config.get('num_labels')}")
    print(f"id2label: {config.get('id2label')}")

# 下载模型
api.snapshot_download(
    repo_id="onnx-community/multilang-pii-ner-ONNX",
    local_dir="./models/multilang-pii/",
    allow_patterns=["*.onnx", "tokenizer.json", "config.json"],
)
```

#### HuggingFace CLI

```bash
# 搜索
huggingface-cli search pii --pipeline-tag token-classification

# 查看模型详情
huggingface-cli model-info onnx-community/multilang-pii-ner-ONNX

# 列出文件
huggingface-cli file-list onnx-community/multilang-pii-ner-ONNX

# 下载
huggingface-cli download onnx-community/multilang-pii-ner-ONNX \
    --local-dir ./models/multilang-pii/ \
    --include "*.onnx" "tokenizer.json" "config.json"
```

### 附录 C：ner_engine.rs 代码补丁模板

#### C.1 添加 token_type_ids 支持

```rust
// 在 infer() 方法中，构建输入张量时增加 token_type_ids
pub fn infer(&mut self, text: &str) -> Result<Vec<EntitySpan>> {
    // ... 现有 tokenizer 编码代码 ...

    let input_ids: Vec<i64> = encoding.get_ids()[..seq_len]
        .iter().map(|&x| x as i64).collect();
    let attention_mask: Vec<i64> = encoding.get_attention_mask()[..seq_len]
        .iter().map(|&x| x as i64).collect();
    // 新增：token_type_ids（全 0，因为我们是单句推理）
    let token_type_ids: Vec<i64> = vec![0i64; seq_len];

    let input_ids_tensor = Tensor::from_array(([1, seq_len], input_ids))?;
    let attention_mask_tensor = Tensor::from_array(([1, seq_len], attention_mask))?;
    let token_type_ids_tensor = Tensor::from_array(([1, seq_len], token_type_ids))?;

    // 根据模型是否需要动态决定是否传入 token_type_ids
    let outputs = if self.need_token_type_ids {
        self.session.run(ort::inputs![
            "input_ids" => input_ids_tensor,
            "attention_mask" => attention_mask_tensor,
            "token_type_ids" => token_type_ids_tensor,
        ])?
    } else {
        self.session.run(ort::inputs![
            "input_ids" => input_ids_tensor,
            "attention_mask" => attention_mask_tensor,
        ])?
    };

    // ... 后续代码不变 ...
}
```

#### C.2 BIO 兼容的标签解析器

```rust
// 新增：将 BIO 标签转换为内部 BIOES 表示
fn parse_bio_label(label: &str) -> BioesLabel {
    if label == "O" {
        return BioesLabel::O;
    }
    if let Some(rest) = label.strip_prefix("B-") {
        BioesLabel::B(rest.to_string())
    } else if let Some(rest) = label.strip_prefix("I-") {
        BioesLabel::I(rest.to_string())
    } else {
        BioesLabel::O
    }
}

// 在 decode_bioes() 中，根据模型标签方案选择解析器
let label = if self.label_scheme == LabelScheme::BIO {
    parse_bio_label(label_str)
} else {
    BioesLabel::parse(label_str)
};
```

#### C.3 从 config.json 动态设置 max_length

```rust
// 在 NerEngine::load() 中，加载完 config.json 后设置 max_length
fn load_max_length(model_dir: &Path) -> usize {
    let config_path = model_dir.join("config.json");
    if let Ok(content) = std::fs::read_to_string(&config_path) {
        if let Ok(config) = serde_json::from_str::<serde_json::Value>(&content) {
            if let Some(max_pos) = config.get("max_position_embeddings").and_then(|v| v.as_u64()) {
                return (max_pos as usize).min(4096); // 设上限防止 OOM
            }
        }
    }
    512 // 默认值
}
```

#### C.4 ONNX 输入名自适应检测

```rust
// 在 NerEngine::load() 中，加载完 session 后检测输入名
fn detect_model_inputs(session: &Session) -> HashSet<String> {
    let mut inputs = HashSet::new();
    // 注意：ort crate 的 API 可能需要通过 session.inputs 获取
    // 此处为概念代码，具体 API 取决于 ort crate 版本
    for input_meta in session.inputs() {
        inputs.insert(input_meta.name.to_string());
    }
    inputs
}
```

---

> 本文档由 AI 辅助生成，持续更新。每次评估新模型时，请使用 6.3 节的评分模板记录评估结果，并在此文档的附录 A 中添加新条目。
