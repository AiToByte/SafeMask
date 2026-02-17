<script setup lang="ts">
import { onMounted } from 'vue';
import { useAppStore } from './stores/useAppStore';
import { Pin, PinOff,Shield, Ghost, Activity, Settings as SettingsIcon } from 'lucide-vue-next';
import Sidebar from './components/Sidebar.vue';
import MagicFeedback from './components/MagicFeedback.vue';
import FileProcessor from './components/FileProcessor.vue';
import HistoryList from './components/HistoryList.vue';
import RuleManager from './components/RuleManager.vue';
import SettingsPage from './components/Settings.vue';
import ExitConfirm from './components/ExitConfirm.vue';
import StatCard from './components/StatCard.vue';

const store = useAppStore();
onMounted(() => store.bootstrap());
</script>




<template>
  <div class="flex h-screen bg-[#0c0b0a] text-amber-50/90 select-none overflow-hidden font-sans">
    <MagicFeedback />
    <Sidebar />

    <main class="flex-1 flex flex-col min-w-0 relative">
      <!-- 琥珀色环境背光 -->
      <div class="absolute top-0 left-1/4 w-[60%] h-[30%] bg-amber-600/[0.02] blur-[120px] pointer-events-none"></div>

      <!-- 🚀 顶栏：高度大幅压缩至 h-20 (80px) -->
      <header class="h-20 flex items-center justify-between px-10 z-40 border-b border-white/[0.03] bg-[#0c0b0a]/60 backdrop-blur-xl">
        <div class="flex items-center gap-5">
          <div class="w-10 h-10 rounded-lg bg-[#141210] border border-amber-500/10 flex items-center justify-center shadow-2xl relative overflow-hidden group">
            <Activity class="text-amber-500 w-4 h-4 relative z-10" />
          </div>
          
          <div>
            <h1 class="text-lg font-bold tracking-tight text-amber-50/90 flex items-center gap-3">
              SafeMask
              <div class="h-3 w-[1px] bg-white/10"></div>
              <span class="text-zinc-500 font-medium text-sm tracking-widest">控制台</span>
            </h1>
            <p class="text-[8px] text-zinc-600 font-bold tracking-[0.1em] uppercase">
              Secure Core Engine · v1.1.3
            </p>
          </div>
        </div>

        <div class="flex items-center gap-3">
          <!-- 置顶按钮：尺寸更小巧 -->
          <button @click="store.toggleAlwaysOnTop"
            class="w-9 h-9 rounded-lg border transition-all duration-300 flex items-center justify-center active:scale-90"
            :class="store.isAlwaysOnTop 
              ? 'bg-amber-500/20 border-amber-500/40 text-amber-300 shadow-[0_0_15px_rgba(245,158,11,0.2)]' 
              : 'bg-white/[0.02] border-white/5 text-zinc-500 hover:border-amber-500/20'">
            <component :is="store.isAlwaysOnTop ? PinOff : Pin" :size="14" />
          </button>

          <!-- 模式切换：极致扁平 -->
          <div @click="store.toggleVaultMode" 
               class="group flex items-center gap-4 bg-[#141210] border border-white/[0.05] h-9 px-4 rounded-xl cursor-pointer hover:border-blue-500/30 transition-all duration-500 active:scale-95">
            <span class="text-[10px] font-bold tracking-widest transition-colors duration-300" 
                  :class="store.settings.shadow_mode_enabled ? 'text-amber-200/80' : 'text-blue-300/80'">
              {{ store.settings.shadow_mode_enabled ? '影子宇宙' : '哨兵宇宙' }}
            </span>
            <div class="w-5 h-5 flex items-center justify-center rounded-md bg-white/[0.02] border border-white/5">
              <component :is="store.settings.shadow_mode_enabled ? Ghost : Shield" :size="10" :class="store.settings.shadow_mode_enabled ? 'text-amber-200' : 'text-blue-300'" />
            </div>
          </div>
        </div>
      </header>

      <!-- 🚀 内容区：py-4 紧凑布局 -->
      <div class="flex-1 overflow-hidden px-10 py-4 flex flex-col">
        <Transition name="page" mode="out-in">
          <div :key="store.activeTab" class="max-w-6xl mx-auto w-full h-full flex flex-col">
            <div v-if="store.activeTab === 'dashboard'" class="flex-1 flex flex-col gap-4">
              
              <!-- 统计卡片：间距 gap-4 -->
              <div class="grid grid-cols-3 gap-4 shrink-0">
                <StatCard title="已装载脱敏规则" :value="store.ruleCount" unit="Patterns" color="text-amber-200" type="amber" clickable @click="store.activeTab = 'rules'"/>
                <StatCard title="累计隐私审计记录" :value="store.historyList.length" unit="Records" color="text-blue-300" type="blue" clickable @click="store.activeTab = 'history'" />
                <StatCard title="脱敏引擎状态" value="无损运行" unit="Normal" color="text-emerald-300" type="emerald" />
              </div>
              
              <!-- 🚀 文件处理区：flex-1 自动填充剩余空间 -->
              <div class="flex-1 min-h-0 relative">
                 <FileProcessor class="h-full bg-[#110f0e]/50 border border-white/[0.02] shadow-2xl" />
              </div>

              <!-- 极简页脚：减小内边距 -->
              <footer class="flex justify-center py-1 opacity-10">
                <p class="text-[7px] font-mono uppercase tracking-[0.5em] text-white">
                  Local Processing Instance
                </p>
              </footer>
            </div>

            <!-- 其他页面 -->
            <div v-else class="flex-1 overflow-y-auto custom-scroll">
               <HistoryList v-if="store.activeTab === 'history'" />
               <RuleManager v-else-if="store.activeTab === 'rules'" />
               <SettingsPage v-else-if="store.activeTab === 'settings'" />
            </div>
          </div>
        </Transition>
        <ExitConfirm />
      </div>
    </main>
  </div>
</template>

<style>
/* 🚀 页面切换平滑动画 */
.page-enter-active, .page-leave-active {
  transition: all 0.4s cubic-bezier(0.16, 1, 0.3, 1);
}
.page-enter-from { opacity: 0; transform: translateY(10px); }
.page-leave-to { opacity: 0; transform: translateY(-5px); }
</style>