<script setup lang="ts">
import { useAppStore } from '../stores/useAppStore';
import { ClipboardCopy, Clock } from 'lucide-vue-next';

const store = useAppStore();

const copyToClipboard = (text: string) => {
  navigator.clipboard.writeText(text);
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

      <div class="grid grid-cols-2 gap-4">
        <!-- 原始数据 -->
        <div class="space-y-2">
          <p class="text-[10px] text-zinc-500 font-bold uppercase ml-1">原始输入</p>
          <div class="bg-black/40 p-4 rounded-2xl text-xs text-zinc-400 font-mono line-clamp-3 relative group/box">
            {{ item.original }}
          </div>
        </div>
        <!-- 脱敏结果 -->
        <div class="space-y-2">
          <p class="text-[10px] text-blue-500/70 font-bold uppercase ml-1">保护后内容</p>
          <div class="bg-blue-500/5 p-4 rounded-2xl text-xs text-blue-100 font-mono line-clamp-3 border border-blue-500/10 relative">
            {{ item.masked }}
            <button @click="copyToClipboard(item.masked)" class="absolute right-2 bottom-2 p-2 rounded-lg bg-zinc-800 opacity-0 group-hover:opacity-100 transition-opacity">
               <ClipboardCopy :size="14" class="text-zinc-400" />
            </button>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>