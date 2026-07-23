import { create } from "zustand";
import { MaskAPI, type AppSettings, type HistoryItem, type Rule, type AiEngineStatus, type EngineInfo } from "@/services/api";
import { normalizeThemeId, type ThemeId } from "@/lib/themes";
import { loadPersistedTheme } from "@/lib/themeStorage";

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
  /**
   * 原子切换主题：乐观更新本地 state → 后端持久化 → 失败回滚。
   * 返回 Promise，UI 层可 await 以决定是否显示错误提示。
   */
  setTheme: (theme: ThemeId) => Promise<void>;
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
    // 从 localStorage 同步读取，避免首帧 store 值与 DOM (由 main.tsx 预应用) 不一致；
    // 后续 bootstrap 从 Rust 加载的 settings 会覆盖此值。
    theme: loadPersistedTheme(),
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

  setTheme: async (theme) => {
    // 规范化输入 —— 防御任何非法值意外进入 store
    const nextTheme = normalizeThemeId(theme);
    const prevSettings = get().settings;
    // 已是目标主题时短路，避免无谓的后端同步
    if (prevSettings.theme === nextTheme) return;

    // 乐观更新：立即切换 UI，让用户感觉即时响应
    const nextSettings: AppSettings = { ...prevSettings, theme: nextTheme };
    set({ settings: nextSettings });

    try {
      await MaskAPI.updateSettings(nextSettings);
    } catch (err) {
      // 竞态防护 (compare-and-swap 语义)：
      // 只在当前 state 仍等于本次调用设置的值时才回滚。
      // 若在此期间用户又切换过主题（新的 setTheme 已经乐观更新），
      // 则新调用的结果优先，不再覆盖回旧值。
      const currentSettings = get().settings;
      if (currentSettings.theme === nextTheme) {
        set({ settings: prevSettings });
      }
      throw err;
    }
  },
}));
