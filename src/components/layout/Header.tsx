import { motion } from "framer-motion";
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
    <header className="h-20 flex items-center justify-between px-10 z-40 border-b border-white/[0.03] bg-[#0c0b0a]/60 backdrop-blur-xl shrink-0">
      {/* Left: Logo + Title */}
      <div className="flex items-center gap-5">
        <motion.div
          className="w-10 h-10 rounded-lg bg-[#141210] border border-amber-500/10 flex items-center justify-center shadow-2xl relative overflow-hidden"
          whileHover={{ scale: 1.05 }}
        >
          <Activity className="text-amber-500 w-4 h-4 relative z-10" />
        </motion.div>

        <div>
          <h1 className="text-lg font-bold tracking-tight text-amber-50/90 flex items-center gap-3">
            SafeMask
            <div className="h-3 w-[1px] bg-white/10" />
            <span className="text-zinc-500 font-medium text-sm tracking-widest">
              控制台
            </span>
          </h1>
          <p className="text-[8px] text-zinc-600 font-bold tracking-[0.1em] uppercase">
            Secure Core Engine · v1.2.4
          </p>
        </div>
      </div>

      {/* Right: Actions */}
      <div className="flex items-center gap-3">
        {/* Pin/Unpin button */}
        <motion.button
          onClick={toggleAlwaysOnTop}
          whileHover={{ scale: 1.05 }}
          whileTap={{ scale: 0.9 }}
          className={cn(
            "w-9 h-9 rounded-lg border transition-all duration-300 flex items-center justify-center",
            isAlwaysOnTop
              ? "bg-amber-500/20 border-amber-500/40 text-amber-300 shadow-[0_0_15px_rgba(245,158,11,0.2)]"
              : "bg-white/[0.02] border-white/5 text-zinc-500 hover:border-amber-500/20",
          )}
        >
          {isAlwaysOnTop ? <PinOff size={14} /> : <Pin size={14} />}
        </motion.button>

        {/* Universe Mode Toggle Capsule */}
        <motion.div
          onClick={handleToggleMode}
          whileHover={{ scale: 1.02 }}
          whileTap={{ scale: 0.95 }}
          className="group relative flex items-center gap-5 bg-[#141210] border border-white/[0.08] h-12 px-6 rounded-2xl cursor-pointer hover:border-amber-500/30 transition-all duration-500 shadow-xl"
        >
          {/* Hover tooltip */}
          <div className="absolute top-full mt-4 right-0 w-72 p-4 rounded-2xl bg-[#1d1b18] border border-amber-500/20 shadow-2xl opacity-0 translate-y-2 pointer-events-none group-hover:opacity-100 group-hover:translate-y-0 transition-all duration-300 z-[100]">
            <div className="flex items-center gap-2 mb-2">
              <div className="w-1.5 h-1.5 rounded-full bg-amber-500" />
              <span className="text-xs font-bold text-amber-200">
                运行模式详情
              </span>
            </div>
            <p className="text-[11px] text-zinc-400 leading-relaxed">
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
            {/* Arrow */}
            <div className="absolute bottom-full right-8 w-3 h-3 bg-[#1d1b18] border-r border-b border-amber-500/20 rotate-45 -translate-y-1.5" />
          </div>

          {/* Mode label */}
          <div className="flex flex-col items-end">
            <span className="text-[9px] font-black text-zinc-600 uppercase tracking-tighter mb-0.5">
              Universe Mode
            </span>
            <span
              className={cn(
                "text-xs font-bold tracking-widest transition-colors duration-300",
                isShadow ? "text-amber-200" : "text-blue-300",
              )}
            >
              {isShadow ? "影子宇宙模式" : "哨兵宇宙模式"}
            </span>
          </div>

          {/* Mode icon */}
          <div className="w-8 h-8 flex items-center justify-center rounded-xl bg-white/[0.02] border border-white/5 relative">
            <motion.div
              className="absolute inset-0 rounded-xl blur-sm opacity-20 animate-pulse"
              animate={{
                backgroundColor: isShadow
                  ? "rgba(245,158,11,0.3)"
                  : "rgba(59,130,246,0.3)",
              }}
            />
            <motion.div
              animate={{ rotate: isShadow ? 0 : 360 }}
              transition={{ duration: 0.4 }}
              key={isShadow ? "ghost" : "shield"}
            >
              {isShadow ? (
                <Ghost size={14} className="text-amber-200" />
              ) : (
                <Shield size={14} className="text-blue-300" />
              )}
            </motion.div>
          </div>
        </motion.div>
      </div>
    </header>
  );
}
