<script setup lang="ts">
/**
 * StatCard 组件
 * 用于展示统计数据，支持自定义颜色、单位以及可选的点击交互效果
 */

interface Props {
  title: string;       // 卡片标题
  value: string | number; // 显示的数值
  color?: string;      // 数值颜色类名 (如 text-blue-400)
  unit?: string;       // 数值后的单位 (如 ITEMS)
  clickable?: boolean; // 🚀 是否启用点击交互逻辑
}

// 显式定义 Props，设置默认值
const props = withDefaults(defineProps<Props>(), {
  clickable: false,
  color: 'text-zinc-100'
});

// 定义组件发射的事件（可选，但推荐规范化）
defineEmits(['click']);
</script>

<template>
  <div 
    @click="$emit('click')"
    class="glass p-8 rounded-[2.5rem] relative overflow-hidden group transition-all duration-300 border border-white/5"
    :class="[
      // 🚀 如果是可点击的，应用交互样式
      props.clickable 
        ? 'cursor-pointer hover:border-blue-500/40 hover:bg-blue-600/5 active:scale-95' 
        : 'cursor-default'
    ]"
  >
    <!-- 背景微光特效 (只在可点击时增加额外的悬浮亮度) -->
    <div 
      class="absolute -right-4 -top-4 w-32 h-32 bg-blue-500/5 rounded-full blur-3xl transition-opacity duration-500"
      :class="props.clickable ? 'group-hover:opacity-100 opacity-40' : 'opacity-20'"
    ></div>
    
    <div class="relative z-10">
      <!-- 标题部分 -->
      <p 
        class="text-zinc-500 text-[10px] uppercase font-extrabold tracking-[0.2em] mb-4 transition-colors"
        :class="{ 'group-hover:text-blue-400': props.clickable }"
      >
        {{ title }}
      </p>
      
      <!-- 数值与单位 -->
      <div class="flex items-baseline gap-2">
        <p class="text-4xl font-mono font-bold tracking-tighter" :class="color">
          {{ value }}
        </p>
        <span v-if="unit" class="text-[10px] text-zinc-600 font-sans font-bold uppercase tracking-tighter">
          {{ unit }}
        </span>
      </div>
    </div>

    <!-- 🚀 如果可点击，在右下角显示一个小箭头暗示交互 (可选增强) -->
    <div 
      v-if="props.clickable" 
      class="absolute right-6 bottom-6 opacity-0 group-hover:opacity-100 transition-all translate-x-2 group-hover:translate-x-0"
    >
      <svg class="w-4 h-4 text-blue-500/50" fill="none" stroke="currentColor" viewBox="0 0 24 24">
        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="3" d="M13 7l5 5m0 0l-5 5m5-5H6" />
      </svg>
    </div>
  </div>
</template>

<style scoped>
/* 使用之前定义的 Glass 样式 */
.glass {
  background: rgba(18, 18, 23, 0.6);
  backdrop-filter: blur(12px);
  box-shadow: 0 4px 24px -1px rgba(0, 0, 0, 0.2);
}

/* 之前 style.css 里的等宽数字类 */
.font-mono-numbers {
  font-family: 'JetBrains Mono', 'Fira Code', monospace;
  font-variant-numeric: tabular-nums;
}
</style>