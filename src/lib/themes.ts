/**
 * 主题注册中心 — 单一事实来源 (Single Source of Truth)
 *
 * ═══════════════════════════════════════════════════════════════════════
 *  如何扩展新主题（3 步）
 * ═══════════════════════════════════════════════════════════════════════
 *   1. 在下方 `THEMES` 数组中追加一项字面量（含预览元数据）
 *   2. 在 `src/style.css` 添加对应的 `[data-theme="<id>"] { ... }` CSS 变量块
 *   3. 无需其他改动 — 类型系统会自动推导出新的 `ThemeId` 联合类型
 *
 * 详细指南参见 `docs/THEMES.md`。
 *
 * ═══════════════════════════════════════════════════════════════════════
 *  类型设计说明
 * ═══════════════════════════════════════════════════════════════════════
 *   • `ThemeConfig` —— 宽结构约束（`id: string`），用于外部接口/参数类型
 *   • `Theme`       —— 从 THEMES 数组推导的**字面量类型**，供内部消费
 *   • `ThemeId`     —— 所有合法 id 的联合类型
 */

// ── Structural Contract ──────────────────────────────────────────────────

/**
 * 单个主题选项的预览色板 —— 供选择器 UI 内嵌 mini-mockup 使用。
 * 与 CSS 变量解耦，允许在预览卡片中呈现"主题肖像"而不必依赖已加载的 CSS。
 */
export interface ThemePreview {
  /** 大背景色（用于卡片外层） */
  readonly bg: string;
  /** 表面/卡片色（次级层） */
  readonly surface: string;
  /** 侧边栏 / 导航区色 */
  readonly sidebar: string;
  /** 强调色（按钮/图标高光） */
  readonly accent: string;
  /** 主文字色（用于 mockup 内的文字块） */
  readonly text: string;
  /** 弱化文字色（用于 mockup 内次级文字） */
  readonly textMuted: string;
}

/**
 * 主题配置项的宽结构约束。仅用于 `satisfies` 与外部参数标注。
 */
export interface ThemeConfig {
  /** 稳定的主题标识符，写入 settings.yaml 与 DOM 的 data-theme 属性 */
  readonly id: string;
  /** 界面显示的主题名称 */
  readonly label: string;
  /** 一行标语（简短，出现在标题下方） */
  readonly tagline: string;
  /** 详细描述（用于卡片脚注/tooltip） */
  readonly description: string;
  /** 预览色板 — 供 mini-mockup 使用 */
  readonly preview: ThemePreview;
}

// ── Theme Registry ───────────────────────────────────────────────────────

/**
 * 已注册的所有主题。顺序即选择器中的显示顺序。
 * 首项被视为"回退主题"，非法输入统一退化到它。
 */
export const THEMES = [
  {
    id: 'default',
    label: 'Amber Dark',
    tagline: '工业级暗黑',
    description: '深黑琥珀基调，高对比度，适合长时间专注工作',
    preview: {
      bg: '#0c0b0a',
      surface: '#141210',
      sidebar: '#0c0c0e',
      accent: '#f59e0b',
      text: '#fef3c7',
      textMuted: '#71717a',
    },
  },
  {
    id: 'claude',
    label: 'Claude',
    tagline: 'Anthropic 品牌配色',
    description: 'Claude.ai 官方视觉语言，暖米色纸感与 Book Cloth 珊瑚橙',
    preview: {
      bg: '#F5F1E8',
      surface: '#FAF7EF',
      sidebar: '#EFEADD',
      accent: '#C96442',
      text: '#3D3929',
      textMuted: '#93896C',
    },
  },
] as const satisfies readonly ThemeConfig[];

// ── Derived Types ────────────────────────────────────────────────────────

export type Theme = (typeof THEMES)[number];
export type ThemeId = Theme['id'];

// ── Constants ────────────────────────────────────────────────────────────

/** 默认（回退）主题 ID */
export const DEFAULT_THEME_ID: ThemeId = THEMES[0].id;

/** DOM 上承载当前主题标识的属性名 */
export const THEME_DATA_ATTRIBUTE = 'data-theme';

/** localStorage key —— 用于首屏预应用，消除 FOWT (Flash of Wrong Theme) */
export const THEME_STORAGE_KEY = 'safemask.theme';

// ── Lookup Table ─────────────────────────────────────────────────────────

const THEMES_BY_ID: Readonly<Record<ThemeId, Theme>> = Object.freeze(
  THEMES.reduce(
    (acc, theme) => {
      acc[theme.id] = theme;
      return acc;
    },
    {} as Record<ThemeId, Theme>,
  ),
);

// ── Helpers ──────────────────────────────────────────────────────────────

export function isValidTheme(id: unknown): id is ThemeId {
  return typeof id === 'string' && Object.prototype.hasOwnProperty.call(THEMES_BY_ID, id);
}

export function normalizeThemeId(id: unknown): ThemeId {
  return isValidTheme(id) ? id : DEFAULT_THEME_ID;
}

export function getTheme(id: unknown): Theme {
  return THEMES_BY_ID[normalizeThemeId(id)];
}

/**
 * @deprecated 使用 `getTheme` 代替。
 */
export const getThemeConfig = getTheme;
