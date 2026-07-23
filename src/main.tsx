/**
 * SafeMask Desktop — React entry point
 * Initializes React 19 + Zustand state management + global styles
 */

import { StrictMode } from "react";
import { createRoot } from "react-dom/client";
import App from "./App";
import { applyThemeToDocument } from "./hooks/useTheme";
import { loadPersistedTheme } from "./lib/themeStorage";

// Global styles (Tailwind directives + custom classes)
import "./style.css";

// ── 主题预应用（消除 FOWT · Flash of Wrong Theme） ─────────────────────
// 在 React 挂载前先从 localStorage 同步读取上次的主题并写入 <html data-theme>。
// Store 的初始值也来自同一 storage 层，两者天然一致，首帧无抖动。
// bootstrap 从 Rust 加载 settings 后若发现不一致（例如用户在其他机器改过配置文件），
// useThemeSync 会在下一个 tick 平滑切换。
applyThemeToDocument(loadPersistedTheme());

const rootEl = document.getElementById("root");
if (!rootEl) throw new Error("Root element #root not found");

createRoot(rootEl).render(
  <StrictMode>
    <App />
  </StrictMode>,
);
