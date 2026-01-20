<script setup lang="ts">
import { onMounted } from 'vue';
import { listen } from "@tauri-apps/api/event";
import { useAppStore } from './stores/useAppStore';
import Sidebar from './components/Sidebar.vue';
import StatCard from './components/StatCard.vue';
import FileProcessor from './components/FileProcessor.vue';

const store = useAppStore();

onMounted(async () => {
  // 1. 初始化统计信息
  await store.fetchStats();

  // 2. 监听后端回传的进度事件
  await listen<{ percentage: number }>("file-progress", (event) => {
    store.progress = event.payload.percentage;
  });

  // 3. 监听脱敏通知
  await listen("masked-event", (event) => {
    console.info("🛡️ SafeMask:", event.payload);
    // 这里可以集成更高级的 Toast 组件
  });
});
</script>

<template>
  <div class="flex h-screen bg-[#09090b] text-zinc-100 select-none overflow-hidden">
    <Sidebar />

    <main class="flex-1 p-12 flex flex-col max-w-6xl mx-auto w-full">
      <header class="flex justify-between items-center mb-12">
        <div>
          <h1 class="text-4xl font-extrabold tracking-tight bg-clip-text text-transparent bg-gradient-to-b from-white to-zinc-500">
            SafeMask 控制台
          </h1>
          <p class="text-zinc-500 mt-2 text-lg">实时保护剪贴板与大规模日志隐私</p>
        </div>

        <!-- 监控开关 -->
        <div class="glass px-6 py-4 rounded-[2rem] flex items-center gap-4">
          <span class="text-sm font-semibold">自动保护</span>
          <button 
            @click="store.toggleMonitor"
            class="w-12 h-6 rounded-full relative transition-colors duration-300"
            :class="store.isMonitorOn ? 'bg-blue-600' : 'bg-zinc-700'"
          >
            <div 
              class="absolute top-1 left-1 bg-white w-4 h-4 rounded-full transition-transform duration-300"
              :class="{ 'translate-x-6': store.isMonitorOn }"
            ></div>
          </button>
        </div>
      </header>

      <!-- 统计栏 -->
      <div class="grid grid-cols-3 gap-8 mb-12">
        <StatCard title="已加载规则" :value="store.ruleCount" />
        <StatCard title="引擎架构" value="HYBRID" color="text-blue-400" />
        <StatCard title="内存占用" value="LOW (MMAP)" color="text-emerald-400" />
      </div>

      <FileProcessor />
    </main>
  </div>
</template>

<style>
.glass {
  background: rgba(24, 24, 27, 0.8);
  backdrop-filter: blur(12px);
  border: 1px solid rgba(255, 255, 255, 0.08);
}
</style>