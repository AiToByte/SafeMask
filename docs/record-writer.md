# Record Writer — 脱敏记录持久化组件

## 1. 背景与目标

SafeMask 需要将脱敏过程（原始内容 → 脱敏内容）持久化到磁盘，用于：

- **AI 训练数据回源**：收集真实世界的脱敏样本对，作为微调和评测数据集
- **隐私合规审计**：记录已处理的敏感信息类型、数量和脱敏方式
- **用户溯源**：允许用户回顾历史脱敏操作（离线、无需依赖前端内存）

### 设计目标

| 目标 | 说明 |
|------|------|
| **非阻塞** | 记录写入不阻塞脱敏主流程（异步 buffer + 后台刷盘） |
| **插件化** | 通过 `trait RecordWriter` 支持多种输出格式（.md / .json / .csv） |
| **可开关** | 用户在设置界面可启用/禁用，无需重启 |
| **零业务侵入** | 核心脱敏引擎不感知记录器，仅在 infra 层注入 hook |

---

## 2. 整体架构

```
┌─────────────────────┐     ┌──────────────────┐     ┌─────────────────────────┐
│  Clipboard Monitor  │────→│  RecordWriter    │────→│  MarkdownRecordWriter   │
│  handler.rs         │     │  trait + Arc      │     │  (tokio::spawn 后台任务) │
├─────────────────────┤     ├──────────────────┤     ├─────────────────────────┤
│  File Processor     │────→│  write(item)     │     │  mpsc::UnboundedChannel │
│  api/files.rs       │     │  flush()         │     │  + timer 双触发 flush   │
└─────────────────────┘     └──────────────────┘     └───────────┬─────────────┘
                                                                  │
                                                                  ▼
                                                        ┌──────────────────┐
                                                        │  .md 文件 (磁盘)  │
                                                        │  YYYY-MM-DD-NNN  │
                                                        └──────────────────┘
```

**配置驱动**：`AppSettings.record_writer_enabled: bool` → `AppState.record_writer: Arc<RwLock<Option<Arc<dyn RecordWriter>>>>`

---

## 3. 核心设计决策

| 决策 | 选择 | 理由 |
|------|------|------|
| 存储格式 | **Markdown (.md) + YAML front matter** | 人类可读、Git 友好、AI 训练可直接作为语料摄取；无需额外依赖 |
| 写入方式 | **Append-only** | 避免读-改-写竞争；天然支持并发追加 |
| Flush 策略 | **双触发**：BATCH_FLUSH_THRESHOLD（10 条）OR FLUSH_INTERVAL（5 秒） | 平衡延迟与吞吐；高频率下批量写入减少 IOPS |
| 文件分割 | **150 条/文件 + 日期序列号** (YYYY-MM-DD-NNN.md) | 单文件不过大、日期分区便于管理、跨日自动切换 |
| 线程安全模型 | **`Arc<RwLock<Option<Arc<dyn T>>>>`** | 支持运行时热切换、无锁竞争读、parking_lot 不中毒 |
| 异步缓冲区 | **`tokio::sync::mpsc::unbounded_channel`** | 非阻塞 send、调用者可来自任意 async context |
| 运行时协程 | **`tokio::spawn` 常驻后台任务** | 独立于调用者生命周期、自动处理 backlog |
| 异步边界 | **parking_lot guard 不跨 `.await`** | 避免 `Send` 编译错误；统一模式：clone Arc 出来再 await |

---

## 4. 数据流

### 4.1 Clipboard 路径

```
process_change()
  ├── engine.mask_line_with_entities(text) → (Cow<[u8]>, Vec<EntitySpanBrief>)
  └── record_privacy_history(original, masked, entities)
        ├── state.add_history(item)              → 内存环形缓冲（50 条上限）
        ├── app.emit("new-history", item)        → 前端事件
        ├── state.record_writer.read().clone()   → 提取 Arc<dyn RecordWriter>（或 None）
        │     ↑ 注意：必须在 await 前完成，parking_lot RwLockGuard 不是 Send
        └── writer.write(item).await             → 进入 mpsc 通道
              └── 后台 writer_task
                    ├── tokio::select! { channel | timer }
                    ├── 批量积累至 10 条 → do_flush()
                    └── 5 秒超时 → do_flush()
```

### 4.2 File 处理路径

```
process_file_gui(input_path)
  ├── processor::process_file() → ProcessStats { entities, ... }
  │     ├── Text (mmap):  per chunk → Arc<Mutex<Vec>> → extend
  │     ├── DOCX:         per XML text node → Vec → extend
  │     ├── XLSX:         per string cell → Vec → extend
  │     └── PDF:          → Vec → direct assign
  ├── tokio::fs::read_to_string(input)  → 原始内容（截断至 2000 字符）
  ├── tokio::fs::read_to_string(output) → 脱敏内容（同上）
  └── writer.write(MaskHistoryItem { mode: "FILE", entities }).await
        └── 同上 mpsc 通道 → 后台 writer_task
```

### 4.3 Lifecycle

```
启动: main → setup_application
        └── init_record_writer(handle)
              ├── 读取 settings.record_writer_enabled
              ├── true  → MarkdownRecordWriter::new(output_dir)
              │             └── tokio::spawn(writer_task)
              └── false → 跳过（state 中为 None）

运行时切换: update_app_settings
        ├── settings_writer_dirty(old, new) → 检测变化
        └── rebuild_record_writer(app, state)
              ├── old_writer.flush().await   → 排空残余缓冲
              ├── 读取新 enabled 值
              ├── true  → Arc::new(MarkdownRecordWriter::new(...))
              └── false → None
              └── *state.record_writer.write() = new_value
```

---

## 5. 关键实现

### 5.1 RecordWriter Trait

```rust
// infra/record_writer/mod.rs
#[async_trait::async_trait]
pub trait RecordWriter: Send + Sync {
    /// 写入一条记录（非阻塞——内部 buffer，异步 flush）
    async fn write(&self, item: MaskHistoryItem);
    /// 强制刷入所有缓冲记录
    async fn flush(&self);
}
```

- `Send + Sync`：trait 对象必须能跨线程传递和共享引用
- `#[async_trait]`：将 `async fn` 转换为返回 `Pin<Box<dyn Future>>`，使之成为对象安全的方法

### 5.2 MarkdownRecordWriter

```rust
pub struct MarkdownRecordWriter {
    sender: mpsc::UnboundedSender<MaskHistoryItem>,
}

impl MarkdownRecordWriter {
    pub fn new(output_dir: PathBuf) -> Self {
        let (tx, rx) = mpsc::unbounded_channel();
        tokio::spawn(writer_task(output_dir, rx));
        Self { sender: tx }
    }
}
```

**关键设计点**：
- `new()` 即 `tokio::spawn`——调用者不需管理后台任务生命周期
- 公开结构体只持有 `UnboundedSender`——`Send + Sync`、可 clone、可 Arc
- `write()` 只是 `sender.send(item)`——O(1)、非阻塞

### 5.3 后台 writer_task

```rust
async fn writer_task(output_dir: PathBuf, mut rx: mpsc::UnboundedReceiver<MaskHistoryItem>) {
    let mut state = WriterState::new(output_dir);
    let mut buffer: Vec<MaskHistoryItem> = Vec::with_capacity(BATCH_FLUSH_THRESHOLD);
    let mut timer = interval(Duration::from_secs(FLUSH_INTERVAL_SECS));
    timer.tick().await;  // 跳过第一次立即 tick

    loop {
        tokio::select! {
            Some(item) = rx.recv() => {
                buffer.push(item);
                // 一次性 drain 所有待处理消息
                while let Ok(item) = rx.try_recv() {
                    buffer.push(item);
                    if buffer.len() >= BATCH_FLUSH_THRESHOLD {
                        do_flush(&mut state, &mut buffer);
                    }
                }
                if buffer.len() >= BATCH_FLUSH_THRESHOLD {
                    do_flush(&mut state, &mut buffer);
                }
            }
            _ = timer.tick() => {
                if !buffer.is_empty() {
                    do_flush(&mut state, &mut buffer);
                }
            }
        }
    }
}
```

**双触发 flush**：
1. **数量触发**：buffer 达到 10 条立即 flush，同时用 `try_recv()` 贪婪收取所有积压消息
2. **时间触发**：5 秒定时器 tick 时若 buffer 非空则 flush

**永不退出**：`loop` 无限循环，所有错误在 `do_flush` 内部处理。

### 5.4 WriterState：文件滚动状态

```rust
struct WriterState {
    output_dir: PathBuf,
    current_date: String,   // "2026-07-15"
    current_seq: u32,       // 当天文件序号
    current_count: u32,     // 当前文件已写记录数
}
```

`WriterState::new()` 启动时扫描磁盘已有文件，恢复 `current_seq` 和 `current_count`：

```
records/2026/
  2026-07-15-001.md  ← 扫描找到 seq=1，读取记录数 count=42
  2026-07-15-002.md  ← 找到 seq=2，count=150 → 达到上限，下一文件 seq=3
```

文件满 150 条 → `seq += 1, count = 0`
日期变更 → `current_date = today, seq = 1, count = 0`

### 5.5 do_flush：写入核心

```rust
fn do_flush(state: &mut WriterState, buffer: &mut Vec<MaskHistoryItem>) {
    for item in buffer.drain(..) {
        if state.current_count >= MAX_RECORDS_PER_FILE {
            state.current_seq += 1;
            state.current_count = 0;
        }
        let path = build_path(state);
        let record = format_record(&item, state.current_count + 1);
        // 新建文件时写入 YAML front matter
        if is_new_file {
            file.write_all("---\ndate: YYYY-MM-DD\n---\n\n")
        }
        file.write_all(record.as_bytes());
        state.current_count += 1;
    }
}
```

### 5.6 输出文件格式

**目录布局**：

```
{app_data_dir}/records/
├── 2026/
│   ├── 2026-07-15-001.md
│   ├── 2026-07-15-002.md    ← 150 条溢出后
│   └── 2026-07-16-001.md    ← 新日期，seq 归 1
└── 2027/
    └── ...
```

**单文件结构**：

```markdown
---
date: 2026-07-15
---

## 记录 1

### 原始内容
```
我的手机号是13800138000
```

### 脱敏后内容
```
我的手机号是138****8000
```

### 识别实体
| 类型  | 起始 | 结束 | 脱敏值        |
|-------|------|------|---------------|
| PHONE | 6    | 17   | 138****8000   |

### 统计
- 模式: SHADOW
- 实体数: 1
- 时间: 14:30:00

---
```

### 5.7 EntitySpanBrief

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EntitySpanBrief {
    pub start: usize,
    pub end: usize,
    pub entity_type: String,  // "PHONE" | "EMAIL" | "ID_CARD" | ...
    pub mask_label: String,   // "138****8000"
}
```

**收集路径**：

```
engine.mask_line_with_entities(bytes)
  │
  ├── Text (mmap):   Rayon par_bridge → Arc<Mutex<Vec>> → ProcessStats.entities
  ├── DOCX:          mask_xml_content() 遍历 XML events → Vec extend
  ├── XLSX:          逐 cell → engine.mask_line_with_entities → Vec extend
  └── PDF:           全书单次调用 → Vec direct
         │
         └── MaskHistoryItem.entities
               │
               ├── Clipboard:  直接传入 record_privacy_history()
               └── File:       从 ProcessStats.entities.clone() 读取
```

---

## 6. 线程安全模型详解

```
AppState.record_writer: Arc<RwLock<Option<Arc<dyn RecordWriter>>>>
                        │     │        │     │          │
                        │     │        │     │          └─ trait object — 动态分发
                        │     │        │     └──────────── Arc — 共享所有权、轻量 clone
                        │     │        └────────────────  Option — None = 已禁用
                        │     └────────────────────────── parking_lot RwLock — 热切换、不中毒
                        └──────────────────────────────── Arc — 全局单例
```

**关键模式：Clone Arc out of lock before await**

```rust
// HANDLER.RS — ✅ 正确
let writer_opt = state.record_writer.read().clone();  // 1. 读锁 → clone Arc 出来
// 锁在此处隐式释放（guard 离开作用域）
if let Some(writer) = writer_opt {
    writer.write(item).await;  // 2. 安全 await，不持锁
}

// ❌ 如果写成这样会编译错误（parking_lot RwLockGuard 不是 Send）：
// let guard = state.record_writer.read();
// guard.as_ref().unwrap().write(item).await;  // error: `RwLockWriteGuard` cannot be sent between threads safely
```

---

## 7. 错误处理策略

| 场景 | 处理方式 | 后果 |
|------|----------|------|
| 后台 writer 文件 IO 失败 | `error!()` 日志 + `continue` | 单条记录丢失，进程不 crash |
| 创建目录失败 | `error!()` 日志 + `continue` | 该次 flush 所有记录丢失 |
| sender 发送后 receiver 已挂 | `let _ = sender.send()` → 静默丢弃 | 记录丢失，不 panic |
| 启动时 settings 读取失败 | propagate `Result` 到 `setup_application` | 应用启动失败（属于致命错误） |
| 通道满 | 使用 unbounded channel → 无背压 | 理论上不会满（受内存限制） |

---

## 8. 单元测试

```rust
// 3 个 inline test，位于 markdown.rs #[cfg(test)] mod
test_format_record  → 验证 markdown 模板包含所有段
test_build_path     → 验证路径格式包含日期+序列号
test_today_str      → 验证日期格式 YYYY-MM-DD
```

运行：`cargo test -p SafeMask -- markdown::tests`

---

## 9. 配置与扩展

### 配置

```rust
// core/config.rs
pub struct AppSettings {
    // ... 其他设置 ...
    pub record_writer_enabled: bool,  // default: false
}
```

前端：`SettingToggle` with `FileText` icon in Kernel section。

### 扩展：实现 JSON 写入器

```rust
pub struct JsonRecordWriter {
    sender: mpsc::UnboundedSender<MaskHistoryItem>,
}

#[async_trait]
impl RecordWriter for JsonRecordWriter {
    async fn write(&self, item: MaskHistoryItem) {
        let _ = self.sender.send(item);
    }
    async fn flush(&self) { /* ... */ }
}
```

只需修改 `rebuild_record_writer()` 或 `init_record_writer()` 中的具体类型即可切换输出格式。

---

## 10. 涉及文件清单

| 文件 | 作用 |
|------|------|
| `infra/record_writer/mod.rs` | Trait 定义 + re-export |
| `infra/record_writer/markdown.rs` | Markdown 实现（271 行） |
| `common/state.rs` | `AppState.record_writer` 字段 + `EntitySpanBrief` + `MaskHistoryItem` |
| `core/config.rs` | `AppSettings.record_writer_enabled` |
| `main.rs` | `init_record_writer()` 启动初始化 |
| `infra/clipboard/handler.rs` | Clipboard 侧 hook：`record_privacy_history()` |
| `api/files.rs` | File 处理侧 hook |
| `api/system.rs` | `rebuild_record_writer()` 运行时热切换 |
| `infra/fs/processor.rs` | Entity 埋点：`ProcessStats.entities` + `mask_line_with_entities` |
| `Cargo.toml` | 依赖：`async-trait = "0.1"` |
