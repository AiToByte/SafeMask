### 一、 严重缺陷与物理损坏漏洞 (Critical Bugs & Corruptions)

#### 1. DOCX XML 还原时的字符未转义漏洞（导致 Word 文档损坏物理损坏）
* **病因位置**：`src-tauri/src/infra/fs/processor.rs` 中的 `mask_xml_content` 函数。
* **技术剖析**：
  在处理 Word 文档的文本节点时，代码执行了 `let raw_text = e.unescape()?;`。这会将 XML 中的实体字符（例如 `&amp;`、`&lt;`）还原为原始字符（`&`、`<`）。
  但在脱敏处理并写回时，代码直接使用了 `BytesText::new(&masked_text)` [doc/脱敏引擎&核心脱敏优化/core-engine-refinement-v2.plan.md]。**`quick_xml` 的 `BytesText::new` 不会自动执行反向转义！**
  如果用户的文本中包含 `&`（例如 `“张三 & 李四”`），写回后的 XML 将直接包含裸露的 `&` 字符。由于 `&` 在 XML 规范中是实体引用的起始符，**这将直接导致生成的 `.docx` 格式失效，用户用 Microsoft Word 打开文件时会报“文件已损坏，无法打开”的灾难性错误**。
* **修复方案**：写回文本前，必须使用 `quick_xml::escape::escape` 重新将敏感字符进行 XML 安全转义。

#### 2. 被 Serde 遮蔽的 `model_download_urls` 导致下载静默失败
* **病因位置**：`src-tauri/src/core/config.rs` 与 `src-tauri/src/api/system.rs`。
* **技术剖析**：
  `AppSettings` 的 `model_download_urls` 标记了 `#[serde(skip_serializing)]` [src-tauri/src/core/config.rs]。这本来是为了防止该字段被写入本地磁盘的 `settings.yaml`。
  然而，Tauri 的 `get_app_settings` 指令通过 JSON 格式将此结构体序列化后发送给前端。由于 `skip_serializing` 的存在，**发往前端的 JSON 数据中会完全缺失此字段**。前端 React 拿到的 `settings.model_download_urls` 始终为 `undefined` [src/hooks/useModelDownloader.ts]，当用户点击“一键下载”时，在底层会因 `TypeError` 静默报错而无任何界面反应。
* **修复方案**：通过在 Tauri Command 中转换为 `serde_json::Value` 并在内存中动态注入该字段，既能维持不写盘的干净设计，又能将下载 URL 安全投递给前端。

#### 3. 本地状态不持久导致 AI 开关状态严重脱节
* **病因位置**：`src/components/settings/SettingsPage.tsx` 中的 `aiLocalEnabled` 状态。
* **技术剖析**：
  在设置页面中，AI 开关的状态交给了组件的 Local State 管理：`const [aiLocalEnabled, setAiLocalEnabled] = useState(true);` [src/components/settings/SettingsPage.tsx]。
  当用户在这个页面关闭了 AI，并切换到“仪表盘”或“历史记录”选项卡时，**`SettingsPage` 组件会被完全卸载，其 Local State 随之被彻底销毁**。当用户再次返回设置页面时，`aiLocalEnabled` 会被重新初始化为默认的 `true`，而此时**后端的 AI 引擎实际上依然是关闭的**。这造成了严重的 UI 状态不一致。
* **修复方案**：必须舍弃 Local State，直接使用 Zustand 全局状态机中持久留存的 `store.aiEngineStatus` 进行双向响应式绑定。

---

### 二、 安全风险与内存安全审计 (Security & Memory Safety)

#### 1. 多线程环境下 `std::env::set_var` 的安全隐患 (Undefined Behavior)
* **病因位置**：`src-tauri/src/main.rs` 及 `src-tauri/src/infra/ai/ner_engine.rs`。
* **技术剖析**：
  在多线程 Rust 程序中，调用 `std::env::set_var` 是一种**未定义行为（Undefined Behavior）**。因为如果有其他线程在同时通过 `std::env::var` 读取环境变量，会发生没有同步保护的内存竞态（Data Race），导致程序崩溃。
  虽然这通常发生在程序刚启动的阶段，但随着 Tauri 插件和系统后台服务的并行化，风险依然极高。
* **修复方案**：
  * 放弃通过设置 `RAYON_NUM_THREADS` 环境变量来限制线程数，改用 Rayon 官方提供的线程安全构建器：`rayon::ThreadPoolBuilder::new().num_threads(n).build_global()`。
  * `ort` 引擎所需的 `intra_threads` 和 `inter_threads` 应直接在 Session 创建时作为参数注入，不依赖外部环境变量。

#### 2. 系统时钟偏置（Clock Skew）引发的下载鉴权拦截风险
* **病因位置**：`src-tauri/src/core/download_auth.rs` 与 `server.js`。
* **技术剖析**：
  服务端对下载 Token 的校验实施了严格的 1 小时限时：`const ageSec = (Date.now() - timestamp * 1000) / 1000; if (ageSec > TOKEN_TTL_SECS) return null;` [workers/download-proxy/server.js]。
  由于 Token 是由客户端根据其本地的系统时钟（`SystemTime::now()`）生成的，如果用户的电脑时钟不准（例如 timezone 错乱、主板电池没电、时钟偏置超过 1 小时），自建服务器就会无条件判定其 Token 过期并返回 `403 Forbidden`。
* **安全与体验平衡建议**：
  保留现有的 302 鉴权作为第一防线（防滥用、防盗链），但鉴于时钟偏差的广泛存在，在前端 `useModelDownloader.ts` 收到 403 错误时，**自动降级切换到无鉴权的直连 OSS 备用下载链路**，确保极佳的用户可用性。

---

### 三、 性能与效率优化建议 (Performance)

#### 1. `Enigo` 句柄频繁创建的物理开销
* **病因位置**：`src-tauri/src/infra/clipboard/magic_paste.rs`。
* **技术剖析**：
  在 `simulate_paste_keys_blocking` 中，每次模拟安全粘贴都会执行 `let mut enigo = Enigo::new();`。
  在 Windows 和 macOS 下，`Enigo::new()` 需要向系统申请底层句柄并连接显示服务器。频繁创建和销毁句柄不仅带来额外的内存开销，还可能在目标应用较卡顿、用户高频点击时引发输入滞后（Lag）。
* **优化建议**：
  由于 Enigo 并不保证绝对的跨线程安全，不推荐直接将其放入全局 State。但在执行器 `MagicPaster` 内部，可以通过 `thread_local!` 进行单线程句柄缓存，或者通过 `once_cell` 配合互斥锁保证句柄的一次性初始化，降低系统开销。

---

### 四、 核心代码重构方案 (Refactoring)

#### 1. 修复 DOCX XML 字符转义漏洞 (Layer 1 - File System)
修改 `src-tauri/src/infra/fs/processor.rs`：

```rust
// src-tauri/src/infra/fs/processor.rs

/// 🚀 修复版：XML 深度脱敏，对文本节点进行脱敏并执行安全的实体反转义写回
fn mask_xml_content(xml_data: &[u8], engine: &Arc<MaskEngine>) -> Result<Vec<u8>> {
    let mut reader = XmlReader::from_reader(xml_data);
    let mut writer = XmlWriter::new(Cursor::new(Vec::new()));
    let mut buf = Vec::new();

    loop {
        match reader.read_event_into(&mut buf) {
            Ok(Event::Text(e)) => {
                let raw_text = e.unescape()?;
                let masked_bytes = engine.mask_line(raw_text.as_bytes());
                let masked_text = String::from_utf8_lossy(&masked_bytes);
                
                // 🚀 核心修复：对脱敏后的明文进行 XML 实体转义，防止生成损坏的 Word 文档
                let escaped_text = quick_xml::escape::escape(&masked_text);
                writer.write_event(Event::Text(BytesText::from_escaped(escaped_text.as_ref())))?;
            },
            Ok(Event::Eof) => break,
            Ok(e) => {
                writer.write_event(e)?;
            },
            Err(e) => return Err(anyhow::anyhow!("XML 解析错误: {}", e)),
        }
        buf.clear();
    }

    Ok(writer.into_inner().into_inner())
}
```

#### 2. 修复 `get_app_settings` 下载链接隐藏漏洞 (Layer 5 - IPC Commands)
修改 `src-tauri/src/api/system.rs`：

```rust
// src-tauri/src/api/system.rs

/// 获取当前应用配置（自动注入 Worker 代理下载 URL，保证前端可见性）
#[tauri::command]
pub async fn get_app_settings(state: State<'_, AppState>) -> AppResult<serde_json::Value> {
    let mut settings = state.settings.read().clone();
    
    // 清除内存中可能残留的旧 Worker 代理 URL
    settings.model_download_urls.retain(|u| !u.contains(download_auth::WORKER_BASE_URL));
    // 前置插入实时生成的 Worker 代理 URL（含 HMAC 令牌）
    let worker_url = download_auth::generate_worker_url(&state.device_id);
    settings.model_download_urls.insert(0, worker_url);

    // 1. 将 settings 序列化为 JSON Value
    let mut json = serde_json::to_value(&settings)
        .map_err(|e| crate::common::errors::AppError::Internal(e.to_string()))?;

    // 2. 🚀 手动向序列化后的 JSON 字典中注入被 skip_serializing 屏蔽的下载链接
    if let Some(obj) = json.as_object_mut() {
        obj.insert(
            "model_download_urls".to_string(),
            serde_json::to_value(&settings.model_download_urls)
                .map_err(|e| crate::common::errors::AppError::Internal(e.to_string()))?,
        );
    }

    Ok(json)
}
```

#### 3. 修复设置页面 AI 状态丢失 Bug (Layer 6 - Presentational UI)
修改 `src/components/settings/SettingsPage.tsx` 中的 AI 切换绑定逻辑，放弃不持久的 local state：

```tsx
// src/components/settings/SettingsPage.tsx

export default function SettingsPage() {
  const store = useAppStore();
  const [isRecording, setRecording] = useState(false);
  const [showKeyWarn, setShowWarn] = useState(false);
  const [elapsed, setElapsed] = useState(0);
  const [emailCopied, setEmail] = useState(false);
  const [selectedModel, setSelectedModel] = useState<string | null>(null);
  const [modelUnselectLock, setModelUnselectLock] = useState(false);
  const [aiToggling, setAiToggling] = useState(false);
  
  // 🚀 核心修复：直接从全局 store 的 aiEngineStatus 计算出 AI 的当前激活态
  // 避免使用 local state 导致切 Tab 后重置为默认值
  const aiLocalEnabled = useMemo(() => {
    if (!store.aiEngineStatus) return false;
    return store.aiEngineStatus.state === "ready" 
        || store.aiEngineStatus.state === "loading" 
        || store.aiEngineStatus.state === "not_loaded";
  }, [store.aiEngineStatus]);

  const { play } = useAudioFeedback(store.settings.enable_audio_feedback);

  // ... 保持其他 Effect 不变 ...

  const handleAiToggle = async (enabled: boolean) => {
    setAiToggling(true);
    try {
      await store.toggleAiEngine(enabled);
      if (enabled) {
        play("ASCEND");
        await message("AI 引擎已启动，正在加载模型...", { title: "AI 引擎", kind: "info" });
      } else {
        play("DESCEND");
        await message("AI 识别已关闭，将使用规则引擎进行脱敏", { title: "AI 引擎", kind: "info" });
      }
    } catch (e) {
      await message("切换 AI 引擎失败: " + e, { title: "错误", kind: "error" });
    } finally {
      setAiToggling(false);
    }
  };

  // ... 保持其他渲染和绑定逻辑不变 ...
}
```

---

### 五、 审计总结与后续开发展望

在修复上述 3 个高优缺陷与安全隐患后，SafeMask 核心脱敏管道（Layer 1 - Layer 4）已具备极佳的工程鲁棒性。

在接下来的开发中，关于**多端兼容与桌面整合**，为您提供以下持续迭代方向：
1. **多格式文件保续（保格式）脱敏**：在 `FileProcessor` 中针对 Excel 表格和 PDF 的处理目前直接提取了纯文本，如果用户有格式高保真还原的需求，后续可以结合 `docx/xlsx` 的原生布局重构机制提供更加无损的表格和 PDF 影子层脱敏。
2. **多进程剪贴板焦点获取**：在 `handler.rs` 的影子宇宙（Shadow Universe）同步中，可以利用 Windows 的 `GetForegroundWindow` / macOS 的 `NSWorkspace` 获取当前正在执行复制/粘贴的来源应用程序名称，为用户提供诸如“只对特定开发软件（VS Code, PyCharm）激活影子脱敏”的高级隐私控制。