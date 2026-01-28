<script setup lang="ts">
import { useAppStore } from '../stores/useAppStore';
import { ClipboardCopy, ClipboardCheck, History, CornerDownRight, Clock } from 'lucide-vue-next';
import { onMounted, ref } from 'vue';
import { MaskAPI } from '../services/api';

const store = useAppStore();
const copiedId = ref("");

// 🚀 核心：进入页面时立即拉取后端存储的历史
onMounted(async () => {
  await store.fetchHistory();
});

// 🚀 调用特殊 API 复制原文
const handleCopyOriginal = async (id: string, text: string) => {
  await MaskAPI.copyOriginal(text);
  copiedId.value = id + "_org";
  setTimeout(() => copiedId.value = "", 2000);
};

// 常规脱敏后内容的复制
const handleCopyMasked = async (id: string, text: string) => {
  await navigator.clipboard.writeText(text);
  copiedId.value = id + "_msk";
  setTimeout(() => copiedId.value = "", 2000);
};
</script>

<template>
  <div class="flex flex-col gap-6 animate-in fade-in slide-in-from-bottom-4 duration-500">
    <div v-if="store.historyList.length === 0" class="flex flex-col items-center justify-center py-20 text-zinc-600">
      <div class="text-5xl mb-4 opacity-20">📭</div>
      <p>暂无脱敏历史记录</p>
    </div>

    <div v-for="item in store.historyList" :key="item.id" class="glass p-6 rounded-[2rem] space-y-4 hover:border-blue-500/30 transition-colors group">
      <div class="flex justify-between items-center">
        <div class="flex items-center gap-2 text-zinc-500 text-xs font-mono">
          <Clock :size="14" />
          {{ item.timestamp }}
        </div>
        <div class="px-3 py-1 rounded-full bg-blue-500/10 text-blue-400 text-[10px] font-bold uppercase tracking-wider">Masked</div>
      </div>

<div class="grid grid-cols-2 gap-6 relative">
        <!-- 1. 原始输入区 -->
        <div class="space-y-3">
          <div class="flex justify-between items-center px-1">
            <p class="text-[10px] text-zinc-500 font-black uppercase tracking-tighter">Raw Original</p>
            <!-- 🚀 关键：复制原文按钮 -->
            <button @click="handleCopyOriginal(item.id, item.original)" 
                    class="text-[10px] flex items-center gap-1.5 transition-all"
                    :class="copiedId === item.id + '_org' ? 'text-emerald-400' : 'text-zinc-500 hover:text-zinc-300'">
              <component :is="copiedId === item.id + '_org' ? ClipboardCheck : ClipboardCopy" :size="12" />
              {{ copiedId === item.id + '_org' ? 'COPIED (CLEAN)' : 'COPY ORIGINAL' }}
            </button>
          </div>
          <div class="bg-black/40 p-5 rounded-3xl text-xs text-zinc-400 font-mono leading-relaxed break-all border border-transparent hover:border-zinc-800 transition-all">
            {{ item.original }}
          </div>
        </div>

        <!-- 装饰箭头 -->
        <div class="absolute left-1/2 top-1/2 -translate-x-1/2 -translate-y-1/2 text-zinc-800 z-10 hidden lg:block">
           <CornerDownRight :size="24" />
        </div>

        <!-- 2. 脱敏结果区 -->
        <div class="space-y-3">
          <div class="flex justify-between items-center px-1">
            <p class="text-[10px] text-blue-500 font-black uppercase tracking-tighter text-glow">Protected Output</p>
            <button @click="handleCopyMasked(item.id, item.masked)" 
                    class="text-[10px] flex items-center gap-1.5 transition-all"
                    :class="copiedId === item.id + '_msk' ? 'text-emerald-400' : 'text-blue-500/70 hover:text-blue-400'">
              <component :is="copiedId === item.id + '_msk' ? ClipboardCheck : ClipboardCopy" :size="12" />
              COPY PROTECTED
            </button>
          </div>
          <div class="bg-blue-500/5 p-5 rounded-3xl text-xs text-blue-100 font-mono leading-relaxed break-all border border-blue-500/10 shadow-inner">
            {{ item.masked }}
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<style scoped>
.text-glow {
  text-shadow: 0 0 8px rgba(59, 130, 246, 0.3);
}
</style>