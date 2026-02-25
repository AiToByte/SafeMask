<script setup lang="ts">
import { useAppStore } from '../stores/useAppStore';
import { 
  ClipboardCopy, ClipboardCheck, CornerDownRight, 
  Clock, Ghost, ShieldAlert, Trash2, Search, X 
} from 'lucide-vue-next';
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

const clearSearch = () => searchQuery.value = "";

// 🚀 修复后的搜索逻辑：支持 原始流、脱敏流、以及 Audit-ID 的全字匹配
const filteredHistory = computed(() => {
  if (!searchQuery.value) return store.historyList;
  
  const q = searchQuery.value.toLowerCase().trim();
  
  return store.historyList.filter(item => {
    // 1. 匹配原始文本
    const matchOriginal = item.original.toLowerCase().includes(q);
    // 2. 匹配脱敏后的文本
    const matchMasked = item.masked.toLowerCase().includes(q);
    // 3. 🚀 修正：仅匹配显示的短 ID (取第一段)
    const displayId = item.id.split('-')[0].toLowerCase();
    const matchId = displayId.includes(q);
    // 4. 🚀 新增：匹配时间戳 (例如用户搜 "17:57" 也能搜到)
    const matchTime = item.timestamp.includes(q);
    
    // 只有这四个可见部分命中，才算匹配成功
    return matchOriginal || matchMasked || matchId || matchTime;
  });
});
</script>

<template>
  <div class="flex flex-col gap-8 animate-in fade-in slide-in-from-bottom-4 duration-500 pb-20">
    
<!-- 顶部工具栏 -->
    <div class="flex flex-col gap-6 px-2">
      <div class="flex justify-between items-end">
        <div class="space-y-1">
          <h2 class="text-xl font-bold text-amber-50/80 tracking-tight">审计账本</h2>
          <p class="text-[10px] text-zinc-600 font-bold uppercase tracking-[0.3em]">Historical Audit Trail</p>
        </div>
        <button @click="store.clearHistory" class="destroy-btn group">
          <Trash2 :size="14" class="group-hover:text-red-400 transition-colors" /> 
          <span>销毁审计记录</span>
        </button>
      </div>

      <!-- 🚀 深度优化的搜索框模块 -->
      <div class="relative w-full max-w-2xl mx-auto group/search">
        
        <!-- 聚焦时的背景扩散扩散光晕 -->
        <div class="absolute -inset-2 bg-amber-500/[0.03] rounded-[2rem] blur-2xl opacity-0 group-focus-within/search:opacity-100 transition-opacity duration-700"></div>
        
        <div class="search-wrapper">
          <!-- 左侧图标：默认明显，聚焦变亮 -->
          <Search class="search-icon" :size="18" />
          
          <input 
            v-model="searchQuery" 
            type="text"
            placeholder="搜索原文、脱敏结果或 Audit-ID..." 
            class="search-input"
          />

          <!-- 快速清空按钮 -->
          <button v-if="searchQuery" @click="clearSearch" class="clear-btn">
            <X :size="14" />
          </button>

          <!-- 装饰：右侧精致的指示标识 -->
          <div class="search-tag">
             <div class="w-1 h-1 rounded-full bg-amber-500/40 mr-2"></div>
             <span>搜索</span>
          </div>
        </div>
      </div>
    </div>

    <!-- 列表内容保持精致度 -->
    <div v-if="filteredHistory.length === 0" class="flex flex-col items-center justify-center py-32 opacity-20">
       <Search :size="48" class="mb-4" />
       <p class="text-sm font-bold tracking-widest uppercase">暂无脱敏记录</p>
    </div>

    <!-- 🚀 搜索无结果时的占位提示 -->
    <div v-if="filteredHistory.length === 0 && searchQuery" 
        class="flex flex-col items-center justify-center py-32 animate-in fade-in zoom-in duration-700">
      <div class="relative mb-6">
        <div class="absolute inset-0 bg-amber-500/10 blur-3xl rounded-full"></div>
        <Search :size="48" class="text-zinc-800 relative z-10" />
      </div>
      <h3 class="text-amber-50/60 font-bold tracking-widest uppercase text-xs">No Audit Matches</h3>
      <p class="text-[10px] text-zinc-600 mt-2">未发现包含 "{{ searchQuery }}" 的审计项，请尝试其他关键词</p>
    </div>

    <div v-for="item in filteredHistory" :key="item.id" 
         class="history-card group/card">
      <div class="flex justify-between items-center mb-6">
        <div class="flex items-center gap-4">
          <div class="timestamp-tag">
            <Clock :size="12" /> {{ item.timestamp }}
          </div>
          
          <div v-if="item.mode === 'SHADOW'" class="mode-badge sentry">
            <Ghost :size="11" /> 影子宇宙侦测
          </div>
          <div v-else class="mode-badge shadow">
            <ShieldAlert :size="11" /> 哨兵宇宙拦截
          </div>
        </div>
        
        <span class="text-[9px] font-mono text-zinc-800 uppercase tracking-widest group-hover/card:text-zinc-600 transition-colors">
          Audit-ID: {{ item.id.split('-')[0] }}
        </span>
      </div>

      <div class="grid grid-cols-1 lg:grid-cols-2 gap-8 relative">
        <div class="space-y-3">
          <div class="flex justify-between items-center px-1">
            <p class="label-text">原始数据流 (Raw)</p>
            <button @click="handleCopy(item.id, item.original, 'org')" 
                    class="copy-action" :class="{ 'copied': copiedId === item.id + '_org' }">
              <component :is="copiedId === item.id + '_org' ? ClipboardCheck : ClipboardCopy" :size="12" />
              {{ copiedId === item.id + '_org' ? '已复制' : '复制原文' }}
            </button>
          </div>
          <div class="code-box original">{{ item.original }}</div>
        </div>

        <div class="absolute left-1/2 top-1/2 -translate-x-1/2 -translate-y-1/2 text-zinc-800 opacity-20 hidden lg:block">
           <CornerDownRight :size="24" />
        </div>

        <div class="space-y-3">
          <div class="flex justify-between items-center px-1">
            <p class="label-text accent" :class="item.mode === 'SHADOW' ? 'text-blue-500/80' : 'text-amber-500/80'">
              脱敏副本 (Masked)
            </p>
            <button @click="handleCopy(item.id, item.masked, 'msk')" 
                    class="copy-action msk" :class="{ 'copied': copiedId === item.id + '_msk' }">
              <component :is="copiedId === item.id + '_msk' ? ClipboardCheck : ClipboardCopy" :size="12" />
              {{ copiedId === item.id + '_msk' ? '已复制副本' : '复制副本' }}
            </button>
          </div>
          <div class="code-box masked">{{ item.masked }}</div>
        </div>
      </div>
    </div>
  </div>
</template>

<style scoped>
/* 🚀 搜索框容器：增强默认可见度 */
.search-wrapper {
  @apply relative flex items-center h-14 px-5 rounded-2xl transition-all duration-500 border;
  /* 默认状态：稍微提高琥珀色边框的透明度，确保在黑灰背景下清晰可见 */
  background: rgba(20, 18, 16, 0.9);
  border-color: rgba(245, 158, 11, 0.25); 
  box-shadow: 
    0 4px 20px -2px rgba(0, 0, 0, 0.5),
    inset 0 1px 1px rgba(255, 255, 255, 0.02);
}

.search-wrapper:hover {
  @apply border-amber-500/40 bg-[#0d0d0f];
  box-shadow: 0 12px 32px -8px rgba(0, 0, 0, 0.7);
}

.search-wrapper:focus-within {
  /* 聚焦状态：强化发光感 */
  @apply border-amber-500/60 scale-[1.01];
  background: #000000;
  box-shadow: 
    0 0 25px rgba(245, 158, 11, 0.08),
    0 20px 40px -15px rgba(0, 0, 0, 0.9);
}

/* 搜索图标：默认颜色加亮 */
.search-icon {
  @apply text-amber-500/60 group-focus-within/search:text-amber-400 transition-colors duration-500;
}

/* 搜索输入文本：象牙白 */
.search-input {
  @apply flex-1 bg-transparent border-none outline-none px-4 text-amber-50 font-medium text-sm placeholder:text-zinc-600 tracking-wide;
}

/* 右侧检索标识 */
.search-tag {
  @apply hidden sm:flex items-center pl-4 border-l border-white/10 text-[9px] font-black text-zinc-500 uppercase tracking-[0.2em];
}

/* 当搜索框有内容时，让标识闪烁，提示正在过滤 */
.search-wrapper:has(.search-input:not(:placeholder-shown)) .search-tag {
  @apply text-amber-500/80;
}

.search-tag {
  @apply hidden sm:flex items-center pl-4 border-l border-white/5 text-[9px] font-black text-zinc-600 uppercase tracking-widest;
}

.clear-btn {
  @apply p-2 mr-1 rounded-lg text-zinc-600 hover:text-amber-200 hover:bg-white/5 transition-all;
}


/* 🚀 按钮与标签样式 */

.timestamp-tag {
  @apply flex items-center gap-2 text-zinc-500 text-[10px] font-mono font-bold bg-black/40 px-3 py-1.5 rounded-lg border border-white/[0.02];
}

.mode-badge {
  @apply flex items-center gap-2 px-3 py-1.5 rounded-lg text-[9px] font-black uppercase border;
}
.mode-badge.shadow { @apply bg-blue-500/10 text-blue-400 border-blue-500/20 shadow-[0_0_15px_rgba(59,130,246,0.1)]; }
.mode-badge.sentry { @apply bg-amber-500/10 text-amber-500 border-amber-500/20 shadow-[0_0_15px_rgba(245,158,11,0.1)]; }

.history-card {
  @apply p-8 rounded-[2.5rem] border border-white/[0.03] bg-[#0c0b0a]/40 hover:bg-[#110f0e]/60 transition-all duration-700;
}

.destroy-btn {
  @apply flex items-center gap-2 text-[10px] font-black text-zinc-600 hover:text-red-400 transition-all uppercase tracking-widest py-2 px-4 rounded-xl border border-white/5 hover:bg-red-500/5;
}

/* 🚀 代码展示区：字号 13px */
.code-box {
  @apply p-6 rounded-[1.5rem] text-[13px] font-mono leading-relaxed break-all border transition-all duration-500 h-40 overflow-y-auto custom-scroll;
}
.code-box.original { @apply bg-black/40 text-zinc-500 border-white/[0.03]; }
.code-box.masked { @apply bg-white/[0.01] text-zinc-200 border-white/[0.03] shadow-inner; }

.label-text { @apply text-[10px] font-black uppercase tracking-[0.2em] text-zinc-600; }

.copy-action {
  @apply flex items-center gap-2 text-[9px] font-bold text-zinc-600 hover:text-amber-100 transition-all px-2.5 py-1.5 rounded-lg bg-white/[0.02] border border-white/[0.05];
}
.copy-action.copied { @apply text-emerald-400 bg-emerald-500/10 border-emerald-500/20; }
.copy-action.msk { @apply text-blue-500/60 hover:text-blue-400; }
.copy-action.msk.copied { @apply text-blue-400 bg-blue-500/10 border-blue-500/20; }

/* 雅致滚动条 */
.custom-scroll::-webkit-scrollbar { width: 3px; }
.custom-scroll::-webkit-scrollbar-thumb { @apply bg-white/5 rounded-full; }
.custom-scroll::-webkit-scrollbar-thumb:hover { @apply bg-amber-500/20; }
</style>