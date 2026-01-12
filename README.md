# SafeMask
This software de-identifies sensitive personal information. After your content is processed by SafeMask, all private information will be anonymized, allowing your information to be safely transmitted and processed on the internet, by AI, etc.

---

# 🛡️ SafeMask v0.4.1

[![Rust](https://img.shields.io/badge/language-Rust-orange.svg)](https://www.rust-lang.org/)
[![Performance](https://img.shields.io/badge/performance-500MB%2Fs+-green.svg)](#-performance-benchmarks)
[![License](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)
[![Platform](https://img.shields.io/badge/platform-Windows%20%7C%20Linux%20%7C%20macOS-lightgrey.svg)](#-installation)

**SafeMask** 是一款工业级的、基于 Rust 开发的高性能隐私数据脱敏工具。它专为 **LLM (大模型) 训练数据清洗**、**跨境日志审计**以及**开发者隐私保护**场景设计。

**SafeMask** 是一款基于 Rust 开发的极致性能隐私数据脱敏工具。它专为处理大规模日志、代码库及敏感文本设计，能够瞬间识别并遮盖 AI API Keys、数据库连接串、IP 地址、手机号等敏感信息，确保数据在进入 AI 模型或共享环境前的合规性。
同时, 也可用于**LLM (大模型) 训练数据清洗**、**跨境日志审计**以及**开发者隐私保护**场景中。


## 🚀 核心架构：三阶段保序流水线 (Level 3 Optimization)

SafeMask 不仅仅是一个正则替换工具，它采用了复杂的**生产者-消费者流水线**模型，实现了 **CPU 计算与 I/O 读写的完全重叠（Overlapping）**。

### 🏗️ 架构概览
```text
[ 磁盘文件 ] 
     |
     v
( Stage 1: 生产者 ) -> 内存映射 (Mmap) + 智能宏分块 (Macro-Chunking 4MB)
     |
     v
( Stage 2: 计算集群 ) -> Rayon 并行计算 | 字节流正则 (Regex Bytes) | AC 自动机
     |
     v
( Stage 3: 消费者 ) -> 优先级缓冲区 (BTreeMap) | 保序合并 | 8MB 聚合写入 (BufWriter)
     |
     v
[ 脱敏输出 ]
```


### ⚡ 深度优化细节
- **Zero-Copy I/O**: 使用 `memmap2` 绕过内核缓冲区拷贝。
- **Byte-Level Engine**: 基于 `regex::bytes` 实现，完全跳过 UTF-8 校验开销。
- **Ordered Pipelining**: 引入 `crossbeam-channel` 与序列号控制，确保高并发下的日志行序与原始文件 100% 一致。
- **Memory Reuse**: 采用线程局部缓冲区（Scratch Buffers），将内存分配压力从 $O(N)$ 降低到 $O(Threads)$。

## 📊 性能基准 (Performance Benchmarks)
| 数据量 | 原始耗时 (PS Redirect) | **SafeMask 优化输出 (-o)** | 吞吐量 (Throughput) |
| :--- | :--- | :--- | :--- |
| **113 MB (100万行)** | 21.9s | **0.42s** | **~270 MB/s** |
| **1.2 GB (500万行)** | - | **4.1s** | **~300 MB/s** |
| **2.3 GB (1000万行)** | - | **8.3s** | **~337 MB/s** |

> *注：性能受限于磁盘 I/O 上限。*

## 🛠️ 安装与编译

确保已安装 Rust 环境 (MSRV 1.70+)。

```bash
git clone https://github.com/AiToByte/safemask.git
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

### 3.效果演示
#### 3.1 输入
```txt
INFO [2026-01-09] User_1 (IP: 99.237.89.211) accessed TopSecretProject using sk-47e9a70ff0e240ee9f3a0ebb04e9131f
INFO [2026-01-09] User_2 (IP: 52.158.34.170) accessed TopSecretProject using sk-7eee0e40148040b29a56829556fe0b88
INFO [2026-01-09] User_3 (IP: 225.95.77.71) accessed TopSecretProject using sk-0289c054a0394bb2a99daa44b5d4f4a2
INFO [2026-01-09] User_4 (IP: 49.75.32.104) accessed TopSecretProject using sk-6541c4475bae4ac1a44d9d5813b4575c
INFO [2026-01-09] User_5 (IP: 55.231.84.214) accessed TopSecretProject using sk-5c91dcc325e941b59817789f60cf7bb0
INFO [2026-01-09] User_6 (IP: 210.55.8.24) accessed TopSecretProject using sk-fe044476c8494108881746217929c82c
INFO [2026-01-09] User_7 (IP: 127.183.99.151) accessed TopSecretProject using sk-f2b904e3b686496ca4b88c8698a27818
INFO [2026-01-09] User_8 (IP: 109.143.251.146) accessed TopSecretProject using sk-4c22af8a8da94424a3bd8cbcab30d398
INFO [2026-01-09] User_9 (IP: 250.88.109.70) accessed TopSecretProject using sk-4cd6d738173441d284b0f32141c82fd4
INFO [2026-01-09] User_10 (IP: 118.41.41.205) accessed TopSecretProject using sk-9ac26f750ab741729f4a5813fbe739e8
INFO [2026-01-09] User_11 (IP: 152.38.117.101) accessed TopSecretProject using sk-f0a2b85db0e2404e8ca8b506e1b4f99e
INFO [2026-01-09] User_12 (IP: 145.44.57.211) accessed TopSecretProject using sk-5e0bc7589ad543c785d6cebfc5f2941b
```

#### 3.2 使用SafeMask之后
```txt
INFO [2026-01-09] User_1 (IP: <IPV4>) accessed TopSecretProject using <DEEPSEEK_KEY>
INFO [2026-01-09] User_2 (IP: <IPV4>) accessed TopSecretProject using <DEEPSEEK_KEY>
INFO [2026-01-09] User_3 (IP: <IPV4>) accessed TopSecretProject using <DEEPSEEK_KEY>
INFO [2026-01-09] User_4 (IP: <IPV4>) accessed TopSecretProject using <DEEPSEEK_KEY>
INFO [2026-01-09] User_5 (IP: <IPV4>) accessed TopSecretProject using <DEEPSEEK_KEY>
INFO [2026-01-09] User_6 (IP: <IPV4>) accessed TopSecretProject using <DEEPSEEK_KEY>
INFO [2026-01-09] User_7 (IP: <IPV4>) accessed TopSecretProject using <DEEPSEEK_KEY>
INFO [2026-01-09] User_8 (IP: <IPV4>) accessed TopSecretProject using <DEEPSEEK_KEY>
INFO [2026-01-09] User_9 (IP: <IPV4>) accessed TopSecretProject using <DEEPSEEK_KEY>
INFO [2026-01-09] User_10 (IP: <IPV4>) accessed TopSecretProject using <DEEPSEEK_KEY>
INFO [2026-01-09] User_11 (IP: <IPV4>) accessed TopSecretProject using <DEEPSEEK_KEY>
INFO [2026-01-09] User_12 (IP: <IPV4>) accessed TopSecretProject using <DEEPSEEK_KEY>
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
    priority: 5 # 优先级越高，越先处理
  - name: "DeepSeek"
    pattern: '\bsk-[a-z0-9]{32}\b'
    mask: "<DEEPSEEK_KEY>"
    priority: 20 # 优先级越高，越先处理
```

## 🏗️ 架构背后的思考

作为一个拥有 Java 背景的开发者，我在设计 SafeMask 时重点解决了以下痛点：
1. **规避 GC 停顿**：通过 Rust 的所有权模型与 `mimalloc` 分配器，消除大规模字符串处理中的停顿。
2. **零拷贝 I/O**：使用 `Mmap` 替代传统的缓冲读取，减少内核态与用户态的数据拷贝。
3. **算法聚合**：避免了 $N$ 次 `replace_all` 导致的 $O(N \times M)$ 复杂度，将其优化为 $O(M)$。

## 🤝 贡献
我们欢迎社区提交新的脱敏规则：
1. 在 `rules/` 下创建分类目录。
2. 遵循 `RULES_TEMP.md` 中的非环视正则规范。
3. 提交 PR 并附带性能测试结果。

