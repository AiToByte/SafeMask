import { AnimatePresence, motion } from "framer-motion";
import { ShieldCheck, Ghost, ShieldAlert, RotateCcw } from "lucide-react";
import { useAppStore, type FeedbackPayload } from "@/hooks/useAppStore";
import { toastVariants } from "@/lib/animations";

const containerClass =
  "fixed top-8 left-1/2 -translate-x-1/2 z-[999] pointer-events-none";
const toastClass =
  "bg-[#0f0f14]/90 backdrop-blur-[20px] border border-white/10 rounded-full px-6 py-3 shadow-2xl shadow-blue-500/10 flex items-center gap-3 text-sm font-bold text-white/90";

/** Toast notification that reads from the Zustand store's activeFeedback state */
export default function MagicFeedback() {
  const activeFeedback = useAppStore((s) => s.activeFeedback);

  return (
    <div className={containerClass}>
      <AnimatePresence mode="wait">
        {activeFeedback && (
          <motion.div
            key={activeFeedback.id}
            variants={toastVariants}
            initial="initial"
            animate="animate"
            exit="exit"
            className={toastClass}
          >
            <ToastContent feedback={activeFeedback} />
          </motion.div>
        )}
      </AnimatePresence>
    </div>
  );
}

function ToastContent({ feedback }: { feedback: FeedbackPayload }) {
  switch (feedback.type) {
    case "MODE_CHANGE":
      return <ModeChangeContent mode={feedback.mode ?? "SHADOW"} />;

    case "PASTE_MASKED":
      return (
        <>
          <ShieldCheck className="w-5 h-5 text-emerald-400 shrink-0" />
          <span>已注入脱敏副本</span>
        </>
      );

    case "PASTE_ORIGINAL":
      return (
        <>
          <RotateCcw className="w-5 h-5 text-amber-400 shrink-0" />
          <span>已回溯粘贴原文</span>
        </>
      );

    case "SUCCESS":
      return (
        <>
          <ShieldCheck className="w-5 h-5 text-blue-400 shrink-0" />
          <span>脱敏成功</span>
        </>
      );

    default:
      return null;
  }
}

function ModeChangeContent({ mode }: { mode: "SHADOW" | "SENTRY" }) {
  const isShadow = mode === "SHADOW";

  return (
    <>
      {isShadow ? (
        <Ghost className="w-5 h-5 text-amber-400 shrink-0" />
      ) : (
        <ShieldAlert className="w-5 h-5 text-blue-400 shrink-0" />
      )}
      <div className="flex flex-col leading-tight">
        <span className="text-xs font-bold">
          {isShadow ? "🌑 SHADOW" : "🛡️ SENTRY"}
        </span>
        <span className="text-[10px] text-zinc-400 font-normal">
          {isShadow
            ? "隐身模式：仅 Alt+V 注入脱敏数据"
            : "哨兵模式：剪贴板全量主动脱敏"}
        </span>
      </div>
    </>
  );
}
