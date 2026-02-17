import { defineStore } from 'pinia';
import { ref } from 'vue';
import { listen } from "@tauri-apps/api/event";
import { MaskAPI } from '../services/api';
export const useAppStore = defineStore('app', () => {
    const settings = ref({
        magic_paste_shortcut: "Alt+V",
        shadow_mode_enabled: true,
        paste_delay_ms: 150,
        enable_visual_feedback: true,
        enable_audio_feedback: true
    });
    const isMonitorOn = ref(true);
    const ruleCount = ref(0);
    const activeTab = ref('dashboard');
    const historyList = ref([]);
    const allRulesList = ref([]);
    const activeFeedback = ref(null);
    const progress = ref(0);
    const isProcessing = ref(false);
    const currentFileName = ref("");
    const appInfo = ref(null);
    const bootstrap = async () => {
        try {
            settings.value = await MaskAPI.getSettings();
            const stats = await MaskAPI.getStats();
            ruleCount.value = stats.rule_count;
            historyList.value = await MaskAPI.getHistory();
            appInfo.value = await MaskAPI.getAppInfo();
            await initAllEventListeners();
        }
        catch (e) {
            console.error("Bootstrap Error:", e);
        }
    };
    const initAllEventListeners = async () => {
        await listen("new-history", (e) => {
            historyList.value.unshift(e.payload);
            if (historyList.value.length > 50)
                historyList.value.pop();
        });
        await listen("magic-feedback", (e) => {
            const p = e.payload;
            if (settings.value.enable_audio_feedback && p.type === 'SUCCESS')
                playFeedbackSound('CLICK');
            if (settings.value.enable_visual_feedback) {
                activeFeedback.value = { ...p, id: Date.now() };
                setTimeout(() => activeFeedback.value = null, 3000);
            }
        });
        await listen("mode-switch-event", (e) => {
            const mode = e.payload;
            settings.value.shadow_mode_enabled = (mode === 'SHADOW');
            if (settings.value.enable_audio_feedback)
                playFeedbackSound(mode === 'SHADOW' ? 'ASCEND' : 'DESCEND');
            activeFeedback.value = { type: 'MODE_CHANGE', mode, id: Date.now() };
            setTimeout(() => activeFeedback.value = null, 3000);
        });
        await listen("file-progress", (e) => progress.value = e.payload.percentage);
    };
    const playFeedbackSound = (type) => {
        const audioCtx = new (window.AudioContext || window.webkitAudioContext)();
        const now = audioCtx.currentTime;
        const playOsc = (freq, dur, gainVal, type = 'triangle') => {
            const osc = audioCtx.createOscillator();
            const gain = audioCtx.createGain();
            osc.type = type;
            osc.frequency.setValueAtTime(freq, now);
            gain.gain.setValueAtTime(gainVal, now);
            gain.gain.exponentialRampToValueAtTime(0.01, now + dur);
            osc.connect(gain);
            gain.connect(audioCtx.destination);
            osc.start();
            osc.stop(now + dur);
        };
        switch (type) {
            case 'CLICK':
                playOsc(1200, 0.08, 0.1, 'square');
                break;
            case 'ASCEND':
                playOsc(440, 0.2, 0.05);
                setTimeout(() => playOsc(880, 0.2, 0.04), 80);
                break;
            case 'DESCEND':
                playOsc(660, 0.2, 0.05);
                setTimeout(() => playOsc(330, 0.2, 0.04), 80);
                break;
            case 'RECORD':
                playOsc(1000, 0.1, 0.08, 'sine');
                break;
            case 'ERROR': // 🚀 新增错误反馈音：低沉的双顿音
                playOsc(200, 0.15, 0.1, 'sawtooth');
                setTimeout(() => playOsc(150, 0.2, 0.1, 'sawtooth'), 120);
                break;
        }
    };
    const toggleVaultMode = async () => {
        const newState = await MaskAPI.toggleVaultMode();
        settings.value.shadow_mode_enabled = newState;
    };
    const isAlwaysOnTop = ref(false);
    const toggleAlwaysOnTop = async () => {
        isAlwaysOnTop.value = !isAlwaysOnTop.value;
        await MaskAPI.setAlwaysOnTop(isAlwaysOnTop.value);
    };
    return {
        settings, isMonitorOn, activeTab, ruleCount, historyList, allRulesList,
        activeFeedback, progress, isProcessing, currentFileName, appInfo,
        bootstrap, toggleVaultMode, fetchStats: async () => ruleCount.value = (await MaskAPI.getStats()).rule_count,
        fetchHistory: async () => historyList.value = await MaskAPI.getHistory(),
        fetchAllRules: async () => allRulesList.value = await MaskAPI.getAllRules(),
        clearHistory: async () => { await MaskAPI.clearHistory(); historyList.value = []; },
        isAlwaysOnTop, toggleAlwaysOnTop, playFeedbackSound
    };
});
//# sourceMappingURL=useAppStore.js.map