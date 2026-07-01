import { useEffect } from "react";
import { AnimatePresence, motion } from "framer-motion";
import { useAppStore } from "@/hooks/useAppStore";
import { useAudioFeedback } from "@/hooks/useAudioFeedback";
import { useTauriEvent } from "@/hooks/useTauriEvents";
import type { HistoryItem } from "@/services/api";
import type { FeedbackPayload as RawFeedback } from "@/hooks/useAppStore";
import Sidebar from "@/components/layout/Sidebar";
import Header from "@/components/layout/Header";
import MagicFeedback from "@/components/feedback/MagicFeedback";
import ExitConfirm from "@/components/overlay/ExitConfirm";
import StatCard from "@/components/dashboard/StatCard";
import FileProcessor from "@/components/dashboard/FileProcessor";
import HistoryList from "@/components/history/HistoryList";
import RuleManager from "@/components/rules/RuleManager";
import SettingsPage from "@/components/settings/SettingsPage";
import { pageTransition } from "@/lib/animations";

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
    <div className="flex h-screen bg-[#0c0b0a] text-amber-50/90 select-none overflow-hidden font-sans">
      {/* Toast notifications */}
      <MagicFeedback />

      {/* Navigation sidebar */}
      <Sidebar />

      {/* Main area */}
      <main className="flex-1 flex flex-col min-w-0 relative">
        {/* Ambient background glow */}
        <div className="absolute top-0 left-1/4 w-[60%] h-[30%] bg-amber-600/[0.02] blur-[120px] pointer-events-none" />

        <Header />

        {/* Content area with page transitions */}
        <div className="flex-1 overflow-hidden px-10 py-4 flex flex-col">
          <AnimatePresence mode="wait">
            <motion.div
              key={activeTab}
              variants={pageTransition}
              initial="initial"
              animate="animate"
              exit="exit"
              className="max-w-6xl mx-auto w-full h-full flex flex-col"
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
                  <HistoryList />
                </div>
              )}

              {activeTab === "rules" && (
                <div className="flex-1 overflow-y-auto custom-scroll">
                  <RuleManager />
                </div>
              )}

              {activeTab === "settings" && (
                <div className="flex-1 overflow-y-auto custom-scroll">
                  <SettingsPage />
                </div>
              )}
            </motion.div>
          </AnimatePresence>
        </div>

        {/* Exit confirmation modal */}
        <ExitConfirm />
      </main>
    </div>
  );
}

// ── Dashboard Sub-page (inline to avoid extra file for small layout) ──

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
    <div className="flex-1 flex flex-col gap-4">
      {/* Stat cards row */}
      <div className="grid grid-cols-3 gap-4 shrink-0">
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

      {/* File processor */}
      <div className="flex-1 min-h-0 relative">
        <FileProcessor className="h-full bg-[#110f0e]/50 border border-white/[0.02] shadow-2xl" />
      </div>

      {/* Footer */}
      <footer className="flex justify-center py-1 opacity-10 shrink-0">
        <p className="text-[7px] font-mono uppercase tracking-[0.5em] text-white">
          Local Processing Instance
        </p>
      </footer>
    </div>
  );
}
