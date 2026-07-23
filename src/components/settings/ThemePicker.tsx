import { useCallback, useRef, type KeyboardEvent } from "react";
import { Check } from "lucide-react";
import { message } from "@tauri-apps/plugin-dialog";
import { useAppStore } from "@/hooks/useAppStore";
import { useAudioFeedback } from "@/hooks/useAudioFeedback";
import { cn } from "@/lib/utils";
import { THEMES, normalizeThemeId, type Theme, type ThemeId } from "@/lib/themes";

/**
 * ThemePicker — 大型可视化主题选择器
 *
 * 灵感来源：Vercel Dashboard / Linear / GitHub 主题预览卡
 *
 * 每张卡片是一个 mini UI mockup，用真实的主题色渲染缩略界面
 * 让用户在切换前就能预见效果。
 *
 * 无障碍性：完整 WAI-ARIA radiogroup + roving tabindex
 *   • Tab / Shift+Tab       进入/离开选择器
 *   • ← / ↑                 上一项并即时切换
 *   • → / ↓                 下一项并即时切换
 *   • Home / End            首/末项
 *   • Space / Enter         触发当前项（重试用）
 */
export default function ThemePicker() {
  const currentTheme = useAppStore((s) => normalizeThemeId(s.settings.theme));
  const setTheme = useAppStore((s) => s.setTheme);
  const audioEnabled = useAppStore((s) => s.settings.enable_audio_feedback);
  const { play } = useAudioFeedback(audioEnabled);

  // roving tabindex 所需的 DOM 引用表
  const optionRefs = useRef<Record<string, HTMLButtonElement | null>>({});

  const applyTheme = useCallback(
    async (target: ThemeId): Promise<void> => {
      if (target === currentTheme) return;
      play("CLICK");
      try {
        await setTheme(target);
      } catch (err) {
        await message(`主题同步失败: ${err}`, { title: "错误", kind: "error" });
      }
    },
    [currentTheme, play, setTheme],
  );

  const handleKeyDown = useCallback(
    (event: KeyboardEvent<HTMLDivElement>) => {
      const currentIndex = THEMES.findIndex((t) => t.id === currentTheme);
      if (currentIndex < 0) return;

      let nextIndex: number | null = null;
      switch (event.key) {
        case "ArrowRight":
        case "ArrowDown":
          nextIndex = (currentIndex + 1) % THEMES.length;
          break;
        case "ArrowLeft":
        case "ArrowUp":
          nextIndex = (currentIndex - 1 + THEMES.length) % THEMES.length;
          break;
        case "Home":
          nextIndex = 0;
          break;
        case "End":
          nextIndex = THEMES.length - 1;
          break;
        default:
          return;
      }

      event.preventDefault();
      const nextTheme = THEMES[nextIndex];
      optionRefs.current[nextTheme.id]?.focus();
      void applyTheme(nextTheme.id);
    },
    [applyTheme, currentTheme],
  );

  return (
    <div className="space-y-6">
      {/* 标题区 */}
      <div className="flex items-end justify-between px-1">
        <div>
          <h3 className="text-lg font-bold text-[color:var(--text-primary)] tracking-tight">
            外观主题
          </h3>
          <p className="text-xs text-[color:var(--text-muted)] mt-1">
            所有界面组件将随主题即时切换 · 支持键盘方向键预览
          </p>
        </div>
        <div className="text-[10px] font-mono uppercase tracking-[0.3em] text-[color:var(--text-muted)]">
          {THEMES.length} 个主题
        </div>
      </div>

      {/* 主题卡片网格 */}
      <div
        role="radiogroup"
        aria-label="选择应用主题"
        onKeyDown={handleKeyDown}
        className="grid grid-cols-1 md:grid-cols-2 gap-5"
      >
        {THEMES.map((theme) => (
          <ThemeCard
            key={theme.id}
            theme={theme}
            selected={theme.id === currentTheme}
            onSelect={applyTheme}
            registerRef={(el) => {
              optionRefs.current[theme.id] = el;
            }}
          />
        ))}
      </div>
    </div>
  );
}

// ── Theme Card ───────────────────────────────────────────────────────────

interface ThemeCardProps {
  theme: Theme;
  selected: boolean;
  onSelect: (id: ThemeId) => void;
  registerRef: (el: HTMLButtonElement | null) => void;
}

/**
 * 单个主题预览卡片。
 *
 * 结构：
 *   ┌─────────────────────────────────┐
 *   │ ▮ Mini Mockup Preview           │  ← 用真实主题色渲染的缩略界面
 *   │   ├─ Sidebar / Header / Card    │
 *   │   └─ Accent chip                │
 *   ├─────────────────────────────────┤
 *   │ Label · Tagline                 │  ← 元数据
 *   └─────────────────────────────────┘
 */
function ThemeCard({ theme, selected, onSelect, registerRef }: ThemeCardProps) {
  const { preview } = theme;

  return (
    <button
      ref={registerRef}
      type="button"
      role="radio"
      aria-checked={selected}
      tabIndex={selected ? 0 : -1}
      onClick={() => onSelect(theme.id)}
      className={cn(
        "group relative overflow-hidden rounded-3xl text-left",
        "border transition-all duration-300 ease-out",
        "hover:-translate-y-0.5 hover:shadow-2xl",
        "focus-visible:outline-none",
        selected
          ? "border-2 shadow-2xl scale-[1.01]"
          : "border-white/[0.06] bg-white/[0.015] hover:bg-white/[0.03] hover:border-white/[0.12]",
      )}
      style={
        selected
          ? {
              borderColor: preview.accent,
              boxShadow: `0 20px 40px -20px ${preview.accent}30, 0 0 0 1px ${preview.accent}20`,
            }
          : undefined
      }
    >
      {/* Mini UI Mockup 预览区 */}
      <div
        className="relative p-4 h-40 overflow-hidden"
        style={{ backgroundColor: preview.bg }}
        aria-hidden="true"
      >
        {/* 装饰性背景光晕 */}
        <div
          className="absolute -top-8 -right-8 w-32 h-32 rounded-full blur-3xl opacity-30 transition-opacity duration-500 group-hover:opacity-50"
          style={{ backgroundColor: preview.accent }}
        />

        <div className="relative flex gap-2 h-full">
          {/* Mini Sidebar */}
          <div
            className="w-8 rounded-lg flex flex-col items-center py-2 gap-1.5 shrink-0"
            style={{ backgroundColor: preview.sidebar }}
          >
            <div
              className="w-4 h-4 rounded-md"
              style={{ backgroundColor: preview.accent }}
            />
            <div className="w-4 h-1 rounded-full bg-white/10" />
            <div className="w-4 h-1 rounded-full bg-white/5" />
            <div className="w-4 h-1 rounded-full bg-white/5" />
          </div>

          {/* Mini Main Content */}
          <div className="flex-1 flex flex-col gap-1.5 min-w-0">
            {/* Mini Header */}
            <div
              className="h-6 rounded-lg flex items-center justify-between px-2"
              style={{ backgroundColor: preview.surface }}
            >
              <div className="flex gap-1">
                <div
                  className="w-1.5 h-1.5 rounded-full"
                  style={{ backgroundColor: preview.accent }}
                />
                <div className="w-8 h-1 rounded-full bg-white/15" />
              </div>
              <div className="w-6 h-2 rounded-sm bg-white/10" />
            </div>

            {/* Mini Cards */}
            <div className="flex-1 grid grid-cols-3 gap-1.5">
              <div
                className="rounded-lg p-1.5 flex flex-col justify-between border"
                style={{
                  backgroundColor: preview.surface,
                  borderColor: `${preview.accent}30`,
                }}
              >
                <div
                  className="w-3 h-3 rounded"
                  style={{ backgroundColor: `${preview.accent}40` }}
                />
                <div
                  className="w-full h-1 rounded-full"
                  style={{ backgroundColor: preview.accent }}
                />
              </div>
              <div
                className="rounded-lg p-1.5 flex flex-col justify-between"
                style={{ backgroundColor: preview.surface }}
              >
                <div className="w-3 h-3 rounded bg-white/10" />
                <div className="w-full h-1 rounded-full bg-white/15" />
              </div>
              <div
                className="rounded-lg p-1.5 flex flex-col justify-between"
                style={{ backgroundColor: preview.surface }}
              >
                <div className="w-3 h-3 rounded bg-white/10" />
                <div className="w-full h-1 rounded-full bg-white/15" />
              </div>
            </div>

            {/* Mini Footer strip */}
            <div
              className="h-4 rounded-lg flex items-center px-2 gap-1"
              style={{ backgroundColor: preview.surface }}
            >
              <div
                className="h-1.5 w-1.5 rounded-full"
                style={{
                  backgroundColor: preview.accent,
                  boxShadow: `0 0 6px ${preview.accent}`,
                }}
              />
              <div className="flex-1 h-1 rounded-full bg-white/10" />
            </div>
          </div>
        </div>

        {/* 选中态勾选徽章 */}
        {selected && (
          <div
            className="absolute top-3 right-3 w-6 h-6 rounded-full flex items-center justify-center shadow-lg"
            style={{
              backgroundColor: preview.accent,
              boxShadow: `0 4px 12px ${preview.accent}60`,
            }}
          >
            <Check size={14} className="text-white" strokeWidth={3} />
          </div>
        )}
      </div>

      {/* 元数据条 */}
      <div
        className="px-5 py-4 backdrop-blur-sm border-t border-white/[0.03]"
        style={{ backgroundColor: "color-mix(in srgb, var(--bg-card) 88%, transparent)" }}
      >
        <div className="flex items-center justify-between gap-3">
          <div className="min-w-0">
            <div className="text-sm font-bold text-[color:var(--text-primary)] truncate">
              {theme.label}
            </div>
            <div className="text-[11px] text-[color:var(--text-muted)] mt-0.5 truncate">
              {theme.tagline}
            </div>
          </div>

          {/* 色板缩略 */}
          <div className="flex gap-1 shrink-0" aria-hidden="true">
            <div
              className="w-4 h-4 rounded-full border border-white/10 shadow-inner"
              style={{ backgroundColor: preview.accent }}
              title="强调色"
            />
            <div
              className="w-4 h-4 rounded-full border border-white/10 shadow-inner"
              style={{ backgroundColor: preview.bg }}
              title="背景色"
            />
          </div>
        </div>
      </div>
    </button>
  );
}
