# 🛡️ SafeMask (v1.1.0) 震撼来袭

**让每一行数据，都能安全地拥抱 AI。**

[![Rust](https://img.shields.io/badge/language-Rust-orange.svg)](https://www.rust-lang.org/)
[![Tauri](https://img.shields.io/badge/framework-Tauri_v2-blue.svg)](https://v2.tauri.app/)
[![Vue](https://img.shields.io/badge/frontend-Vue_3-green.svg)](https://vuejs.org/)
[![License](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)
[![Performance](https://img.shields.io/badge/throughput-300MB%2Fs+-brightgreen.svg)](#-性能基准)

[**立即下载最新版**](https://github.com/AiToByte/SafeMask/releases) | [**英文文档**](README.md)

---

## 💡 为什么需要 SafeMask？

在 LLM（大语言模型）时代，我们将日志、代码或文档粘贴给 ChatGPT/Claude 时，往往面临极高的隐私泄露风险。

*   **痛点 1 (语义破坏)**：传统的 `***` 脱敏会破坏 AI 的理解力。AI 看到 `***` 不知道它原本是 IP、Email 还是 API Key。
*   **痛点 2 (性能瓶颈)**：处理 GB 级的本地日志，传统的文本处理工具（如 Notepad++ 插件）动辄卡死。
*   **痛点 3 (安全焦虑)**：云端脱敏工具不可信，数据在脱敏过程中可能已经泄露。

**SafeMask 完美解决了这些问题。**

---

## ✨ 核心特性

### 1. 🧠 AI 友好型脱敏 (AI-Friendly Semantic Masking)
SafeMask 采用**语义化标签**替换隐私信息。
*   **输入**：`sk-ant-api03-123456...`
*   **SafeMask**：`<CLAUDE_KEY>`
*   **效果**：AI 依然知道这里是一个密钥，能够正确分析逻辑，但无法获取真实内容。

### 2. 🎧 剪贴板隐私盾 (Clipboard Privacy Shield)
*   **静默保护**：开启后，SafeMask 会在后台监听系统剪贴板。
*   **自动拦截**：当检测到敏感隐私（如手机号、密钥）时，自动执行本地脱敏并将安全副本写回剪贴板。
*   **拦截对比**：通过 UI 可视化查看“原文 vs 脱敏文”，确保护隐私万无一失。

### 3. 🚀 工业级文件处理 (Bulk File Processor)
*   **极致性能**：基于 Rust 的 `memmap2` 和 `rayon` 并行引擎。
*   **超大支持**：轻松处理 10GB+ 超大日志文件，内存占用始终保持在极低水平。
*   **保序逻辑**：100% 保持原始行序，不影响后续日志分析。

### 4. 🛠️ 可视化规则引擎 (Visual Rule Manager)
*   **内置规则**：内置涵盖 IP、Email、手机号、身份证、主流 AI API Keys 及数据库连接串的规则库。
*   **动态扩展**：支持通过 UI 快速添加自定义正则规则，即刻生效。

---

## ⚡ 性能基准 (Performance)

*测试环境: Windows 11 / i7-12700K / NVMe SSD*

| 数据规模 | 传统工具 | **SafeMask** | 吞吐量 (Throughput) |
| :--- | :--- | :--- | :--- |
| **100 MB (100万行)** | 20s+ | **0.4s** | **~250 MB/s** |
| **1.2 GB (500万行)** | 无法处理/崩溃 | **4.1s** | **~300 MB/s** |
| **2.3 GB (1000万行)** | - | **8.1s** | **~340 MB/s** |

---

## 🛠️ 技术架构

SafeMask 追求极致的轻量化与安全性：

*   **Core**: 使用 **Rust 2024 Edition**，集成 **Aho-Corasick** 自动机进行固定词过滤，**Regex::bytes** 处理复杂模式。
*   **Runtime**: 基于 **Tauri v2**，相比 Electron 内存占用降低约 90%。
*   **Frontend**: **Vue 3 + Pinia + Tailwind CSS** 提供极致流畅的工业感交互界面。
*   **Security**: **100% 离线运行**，无任何联网权限，规则与数据全链路本地化。

---

## 🚀 快速开始

### 方式 A：直接使用（推荐）
从 [Releases](https://github.com/AiToByte/SafeMask/releases) 下载对应平台的安装包：
*   **Windows**: `.exe` (支持 zh-CN)
*   **macOS**: `.dmg` (支持 Apple Silicon & Intel)
*   **Linux**: `.deb` / `.AppImage`

### 方式 B：从源码编译
确保环境已安装 Rust 和 Node.js。

```bash
# 克隆仓库
git clone https://github.com/AiToByte/SafeMask.git
cd SafeMask

# 安装前端依赖
npm install

# 启动开发环境
npm run tauri dev

# 构建正式版本
npm run tauri build
```

---

## ⚙️ 规则定制示例

SafeMask 使用 YAML 定义脱敏规则，您可以轻松扩展：

```yaml
group: "PRIVATE_PROJECT"
rules:
  - name: "Internal_Project_ID"
    pattern: '\bPROJ-[a-z0-9]{8}\b'
    mask: "<PROJECT_ID>"
    priority: 10
    enabled: true
```

---

## 🛡️ 安全承诺 (Privacy First)

SafeMask 的生命线是“隐私”。
1.  **零联网**：应用源代码中不包含任何网络请求库。
2.  **零上传**：所有脱敏计算均在您的 CPU 上完成。
3.  **零日志**：我们不记录您的原始敏感信息。

---

## 🤝 贡献
我们非常欢迎任何形式的贡献！
*   提交新的脱敏正则规则。
*   改进前端 UI 交互细节。
*   完善不同语言的文档。

---

## 📄 开源协议
本项目采用 [MIT License](LICENSE)。

---

<div align="center">
  <p><b>SafeMask</b> - 让每一行数据都能安全地拥抱 AI。</p>
  <p>由 <b>XiaoSheng</b> 倾力打造</p>
</div>
