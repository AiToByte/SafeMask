<div align="center">

<img src="src-tauri/icons/icon.png" width="128" alt="SafeMask logo"/>

# SafeMask

**在物理宇宙保留真实，在数字宇宙交换安全。**

面向 AI 时代的工业级 **本地优先** 隐私脱敏控制台 — 剪贴板、文件与规则全流程离线。

<br/>

[![Rust](https://img.shields.io/badge/Rust-2024-orange?logo=rust)](https://www.rust-lang.org/) [![Tauri](https://img.shields.io/badge/Tauri-v2-blue)](https://v2.tauri.app/) [![React](https://img.shields.io/badge/React-19-61dafb?logo=react)](https://react.dev/) [![License](https://img.shields.io/badge/License-MIT-gray)](LICENSE) [![Offline](https://img.shields.io/badge/Privacy-100%25%20Offline-teal)](#-隐私与安全) [![Release](https://img.shields.io/badge/Release-v2.1.x-emerald)](https://github.com/AiToByte/SafeMask/releases)
<br/>

**语言：** [English](README.md) · **简体中文** · [日本語](README_JA.md) · [한국어](README_KO.md) · [Русский](README_RU.md)

</div>

---

## 目录

- [为什么需要 SafeMask？](#为什么需要-safemask)
- [效果演示](#效果演示)
- [核心特性](#核心特性)
- [本地 AI 引擎（端侧 NER）](#本地-ai-引擎端侧-ner)
- [安装](#安装)
- [使用方式](#使用方式)
- [开发](#开发)
- [架构概览](#架构概览)
- [文档索引](#文档索引)
- [隐私与安全](#隐私与安全)
- [常见问题](#常见问题)
- [发布](#发布)
- [贡献](#贡献)
- [许可证](#许可证)

---

## 为什么需要 SafeMask？

把日志、代码、纪要粘贴给 ChatGPT、Claude 等大模型时，API Key、手机号、邮箱、内网 IP、真实姓名等敏感信息会一并带走。

**SafeMask** 全程在本机运行：无遥测、不上传内容。规则、正则、可选的本地 AI 模型、历史与脱敏均离线完成。

| 痛点 | SafeMask 方案 |
|------|----------------|
| 误粘贴密钥到 AI | **魔法粘贴**注入脱敏文本后恢复原文剪贴板 |
| 本地编译/配置需要原文 | **影子模式**默认保留系统剪贴板明文 |
| 共享屏幕等高风险场景 | **哨兵模式**复制后自动漂白剪贴板 |
| 正则难以覆盖的人名/地址 | **本地 AI NER** — 量化 ONNX 模型，100% 端侧推理 |
| 超大日志 | **mmap + Rayon** 保序流水线 |
| 组织自定义策略 | **规则管理** + YAML 导入/导出 |

---

## 效果演示

一次按键，隔开你与一次泄密。**脱敏前** —— 你复制的内容：

```text
2026-07-24 10:32:01 INFO user=张伟 email=zhang.wei@example.com phone=13812345678 ip=192.168.31.10 key=sk-a1b2c3d4e5f6g7h8i9j0k1l2m3n4o5p6
```

**脱敏后** —— 真正发给大模型的内容：

```text
2026-07-24 10:32:01 INFO user=<PERSON> email=<EMAIL> phone=<CHINA_MOBILE> ip=<IPv4> key=<OPENAI_KEY>
```

其中人名由 **AI 模型**识别，其余由**内置规则**命中 —— 粘贴完成后，你的原始剪贴板会立即恢复。

---

## 核心特性

### 宇宙双轨剪贴板

- **影子模式（默认）** — `Ctrl+C` 保留真实文本；按 **`Alt+V`**（可配置）执行魔法粘贴：备份 → 脱敏 → 粘贴 → 还原。
- **哨兵模式** — 复制后自动清洗敏感内容。
- **`Alt+M`** 切换模式；支持窗口置顶。

### 混合识别引擎

- **Aho-Corasick** 字典 / 关键字（优先级 100）
- **字节级正则** 高吞吐模式匹配（优先级 90）
- **可选本地 AI NER**（优先级 50）— 见[下文](#本地-ai-引擎端侧-ner)
- **子跨度雕刻式冲突消解** — 窄高优先级跨度雕刻宽低优先级跨度；规则命中会压制重叠的 AI 识别结果

### 规则与配置

- 内置 YAML 规则包（AI 密钥、网络、个人信息、代码、数据库连接串等）
- 自定义规则 + 实时正则沙盒
- **多文件 YAML 导入**（覆盖同名自定义，跳过内置同名）
- **导出自定义规则** + 导入模板下载
- 全局标签样式：`<TAG>` / `[TAG]` — 运行时对所有规则与 AI 标签即时生效

### 文件流水线

- 内存映射 IO、约 8MB 分块、按行安全切分
- 多核保序写出与背压控制
- 超大文件下内存占用稳定

### 界面与主题

- React 19 + Zustand + Tailwind，原生窗口标题栏
- 可扩展主题：**默认工业琥珀** / **Claude 暖米色**

### 可选审计落盘

- 可开启 Markdown 审计写入器，回顾每次脱敏明细（会落盘明文敏感信息，**默认关闭**）

---

## 本地 AI 引擎（端侧 NER）

SafeMask 提供可选的 AI 识别层，用于覆盖规则难以表达的**人名、地址、机构**等上下文相关实体。

### 它是什么

- 基于 `openai/privacy-filter` 的量化（**q4**）ONNX NER 模型 —— 8 层 MoE token 分类器，33 个 BIOES 标签。
- 推理通过 ONNX Runtime + HuggingFace tokenizers **完全在本机进行**，任何文本都不会离开你的设备。
- 模型**不随安装包分发** —— 需要显式的一次性下载。

### 能识别什么

| 模型实体 | 脱敏标签 | 示例 |
|---|---|---|
| `private_person` | `<PERSON>` | 张伟、John Smith |
| `private_email` | `<EMAIL>` | zhang.wei@example.com |
| `private_phone` | `<PHONE>` | +86 138 1234 5678 |
| `private_address` | `<ADDRESS>` | 北京市朝阳区… |
| `account_number` | `<BANK_CARD>` | 6222 0212 3456 7890 |
| `private_date` | `<DATE>` | 个人日期 |
| `private_url` | `<URL>` | 个人链接 |
| `secret` | `<API_KEY>` | 各类令牌密钥 |

### 如何启用

**方式 A —— 一键下载（推荐）**

1. 打开 **设置 → AI 引擎**。
2. 点击**下载**（约 550MB 压缩包；安装过程中需约 2GB 可用磁盘空间）。
3. 模型会从多个镜像源自动回退下载，经 SHA-256 校验、解压后**热加载** —— 无需重启。

**方式 B —— 自助下载镜像**

服务器带宽有限 —— 若应用内下载较慢，可通过以下任一镜像自行获取 `privacy-filter.zip`：

| 镜像 | 链接 | 提取码 |
|---|---|---|
| HuggingFace | [privacy-filter.zip](https://huggingface.co/buckets/XiaoShengCYZ/AI_Models/resolve/privacy-filter.zip?download=true) | — |
| 夸克网盘 | [pan.quark.cn](https://pan.quark.cn/s/51647902f801?pwd=HQ1Y) | `HQ1Y` |
| 百度网盘 | [pan.baidu.com](https://pan.baidu.com/s/1mDBr0mdo2r-guC4LshF87w?pwd=ba7b) | `ba7b` |

下载后将压缩包解压至 `SafeMask.exe` 同级的 `models/privacy-filter/` 目录，然后重启 SafeMask。

**方式 C —— 手动放置**

将以下文件放入 `SafeMask.exe` 同级的 `models/privacy-filter/` 目录：

```text
models/privacy-filter/
├── model_q4.onnx         # 量化模型（入口）
├── model_q4.onnx_data    # 量化权重（约 875MB）
├── tokenizer.json        # HuggingFace 分词器
└── config.json           # id2label 元数据
```

> 启动时还会依次搜索：工作目录 `./models`、应用本地数据目录、应用资源目录。

### 运行行为

- **懒加载** —— 模型在首次执行脱敏时才在后台加载（通常 1–3 分钟，超时上限 5 分钟）。加载状态与耗时显示在设置页；详情记录于 `ai_model_load.log`。
- **规则优先** —— AI 优先级为 50，规则为 90–100，同一段文本上确定性的规则命中始终优先于 AI 推测。
- **运行时开关** —— 设置页可随时启停 AI；检测到模型文件时启动即自动启用。
- **优雅降级** —— 没有模型时其余功能完全一致，SafeMask 自动退回纯规则模式。

### 调优

| 配置项 | 默认值 | 说明 |
|---|---|---|
| `ORT_NUM_THREADS` 环境变量 | `2` | ONNX Runtime 推理线程数 |
| 置信度阈值 | `0.5` | 低于该值的 AI 跨度会被丢弃 |
| 上下文窗口 | 512 tokens | 单次推理长度 |

---

## 安装

### 预编译安装包（推荐）

从 [**GitHub Releases**](https://github.com/AiToByte/SafeMask/releases) 下载：

| 安装包 | 说明 |
|---|---|
| `SafeMask_x.y.z_x64-setup.exe` | Windows NSIS 安装包 —— 推荐 |
| `SafeMask_x.y.z_x64_zh-CN.msi` | Windows MSI |
| macOS / Linux 包 | 由 CI 在每次 `v*` 标签时产出 |

AI 模型为可选项，安装后在应用内单独下载（见[本地 AI 引擎](#本地-ai-引擎端侧-ner)）。

---

## 使用方式

### 剪贴板工作流

1. 照常复制（`Ctrl+C`）—— **影子模式**下原文保留在剪贴板。
2. 聚焦目标输入框（ChatGPT、Claude、网页表单……）。
3. 按 **`Alt+V`** —— SafeMask 备份剪贴板 → 脱敏文本 → 粘贴安全版本 → 恢复原始剪贴板。
4. 需要严格防护？按 **`Alt+M`** 切换**哨兵模式** —— 每次复制后即时清洗。

| 快捷键 | 功能 | 可配置 |
|---|---|---|
| `Alt+V` | 魔法粘贴（脱敏并粘贴） | ✅ 快捷键与粘贴延迟 |
| `Alt+M` | 切换影子 / 哨兵模式 | ❌（固定绑定） |

### 自定义规则

在**规则**页直接添加，或导入 YAML 规则包：

```yaml
group: "MY_COMPANY"
rules:
  - name: "Internal_Project_Code"
    pattern: '\bPRJ-\d{6}\b'
    mask: "<PROJECT_CODE>"
    priority: 10
```

保存前可在内置正则沙盒中实时测试。完整格式见 [docs/RULES_IMPORT.md](docs/RULES_IMPORT.md)。

### 标签样式

偏好方括号？**设置 → 标签样式**可在 `<TAG>` 与 `[TAG]` 之间即时切换 —— 内置规则、自定义规则与 AI 标签全部跟随。

### 文件处理

将日志文件拖入仪表盘（或使用文件选择器）—— SafeMask 通过内存映射、多核流水线流式处理，并在原文件旁输出脱敏副本，超大文件下内存占用依然稳定。

---

## 开发

### 环境

- Node.js 18+
- Rust stable（项目使用 2024 edition）
- [Tauri v2 平台依赖](https://v2.tauri.app/start/prerequisites/)

### 运行

```bash
# 安装前端依赖
npm install

# 完整桌面端（Vite + Tauri）
npm run tauri dev

# 仅前端（http://127.0.0.1:18924）
npm run dev
```

### 构建

```bash
npm run build
npm run tauri build
```

### Rust 检查（仓库根目录）

```bash
cargo check  -p SafeMask
cargo test   -p SafeMask
cargo clippy -p SafeMask -- -D warnings
cargo fmt    -p SafeMask
```

### 环境变量

| 变量 | 默认值 | 用途 |
|---|---|---|
| `SAFEMASK_THREADS` | `2` | Rayon 工作线程数（文件流水线） |
| `ORT_NUM_THREADS` | `2` | ONNX Runtime 推理线程数 |

---

## 架构概览

```
React 19 UI
    │  invoke / events
    ▼
api/*  (Tauri 命令)
    ▼
orchestrator  (Shadow / Sentry)
    ▼
hybrid_engine  → 识别器 → 冲突消解 → 脱敏策略
    ▼
infra  (剪贴板、mmap 文件、ONNX、配置、记录)
```

- `core/` **零** Tauri 依赖，可独立单测
- 偏移量为 UTF-8 **字节偏移**
- 自定义规则：应用 `custom/` 存储路径下的 `user_rules.yaml`

更多：[CLAUDE.md](CLAUDE.md) · [docs/](docs/) · [docs/RULES_IMPORT.md](docs/RULES_IMPORT.md) · [docs/THEMES.md](docs/THEMES.md)

---

## 文档索引

| 文档 | 说明 |
|------|------|
| [README.md](README.md) | English |
| [README_CN.md](README_CN.md) | 简体中文（本文） |
| [README_JA.md](README_JA.md) | 日本語 |
| [README_KO.md](README_KO.md) | 한국어 |
| [README_RU.md](README_RU.md) | Русский |
| [CLAUDE.md](CLAUDE.md) | 架构与协作说明 |
| [DEVELOPMENT.md](DEVELOPMENT.md) | 开发指南 |
| [docs/使用手册.md](docs/使用手册.md) | 用户手册 |
| [docs/RULES_IMPORT.md](docs/RULES_IMPORT.md) | 规则导入导出格式 |
| [docs/THEMES.md](docs/THEMES.md) | 主题系统 |
| [docs/record-writer.md](docs/record-writer.md) | 审计记录写入器 |
| [GitHub Releases](https://github.com/AiToByte/SafeMask/releases) | 预编译安装包 |
| [Issues](https://github.com/AiToByte/SafeMask/issues) | 问题反馈与功能建议 |
| [LICENSE](LICENSE) | MIT |

---

## 隐私与安全

- 脱敏内容**永不上云**
- **AI 推理 100% 端侧进行**；唯一的网络流量是可选的、显式发起的一次性模型下载（仅白名单镜像源）
- 模型下载为可选、用户主动操作
- 审计落盘需手动开启（会保存明文原文到本地）
- 勿导入不信任来源的规则包

---

## 常见问题

**SafeMask 会把我的剪贴板或文件上传到别处吗？**
不会。所有脱敏 —— 规则与 AI —— 均在本地完成，无遥测、无内容上传。

**没有 AI 模型能用吗？**
可以。规则引擎（字典 + 正则）本身即可完整工作。AI 是用于人名、地址等上下文实体的增强层。

**为什么第一次 AI 脱敏很慢？**
约 900MB 的模型在首次使用时才懒加载（通常 1–3 分钟），之后的脱敏即时完成。进度见 设置 → AI 引擎。

**AI 模型存在哪里？**
可执行文件旁的 `models/privacy-filter/`（或应用本地数据 / 资源目录）。删除该文件夹即可彻底移除。

**可以自定义识别规则吗？**
可以 —— 自定义 YAML 规则 + 实时正则沙盒，支持多文件导入导出。见 [docs/RULES_IMPORT.md](docs/RULES_IMPORT.md)。

---

## 发布

预编译安装包：[GitHub Releases](https://github.com/AiToByte/SafeMask/releases)

`v*` 标签触发 CI 产出多平台包（Windows / macOS / Linux）。

---

## 贡献

1. 小步 PR
2. 推送前：`cargo test -p SafeMask` + `npm run build`
3. 保持 `core/` 纯逻辑、`infra/` 系统、`api/` IPC 分层
4. 主题扩展见 [docs/THEMES.md](docs/THEMES.md)
5. 新规则包：YAML 兼容 `RuleGroup` 或纯规则数组

仓库：[AiToByte/SafeMask](https://github.com/AiToByte/SafeMask)

---

## 许可证

[MIT](LICENSE) © SafeMask / AiToByte

---

<div align="center">

[English](README.md) · **简体中文** · [日本語](README_JA.md) · [한국어](README_KO.md) · [Русский](README_RU.md)

</div>
