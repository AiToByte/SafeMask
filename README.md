# SafeMask (v2.0.0)

**"Keep the Truth in the Physical Universe, Exchange Safety in the Digital Universe."**

SafeMask is an **industrial-grade local privacy masking engine** built for the AI era. Powered by Rust 2024 and Tauri v2, it ensures your sensitive data never leaves your machine. Through its innovative "Shadow Mode" and parallel computing architecture, SafeMask achieves a perfect balance between ironclad security and frictionless productivity.

[**Download Latest**](https://github.com/AiToByte/SafeMask/releases) | [**简体中文文档**](README_CN.md)

[![Rust](https://img.shields.io/badge/language-Rust_2024-orange.svg?style=for-the-badge&logo=rust)](https://www.rust-lang.org/)
[![Tauri](https://img.shields.io/badge/framework-Tauri_v2-blue.svg?style=for-the-badge)](https://v2.tauri.app/)
[![Performance](https://img.shields.io/badge/Throughput-300MB%2Fs+-brightgreen.svg?style=for-the-badge)](#-hardcore-tech-high-performance-kernel)
[![Security](https://img.shields.io/badge/Security-100%25_Offline-emerald.svg?style=for-the-badge)](#-privacy-commitment)

---

## What's New in v2.0.0

- **AI NER Entity Recognition**: ONNX Runtime-powered neural entity detection (names, emails, phones, secrets, etc.) alongside traditional regex/dictionary matching
- **Record Persistence**: All masking operations are recorded as structured Markdown files with YAML front matter for AI training analysis
- **Immediate-Save Settings**: Every toggle persists to the Rust backend on click — no Save button needed
- **Configurable Magic Paste Shortcut**: Customize the paste trigger shortcut via the Settings page
- **React 19 Frontend**: Complete rewrite from Vue 3 to React 19 + Zustand, with lazy-loaded components and CSS transitions (no framer-motion)
- **Startup Performance**: Bootstrap split loads critical data first, deferring non-essential IPC calls

---

## Core Innovation: The Dual-Universe Model

SafeMask revolutionizes the traditional "all-or-nothing" interceptor logic by introducing a quantum-state masking experience:

### 1. Shadow Universe (Shadow Mode) — *The Default Elegance*
- **The Phenomenon**: You press `Ctrl+C` to copy. The clipboard still holds your **original plain text**. Local debugging and development continue as usual; SafeMask remains invisible.
- **The Collapse**: When you are ready to send content to ChatGPT/Claude, press your Magic Paste shortcut (default `Ctrl+Shift+V`). SafeMask instantly executes a lightning-fast sequence (~150ms): **Backup → Inject Masked Text → Simulate Paste → Instant Restore**.
- **The Value**: The AI receives a safe `<API_KEY>`, while your physical clipboard "heals" back to the original text the moment the paste is complete.

### 2. Sentry Universe (Sentry Mode) — *Absolute Defense*
- **Logic**: System-level forceful interception. Any sensitive data hitting the clipboard is "bleached" into masked text within milliseconds.
- **Use Case**: Remote meetings/screen sharing, high-security office environments, or working in public spaces.

---

## Hardcore Tech: High-Performance Kernel

### 1. Zero-Copy Mmap Concurrent Pipeline
For giant log files (GB+), SafeMask abandons traditional memory-loading schemes in favor of:
- **Memory Mapping (Mmap)**: Directly maps disk files into the process virtual address space for zero-copy reads.
- **Three-Stage Ordered Pipeline**:
  - **Splitter**: Intelligently carves files into 8MB macro-chunks, locating the nearest newline to ensure line integrity.
  - **Compute (Rayon)**: Multi-core CPU parallel masking, squeezing every bit of power from your hardware.
  - **Reassembly (Ordered Writer)**: Uses a `BTreeMap` buffer and index sequencing to ensure the output file's line order is 100% identical to the input.
- **Throughput**: Real-world benchmarks on NVMe SSDs exceed **340MB/s**, processing a 2GB log file in just 8 seconds.

### 2. Hybrid Matching Engine
- **Aho-Corasick Automaton**: For tens of thousands of literal rules (e.g., project names, employee IDs), it provides O(n) time complexity constant-speed matching.
- **Byte-Level Regex**: Operates directly on `[u8]` byte streams, skipping expensive UTF-8 validation and boosting performance by ~30%.
- **AI NER (ONNX Runtime)**: Neural entity recognition for names, emails, phone numbers, addresses, secrets, and more — catches what fixed rules miss.
- **COW (Copy-On-Write) Optimization**: If no privacy data is found in a line, the engine returns a reference only, incurring **zero memory allocation**.

### 3. Millisecond "Time Backtracking" Algorithm
- **Atomic Lock Control**: Uses Rust's `AtomicBool` to synchronize the listener and executor, completely avoiding "Recursive Masking" deadlocks during simulated pastes.
- **Injection Latency Compensation**: Supports precise 50ms-800ms latency adjustments to ensure accurate injection even in high-load applications.

### 4. AI Training Record Writer
- All masking operations persist as structured Markdown files with YAML front matter (original code, masked code, entity table, stats).
- Auto-rotating files: 150 records per file, daily sequencing (e.g., `2026/2026-07-16-001.md`).
- Toggle on/off from Settings; output directory displayed in the UI.

---

## Rule Sandbox (Rule Lab)

SafeMask is not just a masker; it's a professional Regex debugging terminal:
- **Real-time Simulation**: As you write a regular expression, the sandbox below shows the masking result instantly.
- **Error Backtracking**: If the Regex syntax is invalid (e.g., unclosed parentheses), the sandbox captures the low-level engine error and highlights it.
- **System Lock Mechanism**: Built-in rules are physically protected. Users are encouraged to use the "Save As" logic to build custom private libraries based on industrial-standard templates.

---

## Industrial Design Aesthetics

We believe productivity tools should be as elegant as precision instruments:
- **Amber Ivory Theme**: A deep, warm color palette combined with asymmetric white space significantly reduces eye strain during long sessions.
- **Mechanical Audio System**: Real-time synthesis via Web Audio API. Opening, closing, recording, and errors each have unique physical feedback sounds.
- **Precision Indicators**: A dynamic breathing light in the top-right corner displays the "Universe Mode" status in real-time.

---

## Shortcut Guide

| Shortcut | Action | Semantics |
| :--- | :--- | :--- |
| `Ctrl+Shift+V` (configurable) | **Magic Paste** | Injects the masked copy from the Shadow Universe into the focused window. |
| `Alt + M` | **Universe Switch** | Instantly toggles between "Silent Monitoring" and "Active Interception." |
| Pin Button | **Always on Top** | Pins the console to the front of all apps for real-time monitoring. |

---

## Privacy Commitment

- **100% Offline**: No network permissions requested in configuration; the codebase contains zero HTTP request libraries.
- **Zero Telemetry**: We do not collect usage habits, rules, or masking frequency. Data sovereignty belongs entirely to you.
- **Audit Transparency**: All masking history (audit records) can be physically destroyed with one click, leaving no trace on the disk.

---

## Technical Specifications

- **Kernel**: Rust 2024 (Edition)
- **Frontend**: React 19 + Zustand + Vite 6
- **Communication**: Tauri v2 IPC (Binary stream)
- **Styling**: Tailwind CSS v3 + PostCSS
- **AI Engine**: ONNX Runtime (ort crate) + HuggingFace tokenizers
- **Memory Allocator**: mimalloc
- **Thread Pool**: Rayon (limited to 2 threads)
- **Memory Footprint**: ~40MB at idle

---

## Quick Start

### Get the App
Go to [Releases](https://github.com/AiToByte/SafeMask/releases) to download:
- **Windows**: `.msi` or `.exe` (NSIS Installer)
- **macOS**: Universal Binary `.dmg` (Supports M1/M2/M3/Intel)

### Development & Build
```bash
# 1. Clone
git clone https://github.com/AiToByte/SafeMask.git

# 2. Install Dependencies
npm install

# 3. Start the Development Universe
npx tauri dev

# 4. Build Release Version
npx tauri build
```

---

### Star History
If you find SafeMask useful, please consider starring the repository!

[![Star History Chart](https://api.star-history.com/svg?repos=AiToByte/SafeMask&type=date&legend=top-left)](https://www.star-history.com/#AiToByte/SafeMask&type=date&legend=top-left)

---

<div align="center">
  <p><b>SafeMask</b> - Empowering every line of data to safely embrace AI.</p>
  <p>Developed with ❤️ by <b>XiaoSheng</b></p>
</div>
