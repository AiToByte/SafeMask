# SafeMask 开发手册

> 版本: 1.2.4 | 最后更新: 2026-07-03

---

## 目录

1. [项目概述](#1-项目概述)
2. [技术栈](#2-技术栈)
3. [环境要求](#3-环境要求)
4. [目录结构](#4-目录结构)
5. [开发环境搭建](#5-开发环境搭建)
6. [常用命令](#6-常用命令)
7. [构建与打包](#7-构建与打包)
8. [运行指南](#8-运行指南)
9. [代码规范](#9-代码规范)
10. [发布流程](#10-发布流程)
11. [架构详解](#11-架构详解)
12. [常见问题及排查](#12-常见问题及排查)

---

## 1. 项目概述

SafeMask 是一个使用 Rust + Tauri v2 构建的高性能、有序数据脱敏工具。支持文本、文件（Office/PDF/日志）的脱敏处理，提供剪贴板监听、AI 引擎识别、多策略脱敏等核心功能。

- **仓库**: <https://github.com/AiToByte/SafeMask>
- **作者**: XiaoSheng
- **许可**: MIT

### 核心功能

- **剪贴板监听**: 自动检测剪贴板内容变更，实时脱敏
- **多引擎脱敏**: Aho-Corasick、正则表达式、NER（AI）、校验和识别器
- **影子/哨兵模式**: Shadow（静默脱敏粘贴） / Sentry（确认后粘贴）
- **文件批量处理**: 支持 txt / csv / json / md / docx / xlsx / pdf
- **AI 引擎**: ONNX Runtime + HuggingFace tokenizers 本地 NER 推理
- **规则管理**: 自定义脱敏规则，实时测试沙盒
- **系统托盘**: 后台常驻，快捷键操作

---

## 2. 技术栈

### 前端

| 技术 | 版本 | 用途 |
|------|------|------|
| React | 19.x | UI 框架 |
| TypeScript | 5.7.x | 类型安全 |
| Zustand | 5.x | 状态管理 |
| Tailwind CSS | 3.4.x | 样式 |
| Vite | 6.x | 构建工具 |
| Lucide React | 0.471.x | 图标库 |
| framer-motion | 12.x | 动画（已移除大部分，改用 CSS） |
| clsx + tailwind-merge | — | CSS 类名合并 |

### 后端 (Rust)

| 技术 | 用途 |
|------|------|
| Tauri 2.x | 桌面框架 |
| ort 2.0.0-rc.12 | ONNX Runtime (AI 推理) |
| tokenizers 0.20 | HuggingFace 分词器 |
| aho-corasick 1.1 | 多模式字符串匹配 |
| regex 1.12 | 正则引擎 |
| rayon 1.11 | 并行处理 |
| memmap2 0.9 | 内存映射文件 |
| arboard 3.6 | 剪贴板读写 |
| clipboard-master 4.0 | 剪贴板事件监听 |
| parking_lot 0.12 | 高性能锁 |
| calamine / rust_xlsxwriter | Excel 读写 |
| pdf-extract | PDF 文本提取 |
| quick-xml | DOCX XML 解析 |
| mimalloc | 内存分配器 |

### 开发工具

| 工具 | 用途 |
|------|------|
| Tauri CLI 2.x | Tauri 构建与开发 |
| rustc (Rust 2024 edition) | Rust 编译器 |
| Node.js + npm | 前端依赖管理 |
| TypeScript (tsc) | 前端类型检查 |
| cargo / clippy / fmt | Rust 质量工具 |

---

## 3. 环境要求

### 必需

- **Rust**: 最新 stable (支持 edition 2024)
  - 通过 <https://rustup.rs> 安装
  - 验证: `rustc --version && cargo --version`
- **Node.js**: >= 18.x（推荐 20.x LTS）
  - 验证: `node --version && npm --version`
- **系统依赖**:
  - Windows: WebView2（Win11 自带，Win10 需安装）
  - macOS: 无特殊依赖
  - Linux: `libwebkit2gtk-4.1-dev` 等 (参考 Tauri 文档)

### 网络配置

项目使用 `.cargo/config.toml` 配置了本地代理（`127.0.0.1:7890`）。如无需代理或代理地址不同，请修改或删除该文件：

```bash
# 删除代理配置
rm -rf .cargo/config.toml
```

### 可选

- **ONNX 模型**: 放置到 `src-tauri/models/privacy-filter/` 目录下
  - 需要 `model_q4.onnx` + `tokenizer.json`
  - 无模型时系统以降级模式运行，不影响脱敏功能

---

## 4. 目录结构

```
SafeMask/
├── .cargo/
│   └── config.toml          # Cargo 代理配置
├── .github/
│   └── workflows/           # CI/CD 配置（当前仓库不存在）
├── src-tauri/               # RUST：Tauri 后端
│   ├── src/
│   │   ├── main.rs          # 二进制入口：Tauri Builder，插件注册，初始化
│   │   ├── lib.rs           # 库入口：模块导出（safemask_lib）
│   │   ├── api/             # Tauri IPC 命令处理层
│   │   │   ├── mod.rs
│   │   │   ├── files.rs     # 文件处理命令（process_file_gui）
│   │   │   ├── text.rs      # 文本脱敏命令（mask_text）
│   │   │   └── system.rs    # 系统命令（设置，规则，历史，AI管理）
│   │   ├── common/          # 公共类型层（无 Tauri 依赖）
│   │   │   ├── mod.rs
│   │   │   ├── state.rs     # AppState 全局状态结构
│   │   │   ├── errors.rs    # AppError 错误定义（thiserror）
│   │   │   └── events.rs    # 前端事件常量
│   │   ├── core/            # 核心业务层（纯 Rust，可独立测试）
│   │   │   ├── mod.rs
│   │   │   ├── config.rs    # 应用配置结构
│   │   │   ├── engine.rs    # 基础引擎（MaskEngine）
│   │   │   ├── hybrid_engine.rs  # 混合引擎（组合所有识别器）
│   │   │   ├── rules.rs     # 规则实体
│   │   │   ├── masking/     # 脱敏策略
│   │   │   │   ├── mod.rs
│   │   │   │   ├── engine.rs
│   │   │   │   └── strategies.rs
│   │   │   ├── orchestrator/ # 业务流程编排（SceneMode）
│   │   │   │   ├── mod.rs
│   │   │   │   └── scene.rs
│   │   │   ├── recognizer/  # 可插拔识别器
│   │   │   │   ├── mod.rs
│   │   │   │   ├── aho_corasick_recognizer.rs
│   │   │   │   ├── checksum_recognizer.rs
│   │   │   │   ├── context_enhancer.rs
│   │   │   │   ├── ner_recognizer.rs
│   │   │   │   ├── regex_recognizer.rs
│   │   │   │   ├── registry.rs
│   │   │   │   └── types.rs
│   │   │   └── resolver/    # 冲突解决
│   │   │       └── mod.rs
│   │   ├── infra/           # 基础设施层（OS 交互）
│   │   │   ├── mod.rs
│   │   │   ├── ai/          # AI 引擎（ONNX）
│   │   │   │   ├── mod.rs
│   │   │   │   ├── model_manager.rs
│   │   │   │   └── ner_engine.rs
│   │   │   ├── clipboard/   # 剪贴板相关
│   │   │   │   ├── mod.rs
│   │   │   │   ├── handler.rs
│   │   │   │   ├── magic_paste.rs
│   │   │   │   └── monitor.rs
│   │   │   ├── config/      # 配置持久化
│   │   │   │   ├── mod.rs
│   │   │   │   ├── loader.rs
│   │   │   │   └── shortcut_manager.rs
│   │   │   └── fs/          # 文件系统
│   │   │       ├── mod.rs
│   │   │       └── processor.rs
│   │   └── 架构图.md        # 架构说明（供参考，可能过时）
│   ├── build.rs             # Tauri 构建脚本
│   ├── Cargo.toml           # Rust 包配置
│   ├── tauri.conf.json      # Tauri 应用配置
│   └── capabilities/
│       └── default.json     # IPC 权限配置
├── src/                     # REACT：前端
│   ├── App.tsx              # 根组件（标签页切换，启动引导）
│   ├── main.tsx             # React 入口
│   ├── style.css            # 全局样式（Tailwind + 自定义 CSS 动画）
│   ├── vite-env.d.ts        # Vite 类型声明
│   ├── hooks/
│   │   ├── useAppStore.ts   # Zustand 全局状态（含 bootstrap 启动逻辑）
│   │   ├── useAudioFeedback.ts
│   │   └── useTauriEvents.ts
│   ├── services/
│   │   └── api.ts           # Tauri IPC 封装（invoke 调用）
│   ├── lib/
│   │   └── utils.ts         # 工具函数
│   └── components/
│       ├── dashboard/
│       │   ├── FileProcessor.tsx  # 文件处理面板（懒加载）
│       │   └── StatCard.tsx       # 统计卡片
│       ├── feedback/
│       │   └── MagicFeedback.tsx  # 操作反馈提示（懒加载）
│       ├── history/
│       │   └── HistoryList.tsx    # 脱敏历史列表
│       ├── layout/
│       │   ├── Header.tsx         # 顶部栏
│       │   └── Sidebar.tsx        # 侧边导航栏
│       ├── overlay/
│       │   └── ExitConfirm.tsx    # 退出确认弹窗（懒加载）
│       ├── rules/
│       │   └── RuleManager.tsx    # 规则管理
│       ├── settings/
│       │   └── SettingsPage.tsx   # 设置页面
│       └── ui/
│           ├── Badge.tsx
│           ├── Button.tsx
│           ├── Card.tsx
│           ├── EmptyState.tsx
│           ├── GlassPanel.tsx
│           ├── Input.tsx
│           └── Toggle.tsx
├── index.html                # HTML 入口（含骨架屏）
├── package.json              # 前端依赖与脚本
├── vite.config.ts            # Vite 配置
├── tsconfig.json             # TypeScript 配置
├── tsconfig.node.json        # Vite 配置文件 TS 配置
├── tailwind.config.js        # Tailwind CSS 配置
├── postcss.config.js         # PostCSS 配置
├── Cargo.toml                # Cargo 工作区根配置
├── .gitignore
├── AGENTS.md                 # AI Agent 专用开发备忘录
├── CLAUDE.md                 # 已过时（描述的是 Vue 3 + Pinia，请忽略）
└── DEVELOPMENT.md            # 本文件 - 开发手册
```

---

## 5. 开发环境搭建

### 5.1 首次克隆

```bash
git clone https://github.com/AiToByte/SafeMask.git
cd SafeMask
```

### 5.2 前端依赖安装

```bash
npm install
```

### 5.3 Rust 编译检查

```bash
# 检查 Rust 编译（未安装 Tauri CLI 也可执行）
cargo check -p SafeMask
```

### 5.4 代理配置（可选）

如需保持代理，确保 `.cargo/config.toml` 中的地址正确：

```toml
[http]
proxy = "http://127.0.0.1:7890"

[https]
proxy = "http://127.0.0.1:7890"
```

如无需代理，删除该文件即可。

### 5.5 安装 Tauri CLI（可选，用于 `npm run tauri dev`）

```bash
# Tauri CLI 已包含在 devDependencies 中，无需全局安装
# 通过 npx 或 npm run tauri 调用
npx tauri --version
```

---

## 6. 常用命令

所有命令在项目根目录执行。

### 6.1 前端

```bash
# 启动前端开发服务器（纯浏览器，无 Tauri 窗口）
npm run dev
# → http://127.0.0.1:18924

# 前端类型检查（tsc）
npx tsc --noEmit

# 前端构建（tsc + vite build）
npm run build

# Vite 预览构建产物
npm run preview
```

### 6.2 Rust 后端

```bash
# Rust 编译检查（快）
cargo check -p SafeMask

# 完整编译
cargo build -p SafeMask

# 格式化检查
cargo fmt -p SafeMask

# Clippy lint 检查（警告即错误）
cargo clippy -p SafeMask -- -D warnings

# 运行测试（所有 #[cfg(test)] 内联模块）
cargo test -p SafeMask

# 运行单个测试并显示输出
cargo test -p SafeMask test_name -- --nocapture

# 生产构建（优化 + LTO）
cargo build -p SafeMask --release
```

### 6.3 全栈开发

```bash
# 完整 Tauri 开发模式（前端热更新 + Rust 窗口）
npm run tauri dev

# Tauri 构建（生产打包）
npm run tauri build
```

> **注意**: `npm run tauri dev` 会先启动 Vite 开发服务器，再启动 Tauri 窗口。
> Vite 服务器运行在 `127.0.0.1:18924`，端口严格固定。

### 6.4 快速验证清单

```bash
# 开发时修改代码后的标准验证流程
npm run build          # 前端：tsc 类型检查 + vite 构建
cargo check -p SafeMask  # Rust：编译检查
cargo fmt -p SafeMask    # Rust：格式检查
cargo clippy -p SafeMask -- -D warnings  # Rust：lint 检查
```

---

## 7. 构建与打包

### 7.1 调试构建

```bash
npm run tauri dev
```

- Vite 热更新前端代码
- Rust 代码需重新编译才会生效（Tauri 会自动检测）
- 窗口带有开发者工具

### 7.2 生产构建

```bash
npm run tauri build
```

执行流程：

1. `beforeBuildCommand`: 执行 `npm run build`（tsc → vite build）
2. 前端产物输出到 `dist/`
3. `tauri_build::build()` 处理资源嵌入
4. Rust 编译 `--release` 模式
5. 打包为平台安装包

产物路径：

| 平台 | 产物 |
|------|------|
| Windows | `src-tauri/target/release/bundle/msi/SafeMask_1.2.4_x64_zh-CN.msi` |
| macOS | `src-tauri/target/release/bundle/dmg/SafeMask_1.2.4_x64.dmg` |
| Linux | `src-tauri/target/release/bundle/deb/safemask_1.2.4_amd64.deb` |

### 7.3 构建配置

**关键配置项** (`tauri.conf.json`):

```json
{
  "build": {
    "beforeDevCommand": "npm run dev",
    "beforeBuildCommand": "npm run build",
    "devUrl": "http://127.0.0.1:18924",
    "frontendDist": "../dist"
  },
  "bundle": {
    "targets": "all",
    "createUpdaterArtifacts": true,
    "resources": ["icons/**/*", "rules/**/*", "custom/**/*"],
    "windows": { "wix": { "language": "zh-CN" } }
  }
}
```

### 7.4 Vite 分包策略

`vite.config.ts` 配置了 `manualChunks` 将第三方库拆分为独立 chunk：

| Chunk | 包含 | 大小 |
|-------|------|------|
| `vendor-react` | react, react-dom | ~4 KB |
| `vendor-lucide` | lucide-react | ~20 KB |
| `vendor-tauri` | @tauri-apps/* | ~16 KB |
| `vendor-state` | zustand | ~1 KB |
| `index` | 应用代码 | ~270 KB |

> framer-motion 已从分包中移除（已通过 CSS 动画替代，不再作为主依赖）

### 7.5 增量构建

```bash
# 仅编译 Rust
cargo build -p SafeMask

# 仅构建前端
npm run build

# 仅检查 TS 类型（不输出文件）
npx tsc --noEmit
```

---

## 8. 运行指南

### 8.1 开发模式

```bash
npm run tauri dev
```

此为全栈开发模式：
- 前端：Vite HMR 热更新
- 后端：Rust 代码变更后自动重新编译（可能较慢）
- 初始启动大约需 10-30s（取决于 Rust 编译缓存）

首次启动较慢是正常现象。后续启动仅修改前端时极快。

### 8.2 纯前端开发（无 Tauri 窗口）

```bash
npm run dev
```

浏览器访问 `http://127.0.0.1:18924`

> 注意：纯前端模式下 Tauri API (`invoke`) 不可用，页面会显示错误。
> 适用于 UI 样式 / 组件结构调试。

### 8.3 启动流程详解

应用启动时序：

```
用户点击图标
  ↓
main.rs: main()
  └→ 初始化日志 (env_logger)
  └→ 限制 Rayon 线程数 (SAFEMASK_THREADS)
  └→ 限制 ONNX 线程数 (ORT_NUM_THREADS)
  └→ Tauri Builder 创建
      ├→ 注册插件 (dialog, opener, notification, global-shortcut)
      ├→ setup_application()
      │   ├→ init_app_state()
      │   │   ├→ 加载持久化设置 (settings.yaml)
      │   │   ├→ 加载规则 (rules.yaml)
      │   │   ├→ 编译引擎 (HybridEngine)
      │   │   ├→ AI 引擎初始化（如模型存在）
      │   │   └→ 注入 AppState 到 Tauri 托管
      │   ├→ init_shortcut_service()
      │   │   ├→ 注册安全粘贴快捷键 (默认: Alt+V)
      │   │   └→ 注册模式切换快捷键 (Alt+M)
      │   ├→ init_background_services()
      │   │   └→ 启动剪贴板监听器
      │   ├→ setup_window_handlers()
      │   │   └→ Win32 窗口圆角 (仅 Windows)
      │   │   └→ 关闭拦截事件
      │   └→ setup_system_tray()
      │       └→ 系统托盘图标 + 菜单
      └→ 启动窗口 (WebView)
  ↓
前端: main.tsx
  └→ ReactDOM.createRoot(#root)
  └→ 渲染 App.tsx
  ↓
前端: App.tsx: useEffect(bootstrap)
  └→ Phase 1 (关键):
  │   ├→ 获取设置 (get_app_settings)
  │   └→ 获取统计 (get_rules_stats)
  └→ Phase 2 (延迟 100ms):
      ├→ 获取历史 (get_mask_history)
      ├→ 获取应用信息 (get_app_info)
      ├→ 获取 AI 状态 (get_ai_engine_status)
      └→ 获取引擎信息 (get_engine_info)
  ↓
首屏就绪
```

### 8.4 窗口行为

- **关闭按钮**: 触发 `CloseRequested` 事件 → 弹出 ExitConfirm 确认对话框
- **系统托盘**: 右键菜单可关闭/显示窗口，左键点击切换隐藏/显示
- **快捷键**: `Alt+V` 触发安全粘贴，`Alt+M` 切换影子/哨兵模式
- **窗口置顶**: 通过设置面板切换

---

## 9. 代码规范

### 9.1 Rust 规范

- **Rust edition 2024** — 较新的语法规范，注意与 2021 的差异
- **命名**: `snake_case` 函数/变量，`CamelCase` 类型/trait
- **错误处理**: 使用 `thiserror` 定义 AppError，`anyhow` 用于函数级错误
- **锁**: 优先使用 `parking_lot::Mutex` / `parking_lot::RwLock` 而非 std 版本
- **unsafe**: 仅在必要时使用（如环境变量设置、Win32 FFI）
- **测试**: `#[cfg(test)]` 内联模块，无 `tests/` 目录
- **核心层 (`core/`)**: 禁止导入 Tauri 相关依赖

检视命令：

```bash
cargo fmt -p SafeMask
cargo clippy -p SafeMask -- -D warnings
```

### 9.2 TypeScript 规范

- **严格模式**: `tsconfig.json` 中 `strict: true`
- **命名**: `camelCase` 变量/函数，`PascalCase` 组件/类型/接口
- **导入路径**: 使用 `@/` 别名（映射到 `./src/`）
- **IPC 调用**: 统一通过 `src/services/api.ts` 中的 `MaskAPI` 对象
- **状态管理**: 通过 Zustand `useAppStore` 管理全局状态，避免 prop drilling
- **组件**: 函数式组件 + hooks

检视命令：

```bash
npx tsc --noEmit
npm run build
```

### 9.3 CSS / Tailwind

- 优先使用 Tailwind 工具类
- 自定义组件样式定义在 `@layer components` 内
- CSS 动画使用 `@keyframes`，不使用 framer-motion
- 全局主题色、阴影在 `tailwind.config.js` 的 `extend` 中定义
- 使用 `clsx` + `tailwind-merge` 合并条件类名

### 9.4 注释规范

- 代码中**不要添加任何注释**（这是项目约定）
- 接口定义、类型、API 方法等导出符号可以使用文档注释

### 9.5 Git 规范

- **分支**: 无强制规范，建议 feature 分支命名 `feat/xxx`
- **提交**: 写简洁的描述，不要提交机密信息
- **CI**: 触发 `v*` tag 推送时自动发布（当前仓库无 CI 文件）

---

## 10. 发布流程

### 10.1 手动发布

```bash
# 1. 更新版本号
#    - Cargo.toml (工作区根)
#    - src-tauri/Cargo.toml
#    - package.json
#    - tauri.conf.json (version 字段)

# 2. 构建
npm run tauri build

# 3. 创建 GitHub Release
#    上传构建产物:
#    - Windows: SafeMask_<version>_x64_zh-CN.msi
#    - macOS: SafeMask_<version>_x64.dmg
#    - Linux: safemask_<version>_amd64.deb

# 4. 更新 latest.json（自动更新用）
#    上传至 GitHub Release 附件
```

### 10.2 自动更新机制

通过 `tauri.conf.json` 配置的 `plugins.updater`：

```json
{
  "plugins": {
    "updater": {
      "active": true,
      "endpoints": ["https://github.com/AiToByte/SafeMask/releases/latest/download/latest.json"],
      "pubkey": "dW50cnVzdGVkIGNvbW1lbnQ6..."
    }
  }
}
```

更新过程：
1. 应用启动时检查 `latest.json`
2. 对比本地版本
3. 有新版本时提示用户下载
4. 下载完成后自动安装

### 10.3 构建产物尺寸优化

由于 Rust 编译包含 `ort` (ONNX Runtime)，静态链接后二进制约 30-60 MB。
可通过以下方式优化：

- 使用 `strip = true` 在 `Cargo.toml` 中
- 启用 LTO (`lto = true`)
- 使用 UPX 压缩（可选）

---

## 11. 架构详解

### 11.1 分层架构

```
┌──────────────────────────────────────────────┐
│              前端 (React + Zustand)            │
│  src/components/*       src/hooks/*           │
│  src/services/api.ts    src/App.tsx           │
└──────────────────┬───────────────────────────┘
                   │ Tauri IPC (invoke)
                   ▼
┌──────────────────────────────────────────────┐
│  API 层 (src-tauri/src/api/)                  │
│  #[tauri::command] fn 处理前端请求            │
│  files.rs  text.rs  system.rs                 │
└──────────────────┬───────────────────────────┘
                   │
                   ▼
┌──────────────────────────────────────────────┐
│  Core 层 (src-tauri/src/core/)                │
│  纯业务逻辑，无 Tauri 依赖，可独立测试        │
│  hybrid_engine / recognizer* / resolver       │
│  masking* / rules / config / orchestrator     │
└──────────────────┬───────────────────────────┘
                   │
                   ▼
┌──────────────────────────────────────────────┐
│  Infra 层 (src-tauri/src/infra/)              │
│  与操作系统交互                                │
│  clipboard* / fs / config / ai                │
└──────────────────────────────────────────────┘
```

### 11.2 数据流

```
用户操作 (粘贴/选择文件/点击按钮)
  ↓
前端: api.ts → invoke("command_name", args)
  ↓
Rust: api/xxx.rs → #[tauri::command]
  ↓
Rust: core/xxx → 业务逻辑处理
  ↓
Rust: infra/xxx → OS 交互 (文件读写/剪贴板/AI推理)
  ↓
Rust: 结果返回 → Tauri IPC 序列化
  ↓
前端: invoke 返回值 → Zustand state 更新
  ↓
UI 重新渲染
```

### 11.3 状态管理

**Rust 端** (`AppState` in `common/state.rs`):

```rust
pub struct AppState {
    engine: Arc<RwLock<HybridEngine>>,     // 脱敏引擎
    settings: Arc<RwLock<AppSettings>>,    // 持久化设置
    shadow_store: Arc<RwLock<ShadowClipboard>>, // 影子剪贴板
    is_magic_pasting: Arc<AtomicBool>,     // 正在安全粘贴
    is_monitor_on: Arc<Mutex<bool>>,       // 监听开关
    history: Arc<Mutex<Vec<HistoryItem>>>, // 历史记录
    last_content: Arc<Mutex<String>>,      // 上次剪贴板内容
    is_recording_mode: Arc<AtomicBool>,    // 录制模式
}
```

**前端端** (`useAppStore` in `hooks/useAppStore.ts`):

```typescript
interface AppState {
  settings: AppSettings;
  isMonitorOn: boolean;
  ruleCount: number;
  activeTab: ActiveTab;
  historyList: HistoryItem[];
  allRulesList: Rule[];
  activeFeedback: FeedbackPayload | null;
  progress: number;
  isProcessing: boolean;
  currentFileName: string;
  appInfo: unknown;
  aiEngineStatus: AiEngineStatus | null;
  engineInfo: EngineInfo | null;
  isAlwaysOnTop: boolean;
}
```

### 11.4 识别器体系

```
HybridEngine
├── AhoCorasickRecognizer  # 多模式字符串匹配（高性能）
├── RegexRecognizer        # 正则表达式匹配（灵活）
├── NerRecognizer          # ONNX AI 实体识别（需模型）
└── ChecksumRecognizer     # 校验和验证（银行账号等）
```

### 11.5 启动优化

项目经过两轮启动优化：

1. **IPC 并行化**: 6 个串行 `await` 改为 `Promise.all`
2. **Bootstrap 拆分**: 2 个关键请求（设置 + 统计）先执行，其余 4 个延迟 100ms
3. **首屏免动画**: 跳过首帧入场动画
4. **懒加载**: MagicFeedback、ExitConfirm、FileProcessor 使用 `React.lazy`
5. **字体系统**: 使用系统字体栈 (system-ui)，无外部字体加载
6. **CSS 动画替代**: framer-motion 全部替换为纯 CSS keyframe animation

### 11.6 线程模型

| 线程池 | 用途 | 默认线程数 |
|--------|------|-----------|
| Rayon | 文件并行处理 | 2 (通过 `SAFEMASK_THREADS` 或 `RAYON_NUM_THREADS`) |
| ONNX Runtime | AI 推理 | 2 (通过 `ORT_NUM_THREADS`) |
| Tauri | 异步任务 | tokio 运行时 |

---

## 12. 常见问题及排查

### 12.1 构建失败

**问题**: `cargo check` 失败，提示 edition 2024 不支持

**可能原因**: rustc 版本过旧。Rust 2024 edition 需要 Rust 1.85+。

**解决**:
```bash
rustup update stable
```

**问题**: `npm run build` 失败

**可能原因**: TypeScript 类型错误或 Vite 构建错误

**解决**:
```bash
npx tsc --noEmit  # 确认类型无误
# 或
npx vite build    # 跳过类型检查
```

### 12.2 代理问题

**问题**: `cargo build` 下载依赖极慢

**可能原因**: `.cargo/config.toml` 代理不可用

**解决**:
```bash
# 删除代理配置
Remove-Item -Recurse -Force .cargo
# 或手动修改 .cargo/config.toml
```

### 12.3 启动崩溃

**问题**: Tauri 窗口闪退无日志

**可能原因**:
1. 端口冲突（18924 被占用）
2. WebView2 未安装
3. Vite 未先启动

**解决**:
```bash
# 查看 Rust 错误输出
npm run tauri dev 2>&1

# 确认端口未被占用
netstat -ano | findstr :18924

# 清除 Tauri 缓存
cargo clean -p SafeMask
Remove-Item -Recurse -Force src-tauri/gen
```

### 12.4 AI 引擎不工作

**问题**: AI 引擎状态显示 `not_available`

**可能原因**: 模型文件缺失

**解决**: 确认以下文件存在：
```
src-tauri/models/privacy-filter/
├── model_q4.onnx
└── tokenizer.json
```

AI 引擎为可选功能，不影响脱敏核心功能。

### 12.5 Windows 特定问题

**问题**: Win32 窗口圆角不生效

**可能原因**: Windows 版本低于 21H2

**解决**: `DwmSetWindowAttribute(DWMWCP_ROUND)` 需要 Win10 21H2+ 或 Win11。

**问题**: 托盘图标不显示

**可能原因**: 图标路径错误

**解决**: 确认 `icons/32x32.png` 存在。Tauri 2 需要 PNG 格式。

### 12.6 Rust 编译极慢

**问题**: 每次修改后 Rust 编译需要 30s+

**原因**: `ort` 是大型 C++ 库，需要静态链接 ONNX Runtime

**解决**:
- 使用 `cargo check` 而非 `cargo build`（仅类型检查）
- 利用增量编译（默认启用）
- 减少 `cargo clean` 的使用

### 12.7 AGENTS.md vs CLAUDE.md

`CLAUDE.md` 是旧版文件，描述的是 Vue 3 + Pinia 架构，**已过时**。
当前架构为 **React 19 + Zustand**，详情查阅 `AGENTS.md`。

---

## 附录

### A. 环境变量参考

| 变量 | 默认值 | 作用 |
|------|--------|------|
| `SAFEMASK_THREADS` | 2 | Rayon 线程池大小 |
| `ORT_NUM_THREADS` | 2 | ONNX Runtime 线程数 |
| `RAYON_NUM_THREADS` | 2 | Rayon 备选设置变量 |

### B. 快捷键参考

| 快捷键 | 功能 |
|--------|------|
| `Alt+V` | 安全粘贴（可自定义） |
| `Alt+M` | 切换影子/哨兵模式 |
| 托盘左键 | 切换窗口可见性 |
| 托盘右键 → 菜单 | 显示窗口 / 退出 |

### C. IPC 命令清单

所有 `#[tauri::command]` 及其注册位置：

| 命令 | 文件 | 功能 |
|------|------|------|
| `get_app_settings` | `api/system.rs` | 获取设置 |
| `update_app_settings` | `api/system.rs` | 更新设置 |
| `toggle_vault_mode` | `api/system.rs` | 切换模式 |
| `get_rules_stats` | `api/system.rs` | 规则统计 |
| `get_all_detailed_rules` | `api/system.rs` | 规则列表 |
| `save_rule_api` | `api/system.rs` | 保存规则 |
| `delete_rule_api` | `api/system.rs` | 删除规则 |
| `test_rule_logic` | `api/system.rs` | 规则测试沙盒 |
| `get_mask_history` | `api/system.rs` | 历史记录 |
| `clear_history_cmd` | `api/system.rs` | 清空历史 |
| `toggle_monitor` | `api/system.rs` | 开关监听 |
| `copy_original_cmd` | `api/system.rs` | 复制原文 |
| `get_app_info` | `api/system.rs` | 应用信息 |
| `toggle_always_on_top` | `api/system.rs` | 窗口置顶 |
| `set_recording_mode` | `api/system.rs` | 录制模式 |
| `get_ai_engine_status` | `api/system.rs` | AI 状态 |
| `get_engine_info` | `api/system.rs` | 引擎信息 |
| `toggle_ai_engine` | `api/system.rs` | AI 开关 |
| `get_registered_recognizers` | `api/system.rs` | 识别器列表 |
| `mask_text` | `api/text.rs` | 文本脱敏 |
| `process_file_gui` | `api/files.rs` | 文件处理 |

### D. IPC 权限配置

在 `src-tauri/capabilities/default.json` 中声明。新增命令需要满足：
1. 在 `invoke_handler(generate_handler![...])`（`main.rs`）注册
2. 如需新增权限，在 `default.json` 的 `permissions` 中添加
