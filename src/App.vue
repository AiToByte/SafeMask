<script setup lang="ts">
import { onMounted, onUnmounted } from 'vue';
import { listen, UnlistenFn } from "@tauri-apps/api/event";
import { useAppStore } from './stores/useAppStore';

// 导入重构后的高质量组件
import Sidebar from './components/Sidebar.vue';
import StatCard from './components/StatCard.vue';
import FileProcessor from './components/FileProcessor.vue';
import ExitConfirm from './components/ExitConfirm.vue';
import HistoryList from './components/HistoryList.vue';

const store = useAppStore();

// 存储监听器卸载函数，防止内存泄漏
let unlistenProgress: UnlistenFn;
let unlistenMasked: UnlistenFn;

onMounted(async () => {
  // 1. 初始化从 Rust 后端拉取规则统计信息
  await store.fetchStats();

  // 2. 监听文件处理进度事件 (来自 processor.rs 的保序流水线)
  unlistenProgress = await listen<{ percentage: number }>("file-progress", (event) => {
    // 自动更新 Pinia Store 中的进度状态，FileProcessor 组件会响应式更新 UI
    store.progress = event.payload.percentage;
  });

  // 3. 监听剪贴板脱敏事件 (方案一：原生钩子触发)
  unlistenMasked = await listen<string>("masked-event", (event) => {
    // 可以在此处集成 Toast 通知库，目前先打印日志
    console.info("🛡️ SafeMask Notification:", event.payload);
  });
});

// 组件销毁时取消系统事件监听
onUnmounted(() => {
  if (unlistenProgress) unlistenProgress();
  if (unlistenMasked) unlistenMasked();
});
</script>

<template>
  <!-- 主容器：采用 Flex 布局，H-Screen 撑满窗口 -->
  <div class="flex h-screen bg-[#09090b] text-zinc-100 select-none overflow-hidden font-sans">
    
    <!-- 左侧：固定宽度侧边栏 (已由 Sidebar.vue 封装) -->
    <Sidebar />

    <!-- 右侧：内容主体区域 -->
    <main class="flex-1 flex flex-col min-w-0">
      
      <!-- 顶栏：标题与全局状态开关 -->
      <header class="flex justify-between items-end px-12 pt-12 pb-8 border-b border-zinc-800/30">
        <div class="space-y-1">
          <h1 class="text-3xl font-extrabold tracking-tight bg-clip-text text-transparent bg-gradient-to-br from-white to-zinc-500">
            SafeMask 控制台
          </h1>
          <p class="text-zinc-500 text-sm font-medium">
            极致性能隐私治理引擎 · 实时数据脱敏
          </p>
        </div>

        <!-- 自动保护控制开关 (右侧对齐) -->
        <div class="flex items-center gap-4 bg-zinc-900/50 border border-zinc-800 px-5 py-3 rounded-2xl transition-all hover:border-zinc-700">
          <div class="flex flex-col items-end">
            <span class="text-xs font-bold uppercase tracking-wider text-zinc-400">实时保护</span>
            <span class="text-[10px] text-zinc-600 font-mono">{{ store.isMonitorOn ? 'ACTIVE' : 'DISABLED' }}</span>
          </div>
          <button 
            @click="store.toggleMonitor"
            class="w-12 h-6 rounded-full relative transition-all duration-300 focus:outline-none shadow-inner"
            :class="store.isMonitorOn ? 'bg-blue-600 shadow-blue-500/20' : 'bg-zinc-800'"
          >
            <div 
              class="absolute top-1 left-1 bg-white w-4 h-4 rounded-full transition-transform duration-300 shadow-sm"
              :class="{ 'translate-x-6': store.isMonitorOn }"
            ></div>
          </button>
        </div>
      </header>

       <!-- 动态内容区 -->
      <div class="flex-1 p-12 overflow-y-auto custom-scroll">
        <!-- 页面 1: 仪表盘 -->
        <div v-if="store.activeTab === 'dashboard'" class="space-y-10 animate-in fade-in slide-in-from-bottom-2">
          <div class="grid grid-cols-3 gap-6">
            <StatCard title="已加载规则" :value="store.ruleCount" unit="REG_RULES" />
            <StatCard title="历史拦截" :value="store.historyList.length" color="text-amber-400" />
            <StatCard title="引擎架构" value="HYBRID" color="text-blue-400" />
          </div>
          <FileProcessor class="min-h-[320px]" />
        </div>
        <!-- 页面 2: 历史记录 (这里必须紧跟上面的 v-if) -->
        <HistoryList v-else-if="store.activeTab === 'history'" />

        <!-- 规则库管理页面（预留） -->
        <div v-else-if="store.activeTab === 'rules'" class="text-zinc-500">
          规则配置功能研发中...
        </div>
        
        <!-- 页脚（仅在 Dashboard 显示） -->
        <footer v-if="store.activeTab === 'dashboard'" class="text-center pt-10 opacity-30">
          <p class="text-[10px] font-mono uppercase tracking-widest">Powered by SafeMask Rust Engine v1.0.0</p>
        </footer>
      </div>
    </main>
     <!-- 退出确认组件 -->
    <ExitConfirm />
  </div>
</template>

<style>
/* 全局基础样式补丁 */

/* 1. 隐藏所有滚动条但保留滚动功能 (针对桌面端定制) */
::-webkit-scrollbar {
  display: none;
}

/* 2. 定义玻璃拟态通用背景类 */
.glass {
  background: rgba(18, 18, 23, 0.7);
  backdrop-filter: blur(16px);
  -webkit-backdrop-filter: blur(16px);
  border: 1px solid rgba(255, 255, 255, 0.05);
}

/* 3. 进入/离开动画 */
.fade-enter-active, .fade-leave-active {
  transition: opacity 0.3s ease;
}
.fade-enter-from, .fade-leave-to {
  opacity: 0;
}
</style>