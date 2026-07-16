# 🛡️ SafeMask (v2.0.0)

<div align="center">
  <p align="center">
    <b>“在物理宇宙保留真实，在数字宇宙交换安全。”</b>
  </p>
  <p align="center">
    <strong>Keep the Truth in the Physical Universe, Exchange Safety in the Digital Universe.</strong>
  </p>
</div>

<p align="center">
  <a href="https://www.rust-lang.org/"><img src="https://img.shields.io/badge/Language-Rust_2024-orange.svg?style=for-the-badge&logo=rust" alt="Rust 2024"></a>
  <a href="https://v2.tauri.app/"><img src="https://img.shields.io/badge/Framework-Tauri_v2-blue.svg?style=for-the-badge" alt="Tauri v2"></a>
  <a href="https://react.dev/"><img src="https://img.shields.io/badge/Frontend-React_19-61dafb.svg?style=for-the-badge&logo=react" alt="React 19"></a>
  <br>
  <a href="https://github.com/AiToByte/SafeMask/releases"><img src="https://img.shields.io/badge/Download-Latest_v2.0.0-emerald.svg?style=for-the-badge" alt="Latest version"></a>
  <a href="#-性能指标-benchmarks"><img src="https://img.shields.io/badge/Throughput-340MB%2Fs+-brightgreen.svg?style=for-the-badge" alt="Throughput"></a>
  <a href="#-隐私与安全承诺"><img src="https://img.shields.io/badge/Security-100%25_Offline-teal.svg?style=for-the-badge" alt="100% Offline"></a>
  <a href="LICENSE"><img src="https://img.shields.io/badge/License-MIT-gray.svg?style=for-the-badge" alt="MIT License"></a>
</p>

---

## 🌌 什么是 SafeMask？

**SafeMask** 是一款专为 AI 时代打造的**工业级本地隐私脱敏控制台** [docs/使用手册.md]。

当今时代，我们高频地将代码、日志、会议纪要等数据贴给 ChatGPT, Claude 或 DeepSeek [docs/使用手册.md]。然而，敏感的 `API_KEY`、个人 `手机号`、公司 `内网 IP` 以及 `商业住址` 也在这个过程中悄然泄露。

SafeMask 在确保数据 **100% 离线、不出域** 的前提下 [docs/使用手册.md]，通过**混合识别矩阵**与**宇宙双轨制**交互设计 [doc/架构详解/系统集成、剪切板监控与桌面安全.md]，彻底在物理层面保护您的隐私资产。

---

## ✨ 核心亮点特性

### 1. 宇宙双轨制交互 (The Dual-Universe Model)

SafeMask 抛弃了传统工具“非黑即白”的直接替换，创造性地设计了两个对称运行的平行宇宙 [README_CN.md]：

*   **影子宇宙 (Shadow Mode — 默认雅致)**  
    *   **物理宇宙 (本地)**：您照常按下 `Ctrl+C` 复制，剪贴板里依然是**原始明文**。本地开发、配置、调用一切正常 [README_CN.md]。
    *   **数字宇宙 (AI)**：当您准备贴入 AI 对话框时，按下 **`Alt+V` (魔术粘贴)** [README_CN.md]。SafeMask 在后台毫秒级执行：*“备份原文 $\rightarrow$ 注入脱敏文 $\rightarrow$ 模拟粘贴 $\rightarrow$ 瞬时恢复原文”* 事务 [README_CN.md, .worktrees/shortcut-magic-paste-stability/docs/superpowers/specs/2026-04-09-shortcut-magic-paste-stability-design.md]。AI 拿到了安全的 `<OPENAI_KEY>` [src-tauri/rules/auth/ai/keys.yaml]，而您的物理剪贴板完好无损 [README_CN.md]。
*   **哨兵宇宙 (Sentry Mode — 绝对防御)**  
    *   系统级强力拦截 [README_CN.md]。任何敏感信息一旦触碰剪贴板，在毫秒级内即被强制“洗白”[README_CN.md]，物理防范远程会议演示、公共场所投屏时的不慎泄露。

### 2. 多核保序 Mmap 并行文件处理管线 (Mmap Pipeline)
对于 GB 级别的巨型日志文件，SafeMask 实现了一套高效的异步 I/O 并行管线 [doc/架构详解/高并发保序流水线与IO优化.md]：
*   **零拷贝内存映射 (Mmap)**：利用 `memmap2` 将磁盘文件直投至进程虚拟地址空间，规避传统的系统调用与昂贵的用户态拷贝 [doc/架构详解/高并发保序流水线与IO优化.md]。
*   **三阶段保序流水线**：
    *   *智能宏分块 (Splitter)*：按 8MB 智能切割数据块，自适应寻找最近的 `\n` 换行符，保障每一块的完整性 [doc/架构详解/高并发保序流水线与IO优化.md, src-tauri/src/infra/fs/processor.rs]。
    *   *任务窃取计算 (Rayon)*：压榨多核 CPU 并行脱敏 [doc/架构详解/高并发保序流水线与IO优化.md, README_CN.md]。
    *   *保序重组 (Ordered Writer)*：通过 `BTreeMap` 缓冲与原子序号自增控制，确保**脱敏文件的行顺序与原文 100% 绝对一致** [doc/架构详解/高并发保序流水线与IO优化.md, README_CN.md]。
*   **背压控制 (Backpressure)**：限制内存缓冲区最大积压 32 个块（约 256MB），保证即使处理 100GB 文件，**内存占用也始终稳定在 300MB 左右** [doc/架构详解/高并发保序流水线与IO优化.md]。

### 3. 可插拔混合识别矩阵 (Hybrid Matching Engine)
引擎设计遵循“确定性”与“概率性”的分层协同：
*   **Aho-Corasick 自动机**：针对数万条固定的敏感词、姓名、项目代号，提供 $O(n)$ 时间复杂度的恒速扫描 [doc/架构详解/核心脱敏引擎与冲突算法.md, README_CN.md]。
*   **字节级正则 (Byte-regex)**：直接在 `[u8]` 原生字节流上进行匹配，跳过昂贵的 UTF-8 字符校验，速度提升约 30% [doc/架构详解/核心脱敏引擎与冲突算法.md, README_CN.md]。
*   **本地 AI NER 引擎**：基于 `ort` (ONNX Runtime) 本地零网络开销加载 `openai/privacy-filter` 量化模型，智能提取自然语言自由段落中的人名、机构、地名 [docs/使用手册.md, docs/使用手册.md]。
*   **子区间“雕刻”冲突解决 (Sub-span Carving)**：当 AI 识别出的长住址与正则匹配出的短 IP 重叠时，采用几何级区间裁剪，保留高优规则的同时，将大标签精准细切，避免整块丢弃或粗暴吞噬。

### 4. 异步 AI 训练记录持久化 (Record Writer)
*   引入非阻塞式 `RecordWriter` 架构 [docs/record-writer.md]。当捕获到敏感信息时，在后台以 Markdown + YAML 头部元数据格式追加到本地磁盘。
*   支持单文件 150 条自动滚动分割、按日期年份分区 [docs/record-writer.md]。这些无污染、高质量的脱敏样本对（Pairs）可以直接作为企业私有化大模型的微调（Fine-tuning）与评测数据集 [docs/record-writer.md]。

---

## 📁 架构全景图

SafeMask 采用极佳设计实践的**六层分离架构**，确保了业务逻辑、识别决策、脱敏策略以及跨平台系统交互间的完美解耦：

```
┌─────────────────────────────────────────────────────────────────┐
│  Layer 6: 用户界面层 (React 19 + Zustand)                        │
│  首屏免动画 · 延迟懒加载 · 极简系统字体栈 · 机械物理反馈音效系统  │
└────────────────────────────┬────────────────────────────────────┘
                             │ Tauri IPC (invoke)
┌────────────────────────────▼────────────────────────────────────┐
│  Layer 5: 业务编排层 (Orchestrator)                             │
│  影子宇宙 (Shadow) / 哨兵宇宙 (Sentry) 状态机双轨控制           │
└────────────────────────────┬────────────────────────────────────┘
                             │
┌────────────────────────────▼────────────────────────────────────┐
│  Layer 4: 脱敏策略层 (Masking Strategy)                         │
│  替换 (Replace) · 智能部分遮盖 · 哈希 · 完全删除 · 动态 Token   │
└────────────────────────────┬────────────────────────────────────┘
                             │
┌────────────────────────────▼────────────────────────────────────┐
│  Layer 3: 冲突解决层 (Conflict Resolver)                         │
│  子区间几何雕刻 (Carving) · 容器型实体合并吞噬 · 置信度阈值过滤  │
└────────────────────────────┬────────────────────────────────────┘
                             │
┌────────────────────────────▼────────────────────────────────────┐
│  Layer 2: 识别引擎层 (Recognizer Registry)                       │
│  Aho-C 字典 · 字节级正则 · ONNX AI NER · Checksum 校验位        │
└────────────────────────────┬────────────────────────────────────┘
                             │
┌────────────────────────────▼────────────────────────────────────┐
│  Layer 1: 基础设施层 (Infrastructure)                           │
│  零拷贝 Mmap 管道 · 双向剪贴板监听 · 密钥派生 · 非阻塞 RecordWriter│
└─────────────────────────────────────────────────────────────────┘
```

---

## 📊 性能指标 (Benchmarks)

经过深度多维优化，SafeMask 与传统基于 Python、Node.js 或 Electron 架构的脱敏工具有着降维级别的性能优势：

*   **巨型日志文件处理速度**：
    *   **SafeMask (Rust 2024 内核)**: **340 MB/s** (处理一个 2.3 GB 的生产日志文件仅需 **8.1秒**) [README_CN.md]。
    *   **传统 Python 逐行处理**: **18.4 MB/s** (处理相同文件耗时约 2 分 15 秒)。
*   **内存开销控制**：
    *   **静默待机**: 仅 **40 MB** (传统的 Electron 工具通常起步 300MB+) [README_CN.md]。
    *   **处理 50 GB 文件时**: 稳定限制在 **300 MB 以内** (由于 Mmap + 32-Chunk 限制背压机制) [doc/架构详解/高并发保序流水线与IO优化.md]。
*   **魔术粘贴反应延迟**：
    *   从按下快捷键 `Alt+V` 到目标输入框呈现安全的脱敏文本，整体事务执行仅需约 **150ms**（支持在配置中精细化调优延迟补偿）。

---

## 🛠️ 快速开始

### 1. 下载即用 (推荐)
前往 [SafeMask Releases](https://github.com/AiToByte/SafeMask/releases) 下载适合您平台的双击可执行安装包 [README_CN.md]：
*   **Windows**: 支持标准安装版（`.msi`）以及绿色免安装版（`.zip`） [README_CN.md]。
*   **macOS**: 提供通用二进制磁盘映像（`.dmg`），完美原生支持 Apple Silicon（M1/M2/M3/M4）与 Intel 芯片 [README_CN.md, .worktrees/shortcut-magic-paste-stability/.github/workflows/release.yml]。
*   **Linux**: 提供通用二进制（`.AppImage`）与 Debian 安装包（`.deb`） [.worktrees/shortcut-magic-paste-stability/.github/workflows/release.yml]。

### 2. 源码本地构建
若您希望从零自行编译：

```bash
# 克隆仓库
git clone https://github.com/AiToByte/SafeMask.git
cd SafeMask

# 安装前端运行环境依赖
npm install

# 启动全栈本地开发环境（Vite 6 自动热更新 + Tauri 窗口）
npm run tauri dev

# 打包生产环境高优化版本
npm run tauri build
```

---

## 🧠 AI 离线模型配置指南

SafeMask 自带轻量、高灵敏度的自然语言 NER（命名实体识别）模型 [docs/使用手册.md]。要激活 AI 提取功能，请按照以下步骤进行本地配置 [docs/使用手册.md]：

1. 确认已创建模型专属存储目录 [docs/使用手册.md]：
   `src-tauri/models/privacy-filter/`
2. 将模型资产放入该目录下（在设置页面支持一键自动下载并配置） [src-tauri/src/ai_downloader.rs]：
   ```text
   privacy-filter/
   ├── model_q4.onnx           # ONNX 模型架构 (160 KB)
   ├── model_q4.onnx_data      # 量化后的 4-bit 权重文件 (874 MB)
   ├── tokenizer.json          # 专属高效分词器 (27 MB)
   └── config.json             # 标签映射元配置文件
   ```
3. 启动 SafeMask，进入设置页面，观察 **AI Engine** 的状态指示灯变为绿色，代表您的本地大模型已完全成功载入内存并就绪。

> 模型下载地址
[privacy-filter模型夸克下载地址](https://pan.quark.cn/s/51647902f801)
提取码：HQ1Y

---

[privacy-filter模型百度网盘下载地址](https://pan.baidu.com/s/1mDBr0mdo2r-guC4LshF87w?pwd=ba7b)

---

## 🔒 隐私与安全承诺

*   **100% 本地化**：应用完全离线工作，无网络权限权限声明 [docs/使用手册.md]，无任何外发遥测、上报或监控，数据主权完全归您。
*   **审计销毁权**：所有的历史脱敏记录可一键从物理层彻底擦除、覆写 00，不留任何操作系统痕迹 [README_CN.md, src-tauri/src/api/system.rs]。
*   **最小化权限**：严格限制只获取剪贴板、本地文件读写（用于指定文件处理）与系统托盘注册权限 [doc/架构详解/系统集成、剪切板监控与桌面安全.md]。

---

## 📜 许可证

本项目基于 **MIT 许可证** 开源并分发。

---

<div align="center">
  <p><b>SafeMask</b> — 让每一行数据，都能安全、无畏地拥抱 AI。</p>
  <p>Developed with ❤️ by <b>XiaoSheng</b></p>
</div>