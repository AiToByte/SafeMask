<script setup lang="ts">
interface Props {
  title: string;
  value: string | number;
  color?: string;
  unit?: string;
}
defineProps<Props>();
</script>

<template>
  <div class="glass p-8 rounded-[2.5rem] relative overflow-hidden group transition-all duration-500 hover:border-zinc-700/50 border border-white/5">
    <div class="absolute -right-4 -top-4 w-24 h-24 bg-blue-500/5 rounded-full blur-3xl group-hover:bg-blue-500/10 transition-colors"></div>
    
    <div class="flex justify-between items-start mb-3">
      <p class="text-zinc-500 text-[10px] uppercase tracking-[0.2em] font-bold">
        {{ title }}
      </p>
      <div v-if="color" class="w-1.5 h-1.5 rounded-full bg-blue-500 animate-pulse"></div>
    </div>
    
    <div class="flex items-baseline gap-2">
      <p class="text-4xl font-bold font-mono tracking-tighter" :class="color || 'text-zinc-100'">
        {{ value }}
      </p>
      <span v-if="unit" class="text-[10px] text-zinc-600 font-mono font-bold">{{ unit }}</span>
    </div>
  </div>
</template>