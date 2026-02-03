import { defineStore } from 'pinia';
import { ref } from 'vue';
// 🚀 导入 Rule 和 HistoryItem 类型
import { MaskAPI, type Rule, type HistoryItem, type RuleStats, type AppInfo } from '../services/api';
import { listen } from "@tauri-apps/api/event"; // 🚀 引入事件监听

export const useAppStore = defineStore('app', () => {
  const isMonitorOn = ref(true);
  const ruleCount = ref(0);
  const isProcessing = ref(false);
  const progress = ref(0);
  const currentFileName = ref("");
  const historyList = ref<HistoryItem[]>([]);
  const activeTab = ref('dashboard'); // 切换页面
  const allRulesList = ref<Rule[]>([]);
  const appInfo = ref<AppInfo | null>(null);
  const isAlwaysOnTop = ref(false);

   // 🚀 初始化全局监听：确保只要程序开着，历史记录就在更新
  const initEventListeners = async () => {
    await listen<HistoryItem>("new-history", (event) => {
      // 将新记录插入数组头部（最新在前）
      historyList.value.unshift(event.payload);
      // 保持数组长度，防止长时间运行占用过多内存
      if (historyList.value.length > 50) historyList.value.pop();
    });
  };
  
  const fetchHistory = async () => {
    historyList.value = await MaskAPI.getHistory();
  };

  const fetchAllRules = async () => {
    allRulesList.value = await MaskAPI.getAllRules();
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

  // 获取应用详情
  const fetchAppInfo = async () => {
    appInfo.value = await MaskAPI.getAppInfo();
  };

  // 清除历史记录
  const clearHistory = async () => {
    await MaskAPI.clearHistory();
    historyList.value = [];
  };

  const toggleAlwaysOnTop = async () => {
    isAlwaysOnTop.value = !isAlwaysOnTop.value;
    await MaskAPI.setAlwaysOnTop(isAlwaysOnTop.value);
  };

return { 
     isMonitorOn, ruleCount, isProcessing, progress, 
    currentFileName, historyList, activeTab, allRulesList,
    appInfo, isAlwaysOnTop, // 🚀 必须返回
    fetchStats, fetchHistory, toggleMonitor, fetchAllRules, 
    initEventListeners, fetchAppInfo, clearHistory, // 🚀 必须返回
    toggleAlwaysOnTop
  };
});