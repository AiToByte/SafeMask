<script setup lang="ts">
import { ref } from 'vue';
import { Home, Library, Settings, ShieldCheck, ClipboardCopy } from 'lucide-vue-next';
import { useAppStore } from '../stores/useAppStore';

// 🚀 获取全局状态 Store
const store = useAppStore();
const activeTab = ref('dashboard');

/**
 * 菜单配置项
 * - id: 必须与 App.vue 中 v-if 的判断条件字符串严格对应
 * - icon: Lucide 图标组件
 * - label: 工具提示文本
 */
const menuItems = [
  { id: 'dashboard', icon: Home, label: '仪表盘' },
  { id: 'history', icon: ClipboardCopy, label: '记录对比' },
  { id: 'rules', icon: Library, label: '规则管理' },
]
</script>

<template>
  <nav class="w-20 flex flex-col items-center py-8 bg-[#0c0c0e] border-r border-zinc-800/50 z-50">
    <!-- Logo 区域：添加轻微 hover 动画，提升交互感 -->
    <div class="w-12 h-12 bg-gradient-to-br from-blue-500 to-indigo-600 rounded-2xl flex items-center justify-center shadow-lg shadow-blue-500/20 mb-12 group cursor-pointer transition-all duration-300 hover:shadow-blue-500/40 hover:scale-105">
      <ShieldCheck class="text-white w-7 h-7 group-hover:scale-110 transition-transform duration-300" />
    </div>
    
    <!-- 导航项列表 -->
    <div class="flex flex-col gap-6 flex-1">
      <button 
        v-for="item in menuItems" 
        :key="item.id"
        @click="store.activeTab = item.id"
        :title="item.label"
        class="sidebar-item relative group overflow-hidden"
        :class="[
          store.activeTab === item.id 
            ? 'text-blue-400 shadow-[inset_0_0_12px_rgba(59,130,246,0.1)]' 
            : 'hover:text-zinc-200'
        ]"
      >
        <!-- 背景流动渐变层（科技感流转效果）：仅在激活时显示 -->
        <div 
          v-if="store.activeTab === item.id"
          class="absolute inset-0 bg-gradient-to-r from-blue-600/20 via-indigo-500/20 to-blue-600/20 animate-flow"
        ></div>

        <!-- 图标本身 -->
        <component 
          :is="item.icon" 
          class="w-6 h-6 relative z-10 transition-all duration-300"
          :stroke-width="store.activeTab === item.id ? 2.5 : 2"
          :class="{ 'animate-glow': store.activeTab === item.id }"
        />
        
        <!-- 左侧活动指示条：优化为渐变色，添加轻微动画 -->
        <div 
          v-if="store.activeTab === item.id" 
          class="absolute -left-4 w-1 h-6 bg-gradient-to-b from-blue-500 to-indigo-500 rounded-r-full shadow-[0_0_15px_rgba(59,130,246,0.8)] animate-slide-in"
        ></div>

        <!-- 悬浮 Tooltip：优化为淡入动画，科技蓝边框 -->
        <span class="absolute left-full ml-4 px-3 py-1.5 bg-zinc-800/90 text-white text-xs font-medium rounded-lg border border-blue-500/20 shadow-lg opacity-0 group-hover:opacity-100 transition-all duration-300 translate-x-2 group-hover:translate-x-0 pointer-events-none z-50 whitespace-nowrap">
          {{ item.label }}
        </span>
      </button>
    </div>

    <!-- 设置按钮：底部固定，类似导航项样式 -->
    <button
      @click="store.activeTab = 'settings'"
      class="sidebar-item relative group overflow-hidden mt-auto"
      :class="{ 'text-blue-400': store.activeTab === 'settings' }"
    >
      <!-- 背景流动渐变层 -->
      <div 
        v-if="store.activeTab === 'settings'"
        class="absolute inset-0 bg-gradient-to-r from-blue-600/20 via-indigo-500/20 to-blue-600/20 animate-flow"
      ></div>

      <Settings 
        class="w-6 h-6 relative z-10"
        :class="{ 'animate-glow': store.activeTab === 'settings' }"
      />
    </button>
  </nav>
</template>

<style scoped>
/* 侧边栏项基础样式：圆角、固定尺寸、居中 */
.sidebar-item {
  @apply w-12 h-12 flex items-center justify-center rounded-xl transition-all duration-300;
}

/* 科技感流转效果：线性渐变背景从左到右流动 */
@keyframes flow {
  0% { background-position: 0% 50%; }
  50% { background-position: 100% 50%; }
  100% { background-position: 0% 50%; }
}
.animate-flow {
  background-size: 200% 100%;
  animation: flow 3s ease-in-out infinite;
}

/* Glow 脉冲效果：图标轻微发光，增强科技感 */
@keyframes glow {
  0%, 100% { filter: drop-shadow(0 0 4px rgba(59,130,246,0.3)); }
  50% { filter: drop-shadow(0 0 8px rgba(59,130,246,0.5)); }
}
.animate-glow {
  animation: glow 1.5s ease-in-out infinite;
}

/* 指示条滑入动画：从左淡入 */
@keyframes slide-in {
  from { transform: translateX(-100%); opacity: 0; }
  to { transform: translateX(0); opacity: 1; }
}
.animate-slide-in {
  animation: slide-in 0.3s ease-out forwards;
}
</style>