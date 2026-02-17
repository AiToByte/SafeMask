<script setup lang="ts">
import { ArrowUpRight } from 'lucide-vue-next';
interface Props {
  title: string;
  value: string | number;
  color?: string; // 文本颜色类名
  unit?: string;
  clickable?: boolean;
  type?: 'amber' | 'blue' | 'emerald'; // 🚀 新增：卡片主题类型
}
const props = withDefaults(defineProps<Props>(), {
  clickable: false,
  color: 'text-zinc-100',
  type: 'blue'
});

// 根据类型计算独特的背景阴影和微光颜色
const typeStyles = {
  amber: 'border-amber-500/20 bg-amber-500/[0.015] shadow-amber-900/10',
  blue: 'border-blue-500/20 bg-blue-500/[0.015] shadow-blue-900/10',
  emerald: 'border-emerald-500/20 bg-emerald-500/[0.015] shadow-emerald-900/10'
};
</script>

<template>
  <div @click="$emit('click')"
       class="relative group px-7 py-6 rounded-[1.25rem] border transition-all duration-700 overflow-hidden h-36"
       :class="[
         typeStyles[props.type],
         clickable ? 'cursor-pointer hover:translate-y-[-2px] hover:border-white/20 hover:bg-white/[0.03]' : ''
       ]">
    
    <!-- 🚀 背景氛围光：极淡的对应色光晕 -->
    <div class="absolute -right-8 -bottom-8 w-32 h-32 blur-3xl opacity-20 pointer-events-none"
         :class="props.type === 'amber' ? 'bg-amber-500' : props.type === 'blue' ? 'bg-blue-500' : 'bg-emerald-500'"></div>

    <div class="relative z-10 flex flex-col justify-between h-full">
      <div class="flex justify-between items-start">
        <!-- 标题：使用微黄护眼色 text-amber-50/50 -->
        <p class="text-[10px] font-bold text-zinc-500 group-hover:text-amber-100/60 transition-colors tracking-[0.25em] uppercase">
          {{ title }}
        </p>
        <div v-if="clickable" class="opacity-20 group-hover:opacity-100 transition-all">
           <ArrowUpRight :size="14" class="text-zinc-400" />
        </div>
      </div>
      
      <div class="flex items-baseline gap-3">
        <!-- 🚀 数值：字号缩小至 4xl，使用 font-medium 显得更轻盈雅致 -->
        <p class="text-4xl font-mono font-medium tracking-tighter tabular-nums leading-none" 
           :class="color">
          {{ value }}
        </p>
        <span class="text-[9px] font-black text-zinc-600 uppercase tracking-widest mb-1">
          {{ unit }}
        </span>
      </div>
    </div>

    <!-- 左侧发光灯条：更细、更透 -->
    <div class="absolute left-0 top-0 bottom-0 w-[1.5px] opacity-40 group-hover:opacity-100 transition-opacity"
         :class="props.type === 'amber' ? 'bg-amber-400' : props.type === 'blue' ? 'bg-blue-400' : 'bg-emerald-400'"></div>
  </div>
</template>