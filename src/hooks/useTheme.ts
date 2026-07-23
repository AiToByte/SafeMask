/**
 * 主题相关的 React hooks 与 DOM 应用副作用。
 *
 * 分层：
 *   themes.ts        —— 主题元数据 + 类型 (纯数据)
 *   themeStorage.ts  —— localStorage IO (纯副作用)
 *   useTheme.ts      —— 组合两者 + React 生命周期集成 ← (本文件)
 */

import { useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { useAppStore } from "@/hooks/useAppStore";
import {
  THEME_DATA_ATTRIBUTE,
  getTheme,
  normalizeThemeId,
  type Theme,
  type ThemeId,
} from "@/lib/themes";
import { loadPersistedTheme, savePersistedTheme } from "@/lib/themeStorage";

/** 各主题对应的原生窗口背景色（同步到 Tauri 标题栏） */
const THEME_WINDOW_BG: Record<ThemeId, string> = {
  default: "#0c0b0a",
  claude: "#F5F1E8",
};

// ── DOM Application ──────────────────────────────────────────────────────

/**
 * 把主题 ID 应用到 `<html data-theme="...">` 并持久化到 localStorage。
 * 同步更新 Tauri 窗口背景色与系统标题栏主题（Light/Dark + Windows DWM）。
 */
export function applyThemeToDocument(themeId: unknown): ThemeId {
  const normalized = normalizeThemeId(themeId);
  document.documentElement.setAttribute(THEME_DATA_ATTRIBUTE, normalized);
  savePersistedTheme(normalized);

  const win = getCurrentWindow();
  const bg = THEME_WINDOW_BG[normalized] ?? THEME_WINDOW_BG.default;

  // 前端 API：系统主题 + 窗口底色
  void win.setTheme(normalized === "claude" ? "light" : "dark").catch(() => {});
  void win.setBackgroundColor(bg).catch(() => {});

  // 后端 DWM：强制标题栏背景/文字色（修复 Win11 标题栏仍黑的问题）
  void invoke("apply_window_chrome", { themeId: normalized }).catch(() => {});

  return normalized;
}

/**
 * 读取当前 `<html data-theme>` 属性值 (规范化后)。
 */
export function readDocumentTheme(): ThemeId {
  return normalizeThemeId(document.documentElement.getAttribute(THEME_DATA_ATTRIBUTE));
}

// ── Bootstrap ────────────────────────────────────────────────────────────

export const readPersistedTheme = loadPersistedTheme;

// ── React Hooks ──────────────────────────────────────────────────────────

/**
 * 副作用：订阅 store 中的 `settings.theme`，自动同步到 DOM / localStorage / 原生窗口。
 */
export function useThemeSync(): void {
  const theme = useAppStore((s) => s.settings.theme);
  useEffect(() => {
    const normalized = normalizeThemeId(theme);
    applyThemeToDocument(normalized);
  }, [theme]);
}

/**
 * 返回当前解析后的完整主题元数据（永不返回 undefined）。
 */
export function useCurrentTheme(): Theme {
  const themeId = useAppStore((s) => s.settings.theme);
  return getTheme(themeId);
}
