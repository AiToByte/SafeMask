import { create } from "zustand";
import { MaskAPI, type AppSettings, type HistoryItem, type Rule, type AiEngineStatus, type EngineInfo } from "@/services/api";

// ── Types ──

export interface FeedbackPayload {
  type: "SUCCESS" | "MODE_CHANGE" | "PASTE_MASKED" | "PASTE_ORIGINAL";
  mode?: "SHADOW" | "SENTRY";
  id: number;
}

export type ActiveTab = "dashboard" | "history" | "rules" | "settings";

interface AppState {
  // ── State ──
  settings: AppSettings;
  isMonitorOn: boolean;
  ruleCount: number;
  activeTab: ActiveTab;
  historyList: HistoryItem[];
  allRulesList: Rule[];
  activeFeedback: FeedbackPayload | null;
  progress: number;
  isProcessing: boolean;
  currentFileName: string;
  appInfo: unknown | null;
  aiEngineStatus: AiEngineStatus | null;
  engineInfo: EngineInfo | null;
  isAlwaysOnTop: boolean;

  // ── Actions ──
  bootstrap: () => Promise<void>;
  toggleVaultMode: () => Promise<void>;
  toggleAlwaysOnTop: () => Promise<void>;
  fetchStats: () => Promise<void>;
  fetchHistory: () => Promise<void>;
  fetchAllRules: () => Promise<void>;
  clearHistory: () => Promise<void>;
  fetchAiStatus: () => Promise<void>;
  fetchEngineInfo: () => Promise<void>;
  toggleAiEngine: (enabled: boolean) => Promise<boolean>;
  setActiveTab: (tab: ActiveTab) => void;
  setProgress: (pct: number) => void;
  setProcessing: (v: boolean) => void;
  setCurrentFileName: (name: string) => void;
  setActiveFeedback: (fb: FeedbackPayload | null) => void;
  pushHistory: (item: HistoryItem) => void;
  updateSettings: (s: AppSettings) => void;
}

export const useAppStore = create<AppState>()((set, get) => ({
  // ── Initial State ──
  settings: {
    magic_paste_shortcut: "Alt+V",
    shadow_mode_enabled: true,
    paste_delay_ms: 150,
    enable_visual_feedback: true,
    enable_audio_feedback: true,
    model_download_urls: [],
    record_writer_enabled: false,
    mask_wrapper_style: "angle",
  },
  isMonitorOn: true,
  ruleCount: 0,
  activeTab: "dashboard",
  historyList: [],
  allRulesList: [],
  activeFeedback: null,
  progress: 0,
  isProcessing: false,
  currentFileName: "",
  appInfo: null,
  aiEngineStatus: null,
  engineInfo: null,
  isAlwaysOnTop: false,

  // ── Actions ──

  bootstrap: async () => {
    try {
      // Phase 1: critical — dashboard needs settings + ruleCount
      const [settings, stats] = await Promise.all([
        MaskAPI.getSettings(),
        MaskAPI.getStats(),
      ]);
      set({ settings, ruleCount: stats.rule_count });

      // Phase 2: deferred — non-blocking background fetch
      Promise.all([
        MaskAPI.getHistory(),
        MaskAPI.getAppInfo(),
        MaskAPI.getAiEngineStatus(),
        MaskAPI.getEngineInfo(),
      ]).then(([history, appInfo, aiStatus, engineInfo]) => {
        set({ historyList: history, appInfo, aiEngineStatus: aiStatus, engineInfo });
      });
    } catch (e) {
      console.error("Bootstrap Error:", e);
    }
  },

  toggleVaultMode: async () => {
    const newState = await MaskAPI.toggleVaultMode();
    set((s) => ({
      settings: { ...s.settings, shadow_mode_enabled: newState },
    }));
  },

  toggleAlwaysOnTop: async () => {
    const next = !get().isAlwaysOnTop;
    await MaskAPI.setAlwaysOnTop(next);
    set({ isAlwaysOnTop: next });
  },

  fetchStats: async () => {
    const stats = await MaskAPI.getStats();
    set({ ruleCount: stats.rule_count });
  },

  fetchHistory: async () => {
    set({ historyList: await MaskAPI.getHistory() });
  },

  fetchAllRules: async () => {
    set({ allRulesList: await MaskAPI.getAllRules() });
  },

  clearHistory: async () => {
    await MaskAPI.clearHistory();
    set({ historyList: [] });
  },

  fetchAiStatus: async () => {
    set({ aiEngineStatus: await MaskAPI.getAiEngineStatus() });
  },

  fetchEngineInfo: async () => {
    set({ engineInfo: await MaskAPI.getEngineInfo() });
  },

  toggleAiEngine: async (enabled: boolean) => {
    const result = await MaskAPI.toggleAiEngine(enabled);
    const [status, info] = await Promise.all([
      MaskAPI.getAiEngineStatus(),
      MaskAPI.getEngineInfo(),
    ]);
    set({ aiEngineStatus: status, engineInfo: info });
    return result;
  },

  setActiveTab: (tab) => set({ activeTab: tab }),
  setProgress: (pct) => set({ progress: pct }),
  setProcessing: (v) => set({ isProcessing: v }),
  setCurrentFileName: (name) => set({ currentFileName: name }),
  setActiveFeedback: (fb) => set({ activeFeedback: fb }),
  pushHistory: (item) =>
    set((s) => {
      const updated = [item, ...s.historyList];
      if (updated.length > 50) updated.pop();
      return { historyList: updated };
    }),
  updateSettings: (s) => set({ settings: s }),
}));
