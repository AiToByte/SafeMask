import { ShieldCheck, Ghost, ShieldAlert, RotateCcw } from "lucide-react";
import { useAppStore, type FeedbackPayload } from "@/hooks/useAppStore";
import { useEffect, useState } from "react";

const containerClass = "relative w-full h-full flex items-center justify-center pointer-events-none";
const toastClass = "bg-[#141210] border border-amber-500/10 rounded-2xl px-6 py-2.5 shadow-xl shadow-black/40 flex items-center gap-3 text-sm font-bold text-white/90 toast-animate pointer-events-none";

export default function MagicFeedback() {
  const activeFeedback = useAppStore((s) => s.activeFeedback);
  const [exiting, setExiting] = useState(false);
  const [current, setCurrent] = useState<FeedbackPayload | null>(null);

  useEffect(() => {
    if (activeFeedback) {
      setCurrent(activeFeedback);
      setExiting(false);
    } else if (current) {
      setExiting(true);
      const timer = setTimeout(() => {
        setCurrent(null);
        setExiting(false);
      }, 200);
      return () => clearTimeout(timer);
    }
  }, [activeFeedback, current]);

  if (!current && !exiting) return null;

  const show = current || activeFeedback;

  return (
    <div className={containerClass}>
      <div
        className={`${toastClass} ${exiting ? "toast-exit" : "toast-enter"}`}
      >
        <ToastContent feedback={show!} />
      </div>
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
          <ShieldCheck className="w-5 h-5 text-emerald-400 shrink-0 animate-pulse" />
          <span className="text-xs tracking-wide">已注入脱敏副本</span>
        </>
      );

    case "PASTE_ORIGINAL":
      return (
        <>
          <RotateCcw className="w-5 h-5 text-amber-400 shrink-0 animate-spin" />
          <span className="text-xs tracking-wide">已回溯粘贴原文</span>
        </>
      );

    case "SUCCESS":
      return (
        <>
          <ShieldCheck className="w-5 h-5 text-blue-400 shrink-0" />
          <span className="text-xs tracking-wide">文本脱敏成功</span>
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
        <span className="text-xs font-black tracking-wider text-amber-50/90">
          {isShadow ? '影子宇宙模式激活' : '哨兵宇宙模式激活'}
        </span>
        <span className="text-[8px] text-zinc-500 font-bold uppercase tracking-widest mt-0.5">
          {isShadow ? '手动按需脱敏粘贴' : '全局自动实时洗白'}
        </span>
      </div>
    </>
  );
}
