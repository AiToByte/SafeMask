# 🛡️ SafeMask (v1.2.2)

**“在物理宇宙保留真实，在数字宇宙交换安全。”**

SafeMask 是一款专为 AI 时代打造的**工业级本地隐私脱敏引擎**。基于 Rust 2024 与 Tauri v2 构建，它在确保隐私数据 100% 不出域的前提下，通过创新的“影子模式”与并行计算架构，实现了安全与生产力的完美平衡。

[![Rust](https://img.shields.io/badge/language-Rust_2024-orange.svg?style=for-the-badge&logo=rust)](https://www.rust-lang.org/)
[![Tauri](https://img.shields.io/badge/framework-Tauri_v2-blue.svg?style=for-the-badge)](https://v2.tauri.app/)
[![Performance](https://img.shields.io/badge/Throughput-300MB%2Fs+-brightgreen.svg?style=for-the-badge)](#-黑科技-高性能内核)
[![Security](https://img.shields.io/badge/Security-100%25_Offline-emerald.svg?style=for-the-badge)](#-隐私承诺)

---

## 🌌 核心创新：宇宙双轨制 (The Dual-Universe Model)

SafeMask 彻底改变了传统拦截器“非白即黑”的逻辑，引入了量子态式的脱敏体验：

### 1. 影子宇宙 (Shadow Mode) — 默认的优雅
*   **现象**：您按下 `Ctrl+C` 复制，剪贴板里依然是**原始明文**。开发调试、本地运行一切照旧，SafeMask 毫无存在感。
*   **坍缩**：当您准备将内容发给 ChatGPT/Claude 时，按下 `Alt+V`。SafeMask 瞬间执行 **“备份 → 注入脱敏文 → 模拟粘贴 → 瞬时还原”** 的闪电序列（150ms）。
*   **价值**：AI 拿到了安全的 `<API_KEY>`，而您的物理剪贴板在粘贴完成的那一刻已自动“愈合”为原文。
![演示Alt + V粘贴](https://github.com/user-attachments/assets/8a9e129a-8542-4e93-8dc0-8128a4e99b0a)

### 2. 哨兵宇宙 (Sentry Mode) — 绝对的防御
*   **逻辑**：系统级强力拦截。任何敏感数据在触碰剪贴板的毫秒内即被洗白。
*   **场景**：远程会议演示、高保密办公环境、公共场所操作。
![剪切板 哨兵宇宙](https://github.com/user-attachments/assets/a665ad4e-62ba-482c-a008-12a2cad0aee4)

---

## 🚀 “黑科技”：高性能内核解析

### 1. 零拷贝 Mmap 高并发流水线
针对 GB 级别的巨型日志文件，SafeMask 抛弃了传统的内存读取方案：
*   **内存映射 (Mmap)**：直接将磁盘文件映射至进程虚拟地址空间，实现零拷贝读取。
*   **三阶段保序流水线**：
    *   **分块 (Splitter)**：将文件智能切割为 8MB 宏分块，寻找最近换行符确保行完整性。
    *   **计算 (Rayon)**：多核 CPU 并行脱敏，压榨每一颗核心的算力。
    *   **重组 (Ordered Writer)**：通过 `BTreeMap` 缓冲和序号索引，确保输出文件的行序与输入 100% 一致。
*   **吞吐量**：在 NVMe SSD 上实测突破 **340MB/s**，处理 2GB 日志仅需 8 秒。

### 2. 混合匹配引擎 (Hybrid Engine)
*   **Aho-Corasick 自动机**：针对数万条固定词规则（如项目名、员工 ID），提供 $O(n)$ 时间复杂度的恒速匹配。
*   **字节级正则 (Byte-regex)**：直接在 `[u8]` 字节流上操作，跳过昂贵的 UTF-8 校验，性能提升约 30%。
*   **COW (Copy-On-Write) 优化**：如果一行文本未发现隐私，引擎仅返回引用，**不产生任何内存分配**。

### 3. 毫秒级“时间回溯”算法
*   **原子锁控制**：使用 Rust `AtomicBool` 协调监听器与执行器，彻底规避模拟粘贴时的“循环脱敏”死锁。
*   **注入延迟补偿**：根据不同应用的响应速度，支持 50ms-800ms 的精密延迟调节，确保在高负载应用中也能准确注入。

---

## 🧪 规则实验室 (Rule Sandbox)

SafeMask 不仅能脱敏，还是一个专业的正则调试终端：
*   **实时仿真**：在编写正则表达式时，下方沙盒会实时显示脱敏效果。
*   **错误回溯**：如果正则语法有误（如括号未闭合），沙盒将精准捕获底层引擎报错并高亮提示。
*   **系统锁机制**：内置规则受物理保护，支持“另存为”逻辑，鼓励用户基于工业标准模板构建自定义私有库。
![规则页面 规则实验室](https://github.com/user-attachments/assets/0e858ed3-eff7-4433-a4e4-9bb829f581d8)
---

## 🎨 工业设计美学

我们认为生产力工具应当像精密仪器一样雅致：
*   **琥珀象牙 (Amber Ivory) 主题**：深邃的暖色调布局，配合非对称留白，极大地缓解了长时间盯着屏幕的眼部疲劳。
*   **机械音效系统**：基于 Web Audio API 实时合成，开启、关闭、录制、错误均有专属的物理反馈音。
*   **精密指示灯**：右上角动态呼吸灯实时展示“宇宙模式”状态。
![首页GIF图](https://github.com/user-attachments/assets/97497bfa-673b-498c-a956-71965e5cb2d8)
---

## ⌨️ 快捷键指南

| 快捷键 | 动作 | 语义 |
| :--- | :--- | :--- |
| `Alt + V` | **魔术粘贴** | 将影子宇宙中的脱敏副本注入当前焦点窗口 |
| `Alt + M` | **宇宙切换** | 在“静默监测”与“主动拦截”模式间瞬时切换 |

---

## 🔒 隐私与合规 (Privacy First)

*   **100% 离线**：配置文件中未开启任何网络权限，代码库不包含任何 HTTP 请求库。
*   **零遥测**：不收集您的任何使用习惯、规则或脱敏频率，数据主权完全归您所有。
*   **审计透明**：所有的脱敏历史（审计记录）均可一键物理销毁，不留任何磁盘痕迹。

---

## 🛠️ 技术规格

*   **内核**: Rust 2024 (Edition)
*   **前端**: Vue 3 + Pinia + Vite 6
*   **通信**: Tauri v2 IPC (Binary stream)
*   **样式**: Tailwind CSS v3 + PostCSS
*   **内存**: 静态待机约 40MB (得益于 Rust 内存控制)

---

## 🚀 快速开始

### 获取产物
前往 [Releases](https://github.com/AiToByte/SafeMask/releases) 下载：
*   **Windows**: `.msi` 或绿色版 `.zip` (推荐)
*   **macOS**: 通用二进制 `.dmg` (支持 M1/M2/M3)

### 开发与构建
```bash
# 1. 克隆
git clone https://github.com/AiToByte/SafeMask.git

# 2. 安装前端
npm install

# 3. 启动开发宇宙
npm run tauri dev

# 4. 构建发布版本
npm run tauri build
```

---

<div align="center">
  <p><b>SafeMask</b> - 让每一行数据，都能安全地拥抱 AI。</p>
  <p>Developed with ❤️ by <b>XiaoSheng</b></p>
</div>