对于一个 5 年经验、追求高质量产出的工程师来说，GitHub Release 不仅仅是上传一个 `.exe` 文件，它是一份**产品的身份说明书**。

以下是为你设计的 **SafeMask v0.1.0** 发布说明模板。你可以直接复制到 GitHub 的 "Release Notes" 框中，并根据实际情况微调。

---

## 🚀 SafeMask v0.1.0 - High Performance Privacy Guard

这是 **SafeMask** 的第一个正式发布版本。SafeMask 是一款专为开发者和运维工程师设计的、基于 Rust 的极致性能脱敏工具，旨在解决 AI 辅助开发流中的数据隐私合规问题。

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

### 💡 发布前的小建议 (Checklist)

1.  **打包规则文件夹**：
    由于你的程序依赖 `rules/` 目录下的 YAML 文件，用户下载后如果没有这个目录会报错。
    *   **建议**：将 `safemask.exe` 和 `rules/` 文件夹一起打包进一个 `.zip` 文件。
2.  **静态链接 (Static Linking)**：
    为了防止用户电脑缺少某些 DLL，建议在 Windows 编译时使用以下命令（如果是用 MSVC 环境）：
    ```powershell
    # 这样生成的 .exe 就不依赖特定的 VC 运行时库
    $env:RUSTFLAGS="-C target-feature=+crt-static"
    cargo build --release
    ```
3.  **计算 Hash 值**（专业体现）：
    在 Release 说明的末尾，放上二进制文件的 SHA256 校验码，防止文件被篡改：
    ```powershell
    # Windows 下获取哈希值
    Get-FileHash ./target/release/safemask.exe -Algorithm SHA256
    ```
4.  **LICENSE 文件**：
    确保仓库里有一个 `LICENSE` 文件（推荐 MIT 或 Apache 2.0），这对远程工作的合规性加分很重要。

这份说明体现了你作为一个**资深后端工程师**对性能、易用性和安全性的全面考虑。发布后，这不仅是一个工具，更是你 Rust 工程能力的有力证言。