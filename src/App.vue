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
              Secure Core Engine · v1.2.2
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
          <!-- 修改 App.vue 中的模式切换胶囊部分 -->
<div @click="store.toggleVaultMode" 
     class="group relative flex items-center gap-5 bg-[#141210] border border-white/[0.08] h-12 px-6 rounded-2xl cursor-pointer hover:border-amber-500/30 transition-all duration-500 active:scale-95 shadow-xl">
  
  <!-- 🚀 雅致的自定义悬浮说明浮层 -->
  <div class="absolute top-full mb-4 right-0 w-72 p-4 rounded-2xl bg-[#1d1b18] border border-amber-500/20 shadow-2xl opacity-0 translate-y-2 pointer-events-none group-hover:opacity-100 group-hover:translate-y-0 transition-all duration-300 z-[100]">
    <div class="flex items-center gap-2 mb-2">
      <div class="w-1.5 h-1.5 rounded-full bg-amber-500"></div>
      <span class="text-xs font-bold text-amber-200">运行模式详情</span>
    </div>
      <p class="text-[11px] text-zinc-400 leading-relaxed">
        <template v-if="store.settings.shadow_mode_enabled">
          <strong class="text-amber-200/80">影子宇宙模式：</strong> 
          系统仅在后台静默记录敏感信息，不改变剪贴板。需按下 <code class="bg-black/40 px-1 rounded text-amber-500">{{ store.settings.magic_paste_shortcut }}</code> 才会粘贴脱敏副本。
        </template>
        <template v-else>
          <strong class="text-blue-400/80">哨兵宇宙模式：</strong> 
          全自动强力拦截。检测到敏感隐私时，系统会自动实时洗白剪贴板，确保存储与发送的始终是脱敏数据。
        </template>
      </p>
        <!-- 装饰三角 -->
        <div class="absolute bottom-full right-8 w-3 h-3 bg-[#1d1b18] border-r border-b border-amber-500/20 rotate-45 -translate-y-1.5"></div>
          </div>

            <div class="flex flex-col items-end">
              <span class="text-[9px] font-black text-zinc-600 uppercase tracking-tighter mb-0.5">Universe Mode</span>
              <span class="text-xs font-bold tracking-widest transition-colors duration-300" 
                    :class="store.settings.shadow_mode_enabled ? 'text-amber-200' : 'text-blue-300'">
                {{ store.settings.shadow_mode_enabled ? '影子宇宙模式' : '哨兵宇宙模式' }}
              </span>
            </div>

            <div class="w-8 h-8 flex items-center justify-center rounded-xl bg-white/[0.02] border border-white/5 relative">
              <div class="absolute inset-0 rounded-xl blur-sm opacity-20 animate-pulse"
                  :class="store.settings.shadow_mode_enabled ? 'bg-amber-400' : 'bg-blue-400'"></div>
              <component :is="store.settings.shadow_mode_enabled ? Ghost : Shield" 
                        :size="14" 
                        :class="store.settings.shadow_mode_enabled ? 'text-amber-200' : 'text-blue-300'" />
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