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
    <header
      className="h-20 md:h-24 flex items-center justify-between px-6 md:px-10 z-40 border-b border-white/[0.03] backdrop-blur-xl shrink-0"
      style={{ backgroundColor: 'color-mix(in srgb, var(--bg-root) 60%, transparent)' }}
    >
      {/* 左侧：Logo 与标题（随视口流式缩放） */}
      <div className="flex items-center gap-3 md:gap-5 shrink-0">
        <div
          className="w-10 h-10 md:w-12 md:h-12 rounded-lg border border-amber-500/10 flex items-center justify-center shadow-2xl relative overflow-hidden transition-transform duration-200 hover:scale-105"
          style={{ backgroundColor: 'var(--bg-elevated)' }}
        >
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
            Secure Core Engine · v{__APP_VERSION__}
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
          <div
            className="absolute top-full mt-4 right-0 w-72 p-4 rounded-3xl border border-amber-500/20 shadow-2xl opacity-0 pointer-events-none group-hover:opacity-100 z-[100] transition-none"
            style={{ backgroundColor: 'var(--bg-elevated)' }}
          >
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
            <div
              className="absolute bottom-full right-8 w-3 h-3 border-r border-b border-amber-500/20 rotate-45 -translate-y-1.5"
              style={{ backgroundColor: 'var(--bg-elevated)' }}
            />
          </div>

          <div
            onClick={handleToggleMode}
            className="flex items-center gap-3 md:gap-6 border border-white/[0.08] h-11 md:h-14 px-4 md:px-8 rounded-2xl md:rounded-3xl cursor-pointer hover:border-amber-500/30 transition-all duration-500 shadow-xl hover:scale-[1.02] active:scale-[0.95]"
            style={{ backgroundColor: 'var(--bg-elevated)' }}
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
                  "scale-100",
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
