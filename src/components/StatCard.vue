<script setup lang="ts">
interface Props {
  title: string;
  value: string | number;
  color?: string; // 可选的文字颜色类名
}

defineProps<Props>();
</script>

<template>
  <div class="glass p-8 rounded-[2.5rem] relative overflow-hidden group transition-all duration-500 hover:border-zinc-700">
    <!-- 背景微光特效 -->
    <div class="absolute -right-4 -top-4 w-24 h-24 bg-blue-500/5 rounded-full blur-3xl group-hover:bg-blue-500/10 transition-colors"></div>
    
    <p class="text-zinc-500 text-xs uppercase tracking-[0.2em] mb-3 font-semibold">
      {{ title }}
    </p>
    
    <div class="flex items-baseline gap-2">
      <p 
        class="text-4xl font-bold font-mono tracking-tighter transition-all"
        :class="color || 'text-zinc-100'"
      >
        {{ value }}
      </p>
      
      <!-- 如果是 ACTIVE 状态，增加一个小呼吸灯 -->
      <span v-if="value === 'HYBRID'" class="flex h-2 w-2 relative">
        <span class="animate-ping absolute inline-flex h-full w-full rounded-full bg-blue-400 opacity-75"></span>
        <span class="relative inline-flex rounded-full h-2 w-2 bg-blue-500"></span>
      </span>
    </div>
  </div>
</template>

<style scoped>
.glass {
  background: rgba(18, 18, 23, 0.6);
  backdrop-filter: blur(12px);
  border: 1px solid rgba(255, 255, 255, 0.05);
  box-shadow: 0 8px 32px 0 rgba(0, 0, 0, 0.3);
}
</style>