import { defineStore } from 'pinia';
import { ref } from 'vue';
import { listen } from "@tauri-apps/api/event";
import { MaskAPI, type Rule, type HistoryItem, type AppSettings } from '../services/api';

export const useAppStore = defineStore('app', () => {
  const settings = ref<AppSettings>({
    magic_paste_shortcut: "Alt+V",
    shadow_mode_enabled: true,
    paste_delay_ms: 150,
    enable_visual_feedback: true,
    enable_audio_feedback: true
  });

  const isMonitorOn = ref(true);
  const ruleCount = ref(0);
  const activeTab = ref('dashboard');
  const historyList = ref<HistoryItem[]>([]);
  const allRulesList = ref<Rule[]>([]);
  const activeFeedback = ref<any>(null);
  const progress = ref(0);
  const isProcessing = ref(false);
  const currentFileName = ref("");
  const appInfo = ref<any>(null);

  const bootstrap = async () => {
    try {
      settings.value = await MaskAPI.getSettings();
      const stats = await MaskAPI.getStats();
      ruleCount.value = stats.rule_count;
      historyList.value = await MaskAPI.getHistory();
      appInfo.value = await MaskAPI.getAppInfo();
      await initAllEventListeners();
    } catch (e) { console.error("Bootstrap Error:", e); }
  };

  const initAllEventListeners = async () => {
    await listen<HistoryItem>("new-history", (e) => {
      historyList.value.unshift(e.payload);
      if (historyList.value.length > 50) historyList.value.pop();
    });

    await listen<any>("magic-feedback", (e) => {
      const p = e.payload;
      if (settings.value.enable_audio_feedback && p.type === 'SUCCESS') playSound('CLICK');
      if (settings.value.enable_visual_feedback) {
        activeFeedback.value = { ...p, id: Date.now() };
        setTimeout(() => activeFeedback.value = null, 3000);
      }
    });

    await listen<string>("mode-switch-event", (e) => {
      const mode = e.payload;
      settings.value.shadow_mode_enabled = (mode === 'SHADOW');
      if (settings.value.enable_audio_feedback) playSound(mode === 'SHADOW' ? 'ASCEND' : 'DESCEND');
      activeFeedback.value = { type: 'MODE_CHANGE', mode, id: Date.now() };
      setTimeout(() => activeFeedback.value = null, 3000);
    });

    await listen<{ percentage: number }>("file-progress", (e) => progress.value = e.payload.percentage);
  };

  const playSound = (type: 'CLICK' | 'ASCEND' | 'DESCEND') => {
    const ctx = new AudioContext();
    const osc = ctx.createOscillator();
    const gain = ctx.createGain();
    osc.connect(gain); gain.connect(ctx.destination);
    const now = ctx.currentTime;
    if (type === 'CLICK') {
      osc.frequency.setValueAtTime(1200, now);
      osc.frequency.exponentialRampToValueAtTime(40, now + 0.1);
      gain.gain.setValueAtTime(0.1, now);
    } else if (type === 'ASCEND') {
      osc.frequency.setValueAtTime(440, now);
      osc.frequency.exponentialRampToValueAtTime(880, now + 0.15);
      gain.gain.setValueAtTime(0.05, now);
    } else {
      osc.frequency.setValueAtTime(660, now);
      osc.frequency.exponentialRampToValueAtTime(330, now + 0.15);
      gain.gain.setValueAtTime(0.05, now);
    }
    osc.start(); osc.stop(now + 0.2);
  };

  const toggleVaultMode = async () => {
    const newState = await MaskAPI.toggleVaultMode();
    settings.value.shadow_mode_enabled = newState;
  };

  return { 
    settings, isMonitorOn, activeTab, ruleCount, historyList, allRulesList, 
    activeFeedback, progress, isProcessing, currentFileName, appInfo,
    bootstrap, toggleVaultMode, fetchStats: async () => ruleCount.value = (await MaskAPI.getStats()).rule_count,
    fetchHistory: async () => historyList.value = await MaskAPI.getHistory(),
    fetchAllRules: async () => allRulesList.value = await MaskAPI.getAllRules(),
    clearHistory: async () => { await MaskAPI.clearHistory(); historyList.value = []; }
  };
});