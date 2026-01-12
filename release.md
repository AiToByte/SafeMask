根据你新增的**法定目录规则加载**和**混合引擎（AC + Regex）**功能，我为你优化了 `README.md`。

这次更新重点突出了**“零配置自定义”**、**“混合动力引擎”**以及**“法定目录规范”**，使文档更具工具书的专业感。

---

# 🛡️ SafeMask v0.4.2

<p align="center">
  <strong>SafeMask</strong> - <em>让每一行数据都能安全地拥抱 AI。</em>
</p>

[![Rust](https://img.shields.io/badge/language-Rust-orange.svg)](https://www.rust-lang.org/)
[![Performance](https://img.shields.io/badge/performance-300MB%2Fs+-green.svg)](#-性能基准)
[![License](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)
[![Platform](https://img.shields.io/badge/platform-Windows%20%7C%20Linux%20%7C%20macOS-lightgrey.svg)](#-使用指南)

**SafeMask** 是一款工业级的隐私数据脱敏工具，专为 AI 开发者、安全审计员设计。它不仅能瞬间处理 GB 级日志，更能通过**语义化脱敏**，在保护隐私的同时完整保留数据的逻辑上下文，让 AI 辅助分析不再受阻。

---

## ✨ v0.4.2 新特性：灵活定制，极致性能

*   **🗂️ 法定目录管理**：自动扫描 `rules/` (系统内置) 与 `custom/` (用户自定义) 目录，规则变更无需重新编译。
*   **🚀 混合动力引擎**：
    *   **固定词过滤**：自动识别纯文本规则（如人名、项目名），采用 **Aho-Corasick** 算法，实现 $O(n)$ 级极速过滤。
    *   **模式匹配**：复杂隐私模式采用 **高性能字节正则**，分层优先级处理。
*   **🧠 AI 友好型语义**：支持将敏感信息替换为 `<EMAIL>`、`<PROJECT_ID>` 等标签，而非破坏性的 `***`。

---

## 🏗️ 核心架构

SafeMask 采用三阶段保序流水线，实现了 CPU 计算与磁盘 I/O 的完全重叠：

```text
[ 原始数据 ] 
     |
     v
( Stage 1: 生产者 ) ➔ 内存映射 (Mmap) 自动分块
     |
     v
( Stage 2: 混合计算 ) ➔ AC 自动机 (固定词) + 分层正则 (模式) 
     |
     v
( Stage 3: 消费者 ) ➔ BTreeMap 保序缓冲区 ➔ 聚合写入 (BufWriter)
     |
     v
[ 脱敏产物 ]
```

---

## ⚙️ 规则定制指南

SafeMask 强制执行目录化管理，你只需将 YAML 规则文件放入对应目录即可生效。

### 1. 目录结构
```text
.
├── safemask.exe       # 执行文件
├── rules/             # [系统级] 内置规则 (IP, Email, API Keys等)
└── custom/            # [用户级] 在这里添加你的私有规则
    ├── private.yaml
    └── internal.yaml
```

### 2. 配置示例 (`custom/my_rules.yaml`)
```yaml
group: "MY_CUSTOM_RULES"
rules:
  # 固定字符串匹配 (极速模式)
  - name: "PersonalName"
    pattern: "xiaosheng"
    mask: "<MY_NAME>"
    priority: 100

  # 正则模式匹配
  - name: "InternalProject"
    pattern: 'PROJ-[0-9]{5,}'
    mask: "<PROJECT_ID>"
    priority: 80
```

---

## 📊 性能基准 (Performance Benchmarks)
*测试环境: Windows 11 / i7-12700K / NVMe SSD*

| 数据规模 | 原始处理耗时 (PS) | **SafeMask 耗时** | 吞吐量 (Throughput) |
| :--- | :--- | :--- | :--- |
| **113 MB (100万行)** | 21.9s | **0.42s** | **~270 MB/s** |
| **1.2 GB (500万行)** | - | **4.1s** | **~300 MB/s** |
| **2.3 GB (1000万行)** | - | **8.3s** | **~337 MB/s** |

---

## 📖 使用指南

### 1. 剪贴板模式 (与 AI 对话神器)
最适合在将代码或日志粘贴给 ChatGPT/Claude 前使用。程序会自动读取剪贴板内容，脱敏后回写：
```powershell
./safemask --mode clipboard
```

### 2. 文件模式 (大规模清洗)
处理本地大文件，支持保序输出：
```powershell
./safemask --mode file --path ./input.log --output ./output_masked.log
```

---

## 🏗️ 设计哲学

1.  **语义保留**：脱敏后的数据应保持“可读性”。AI 应该知道这是一个 `<DATABASE_URL>`，而不是一串星号。
2.  **零拷贝优先**：利用 `Mmap` 和字节流处理，将内存分配压力降至最低。
3.  **宁过错杀，不可漏过**：引擎设计倾向于覆盖更广的隐私特征，确保安全合规。

## 🤝 贡献
欢迎提交 PR 增加更多内置规则：
1. 在 `rules/` 下创建新的分类。
2. 确保正则不包含不支持的环视（Lookaround）语法。
3. 提交前请运行性能测试。

---
<p align="center">
  <strong>SafeMask</strong> - <em>Empowering Data Security in the AI Era.</em>
</p>