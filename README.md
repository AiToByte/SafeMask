# 🛡️ SafeMask (v1.1.2) Shocking Arrival

**Enabling every line of data to safely embrace AI.**

[![Rust](https://img.shields.io/badge/language-Rust-orange.svg)](https://www.rust-lang.org/)
[![Tauri](https://img.shields.io/badge/framework-Tauri_v2-blue.svg)](https://v2.tauri.app/)
[![Vue](https://img.shields.io/badge/frontend-Vue_3-green.svg)](https://vuejs.org/)
[![License](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)
[![Performance](https://img.shields.io/badge/throughput-300MB%2Fs+-brightgreen.svg)](#-performance-benchmarks)

[**Download Latest**](https://github.com/AiToByte/SafeMask/releases) | [**简体中文文档**](README_CN.md)

---

## 💡 Why SafeMask?

In the era of LLMs (Large Language Models), pasting logs, code, or documents into ChatGPT/Claude poses significant privacy risks. 

*   **Pain Point 1 (Semantic Loss)**: Traditional `***` masking destroys AI's reasoning. AI sees `***` and cannot distinguish if it was an IP, Email, or API Key.
*   **Pain Point 2 (Performance Bottlenecks)**: Traditional text tools (like Notepad++ plugins) freeze when handling GB-level local logs.
*   **Pain Point 3 (Security Anxiety)**: Cloud-based masking tools are untrustworthy; your data might leak during the process itself.

**SafeMask solves these problems elegantly.**

---

## ✨ Core Features

### 1. 🧠 AI-Friendly Semantic Masking
SafeMask uses **semantic labels** to replace sensitive information.
*   **Input**: `sk-ant-api03-123456...`
*   **SafeMask**: `<CLAUDE_KEY>`
*   **Outcome**: The AI still knows it is a key, allowing it to analyze logic correctly without seeing the actual secret.

### 2. 🎧 Clipboard Privacy Shield
*   **Silent Protection**: When enabled, SafeMask monitors the system clipboard in the background.
*   **Auto-Interception**: If sensitive data (e.g., phone numbers, keys) is detected, it is masked locally, and the safe version is written back to the clipboard.
*   **History Comparison**: View "Original vs. Masked" via the UI to ensure your privacy is perfectly protected.

### 3. 🚀 Industrial-Grade File Processor
*   **Extreme Performance**: Powered by Rust’s `memmap2` and `rayon` parallel engine.
*   **Ultra-Large Support**: Handles 10GB+ log files with ease, maintaining a minimal memory footprint.
*   **Order Preservation**: 100% maintains the original line order, ensuring no disruption to subsequent log analysis.

### 4. 🛠️ Visual Rule Manager
*   **Built-in Rules**: Includes a library covering IPs, Emails, Phone Numbers, IDs, major AI API Keys, and Database connection strings.
*   **Dynamic Expansion**: Add custom Regex rules via the UI and apply them instantly without restarting.

---

## ⚡ Performance Benchmarks

*Environment: Windows 11 / i7-12700K / NVMe SSD*

| Data Scale | Traditional Tools | **SafeMask** | Throughput |
| :--- | :--- | :--- | :--- |
| **100 MB (1M lines)** | 20s+ | **0.4s** | **~250 MB/s** |
| **1.2 GB (5M lines)** | Fail/Crash | **4.1s** | **~300 MB/s** |
| **2.3 GB (10M lines)** | - | **8.1s** | **~340 MB/s** |

---

## 🛠️ Technical Architecture

SafeMask is built for lightness and security:

*   **Core**: Written in **Rust 2024 Edition**, utilizing **Aho-Corasick** for literal filtering and **Regex::bytes** for pattern matching.
*   **Runtime**: Based on **Tauri v2**, reducing memory usage by ~90% compared to Electron.
*   **Frontend**: **Vue 3 + Pinia + Tailwind CSS** provides a smooth, industrial-grade user experience.
*   **Security**: **100% Offline**, no internet permissions required. Rules and data stay strictly on your local machine.

---

## 🚀 Getting Started

### Method A: Direct Download (Recommended)
Download the installer for your platform from [Releases](https://github.com/AiToByte/SafeMask/releases):
*   **Windows**: `.exe` (supports zh-CN/en-US)
*   **macOS**: `.dmg` (Universal: Apple Silicon & Intel)
*   **Linux**: `.deb` / `.AppImage`

### Method B: Build from Source
Ensure you have Rust and Node.js installed.

```bash
# Clone the repository
git clone https://github.com/AiToByte/SafeMask.git
cd SafeMask

# Install frontend dependencies
npm install

# Run in development mode
npm run tauri dev

# Build the production version
npm run tauri build
```

---

## ⚙️ Rule Customization Example

SafeMask defines masking rules using YAML. You can easily extend them:

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

## 🛡️ Privacy Commitment (Zero-Trust)

Privacy is the lifeblood of SafeMask.
1.  **Zero Networking**: The source code contains no network request libraries.
2.  **Zero Upload**: All masking calculations are performed on your local CPU.
3.  **Zero Telemetry**: we do not track or record your original sensitive information.

---

## 🤝 Contributing
We welcome contributions of all forms!
*   Submit new regex rules for common privacy items.
*   Improve UI/UX details in the frontend.
*   Help with translations for different languages.

---

## 📄 License
This project is licensed under the [MIT License](LICENSE).

---

<div align="center">
  <p><b>SafeMask</b> - Empowering every line of data to safely embrace AI.</p>
  <p>Developed with ❤️ by <b>XiaoSheng</b></p>
</div>