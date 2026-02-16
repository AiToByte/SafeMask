<script setup lang="ts">
import { useAppStore } from '../stores/useAppStore';
import { ClipboardCopy, ClipboardCheck, CornerDownRight, Clock, Ghost, ShieldAlert, Trash2, Search } from 'lucide-vue-next';
import { onMounted, ref, computed } from 'vue';
import { MaskAPI } from '../services/api';

const store = useAppStore();
const copiedId = ref("");
const searchQuery = ref("");

onMounted(() => store.fetchHistory());

const handleCopy = async (id: string, text: string, type: 'org' | 'msk') => {
  if (type === 'org') await MaskAPI.copyOriginal(text);
  else await navigator.clipboard.writeText(text);
  copiedId.value = id + "_" + type;
  setTimeout(() => copiedId.value = "", 2000);
};

const filteredHistory = computed(() => {
  if (!searchQuery.value) return store.historyList;
  const q = searchQuery.value.toLowerCase();
  return store.historyList.filter(i => i.original.toLowerCase().includes(q) || i.masked.toLowerCase().includes(q));
});
</script>

<template>
  <div class="flex flex-col gap-6 animate-in fade-in slide-in-from-bottom-4 duration-500 pb-20">
    <div class="flex justify-between items-center px-2">
      <div class="relative w-72 group">
        <Search class="absolute left-3 top-1/2 -translate-y-1/2 text-zinc-500" :size="14" />
        <input v-model="searchQuery" placeholder="搜索泄露项内容..." class="w-full bg-zinc-900/50 border border-white/5 rounded-xl py-2.5 pl-10 pr-4 text-xs outline-none focus:border-blue-500/30" />
      </div>
      <button @click="store.clearHistory" class="text-[10px] font-black text-zinc-600 hover:text-red-400 transition-colors uppercase tracking-[0.2em] flex items-center gap-2">
        <Trash2 :size="12" /> Destroy All Records
      </button>
    </div>

    <div v-for="item in filteredHistory" :key="item.id" class="p-6 rounded-[2.5rem] border border-white/5 bg-white/[0.01] hover:bg-white/[0.03] transition-all">
      <div class="flex justify-between items-center mb-5">
        <div class="flex items-center gap-3">
          <div class="flex items-center gap-1.5 text-zinc-500 text-[10px] font-mono font-bold bg-black/40 px-3 py-1 rounded-lg">
            <Clock :size="12" /> {{ item.timestamp }}
          </div>
          <div v-if="item.mode === 'SHADOW'" class="flex items-center gap-1.5 px-3 py-1 rounded-lg text-[9px] font-black uppercase bg-blue-500/10 text-blue-400 border border-blue-500/20">
            <Ghost :size="10" /> 影子侦测
          </div>
          <div v-else class="flex items-center gap-1.5 px-3 py-1 rounded-lg text-[9px] font-black uppercase bg-amber-500/10 text-amber-500 border border-amber-500/20">
            <ShieldAlert :size="10" /> 哨兵拦截
          </div>
        </div>
      </div>

      <div class="grid grid-cols-1 lg:grid-cols-2 gap-6 relative">
        <div class="space-y-3">
          <div class="flex justify-between items-center px-1">
            <p class="text-[10px] text-zinc-600 font-black uppercase tracking-widest">Original Input</p>
            <button @click="handleCopy(item.id, item.original, 'org')" class="text-[9px] font-bold text-zinc-500 hover:text-white flex items-center gap-1">
              <component :is="copiedId === item.id + '_org' ? ClipboardCheck : ClipboardCopy" :size="12" />
              {{ copiedId === item.id + '_org' ? 'COPIED' : 'COPY RAW' }}
            </button>
          </div>
          <div class="p-5 rounded-3xl text-xs font-mono bg-black/40 text-zinc-500 border border-white/5 h-32 overflow-y-auto custom-scroll break-all">{{ item.original }}</div>
        </div>
        <div class="space-y-3">
          <div class="flex justify-between items-center px-1">
            <p class="text-[10px] font-black uppercase tracking-widest text-blue-500">Masked Output</p>
            <button @click="handleCopy(item.id, item.masked, 'msk')" class="text-[9px] font-bold text-blue-500/70 hover:text-blue-400 flex items-center gap-1">
              <component :is="copiedId === item.id + '_msk' ? ClipboardCheck : ClipboardCopy" :size="12" />
              COPY PROTECTED
            </button>
          </div>
          <div class="p-5 rounded-3xl text-xs font-mono bg-white/[0.02] text-zinc-200 border border-white/5 h-32 overflow-y-auto custom-scroll break-all shadow-inner">{{ item.masked }}</div>
        </div>
        <div class="absolute left-1/2 top-1/2 -translate-x-1/2 -translate-y-1/2 text-zinc-800 opacity-20 hidden lg:block"><CornerDownRight :size="20" /></div>
      </div>
    </div>
  </div>
</template>