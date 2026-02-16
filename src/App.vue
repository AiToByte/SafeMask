<script setup lang="ts">
import { onMounted } from 'vue';
import { useAppStore } from './stores/useAppStore';
import { Shield, Ghost, Activity, Settings as SettingsIcon } from 'lucide-vue-next';
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
  <div class="flex h-screen bg-[#09090b] text-zinc-100 select-none overflow-hidden font-sans">
    <MagicFeedback />
    <Sidebar />

    <main class="flex-1 flex flex-col min-w-0 relative">
      <header class="h-28 flex items-center justify-between px-10 border-b border-white/[0.03] bg-[#09090b]/50 backdrop-blur-md z-40">
        <div class="flex items-center gap-5">
          <div class="p-3 bg-blue-600/10 rounded-2xl">
            <Activity class="text-blue-500 w-6 h-6" />
          </div>
          <div>
            <h1 class="text-2xl font-bold text-white">SafeMask <span class="text-zinc-500 font-medium text-xl ml-1">脱敏控制台</span></h1>
            <p class="text-[10px] text-zinc-500 uppercase tracking-[0.25em] font-bold mt-1">Version {{ store.appInfo?.version || '1.1.3' }}</p>
          </div>
        </div>

        <div class="flex items-center gap-4">
          <!-- 宇宙模式状态胶囊 -->
          <div @click="store.toggleVaultMode" class="flex items-center gap-4 bg-zinc-900 border border-zinc-800 h-14 px-6 rounded-2xl shadow-inner cursor-pointer hover:border-zinc-600 transition-all">
            <div class="flex flex-col items-start min-w-[100px]">
              <span class="text-xs font-black uppercase tracking-widest" :class="store.settings.shadow_mode_enabled ? 'text-blue-400' : 'text-amber-500'">
                {{ store.settings.shadow_mode_enabled ? '影子宇宙' : '哨兵宇宙' }}
              </span>
              <span class="text-[10px] text-zinc-500 font-bold">ALT + M 快速切换</span>
            </div>
            <div class="p-2 rounded-xl" :class="store.settings.shadow_mode_enabled ? 'bg-blue-600/10' : 'bg-amber-600/10'">
              <component :is="store.settings.shadow_mode_enabled ? Ghost : Shield" :size="18" :class="store.settings.shadow_mode_enabled ? 'text-blue-400' : 'text-amber-500'" />
            </div>
          </div>
        </div>
      </header>

      <div class="flex-1 overflow-y-auto custom-scroll px-10 py-10">
        <div v-if="store.activeTab === 'dashboard'" class="space-y-10 animate-in fade-in duration-500">
          <div class="grid grid-cols-3 gap-8">
            <StatCard title="已加载规则" :value="store.ruleCount" unit="条规则" clickable @click="store.activeTab = 'rules'"/>
            <StatCard title="宇宙发现记录" :value="store.historyList.length" unit="条审计" color="text-blue-400" clickable @click="store.activeTab = 'history'" />
            <StatCard title="引擎架构" value="HYBRID" unit="HighPerf" color="text-emerald-400" />
          </div>
          <FileProcessor class="min-h-[380px]" />
        </div>
        <HistoryList v-else-if="store.activeTab === 'history'" />
        <RuleManager v-else-if="store.activeTab === 'rules'" />
        <SettingsPage v-else-if="store.activeTab === 'settings'" />
      </div>
    </main>
    <ExitConfirm />
  </div>
</template>