import { useEffect, lazy, Suspense } from "react";
import { useAppStore } from "@/hooks/useAppStore";
import { useAudioFeedback } from "@/hooks/useAudioFeedback";
import { useTauriEvent } from "@/hooks/useTauriEvents";
import type { HistoryItem } from "@/services/api";
import type { FeedbackPayload as RawFeedback } from "@/hooks/useAppStore";
import Sidebar from "@/components/layout/Sidebar";
import Header from "@/components/layout/Header";
import StatCard from "@/components/dashboard/StatCard";
import FileProcessor from "@/components/dashboard/FileProcessor";

const HistoryList = lazy(() => import("@/components/history/HistoryList"));
const RuleManager = lazy(() => import("@/components/rules/RuleManager"));
const SettingsPage = lazy(() => import("@/components/settings/SettingsPage"));
const MagicFeedback = lazy(() => import("@/components/feedback/MagicFeedback"));
const ExitConfirm = lazy(() => import("@/components/overlay/ExitConfirm"));

export default function App() {
  const bootstrap = useAppStore((s) => s.bootstrap);
  const activeTab = useAppStore((s) => s.activeTab);
  const settings = useAppStore((s) => s.settings);
  const ruleCount = useAppStore((s) => s.ruleCount);
  const historyList = useAppStore((s) => s.historyList);
  const pushHistory = useAppStore((s) => s.pushHistory);
  const setActiveFeedback = useAppStore((s) => s.setActiveFeedback);
  const setActiveTab = useAppStore((s) => s.setActiveTab);
  const { play } = useAudioFeedback(settings.enable_audio_feedback);

  // Bootstrap on mount
  useEffect(() => {
    bootstrap();
  }, [bootstrap]);

  // Background preload lazy chunks after dashboard renders
  useEffect(() => {
    const timer = setTimeout(() => {
      import("@/components/history/HistoryList");
      import("@/components/rules/RuleManager");
      import("@/components/settings/SettingsPage");
    }, 500);
    return () => clearTimeout(timer);
  }, []);

  // ── Tauri Event Listeners ──

  useTauriEvent<HistoryItem>("new-history", (item) => {
    pushHistory(item);
  });

  useTauriEvent<RawFeedback>("magic-feedback", (payload) => {
    if (settings.enable_audio_feedback && (payload as any).type === "SUCCESS") {
      play("CLICK");
    }
    if (settings.enable_visual_feedback) {
      const fb = { ...(payload as any), id: Date.now() };
      setActiveFeedback(fb);
      setTimeout(() => setActiveFeedback(null), 3000);
    }
  });

  useTauriEvent<string>("mode-switch-event", (mode) => {
    if (settings.enable_audio_feedback) {
      play(mode === "SHADOW" ? "ASCEND" : "DESCEND");
    }
    const fb = { type: "MODE_CHANGE" as const, mode: mode as "SHADOW" | "SENTRY", id: Date.now() };
    setActiveFeedback(fb);
    setTimeout(() => setActiveFeedback(null), 3000);
  });

  useTauriEvent<{ percentage: number }>("file-progress", (p) => {
    useAppStore.getState().setProgress(p.percentage);
  });

  return (
    <div className="flex flex-col h-screen bg-[#0c0b0a] text-amber-50/90 select-none overflow-hidden font-sans relative">
      <Suspense fallback={null}>
        <MagicFeedback />
      </Suspense>

      <div className="flex flex-1 overflow-hidden">
        <Sidebar />

        <main className="flex-1 flex flex-col min-w-0 relative">
          <div className="absolute top-0 left-1/4 w-[60%] h-[30%] bg-amber-600/[0.02] blur-[120px] pointer-events-none" />

          <Header />

          <div className="flex-1 overflow-hidden px-12 py-6 flex flex-col">
            <div
              key={activeTab}
              className="w-full h-full flex flex-col page-active"
            >
              {activeTab === "dashboard" && (
                <DashboardPage
                  ruleCount={ruleCount}
                  historyCount={historyList.length}
                  onNavRules={() => setActiveTab("rules")}
                  onNavHistory={() => setActiveTab("history")}
                />
              )}

              {activeTab === "history" && (
                <div className="flex-1 overflow-y-auto custom-scroll">
                  <Suspense fallback={<PageFallback />}>
                    <HistoryList />
                  </Suspense>
                </div>
              )}

              {activeTab === "rules" && (
                <div className="flex-1 overflow-y-auto custom-scroll">
                  <Suspense fallback={<PageFallback />}>
                    <RuleManager />
                  </Suspense>
                </div>
              )}

              {activeTab === "settings" && (
                <div className="flex-1 overflow-y-auto custom-scroll">
                  <Suspense fallback={<PageFallback />}>
                    <SettingsPage />
                  </Suspense>
                </div>
              )}
            </div>
          </div>

          <Suspense fallback={null}>
            <ExitConfirm />
          </Suspense>
        </main>
      </div>
    </div>
  );
}

function PageFallback() {
  return (
    <div className="flex items-center justify-center h-full">
      <div className="w-6 h-6 border-2 border-amber-500/30 border-t-amber-500 rounded-full animate-spin" />
    </div>
  );
}

function DashboardPage({
  ruleCount,
  historyCount,
  onNavRules,
  onNavHistory,
}: {
  ruleCount: number;
  historyCount: number;
  onNavRules: () => void;
  onNavHistory: () => void;
}) {
  return (
    <div className="flex-1 flex flex-col gap-6">
      <div className="grid grid-cols-3 gap-6 shrink-0">
        <StatCard
          title="已装载脱敏规则"
          value={ruleCount}
          unit="Patterns"
          color="text-amber-200"
          type="amber"
          clickable
          onClick={onNavRules}
        />
        <StatCard
          title="累计隐私审计记录"
          value={historyCount}
          unit="Records"
          color="text-blue-300"
          type="blue"
          clickable
          onClick={onNavHistory}
        />
        <StatCard
          title="脱敏引擎状态"
          value="无损运行"
          unit="Normal"
          color="text-emerald-300"
          type="emerald"
        />
      </div>

      <div className="flex-1 min-h-0 relative">
        <FileProcessor className="h-full bg-[#110f0e]/50 border border-white/[0.02] shadow-2xl" />
      </div>

      <footer className="flex justify-center py-1 opacity-30 shrink-0">
        <p className="text-[9px] font-mono uppercase tracking-[0.5em] text-white">
          Local Processing Instance
        </p>
      </footer>
    </div>
  );
}
