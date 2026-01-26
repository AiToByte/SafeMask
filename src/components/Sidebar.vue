<script setup lang="ts">
import { ref } from 'vue';
import { Home, Library, Settings, ShieldCheck, ClipboardCopy } from 'lucide-vue-next';
import { useAppStore } from '../stores/useAppStore';

// 🚀 获取全局状态 Store
const store = useAppStore();
const activeTab = ref('dashboard');

/**
 * 菜单配置项
 * id 必须与 App.vue 中 v-if 的判断条件字符串严格对应
 */
const menuItems = [
  { id: 'dashboard', icon: Home, label: '仪表盘' },
  { id: 'history', icon: ClipboardCopy, label: '记录对比' }, // 修改 ID 为 history 以匹配 App.vue
  { id: 'rules', icon: Library, label: '规则管理' },
];
</script>

<template>
  <nav class="w-20 flex flex-col items-center py-8 bg-[#0c0c0e] border-r border-zinc-800/50 z-50">
    <!-- Logo -->
    <div class="w-12 h-12 bg-gradient-to-br from-blue-500 to-indigo-600 rounded-2xl flex items-center justify-center shadow-lg shadow-blue-500/20 mb-12 group cursor-pointer">
      <ShieldCheck class="text-white w-7 h-7 group-hover:scale-110 transition-transform" />
    </div>
    
    <!-- 导航项 -->
    <div class="flex flex-col gap-6 flex-1">
      <button 
        v-for="item in menuItems" 
        :key="item.id"
        @click="store.activeTab = item.id"
        :title="item.label"
        class="sidebar-item group"
        :class="[
          store.activeTab === item.id 
            ? 'bg-blue-600/10 text-blue-400 !border-blue-500/30 shadow-[inset_0_0_12px_rgba(59,130,246,0.1)]' 
            : 'hover:bg-zinc-800/50 hover:text-zinc-200 border-transparent'
        ]"
      >
        <component :is="item.icon" class="w-6 h-6" :stroke-width="store.activeTab === item.id ? 2.5 : 2" />
        
        <!-- 活动指示条 (左侧蓝条) -->
        <div 
          v-if="store.activeTab === item.id" 
          class="absolute -left-4 w-1 h-6 bg-blue-500 rounded-r-full shadow-[0_0_15px_rgba(59,130,246,0.8)]"
        ></div>

        <!-- 悬浮 Tooltip 提示 -->
        <span class="absolute left-full ml-4 px-2 py-1 bg-zinc-800 text-white text-[10px] rounded opacity-0 group-hover:opacity-100 transition-opacity whitespace-nowrap pointer-events-none z-50">
          {{ item.label }}
        </span>
      </button>
    </div>

    <!-- 设置 -->
    <button class="sidebar-item mt-auto hover:bg-zinc-800/50 hover:text-zinc-200 border-transparent">
      <Settings class="w-6 h-6" />
    </button>
  </nav>
</template>

<style scoped>
/* 可以在这里添加一些特定于侧边栏的细微过渡 */
.sidebar-item svg {
    transition: transform 0.2s ease;
}
.sidebar-item:active svg {
    transform: scale(0.9);
}
</style>