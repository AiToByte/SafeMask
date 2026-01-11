# 🛡️ SafeMask v0.5.0 (Ultra-Performance)

[![Rust](https://img.shields.io/badge/language-Rust-orange.svg)](https://www.rust-lang.org/)
[![Performance](https://img.shields.io/badge/performance-500MB%2Fs+-green.svg)](#-performance-benchmarks)
[![License](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)
[![Platform](https://img.shields.io/badge/platform-Windows%20%7C%20Linux%20%7C%20macOS-lightgrey.svg)](#-installation)

**SafeMask** 是一款工业级的、基于 Rust 开发的高性能隐私数据脱敏工具。它专为 **LLM (大模型) 训练数据清洗**、**跨境日志审计**以及**开发者隐私保护**场景设计。

通过深度利用 Rust 的底层系统编程特性，SafeMask 在保证数据**绝对时序性（Ordered）**的前提下，将文本处理吞吐量推向了硬件 I/O 的极限。

---

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

---

## 📊 性能基准 (Performance Benchmarks)

在测试环境（i7-12700K / NVMe SSD / 1.2GB Log File）下：

| 模式 | 吞吐量 (MB/s) | 1.2GB 处理耗时 | 备注 |
| :--- | :--- | :--- | :--- |
| 传统正则 (Python/Java) | ~15-30 MB/s | ~60s | 存在 GC 抖动/性能瓶颈 |
| SafeMask v0.1.0 | ~100 MB/s | 12s | 初步并行化 |
| **SafeMask v0.5.0** | **450 - 650 MB/s** | **~2.2s** | **三阶段流水线 + 字节引擎** |

> **结论**：SafeMask v0.5.0 的速度主要受限于磁盘 I/O 带宽，计算层已基本实现零阻塞。

---

## 🛠️ 安装与快速开始

### 1. 编译
我们推荐使用全量编译优化（PGO/LTO）：
```bash
cargo build --release
```
二进制产物位于 `target/release/safemask`。

### 2. 使用
- **剪贴板监听**：`./safemask --mode clipboard`
- **文件高速处理**：`./safemask --mode file --path test.log --output masked.log`

---

## ⚙️ 规则配置化 (Modular Rules)

SafeMask 允许通过 YAML 动态配置脱敏规则，支持分包管理：

```yaml
# rules/ai/keys.yaml
group: "AI_AUTH_KEYS"
rules:
  - name: "OpenAI"
    pattern: '\bsk-[a-zA-Z0-9]{48}\b'
    mask: "<OPENAI_KEY>"
```

---

## 💎 为什么选择 SafeMask？

作为一名拥有 5 年以上后端开发经验的工程师，我在构建 SafeMask 时融入了对**分布式系统**和**高性能底层开发**的深度理解：
1. **开发者友好**：提供 CLI 和 GUI (Tauri 版) 双重选择。
2. **安全至上**：100% 离线，无任何网络请求，符合 GDPR 标准。
3. **极简运维**：提供全平台静态链接的二进制包，无任何运行环境依赖（No Runtime/No JVM）。

---

## 🤝 参与贡献

我们欢迎社区提交新的脱敏规则：
1. 在 `rules/` 下创建分类目录。
2. 遵循 `RULES_TEMP.md` 中的非环视正则规范。
3. 提交 PR 并附带性能测试结果。