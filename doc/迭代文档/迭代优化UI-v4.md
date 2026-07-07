这是一个非常经典且关键的**响应式布局（Responsive Layout）**问题。

### 🔍 根本原因分析

目前导致右侧模式胶囊被覆盖的根本原因有两点：
1. **中部空白容器的硬宽限制**：
   在 `Header.tsx` 中，中部的 Feedback 容器被设置了 `min-w-[340px]`：
   ```tsx
   <div className="flex-1 flex justify-center items-center px-6 h-full min-w-[340px]">
   ```
   这就意味着，无论窗体缩到多小，中部的空白容器（即使此时没有弹窗）也会**强行霸占至少 `340px` 的宽度**。随着窗体变窄，它会无情地将右侧组件强行挤出屏幕。
2. **缺乏多端自适应尺寸流（Fluid Typography & Padding）**：
   左右两侧的组件（如 Logo、文字、模式切换胶囊等）在所有窗体宽度下都使用了固定的大小和内边距（如 `px-10`、`gap-5`、`h-24`）。在狭窄的窗体下，这些大尺寸元素会产生物理空间冲突，导致整体布局崩溃。

---

### 🛠️ 解决方案

1. **中部空白区彻底弹性化**：
   将中部 Feedback 容器的 `min-w-[340px]` 改为 **`min-w-0`** 且配合 `overflow-hidden`。在日常无弹窗提示时，该容器的宽度将完美收缩为 `0`，把 100% 的空间优先让给左侧标题与右侧模式展示区，从而彻底解决挤压冲突。
2. **两端组件流式响应式（Responsive Utility Scaling）**：
   利用 Tailwind CSS 的响应式断点（如 `md:`、`sm:`），对 Header 整体进行流式微调：
   * **Header 高度和边距**：小窗体下为较紧凑的 `h-20 px-6`，大窗体下自适应展开为 `h-24 px-10`。
   * **左侧 Logo 与文字**：小窗体下 Logo 容器收敛为 `w-10 h-10`，标题字号自适应调整为 `text-base`；大窗体下自动舒展为 `w-12 h-12` 和 `text-xl`。
   * **右侧模式切换胶囊**：小窗体下自动收起繁琐的 `Alt+M Universe Mode` 快捷键提示，胶囊高度收敛为 `h-11 px-4`，字号降为 `text-xs`，图标微调为 `w-7 h-7`，保证在极小窗口下依然能够优雅容纳、不换行、不溢出。

---

### 💾 优化后的 `src/components/layout/Header.tsx` 完整代码

请用以下经过流式响应式重塑后的代码替换原文件，它能保证 SafeMask 控制台在**任意拖分屏、小窗口状态**下，所有头部组件均能完美自适应、流畅重绘：

```tsx
import {
  Activity,
  Pin,
  PinOff,
  Ghost,
  Shield,
} from "lucide-react";
import { useAppStore } from "@/hooks/useAppStore";
import { useAudioFeedback } from "@/hooks/useAudioFeedback";
import { cn } from "@/lib/utils";
import MagicFeedback from "@/components/feedback/MagicFeedback";

export default function Header() {
  const settings = useAppStore((s) => s.settings);
  const isAlwaysOnTop = useAppStore((s) => s.isAlwaysOnTop);
  const toggleAlwaysOnTop = useAppStore((s) => s.toggleAlwaysOnTop);
  const toggleVaultMode = useAppStore((s) => s.toggleVaultMode);
  const { play } = useAudioFeedback(settings.enable_audio_feedback);

  const isShadow = settings.shadow_mode_enabled;

  const handleToggleMode = async () => {
    await toggleVaultMode();
    play(isShadow ? "DESCEND" : "ASCEND");
  };

  return (
    <header className="h-20 md:h-24 flex items-center justify-between px-6 md:px-10 z-40 border-b border-white/[0.03] bg-[#0c0b0a]/60 backdrop-blur-xl shrink-0">
      {/* 左侧：Logo 与标题（随视口流式缩放） */}
      <div className="flex items-center gap-3 md:gap-5 shrink-0">
        <div className="w-10 h-10 md:w-12 md:h-12 rounded-lg bg-[#141210] border border-amber-500/10 flex items-center justify-center shadow-2xl relative overflow-hidden transition-transform duration-200 hover:scale-105">
          <Activity className="text-amber-500 w-4 h-4 md:w-5 md:h-5 relative z-10" />
        </div>

        <div>
          <h1 className="text-base md:text-xl font-bold tracking-tight text-amber-50/90 flex items-center gap-2 md:gap-3">
            SafeMask
            <div className="h-3 w-[1px] bg-white/10" />
            <span className="text-zinc-500 font-medium text-xs md:text-sm tracking-widest">
              控制台
            </span>
          </h1>
          <p className="text-[8px] md:text-[10px] text-zinc-600 font-bold tracking-[0.1em] uppercase">
            Secure Core Engine · v1.2.4
          </p>
        </div>
      </div>

      {/* 🚀 中部：彻底弹性化，min-w-0 确保空状态时不占用任何像素，有反馈时弹性伸缩 */}
      <div className="flex-1 flex justify-center items-center px-4 h-full min-w-0 overflow-hidden">
        <MagicFeedback />
      </div>

      {/* 右侧：始终置顶与模式切换（小视口下智能紧凑化） */}
      <div className="flex items-center gap-2 md:gap-3 shrink-0">
        <button
          type="button"
          onClick={toggleAlwaysOnTop}
          className={cn(
            "w-9 h-9 md:w-10 md:h-10 rounded-lg border transition-all duration-300 flex items-center justify-center hover:scale-105 active:scale-90",
            isAlwaysOnTop
              ? "bg-amber-500/20 border-amber-500/40 text-amber-300 shadow-amber-glow"
              : "bg-white/[0.02] border-white/5 text-zinc-500 hover:border-amber-500/20",
          )}
        >
          {isAlwaysOnTop ? <PinOff size={14} /> : <Pin size={14} />}
        </button>

        <div className="group relative">
          <div className="absolute top-full mt-4 right-0 w-72 p-4 rounded-3xl bg-[#1d1b18] border border-amber-500/20 shadow-2xl opacity-0 pointer-events-none group-hover:opacity-100 z-[100] transition-none">
            <div className="flex items-center gap-2 mb-2">
              <div className="w-1.5 h-1.5 rounded-full bg-amber-500" />
              <span className="text-xs font-bold text-amber-200">
                运行模式详情
              </span>
            </div>
            <p className="text-xs text-zinc-400 leading-relaxed">
              {isShadow ? (
                <>
                  <strong className="text-amber-200/80">影子宇宙模式：</strong>
                  系统仅在后台静默记录敏感信息，不改变剪贴板。需按下{" "}
                  <code className="bg-black/40 px-1 rounded text-amber-500">
                    {settings.magic_paste_shortcut}
                  </code>{" "}
                  才会粘贴脱敏副本。
                </>
              ) : (
                <>
                  <strong className="text-blue-400/80">哨兵宇宙模式：</strong>
                  全自动强力拦截。检测到敏感隐私时，系统会自动实时洗白剪贴板，确保存储与发送的始终是脱敏数据。
                </>
              )}
            </p>
            <div className="absolute bottom-full right-8 w-3 h-3 bg-[#1d1b18] border-r border-b border-amber-500/20 rotate-45 -translate-y-1.5" />
          </div>

          <div
            onClick={handleToggleMode}
            className="flex items-center gap-3 md:gap-6 bg-[#141210] border border-white/[0.08] h-11 md:h-14 px-4 md:px-8 rounded-2xl md:rounded-3xl cursor-pointer hover:border-amber-500/30 transition-all duration-500 shadow-xl hover:scale-[1.02] active:scale-[0.95]"
          >
            <div className="flex flex-col items-end leading-none">
              {/* 🚀 小视口下自动隐藏修饰快捷键和文字说明，仅保留核心模式文字 */}
              <span className="text-[8px] md:text-[9px] font-black text-zinc-700 uppercase tracking-[0.2em] mb-1 hidden md:flex items-center">
                <kbd className="px-1.5 py-0.5 bg-white/[0.04] rounded border border-white/[0.06] font-mono text-zinc-600">Alt+M</kbd>
                <span className="mx-1.5">Universe Mode</span>
              </span>
              <span
                className={cn(
                  "text-xs md:text-sm font-bold tracking-wider md:tracking-widest transition-colors duration-300",
                  isShadow ? "text-amber-200" : "text-blue-300",
                )}
              >
                {isShadow ? "影子宇宙" : "哨兵宇宙"}
              </span>
            </div>

            <div className="w-7 h-7 md:w-10 md:h-10 flex items-center justify-center rounded-lg md:rounded-xl bg-white/[0.02] border border-white/5 relative shrink-0">
              <div
                className={cn(
                  "absolute inset-0 rounded-lg md:rounded-xl blur-sm transition-colors duration-1000",
                  isShadow ? "bg-amber-500/25" : "bg-blue-500/25",
                )}
                style={{
                  animation: "pulse-opacity 2s ease-in-out infinite",
                }}
              />
              <div
                className={cn(
                  "transition-all duration-500",
                  isShadow ? "rotate-0 scale-100" : "rotate-180 scale-100",
                )}
              >
                {isShadow ? (
                  <Ghost className="w-4 h-4 md:w-5 md:h-5 text-amber-200" />
                ) : (
                  <Shield className="w-4 h-4 md:w-5 md:h-5 text-blue-300" />
                )}
              </div>
            </div>
          </div>
        </div>
      </div>
    </header>
  );
}
```