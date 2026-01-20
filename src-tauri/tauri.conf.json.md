由于你已经将前端升级到了 **Vue 3 + Vite** 架构，原有的 `tauri.conf.json` 需要针对 **开发服务器地址**、**构建产物路径** 以及 **Tauri v2 的安全权限体系** 进行大幅度优化。

以下是为你优化后的完整版本，它能够完美适配 Vite 的开发流，并提升了桌面的交互质感。

### 优化后的 `safemask/src-tauri/tauri.conf.json`

```json
{
  "$schema": "https://schema.tauri.app/config/2",
  "productName": "SafeMask",
  "version": "1.0.0",
  "identifier": "com.aitobyte.safemask",
  "build": {
    "beforeDevCommand": "npm run dev",
    "beforeBuildCommand": "npm run build",
    "devUrl": "http://localhost:5173",
    "frontendDist": "../dist"
  },
  "app": {
    "withGlobalTauri": true,
    "windows": [
      {
        "title": "SafeMask - 极致性能隐私脱敏控制台",
        "width": 1024,
        "height": 720,
        "minWidth": 800,
        "minHeight": 600,
        "resizable": true,
        "fullscreen": false,
        "center": true,
        "theme": "Dark"
      }
    ],
    "security": {
      "csp": null
    }
  },
  "bundle": {
    "active": true,
    "targets": "all",
    "icon": [
      "icons/32x32.png",
      "icons/128x128.png",
      "icons/128x128@2x.png",
      "icons/icon.icns",
      "icons/icon.ico"
    ],
    "publisher": "AiToByte"
  },
  "plugins": {
    "opener": {
      "default": true
    },
    "dialog": {
      "default": true
    },
    "notification": {
      "default": true
    },
    "global-shortcut": {
      "default": true
    }
  }
}
```

---

### 核心改进说明（针对 Vue 3 升级）：

1.  **构建流程对齐 (`build` 节点)**：
    *   **`beforeDevCommand`**: 设置为 `npm run dev`。当你运行 `cargo tauri dev` 时，它会自动帮你启动 Vite 开发服务器。
    *   **`devUrl`**: 设置为 `http://localhost:5173`。这是 Vite 默认的预览地址，Tauri 会将 Webview 挂载到该地址。
    *   **`frontendDist`**: 修改为 `../dist`。Vue 3 打包后的默认输出目录是 `dist` 而非原来的 `src`。

2.  **窗体质感提升 (`windows` 节点)**：
    *   **尺寸调整**: 将默认尺寸增大到 `1024x720`，更适合展示仪表盘和复杂的统计图表。
    *   **限制最小尺寸**: 设置 `minWidth` 和 `minHeight`，防止用户缩放窗体过小导致 Vue 组件布局崩坏。
    *   **主题锁定**: 强制 `Dark` 主题，确保界面始终符合你设计的深色工业风。

3.  **插件显式声明 (`plugins` 节点)**：
    *   Tauri v2 要求在配置文件中开启对应的插件。我已经帮你加上了 `dialog`（文件选择）、`notification`（脱敏成功通知）和 `global-shortcut`（热键功能）。

---

### ⚠️ 必须配套执行的操作

在 Tauri v2 中，仅仅修改 `tauri.conf.json` 还是不够的，你还需要确保**权限能力（Capabilities）**文件已同步更新。

请检查 `safemask/src-tauri/capabilities/default.json`，确保包含以下权限，否则 Vue 前端无法调用 Rust 命令：

```json
{
  "$schema": "../gen/schemas/desktop-schema.json",
  "identifier": "default",
  "description": "Capability for the main window",
  "windows": ["main"],
  "permissions": [
    "core:default",
    "opener:default",
    "dialog:allow-open",
    "notification:allow-notify",
    "global-shortcut:allow-is-registered",
    "global-shortcut:allow-register",
    "core:event:allow-listen",
    "core:event:allow-emit"
  ]
}
```

### 接下来建议执行：
1.  **清理缓存**：`cargo clean`（特别是你修改了 `identifier` 后）。
2.  **正式运行**：`npx tauri dev`。

现在的配置已经完全工程化，足以支撑你后续增加的所有 Vue 3 功能。