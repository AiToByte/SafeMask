# SafeMask
This software de-identifies sensitive personal information. After your content is processed by SafeMask, all private information will be anonymized, allowing your information to be safely transmitted and processed on the internet, by AI, etc.

---

# 🛡️ SafeMask v0.4.1

[![Rust](https://img.shields.io/badge/language-Rust-orange.svg)](https://www.rust-lang.org/)
[![Performance](https://img.shields.io/badge/performance-500MB%2Fs+-green.svg)](#-performance-benchmarks)
[![License](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)
[![Platform](https://img.shields.io/badge/platform-Windows%20%7C%20Linux%20%7C%20macOS-lightgrey.svg)](#-installation)

## 📝 介绍
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
在 Windows 11 / i7-12700K 环境下对真实日志进行测试：
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
INFO [2026-01-09] REQ_ID:c6c146f4-5f59-49fb-9af3-ae53dffd80fe | Client: 158.209.138.172 | Phone: 13184327690 | Email: user_c6c146f4@internal.cloud | DNS: node-923.api.service.io | DB: postgres://admin:pwd6435@/db_main | Key: sk-c6c146f4-5f59-49fb-9af3-ae53dffd80fec6c146f4-5f59-49fb-9af3-ae53dffd80fe
INFO [2026-01-09] REQ_ID:fdfbc6fe-6a6f-4a29-ad25-33800e07a54c | Client: 199.203.32.197 | Phone: 15018443387 | Email: user_fdfbc6fe@internal.cloud | DNS: node-806.prod.corp | DB: postgres://admin:pwd6920@/db_main | Key: sk-fdfbc6fe-6a6f-4a29-ad25-33800e07a54cfdfbc6fe-6a6f-4a29-ad25-33800e07a54c
INFO [2026-01-09] REQ_ID:1cee87c0-c759-4a28-9c5a-f53d0795fa33 | Client: 170.35.237.6 | Phone: 15025377154 | Email: user_1cee87c0@prod.corp | DNS: node-176.secure.node | DB: postgres://admin:pwd5656@/db_main | Key: sk-1cee87c0-c759-4a28-9c5a-f53d0795fa331cee87c0-c759-4a28-9c5a-f53d0795fa33
INFO [2026-01-09] REQ_ID:79909f77-e7b6-4cbc-84eb-802894deb6cd | Client: 89.153.179.13 | Phone: 13887270345 | Email: user_79909f77@prod.corp | DNS: node-725.dev.local | DB: postgres://admin:pwd9011@/db_main | Key: sk-79909f77-e7b6-4cbc-84eb-802894deb6cd79909f77-e7b6-4cbc-84eb-802894deb6cd
INFO [2026-01-09] REQ_ID:8c18cfe5-33f9-49f8-a958-2f00d6018dda | Client: 99.21.87.115 | Phone: 18833721927 | Email: user_8c18cfe5@secure.node | DNS: node-448.prod.corp | DB: postgres://admin:pwd4623@/db_main | Key: sk-8c18cfe5-33f9-49f8-a958-2f00d6018dda8c18cfe5-33f9-49f8-a958-2f00d6018dda
INFO [2026-01-09] REQ_ID:4eb8c76e-4626-4b1c-a4b2-23556d727cf5 | Client: 150.161.35.52 | Phone: 18953035548 | Email: user_4eb8c76e@api.service.io | DNS: node-564.prod.corp | DB: postgres://admin:pwd7753@/db_main | Key: sk-4eb8c76e-4626-4b1c-a4b2-23556d727cf54eb8c76e-4626-4b1c-a4b2-23556d727cf5
INFO [2026-01-09] REQ_ID:e9915291-850f-4043-8184-053b51932395 | Client: 158.5.64.110 | Phone: 15026107635 | Email: user_e9915291@internal.cloud | DNS: node-169.secure.node | DB: postgres://admin:pwd4699@/db_main | Key: sk-e9915291-850f-4043-8184-053b51932395e9915291-850f-4043-8184-053b51932395
INFO [2026-01-09] REQ_ID:9c442dbb-24c2-4ba1-a666-22c4dfb32ab8 | Client: 211.100.85.125 | Phone: 15062438861 | Email: user_9c442dbb@secure.node | DNS: node-531.prod.corp | DB: postgres://admin:pwd1412@/db_main | Key: sk-9c442dbb-24c2-4ba1-a666-22c4dfb32ab89c442dbb-24c2-4ba1-a666-22c4dfb32ab8
INFO [2026-01-09] REQ_ID:8aa7a501-7aa1-4cf3-92d9-944652605994 | Client: 123.208.175.66 | Phone: 13174268227 | Email: user_8aa7a501@dev.local | DNS: node-550.internal.cloud | DB: postgres://admin:pwd6410@/db_main | Key: sk-8aa7a501-7aa1-4cf3-92d9-9446526059948aa7a501-7aa1-4cf3-92d9-944652605994
```

#### 3.2 使用SafeMask之后
```txt
INFO [2026-01-09] REQ_ID:c6c146f4-5f59-49fb-9af3-ae53dffd80fe | Client: <IPv4> | Phone: <CHINA_MOBILE> | Email: <EMAIL> | DNS: 
<DOMAIN> | DB: <POSTGRES_URI> | Key: <OPENAI_KEY>
INFO [2026-01-09] REQ_ID:fdfbc6fe-6a6f-4a29-ad25-33800e07a54c | Client: <IPv4> | Phone: <CHINA_MOBILE> | Email: <EMAIL> | DNS: 
<DOMAIN> | DB: <POSTGRES_URI> | Key: <OPENAI_KEY>
INFO [2026-01-09] REQ_ID:1cee87c0-c759-4a28-9c5a-f53d0795fa33 | Client: <IPv4> | Phone: <CHINA_MOBILE> | Email: <EMAIL> | DNS: 
<DOMAIN> | DB: <POSTGRES_URI> | Key: <OPENAI_KEY>
INFO [2026-01-09] REQ_ID:79909f77-e7b6-4cbc-84eb-802894deb6cd | Client: <IPv4> | Phone: <CHINA_MOBILE> | Email: <EMAIL> | DNS: 
<DOMAIN> | DB: <POSTGRES_URI> | Key: <OPENAI_KEY>
INFO [2026-01-09] REQ_ID:8c18cfe5-33f9-49f8-a958-2f00d6018dda | Client: <IPv4> | Phone: <CHINA_MOBILE> | Email: <EMAIL> | DNS: 
<DOMAIN> | DB: <POSTGRES_URI> | Key: <OPENAI_KEY>
INFO [2026-01-09] REQ_ID:4eb8c76e-4626-4b1c-a4b2-23556d727cf5 | Client: <IPv4> | Phone: <CHINA_MOBILE> | Email: <EMAIL> | DNS: 
<DOMAIN> | DB: <POSTGRES_URI> | Key: <OPENAI_KEY>
INFO [2026-01-09] REQ_ID:e9915291-850f-4043-8184-053b51932395 | Client: <IPv4> | Phone: <CHINA_MOBILE> | Email: <EMAIL> | DNS: 
<DOMAIN> | DB: <POSTGRES_URI> | Key: <OPENAI_KEY>
INFO [2026-01-09] REQ_ID:9c442dbb-24c2-4ba1-a666-22c4dfb32ab8 | Client: <IPv4> | Phone: <CHINA_MOBILE> | Email: <EMAIL> | DNS: 
<DOMAIN> | DB: <POSTGRES_URI> | Key: <OPENAI_KEY>
INFO [2026-01-09] REQ_ID:8aa7a501-7aa1-4cf3-92d9-944652605994 | Client: <IPv4> | Phone: <CHINA_MOBILE> | Email: <EMAIL> | DNS: 
<DOMAIN> | DB: <POSTGRES_URI> | Key: <OPENAI_KEY> 
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

