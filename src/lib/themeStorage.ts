/**
 * 主题持久化存储层 —— 封装 localStorage IO 与容错逻辑。
 *
 * 作用：
 *   • 隔离 storage 读写细节，让上层 (hooks/components) 只关心业务
 *   • 统一处理不可用/损坏/被 QuotaExceeded 拒绝等边界
 *   • 自动清理非法值，防止陈旧数据反复触发规范化开销
 *
 * 失败策略：静默降级。存储失败不会阻塞主题切换本身，仅牺牲下次启动的预应用。
 */

import {
  DEFAULT_THEME_ID,
  THEME_STORAGE_KEY,
  isValidTheme,
  normalizeThemeId,
  type ThemeId,
} from "./themes";

/**
 * 读取上次持久化的主题 ID。
 * 未存储、值非法、或 storage 不可用时返回默认主题。
 * 若发现值非法会顺手清理，避免陈旧脏数据滞留。
 */
export function loadPersistedTheme(): ThemeId {
  try {
    const raw = localStorage.getItem(THEME_STORAGE_KEY);
    if (raw === null) return DEFAULT_THEME_ID;
    if (isValidTheme(raw)) return raw;

    // 发现非法值 —— 顺手清理，避免下次继续走 normalize 分支
    localStorage.removeItem(THEME_STORAGE_KEY);
    return DEFAULT_THEME_ID;
  } catch {
    // storage 不可用（隐私模式、SecurityError 等）
    return DEFAULT_THEME_ID;
  }
}

/**
 * 持久化主题 ID。写入前先 normalize，杜绝非法值污染 storage。
 * 失败静默返回 false，成功返回 true (供调试/测试使用)。
 */
export function savePersistedTheme(themeId: ThemeId): boolean {
  try {
    localStorage.setItem(THEME_STORAGE_KEY, normalizeThemeId(themeId));
    return true;
  } catch {
    return false;
  }
}

/**
 * 清空持久化的主题（例如"恢复默认设置"操作）。
 * 静默失败。
 */
export function clearPersistedTheme(): void {
  try {
    localStorage.removeItem(THEME_STORAGE_KEY);
  } catch {
    // ignore
  }
}
