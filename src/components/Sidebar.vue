<script setup lang="ts">
import { ref } from 'vue';
import { Home, Library, Settings, ShieldCheck } from 'lucide-vue-next';

// 当前激活的菜单项
const activeTab = ref('dashboard');

const menuItems = [
  { id: 'dashboard', icon: Home, label: '仪表盘' },
  { id: 'rules', icon: Library, label: '规则库' },
];
</script>

<template>
  <nav class="w-20 flex flex-col items-center py-8 bg-[#0c0c0e] border-r border-zinc-800/50 z-50">
    <!-- Logo 区域 -->
    <div class="w-12 h-12 bg-gradient-to-br from-blue-500 to-indigo-600 rounded-2xl flex items-center justify-center shadow-lg shadow-blue-500/20 mb-12 group cursor-pointer">
      <ShieldCheck class="text-white w-7 h-7 group-hover:scale-110 transition-transform" />
    </div>
    
    <!-- 中间导航项 -->
    <div class="flex flex-col gap-6 flex-1">
      <button 
        v-for="item in menuItems" 
        :key="item.id"
        @click="activeTab = item.id"
        :title="item.label"
        class="w-12 h-12 flex items-center justify-center rounded-2xl transition-all duration-300 group relative"
        :class="activeTab === item.id ? 'bg-blue-600/10 text-blue-400 border border-blue-500/20' : 'text-zinc-500 hover:bg-zinc-800/50 hover:text-zinc-200'"
      >
        <component :is="item.icon" class="w-6 h-6" />
        
        <!-- 活动状态的小指示条 -->
        <div 
          v-if="activeTab === item.id" 
          class="absolute -left-4 w-1 h-6 bg-blue-500 rounded-r-full shadow-[0_0_8px_rgba(59,130,246,0.5)]"
        ></div>
      </button>
    </div>

    <!-- 底部设置项 -->
    <button class="w-12 h-12 flex items-center justify-center rounded-2xl text-zinc-500 hover:bg-zinc-800/50 hover:text-zinc-200 transition-all">
      <Settings class="w-6 h-6" />
    </button>
  </nav>
</template>