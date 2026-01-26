import { defineStore } from 'pinia';
import { ref } from 'vue';
// 🚀 导入 Rule 和 HistoryItem 类型
import { MaskAPI } from '../services/api';
import { listen } from "@tauri-apps/api/event"; // 🚀 引入事件监听
export const useAppStore = defineStore('app', () => {
    const isMonitorOn = ref(true);
    const ruleCount = ref(0);
    const isProcessing = ref(false);
    const progress = ref(0);
    const currentFileName = ref("");
    const historyList = ref([]);
    const activeTab = ref('dashboard'); // 切换页面
    const allRulesList = ref([]);
    // 🚀 初始化全局监听：确保只要程序开着，历史记录就在更新
    const initEventListeners = async () => {
        await listen("new-history", (event) => {
            // 将新记录插入数组头部（最新在前）
            historyList.value.unshift(event.payload);
            // 保持数组长度，防止长时间运行占用过多内存
            if (historyList.value.length > 50)
                historyList.value.pop();
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
    return {
        isMonitorOn, ruleCount, isProcessing, progress,
        currentFileName, historyList, activeTab,
        fetchStats, fetchHistory, toggleMonitor,
        allRulesList, fetchAllRules, initEventListeners
    };
});
//# sourceMappingURL=useAppStore.js.map