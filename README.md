# SafeMask
This software de-identifies sensitive personal information. After your content is processed by SafeMask, all private information will be anonymized, allowing your information to be safely transmitted and processed on the internet, by AI, etc.

---

# 🛡️ SafeMask

[![Rust](https://img.shields.io/badge/language-Rust-orange.svg)](https://www.rust-lang.org/)
[![License](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)
[![Performance](https://img.shields.io/badge/performance-Ultra--High-green.svg)](#performance)

**SafeMask** 是一款基于 Rust 开发的极致性能隐私数据脱敏工具。它专为处理大规模日志、代码库及敏感文本设计，能够瞬间识别并遮盖 AI API Keys、数据库连接串、IP 地址、手机号等敏感信息，确保数据在进入 AI 模型或共享环境前的合规性。

## ✨ 核心特性

- 🚀 **极致吞吐**：基于内存映射（Mmap）与单次扫描（Single-Pass）正则引擎，支持 GB 级数据秒级处理。
- 🧵 **多核并发**：利用 Rayon 并行流水线，自动压榨多核 CPU 性能。
- 🧠 **混合动力引擎**：
  - **Aho-Corasick 算法**：毫秒级处理成千上万个固定关键词。
  - **DFA 超级正则**：聚合多维规则，无论多少正则，文本仅需扫描一遍。
- 📦 **模块化规则**：支持通过 YAML 文件动态配置规则，按包（Package）和分类管理。
- 💾 **极低内存**：通过内存映射技术（Memory Mapping），处理超大文件时的内存占用仅为数十 MB。
- 📋 **剪贴板集成**：一键处理剪贴板内容，无缝衔接 AI 辅助开发流。

## ⚡ 性能表现 (Benchmark)

在 Windows 11 / i7-12700K 环境下对真实日志进行测试：

| 数据量 | 原始耗时 (PS Redirect) | **SafeMask 优化输出 (-o)** | 吞吐量 (Throughput) |
| :--- | :--- | :--- | :--- |
| **113 MB (100万行)** | 21.9s | **0.42s** | **~270 MB/s** |
| **1.2 GB (1000万行)** | - | **4.1s** | **~300 MB/s** |

> *注：性能受限于磁盘 I/O 上限。*

## 🛠️ 安装与编译

确保已安装 Rust 环境 (MSRV 1.70+)。

```bash
git clone https://github.com/YourUsername/safemask.git
cd safemask

# 必须使用 --release 模式以开启所有编译优化
cargo build --release
```

编译产物位于 `./target/release/safemask`。

## 📖 使用指南

### 1. 剪贴板模式
最适合在将代码或日志粘贴给 ChatGPT/Claude 前使用：
```powershell
./safemask --mode clipboard
```

### 2. 文件模式
处理大规模日志文件，并直接输出到指定文件（推荐）：
```powershell
./safemask --mode file --path ./input.log --output ./output_masked.log
```

## ⚙️ 规则配置

规则以 YAML 格式存储在 `rules/` 目录下，支持多层文件夹分类：

```yaml
# rules/ai/keys.yaml
group: "AI_API_KEYS"
rules:
  - name: "OpenAI"
    pattern: '\bsk-[a-zA-Z0-9]{48}\b'
    mask: "<OPENAI_KEY>"
  - name: "DeepSeek"
    pattern: '\bsk-[a-z0-9]{32}\b'
    mask: "<DEEPSEEK_KEY>"
```

## 🏗️ 架构背后的思考

作为一个拥有 Java 背景的开发者，我在设计 SafeMask 时重点解决了以下痛点：
1. **规避 GC 停顿**：通过 Rust 的所有权模型与 `mimalloc` 分配器，消除大规模字符串处理中的停顿。
2. **零拷贝 I/O**：使用 `Mmap` 替代传统的缓冲读取，减少内核态与用户态的数据拷贝。
3. **算法聚合**：避免了 $N$ 次 `replace_all` 导致的 $O(N \times M)$ 复杂度，将其优化为 $O(M)$。

## 🤝 贡献

**欢迎提交 Issue 或 Pull Request 来增加更多的脱敏规则！**

