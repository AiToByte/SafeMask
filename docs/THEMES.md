# SafeMask 主题系统

本文档面向想为 SafeMask 添加新主题、或修改现有主题外观的贡献者。

---

## 架构总览

```
┌──────────────────────────────────────────────────────────────────┐
│                       用户交互 (ThemePicker.tsx)                  │
│                              │                                    │
│                              ▼                                    │
│              store.setTheme()  ← 乐观更新 + 竞态防护              │
│                              │                                    │
│              ┌───────────────┴──────────────┐                     │
│              ▼                              ▼                     │
│   MaskAPI.updateSettings()         useThemeSync() 观察 store     │
│              │                              │                     │
│              ▼                              ▼                     │
│      Rust settings.yaml         applyThemeToDocument()            │
│      (跨会话持久化)                          │                     │
│                                  ┌──────────┴──────────┐          │
│                                  ▼                     ▼          │
│                        <html data-theme>       localStorage       │
│                                  │             (首屏预应用)        │
│                                  ▼                                │
│                          CSS 变量级联渲染                          │
└──────────────────────────────────────────────────────────────────┘
```

## 文件分层

| 文件 | 职责 |
|------|------|
| `src/lib/themes.ts` | 主题元数据、类型定义、规范化辅助（**纯数据，无副作用**） |
| `src/lib/themeStorage.ts` | localStorage IO 与容错（**纯副作用，无 React**） |
| `src/hooks/useTheme.ts` | React hooks (`useThemeSync`, `useCurrentTheme`) 与 DOM 应用 |
| `src/hooks/useAppStore.ts` | Zustand store 中的 `setTheme` action，含竞态保护 |
| `src/components/settings/ThemePicker.tsx` | 主题选择器 UI，支持键盘 radiogroup 导航 |
| `src/style.css` | CSS 变量定义（每个主题一个 `[data-theme="<id>"]` 块） |
| `src/main.tsx` | 首屏预应用，消除 FOWT (Flash of Wrong Theme) |
| `src-tauri/src/core/config.rs` | `AppSettings.theme` 字段（持久化到 YAML） |

---

## 添加新主题（3 步）

假设要添加一个"海洋主题" (`ocean`)。

### 步骤 1 — 注册元数据

编辑 `src/lib/themes.ts`，在 `THEMES` 数组末尾追加：

```typescript
{
  id: 'ocean',
  label: '海洋主题',
  description: '冷蓝深海 · 静谧沉浸风格',
  accentColor: '#0891b2',   // 主题选择器预览用主色
  bgColor: '#0a1520',       // 主题选择器预览用背景色
},
```

**类型系统会自动感知新主题**：`ThemeId` 联合类型立刻扩展为 `'default' | 'claude' | 'ocean'`，
所有类型守卫（`isValidTheme` / `normalizeThemeId`）无需任何改动。

### 步骤 2 — 定义 CSS 变量

编辑 `src/style.css`，复制现有主题块并修改颜色：

```css
[data-theme="ocean"] {
  --bg-root:     #0a1520;
  --bg-card:     #0e1a26;
  --bg-sidebar:  #0b1622;
  --bg-input:    #071018;
  --bg-elevated: #12212f;
  --bg-glass:    rgba(10, 21, 32, 0.82);

  --accent:         #0891b2;
  --accent-rgb:     8, 145, 178;
  --accent-bright:  #22b8d3;
  --accent-dim:     rgba(8, 145, 178, 0.15);
  --accent-border:  rgba(8, 145, 178, 0.30);

  --ring-offset-bg: #0a1520;
  --ring-color:     rgba(8, 145, 178, 0.6);

  --scrollbar-thumb:       rgba(8, 145, 178, 0.15);
  --scrollbar-thumb-hover: rgba(8, 145, 178, 0.25);
}
```

### 步骤 3 — 验证

```bash
npm run build          # 前端类型检查 + 构建
cargo test -p SafeMask # Rust 单元测试
npm run tauri dev      # 打开设置页面 → 外观主题 → 选择"海洋"
```

完成。

---

## 变量参考

所有主题必须完整定义以下变量。缺失的变量会级联使用 `:root` 中的默认主题值，
可能导致视觉不一致。

### 背景层 (`--bg-*`)

| 变量 | 用途 | 建议规律 |
|------|------|---------|
| `--bg-root` | 应用根背景 | 最深 |
| `--bg-card` | 卡片表面 | 比 root 略亮 |
| `--bg-sidebar` | 左侧导航栏 | 接近 root |
| `--bg-input` | 输入框、textarea | 通常比 root 略深 |
| `--bg-elevated` | 悬浮元素（tooltip、popover、按钮） | 明显比 card 亮 |
| `--bg-glass` | 玻璃拟态半透明基色 | 需含 alpha 通道 |

### 强调色 (`--accent-*`)

| 变量 | 用途 |
|------|------|
| `--accent` | 品牌基色 |
| `--accent-rgb` | 上述颜色的 RGB 分量，逗号分隔 (供 `rgba()` 用) |
| `--accent-bright` | 高亮态（hover、slider thumb 头部） |
| `--accent-dim` | 弱化态（选中项半透明背景） |
| `--accent-border` | 强调边框 |

### 焦点环 (`--ring-*`)

| 变量 | 用途 |
|------|------|
| `--ring-offset-bg` | 焦点环外侧的偏移色（通常等于 `--bg-root`） |
| `--ring-color` | 焦点环本体色（通常是 `--accent` 的半透明版） |

### 滚动条 (`--scrollbar-*`)

| 变量 | 用途 |
|------|------|
| `--scrollbar-thumb` | 滚动条滑块常态 |
| `--scrollbar-thumb-hover` | 滑块悬停态 |

---

## 设计原则

1. **CSS 变量是真相，TypeScript 只存元数据**  
   实际渲染颜色只出现在 CSS，`themes.ts` 中的 `accentColor`/`bgColor` **仅供主题选择器预览用**。
   如果只想调整颜色而不改元数据，只需改 CSS 一处。

2. **宽泛输入，严格输出**  
   IPC 边界（`AppSettings.theme: string`）保持宽泛，防止旧版本用户降级失败。
   前端在渲染前用 `normalizeThemeId()` 收敛，非法值一律回退到 `default`。

3. **首屏预应用消除 FOWT**  
   `main.tsx` 在 React 渲染前从 localStorage 读取主题应用到 `<html>`，
   Zustand store 初始值也来自同一 storage，两者天然一致 → 首帧就是正确主题。

4. **过渡尊重用户偏好**  
   `html`/`body` 有 220ms 背景色过渡；`prefers-reduced-motion` 用户跳过过渡。

5. **无障碍性优先**  
   ThemePicker 遵循 WAI-ARIA `radiogroup` 模式：Tab 进入组，箭头键切换，
   Home/End 跳首末，roving tabindex 保证只有选中项在 Tab 序列内。

---

## 常见问题

**Q: 我改了 `THEMES` 但类型没更新？**  
A: 确保数组尾部有 `as const satisfies readonly ThemeConfig[]`。IDE 可能需要重启 TS server。

**Q: 首屏切换有闪烁？**  
A: 检查 `main.tsx` 是否在 React 渲染前调用了 `applyThemeToDocument(loadPersistedTheme())`。
如果自定义了入口，确保这行在 `createRoot().render()` 之前。

**Q: 主题切换失败会怎样？**  
A: `setTheme` 会在后端 IPC 失败时把 store 回滚到旧主题（compare-and-swap 语义），
UI 层收到 throw 后弹出对话框。用户看到的最终状态与后端持久化保持一致。

**Q: 想在组件里读当前主题元数据？**  
A: 使用 `useCurrentTheme()`：
```tsx
import { useCurrentTheme } from "@/hooks/useTheme";
const theme = useCurrentTheme();
return <span style={{ color: theme.accentColor }}>{theme.label}</span>;
```
