对于一个 5 年经验、追求高质量产出的工程师来说，GitHub Release 不仅仅是上传一个 `.exe` 文件，它是一份**产品的身份说明书**。

以下是为你设计的 **SafeMask v0.1.0** 发布说明模板。你可以直接复制到 GitHub 的 "Release Notes" 框中，并根据实际情况微调。

---

## 🚀 SafeMask v0.4.1 - High Performance Privacy Guard
1. 采用三阶段


### 🌟 核心特性 (Key Features)

*   **极致性能引擎**：采用 **Aho-Corasick** 算法与 **超级正则 (Combined DFA)** 混合驱动，实现单次扫描脱敏。
*   **零拷贝 I/O**：深度利用 **Memory Mapping (Mmap)** 技术，处理 GB 级别大文件时内存占用极低。
*   **多核并发处理**：通过 **Rayon** 并行流，自动利用多核 CPU 算力，100MB 级别日志处理仅需 **0.4s**。
*   **模块化配置**：支持 YAML 定义规则包，涵盖 OpenAI/Claude/DeepSeek API Keys、主流数据库连接串、IP/Email 等。
*   **双模式支持**：
    *   **Clipboard 模式**：一键处理剪贴板内容，无缝对接 ChatGPT/Claude 对话。
    *   **File 模式**：支持大规模本地日志文件的脱敏导出。

### 📊 性能表现 (Benchmarks)

在 Windows 11 环境下，处理包含百万行数据的真实日志文件（113 MB）：
*   **吞吐量**: ~270 MB/s
*   **总耗时**: **420ms** (计算+磁盘写入)
*   **内存峰值**: < 50 MB

### 📂 产物说明 (Artifacts)

| 文件名 | 适用平台 | 说明 |
| :--- | :--- | :--- |
| `safemask-v0.1.0-windows-x64.zip` | Windows 10/11 | 包含可执行文件及示例规则文件夹 |
| `rules.zip` | 全平台 | 预置的脱敏规则包（AI, Database, Network） |

### 🛠️ 安装与使用 (Quick Start)

1.  下载并解压 `safemask-v0.1.0-windows-x64.zip`。
2.  确保 `safemask.exe` 同级目录下存在 `rules/` 文件夹。
3.  **剪贴板脱敏**：
    ```powershell
    ./safemask --mode clipboard
    ```
4.  **大规模文件处理**：
    ```powershell
    ./safemask --mode file --path test.log --output masked.log
    ```

### 🔒 安全承诺

*   **100% 离线运行**：所有计算均在本地完成，绝不上传任何数据。
*   **开源透明**：核心算法逻辑完全透明，接受社区审计。

---