# 🛡️ SafeMask (v2.0.x)

<div align="center">
  <p align="center">
    <b>"Keep the Truth in the Physical Universe, Exchange Safety in the Digital Universe."</b>
  </p>
</div>

<p align="center">
  <a href="https://www.rust-lang.org/"><img src="https://img.shields.io/badge/Language-Rust_2024-orange.svg?style=for-the-badge&logo=rust" alt="Rust 2024"></a>
  <a href="https://v2.tauri.app/"><img src="https://img.shields.io/badge/Framework-Tauri_v2-blue.svg?style=for-the-badge" alt="Tauri v2"></a>
  <a href="https://react.dev/"><img src="https://img.shields.io/badge/Frontend-React_19-61dafb.svg?style=for-the-badge&logo=react" alt="React 19"></a>
  <br>
  <a href="README_CN.md"><img src="https://img.shields.io/badge/Documentation-简体中文-amber.svg?style=for-the-badge" alt="简体中文"></a>
  <a href="https://github.com/AiToByte/SafeMask/releases"><img src="https://img.shields.io/badge/Download-Latest_v2.0.2-emerald.svg?style=for-the-badge" alt="Latest version"></a>
  <a href="#-benchmarks"><img src="https://img.shields.io/badge/Throughput-340MB%2Fs+-brightgreen.svg?style=for-the-badge" alt="Throughput"></a>
  <a href="#-privacy--security-commitment"><img src="https://img.shields.io/badge/Security-100%25_Offline-teal.svg?style=for-the-badge" alt="100% Offline"></a>
  <a href="LICENSE"><img src="https://img.shields.io/badge/License-MIT-gray.svg?style=for-the-badge" alt="MIT License"></a>
</p>

---

## 🌌 What is SafeMask?

**SafeMask** is an **industrial-grade local privacy masking engine and control console** built for the AI era [docs/使用手册.md].

When interacting with Large Language Models (such as ChatGPT, Claude, or DeepSeek), we frequently copy-paste codebases, system logs, meeting transcripts, and configurations [docs/使用手册.md]. However, sensitive identifiers like `API_KEY`s, personal `phone numbers`, internal `IP addresses`, and physical `addresses` are often leaked in the process.

SafeMask ensures your sensitive data **never leaves your machine** [docs/使用手册.md]. Operating completely **offline** [docs/使用手册.md], it implements a **hybrid matching matrix** and a **Dual-Universe clipboard workflow** to protect your local privacy assets without breaking your productivity [doc/架构详解/系统集成、剪切板监控与桌面安全.md].

> 🌐 **Bilingual Documentation**: For the Simplified Chinese documentation, please refer to [README_CN.md](README_CN.md) [README.md].

---

## ✨ Key Features

### 1. The Dual-Universe Clipboard Model
SafeMask moves away from rigid "all-or-nothing" clipboard blocking, implementing a quantum-state-inspired approach to clipboard handling [README_CN.md]:

*   **Shadow Universe (Shadow Mode — *Default & Non-Intrusive*)**  
    *   *Physical Universe (Local)*: Standard `Ctrl+C` copying preserves your **original plaintext** [README_CN.md]. Local compilers, configurations, and internal runs execute without any interference [README_CN.md].
    *   *Digital Universe (AI)*: When pasting to an AI prompt, press **`Alt+V` (Magic Paste)** [README_CN.md]. SafeMask instantly triggers a millisecond-level transaction: *“Backup plaintext $\rightarrow$ Inject masked text $\rightarrow$ Send native paste $\rightarrow$ Restore original text”* [README_CN.md, .worktrees/shortcut-magic-paste-stability/docs/superpowers/specs/2026-04-09-shortcut-magic-paste-stability-design.md]. The AI receives a sanitized `<OPENAI_KEY>` [src-tauri/rules/auth/ai/keys.yaml], while your physical clipboard automatically heals back to the original plaintext [README_CN.md].
*   **Sentry Universe (Sentry Mode — *Active Defense*)**  
    *   System-level active clipboard bleaching [README_CN.md]. Any sensitive information hitting the clipboard is sanitized within milliseconds [README_CN.md]. This prevents accidental leaks during screen sharing, remote meetings, or working in public spaces.

### 2. High-Performance Zero-Copy Mmap Pipeline
For gigabyte-scale log files, SafeMask implements an asynchronous, thread-safe I/O processing pipeline [doc/架构详解/高并发保序流水线与IO优化.md]:
*   **Zero-Copy Memory Mapping (Mmap)**: Maps disk files directly into the virtual memory address space via `memmap2`, eliminating expensive user-to-kernel memory copies and traditional read system calls [doc/架构详解/高并发保序流水线与IO优化.md].
*   **Three-Stage Ordered Pipeline**:
    *   *Smart Splitter*: Chunking files into 8MB blocks while dynamically locating the nearest `\n` to guarantee structural line integrity [doc/架构详解/高并发保序流水线与IO优化.md, src-tauri/src/infra/fs/processor.rs].
    *   *Work-Stealing Compute*: Squeezing multi-core CPU power using `Rayon` task-stealing algorithms [doc/架构详解/高并发保序流水线与IO优化.md, README_CN.md].
    *   *Ordered Writing*: Utilizing a `BTreeMap` and an atomic counter to ensure **the output line order matches the input file with 100% precision** [doc/架构详解/高并发保序流水线与IO优化.md, README_CN.md].
*   **Backpressure Flow-Control**: Restricts in-flight memory chunks to 32 (approx. 256MB), guaranteeing that **RAM usage remains constant at ~300MB even when processing 100GB+ files** [doc/架构详解/高并发保序流水线与IO优化.md].

### 3. Pluggable Hybrid Matching Matrix
The detection engine coordinates high-performance deterministic rules with probabilistic ML models:
*   **Aho-Corasick Automaton**: High-speed matching of static patterns, literal keywords, and project codenames in $O(n)$ time complexity [doc/架构详解/核心脱敏引擎与冲突算法.md, README_CN.md].
*   **Byte-Level Regex**: Operates directly on raw `[u8]` byte streams, bypassing Rust's default UTF-8 validation overhead for a ~30% performance boost [doc/架构详解/核心脱敏引擎与冲突算法.md, README_CN.md].
*   **ONNX Local NER Engine**: Runs token classification locally using `ort` (ONNX Runtime) to extract unstructured names, organizations, and addresses via a q4 quantized `openai/privacy-filter` model [docs/使用手册.md, docs/使用手册.md].
*   **Sub-Span Carving Conflict Resolution**: When an AI-detected address overlaps with a regex-matched IP, the engine carves the overlapping ranges geometrically, preserving both high-priority rules and wide semantic contexts rather than dropping entire spans.

### 4. Asynchronous PII Dataset Record Writer
*   Implements a non-blocking `RecordWriter` [docs/record-writer.md]. When sensitive data is encountered, it asynchronously appends the original-to-masked mapping to your local storage in Markdown format with YAML front matter [docs/record-writer.md].
*   Supports automatic 150-record file splitting and yearly directory partitioning [docs/record-writer.md]. These offline, sanitized, and structured datasets can be directly utilized as clean corpora for training and evaluating private LLMs [docs/record-writer.md].

---

## 📁 System Architecture

SafeMask is engineered with a **6-Layer Decoupled Architecture**, isolating presentation, state machine orchestration, conflict resolution, detection registry, and cross-platform infrastructure:

```
┌─────────────────────────────────────────────────────────────────┐
│              Layer 6: Presentation Layer (React 19)              │
│  Staged Bootstrap · Lazy-Loading · System Fonts · Web Audio SFX  │
└────────────────────────────┬────────────────────────────────────┘
                             │ IPC (Tauri Commands)
┌────────────────────────────▼────────────────────────────────────┐
│              Layer 5: Orchestration Layer                       │
│  Dual-Universe (Shadow vs. Sentry) State Machine                │
└────────────────────────────┬────────────────────────────────────┘
                             │
┌────────────────────────────▼────────────────────────────────────┐
│              Layer 4: Masking Strategy Layer                    │
│  Replace · Semantic Partial Mask · Hash · Redact · Token · Temp │
└────────────────────────────┬────────────────────────────────────┘
                             │
┌────────────────────────────▼────────────────────────────────────┐
│              Layer 3: Conflict Resolution Layer                 │
│  Sub-Span Carving · Container Swallowing · Confidence Filtering │
└────────────────────────────┬────────────────────────────────────┘
                             │
┌────────────────────────────▼────────────────────────────────────┐
│              Layer 2: Pluggable Detection Layer                 │
│  AC Dictionary · Byte Regex · ONNX AI NER · Checksum Validator  │
└────────────────────────────┬────────────────────────────────────┘
                             │
┌────────────────────────────▼────────────────────────────────────┐
│              Layer 1: Infrastructure Layer                      │
│  Zero-Copy Mmap · Polling Clipboard · Async RecordWriter · HMAC │
└─────────────────────────────────────────────────────────────────┘
```

---

## 📊 Benchmarks

SafeMask's low-level optimizations provide significant throughput advantages over traditional Python, Node.js, or Electron-based tools:

*   **Large Log File Throughput**:
    *   **SafeMask (Rust 2024 Core)**: **340 MB/s** (Processes a 2.3 GB production log file in **8.1 seconds**) [README_CN.md].
    *   **Traditional Python (Line-by-line)**: **18.4 MB/s** (Takes approx. 2 mins 15 seconds).
*   **Memory Footprint**:
    *   **Standby/Idle**: Only **40 MB** (compared to typical Electron footprint of 300MB+) [README_CN.md].
    *   **During 50 GB File Runs**: Strictly limited to **under 300 MB** due to Mmap and backpressure queue bounds [doc/架构详解/高并发保序流水线与IO优化.md].
*   **Magic Paste Latency**:
    *   Total key simulation, swap sequence, and restore transaction executes in approx. **150ms** (fully configurable).

---

## 🛠️ Quick Start

### 1. Download & Run (Recommended)
Go to the [SafeMask Releases](https://github.com/AiToByte/SafeMask/releases) page and download the executable built for your platform [README_CN.md]:
*   **Windows**: Standard installers (`.msi`) and zero-installation portable archives (`.zip`) are both supported [README_CN.md, .worktrees/shortcut-magic-paste-stability/.github/workflows/release.yml].
*   **macOS**: Universal binaries (`.dmg`) compiled natively for both Apple Silicon (M1/M2/M3/M4) and Intel architectures [README_CN.md, .worktrees/shortcut-magic-paste-stability/.github/workflows/release.yml].
*   **Linux**: Universal AppImage packages (`.AppImage`) and Debian installers (`.deb`) [.worktrees/shortcut-magic-paste-stability/.github/workflows/release.yml].

### 2. Manual Source Compilation
If you prefer compiling the workspace from scratch:

```bash
# Clone the repository
git clone https://github.com/AiToByte/SafeMask.git
cd SafeMask

# Install frontend node modules
npm install

# Run the full-stack development environment (Vite 6 + Tauri Window)
npm run tauri dev

# Build the release bundle for your current platform
npm run tauri build
```

---

## 🧠 Local AI Model Configuration

SafeMask utilizes a quantized token-classification model for Named Entity Recognition [docs/使用手册.md]. To activate local AI processing [docs/使用手册.md]:

1. Ensure the model directory exists [docs/使用手册.md]:
   `src-tauri/models/privacy-filter/`
2. Download and place the following asset bundle into the directory (you can also use the one-click download option directly from the Settings page) [src-tauri/src/ai_downloader.rs]:
   ```text
   privacy-filter/
   ├── model_q4.onnx           # ONNX model structure (160 KB)
   ├── model_q4.onnx_data      # Quantized 4-bit weights file (874 MB)
   ├── tokenizer.json          # Fast HF Tokenizer configuration (27 MB)
   └── config.json             # ID-to-Label mapping metadata
   ```
3. Open the app, navigate to the Settings page, and verify the **AI Engine** status indicator glows green, indicating the local model has successfully loaded into memory.

---

> 模型下载地址
[privacy-filter Model Quark download link](https://pan.quark.cn/s/51647902f801?pwd=HQ1Y)
[privacy-filter Baidu Netdisk download link for the model](https://pan.baidu.com/s/1mDBr0mdo2r-guC4LshF87w?pwd=ba7b)

---

## 🔒 Privacy & Security Commitment

*   **100% Offline**: All masking, rules, and AI inferences execute entirely in local sandbox environments with zero network calls [docs/使用手册.md]. The codebase contains no telemetry or remote reporting.
*   **Audit Sanitization**: All historical logs and audit records can be permanently deleted and zeroed out from the disk with a single click [README_CN.md, src-tauri/src/api/system.rs].
*   **Least Privilege**: The application operates under a strict, minimal permission model [doc/架构详解/系统集成、剪切板监控与桌面安全.md], requesting only clipboard and local file I/O permissions.

---

## 📜 License

This project is open-source and distributed under the terms of the **MIT License**.

---

<div align="center">
  <p><b>SafeMask</b> — Empowering every line of data to safely and fearlessly embrace AI.</p>
  <p>Developed with ❤️ by <b>XiaoSheng</b></p>
</div>
