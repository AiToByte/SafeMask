# Plan: 修复 AI 模型加载进度条问题

## 问题分析

### 根本原因

进度条"反复加载、始终无法完成"有 **3 个层面的问题**：

#### 1. 进度条是纯 CSS 动画，与实际加载无关

```css
/* 当前实现：无限循环，与模型加载状态无关 */
@keyframes progress {
  0% { width: 0%; }
  50% { width: 70%; }
  100% { width: 95%; }
}
.animate-progress {
  animation: progress 3s ease-in-out infinite;  /* ← 永远循环 */
}
```

- 进度条从 0% → 95% → 重置 → 0% → 95% → ... 无限循环
- **没有任何逻辑将进度条与实际加载进度关联**

#### 2. 模型加载时间过长（874MB）

- `model.onnx_data` 文件 874MB
- ONNX Runtime 加载需要解析整个文件
- 在普通 PC 上可能需要 **3-10 分钟**
- 用户看到进度条"差一点就完成"是因为 CSS 动画周期刚好 3 秒

#### 3. 加载失败后无法恢复

```rust
// 当前逻辑：失败后永久卡在 Error 状态
ModelState::Error(_) => return false,  // ← 永远不再重试
```

- 如果加载失败（超时、内存不足等），状态变为 `Error`
- 之后每次调用都直接返回 `false`
- **没有重试机制**

### 日志文件位置问题

```rust
let log_file = std::path::PathBuf::from("ai_model_load.log");
// ← 相对路径，写入 src-tauri/ 目录，用户难以找到
```

## 修复方案

### Step 1: 用真实状态替换 CSS 动画

**文件**: `src/components/Settings.vue`

将进度条改为**基于状态的确定性显示**：

| 状态 | 进度条行为 |
|------|-----------|
| `not_loaded` | 不显示进度条 |
| `loading` | 显示**不确定进度**的脉冲动画（不是假进度条） |
| `ready` | 不显示进度条，显示绿色✓ |
| `error` | 不显示进度条，显示红色错误信息 |

### Step 2: 添加加载超时保护

**文件**: `src-tauri/src/core/recognizer/ner_recognizer.rs`

- 添加 5 分钟加载超时
- 超时后自动标记为 `Error`，避免永久卡在 `Loading`

### Step 3: 添加重试机制

**文件**: `src-tauri/src/core/recognizer/ner_recognizer.rs`

- 加载失败后，等待 30 秒自动重试一次
- 或者用户点击"重试"按钮手动触发

### Step 4: 改进日志位置

**文件**: `src-tauri/src/infra/ai/ner_engine.rs`

- 日志写入应用数据目录（而非相对路径）
- 添加加载耗时统计

### Step 5: 前端显示加载详情

**文件**: `src/components/Settings.vue`

- 显示当前加载阶段（"加载模型文件..." / "初始化分词器..." / "加载完成"）
- 显示已用时间
- 添加"重试"按钮（仅 error 状态）

## 文件变更清单

| 文件 | 变更 |
|------|------|
| `src/components/Settings.vue` | 修复进度条，添加重试按钮 |
| `src-tauri/src/core/recognizer/ner_recognizer.rs` | 添加超时和重试逻辑 |
| `src-tauri/src/infra/ai/ner_engine.rs` | 改进日志位置和加载统计 |
| `src-tauri/src/api/system.rs` | 添加重试 API |

## 验证标准

- [ ] 进度条不再无限循环
- [ ] loading 状态显示脉冲动画（非假进度条）
- [ ] ready 状态显示绿色✓
- [ ] error 状态显示错误信息和重试按钮
- [ ] 加载超时后自动标记为 Error
- [ ] 点击重试按钮可重新加载

**WAITING FOR CONFIRMATION**: 是否确认此修复方案？
