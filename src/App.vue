<script setup lang="ts">
import { onMounted, onUnmounted } from 'vue';
import { listen, UnlistenFn } from "@tauri-apps/api/event";
import { useAppStore } from './stores/useAppStore';
import { Pin, PinOff, Shield, ShieldOff, Activity } from 'lucide-vue-next';

// 导入组件
import Sidebar from './components/Sidebar.vue';
import StatCard from './components/StatCard.vue';
import FileProcessor from './components/FileProcessor.vue';
import ExitConfirm from './components/ExitConfirm.vue';
import HistoryList from './components/HistoryList.vue';
import RuleManager from './components/RuleManager.vue';
import SettingsPage from './components/Settings.vue';

const store = useAppStore();

let unlistenProgress: UnlistenFn;
let unlistenMasked: UnlistenFn;

onMounted(async () => {
  try {
    await store.fetchStats();
    await store.fetchHistory();
    await store.initEventListeners();
  } catch (e) {
    console.error("初始化失败:", e);
  }

  unlistenProgress = await listen<{ percentage: number }>("file-progress", (event) => {
    store.progress = event.payload.percentage;
  });

  unlistenMasked = await listen<string>("masked-event", (event) => {
    console.info("🛡️ SafeMask 通知:", event.payload);
  });
});

onUnmounted(() => {
  if (unlistenProgress) unlistenProgress();
  if (unlistenMasked) unlistenMasked();
});
</script>

<template>
  <div class="flex h-screen bg-[#09090b] text-zinc-100 select-none overflow-hidden font-sans">
    <!-- 左侧侧边栏 -->
    <Sidebar />

    <!-- 右侧主体 -->
    <main class="flex-1 flex flex-col min-w-0 relative">
      
      <!-- 🚀 顶栏：强化了控制胶囊的视觉圆润感 -->
      <header class="h-28 flex items-center justify-between px-10 border-b border-white/[0.03] bg-[#09090b]/50 backdrop-blur-md z-40">
        <!-- 左侧标题 -->
        <div class="flex items-center gap-5">
          <div class="p-3 bg-blue-600/10 rounded-2xl hidden sm:block">
            <Activity class="text-blue-500 w-6 h-6" />
          </div>
          <div>
            <h1 class="text-2xl font-bold tracking-tight text-white">
              SafeMask <span class="text-zinc-500 font-medium text-xl ml-1">脱敏控制台</span>
            </h1>
            <p class="text-xs text-zinc-500 uppercase tracking-[0.25em] font-bold mt-1">隐私防护引擎 v1.1.3</p>
          </div>
        </div>

        <!-- 右侧系统控制组 -->
        <div class="flex items-center gap-4">
          
          <!-- 置顶按钮 -->
          <button 
            @click="store.toggleAlwaysOnTop"
            class="p-3.5 rounded-2xl border transition-all duration-300 flex items-center justify-center hover:scale-105 active:scale-95"
            :class="store.isAlwaysOnTop 
              ? 'bg-blue-600 border-blue-400 text-white shadow-[0_0_20px_rgba(59,130,246,0.5)]' 
              : 'bg-zinc-900 border-zinc-800 text-zinc-500 hover:border-zinc-700'"
            :title="store.isAlwaysOnTop ? '取消窗口置顶' : '固定窗口至最前'"
          >
            <component :is="store.isAlwaysOnTop ? PinOff : Pin" :size="20" />
          </button>

          <!-- 🛡️ 状态胶囊：文字加大且视觉更圆润 -->
          <div class="flex items-center gap-5 bg-zinc-900 border border-zinc-800 h-16 px-6 rounded-[1.5rem] shadow-inner">
            <div class="flex flex-col items-start">
              <!-- 主状态：字号 text-[15px]，半粗体 font-semibold 显得更柔和 -->
              <span 
                class="text-[15px] font-semibold leading-none mb-2 tracking-wide transition-colors duration-300" 
                :class="store.isMonitorOn ? 'text-blue-400' : 'text-zinc-500'"
              >
                {{ store.isMonitorOn ? '自动保护已开启' : '实时防护已关闭' }}
              </span>
              <!-- 副标题：字号 text-[12px] -->
              <span class="text-[12px] font-medium text-zinc-500 leading-none tracking-[0.15em] opacity-80">
                系统哨兵监控模式
              </span>
            </div>
            
            <!-- 开关按钮：稍微放大以匹配整体比例 -->
            <button 
              @click="store.toggleMonitor"
              class="w-14 h-7 rounded-full relative transition-all duration-500 focus:outline-none overflow-hidden"
              :class="store.isMonitorOn ? 'bg-blue-600' : 'bg-zinc-800'"
            >
              <!-- 开关轨道内的发光装饰 -->
              <div v-if="store.isMonitorOn" class="absolute inset-0 bg-gradient-to-r from-blue-400/20 to-transparent"></div>
              
              <div 
                class="absolute top-1 left-1 bg-white w-5 h-5 rounded-full transition-transform duration-500 shadow-xl z-10"
                :class="{ 'translate-x-7': store.isMonitorOn }"
              ></div>
            </button>
          </div>
        </div>
      </header>

      <!-- 动态内容区 -->
      <div class="flex-1 overflow-y-auto custom-scroll px-10 py-10">
        <!-- 页面 1: 仪表盘 -->
        <div v-if="store.activeTab === 'dashboard'" class="space-y-10 animate-in fade-in duration-500">
          <div class="grid grid-cols-3 gap-8">
            <StatCard title="已加载规则" :value="store.ruleCount" unit="条规则" clickable @click="store.activeTab = 'rules'"/>
            <StatCard title="历史拦截" :value="store.historyList.length" unit="条记录" color="text-amber-400" clickable @click="store.activeTab = 'history'" />
            <StatCard title="引擎架构" value="HYBRID" unit="ENGINE" color="text-blue-400" />
          </div>
          <FileProcessor class="min-h-[380px]" />
        </div>

        <HistoryList v-else-if="store.activeTab === 'history'" />
        <RuleManager v-else-if="store.activeTab === 'rules'" />
        <SettingsPage v-else-if="store.activeTab === 'settings'" />

        <!-- 汉化页脚 -->
        <footer v-if="store.activeTab === 'dashboard'" class="py-12 flex justify-center items-center gap-4 opacity-20">
          <div class="h-px w-10 bg-zinc-500"></div>
          <p class="text-[10px] font-mono uppercase tracking-[0.3em] text-zinc-400">全本地化安全运行环境</p>
          <div class="h-px w-10 bg-zinc-500"></div>
        </footer>
      </div>
      
      <!-- 置顶反馈边框 -->
      <div v-if="store.isAlwaysOnTop" class="absolute inset-0 pointer-events-none border-2 border-blue-500/20 z-50"></div>
    </main>

    <ExitConfirm />
  </div>
</template>

<style>
/* 保持滚动条隐藏 */
::-webkit-scrollbar {
  display: none;
}

.custom-scroll {
  scrollbar-width: none;
  -ms-overflow-style: none;
}

/* 统一玻璃背景 */
.glass {
  background: rgba(18, 18, 23, 0.7);
  backdrop-filter: blur(16px);
  -webkit-backdrop-filter: blur(16px);
  border: 1px solid rgba(255, 255, 255, 0.05);
}
</style>