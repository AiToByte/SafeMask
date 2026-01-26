import { defineStore } from 'pinia';
import { ref } from 'vue';
import { MaskAPI } from '../services/api';

export interface HistoryItem {
  id: string;
  timestamp: string;
  original: string;
  masked: string;
}

export const useAppStore = defineStore('app', () => {
  const isMonitorOn = ref(true);
  const ruleCount = ref(0);
  const isProcessing = ref(false);
  const progress = ref(0);
  const currentFileName = ref("");
  const historyList = ref<HistoryItem[]>([]);
  const activeTab = ref('dashboard'); // 切换页面

  const fetchHistory = async () => {
    historyList.value = await MaskAPI.getHistory();
  };

  // 初始化统计
  const fetchStats = async () => {
    const stats = await MaskAPI.getStats();
    ruleCount.value = stats.rule_count;
  };

  // 切换监控
  const toggleMonitor = async () => {
    isMonitorOn.value = !isMonitorOn.value;
    await MaskAPI.toggleMonitor(isMonitorOn.value);
  };

return { 
    isMonitorOn, ruleCount, isProcessing, progress, 
    currentFileName, historyList, activeTab,
    fetchStats, fetchHistory, toggleMonitor 
  };
});