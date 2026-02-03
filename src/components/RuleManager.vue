<script setup lang="ts">
import { ref, onMounted, computed } from 'vue';
import { useAppStore } from '../stores/useAppStore';
import { MaskAPI } from '../services/api';
import { Plus, Info, Layers, Trash2, ShieldCheck, Search, Hash, AlignLeft, Fingerprint, ArrowUpDown, Save, Loader2 } from 'lucide-vue-next';
import { confirm } from '@tauri-apps/plugin-dialog';

const store = useAppStore();
const form = ref({ name: '', pattern: '', mask: '<LABEL>', priority: 10, is_custom: true });
const isSubmitting = ref(false);
const message = ref("");
const searchQuery = ref("");

onMounted(() => store.fetchAllRules());

const handleSave = async () => {
  if (!form.value.name || !form.value.pattern) return;
  isSubmitting.value = true;
  try {
    await MaskAPI.saveRule({ ...form.value });
    message.value = "✅ 规则保存成功";
    await store.fetchAllRules();
    await store.fetchStats();
    form.value = { name: '', pattern: '', mask: '<LABEL>', priority: 10, is_custom: true };
  } catch (e) {
    message.value = "❌ 保存失败: " + e;
  } finally {
    isSubmitting.value = false;
    setTimeout(() => message.value = "", 3000);
  }
};

const handleDelete = async (name: string) => {
  const confirmation = await confirm(
    `确定要永久删除自定义规则 [${name}] 吗？`, 
    { title: '删除确认', kind: 'warning', okLabel: '确定删除', cancelLabel: '取消' }
  );
  if (confirmation) {
    try {
      await MaskAPI.deleteRule(name);
      await store.fetchAllRules();
      await store.fetchStats();
    } catch (e) { console.error("删除失败:", e); }
  }
};

const sortedRules = computed(() => {
  let filtered = store.allRulesList;
  if (searchQuery.value) {
    filtered = filtered.filter(r => r.name.toLowerCase().includes(searchQuery.value.toLowerCase()));
  }
  return [...filtered].sort((a, b) => {
    if (a.is_custom !== b.is_custom) return a.is_custom ? -1 : 1;
    return b.priority - a.priority;
  });
});
</script>

<template>
  <div class="flex items-stretch gap-6 h-full max-h-[680px] animate-in fade-in duration-500 font-sans p-1">
    
    <!-- 左侧列表 -->
    <div class="flex-[1.5] min-w-0 flex flex-col glass rounded-[2rem] border border-white/5 overflow-hidden shadow-2xl shadow-black/20">
      <!-- 搜索头部 -->
      <div class="px-6 py-5 border-b border-white/5 flex items-center justify-between bg-white/[0.01]">
        <div class="flex items-center gap-3">
          <div class="p-2 bg-blue-500/10 rounded-xl border border-blue-500/10">
            <Layers :size="18" class="text-blue-400" />
          </div>
          <div>
            <h3 class="font-bold text-zinc-200 tracking-tight text-base">规则列表</h3>
            <p class="text-[10px] text-zinc-500 font-medium">Manage Masking Rules</p>
          </div>
        </div>
        <div class="relative group">
          <Search class="absolute left-3 top-1/2 -translate-y-1/2 text-zinc-500 group-focus-within:text-blue-400 transition-colors" :size="14" />
          <input v-model="searchQuery" placeholder="搜索..." 
                 class="bg-black/20 border border-white/5 rounded-xl py-2 pl-9 pr-4 text-xs outline-none focus:border-blue-500/30 focus:bg-black/40 w-40 transition-all focus:w-56 text-zinc-300 placeholder:text-zinc-600" />
        </div>
      </div>
      
      <!-- 列表滚动区 -->
      <div class="flex-1 overflow-y-auto p-4 space-y-2.5 custom-scroll">
        <div v-for="rule in sortedRules" :key="rule.name" 
             class="group relative p-4 rounded-xl border transition-all duration-300 flex items-center bg-white/[0.02] border-white/5 hover:bg-white/[0.06] hover:border-blue-500/20 hover:shadow-lg hover:shadow-black/20 overflow-hidden cursor-default">
          
          <!-- 选中高亮条 -->
          <div class="absolute left-0 top-0 bottom-0 w-0.5 bg-blue-500 opacity-0 group-hover:opacity-100 transition-opacity"></div>

          <!-- 信息主体 -->
          <div class="flex-1 min-w-0 pr-4">
            <div class="flex items-center gap-2 mb-1.5">
              <span class="text-sm font-bold truncate text-zinc-200 group-hover:text-white transition-colors">
                {{ rule.name }}
              </span>
              <span v-if="rule.is_custom" class="shrink-0 text-[9px] bg-indigo-500/20 text-indigo-300 px-1.5 py-0.5 rounded border border-indigo-500/20 font-bold uppercase tracking-wider">
                Custom
              </span>
              <span v-else class="shrink-0 text-[9px] bg-zinc-800 text-zinc-500 px-1.5 py-0.5 rounded border border-zinc-700/50 font-bold uppercase tracking-wider">
                System
              </span>
            </div>
            
            <div class="flex items-center gap-2">
               <code class="text-[10px] font-mono text-zinc-500 bg-black/30 px-2 py-0.5 rounded border border-white/5 truncate max-w-full block" :title="rule.pattern">
                 {{ rule.pattern }}
               </code>
            </div>
          </div>

          <!-- 右侧状态与操作 -->
          <div class="flex items-center gap-3 shrink-0 ml-auto pl-2">
            <span class="text-[10px] font-mono font-medium text-emerald-400 bg-emerald-500/5 px-2 py-1 rounded border border-emerald-500/10 whitespace-nowrap">
              {{ rule.mask }}
            </span>

            <div class="w-8 flex justify-center">
              <button v-if="rule.is_custom" 
                      @click.stop="handleDelete(rule.name)"
                      class="p-1.5 rounded-lg text-zinc-500 hover:text-red-400 hover:bg-red-500/10 transition-all opacity-0 group-hover:opacity-100"
                      title="删除规则">
                <Trash2 :size="15" />
              </button>
              <div v-else class="text-zinc-700" title="系统内置规则不可删除">
                <ShieldCheck :size="15" />
              </div>
            </div>
          </div>
        </div>
      </div>
    </div>

    <!-- 右侧面板：宽度固定，改为Flex布局解决溢出 -->
    <div class="w-[380px] shrink-0 flex flex-col glass rounded-[2rem] border border-blue-500/10 overflow-hidden relative shadow-2xl shadow-blue-900/5">
      <!-- 动态背景 -->
      <div class="absolute -right-20 -top-20 w-64 h-64 bg-blue-600/5 blur-[100px] rounded-full pointer-events-none"></div>
      <div class="absolute -left-20 bottom-0 w-64 h-64 bg-indigo-600/5 blur-[100px] rounded-full pointer-events-none"></div>

      <!-- 1. 头部：固定 -->
      <div class="shrink-0 px-8 pt-8 pb-4">
        <h3 class="text-lg font-bold flex items-center gap-3 text-white">
          <div class="p-2 bg-gradient-to-br from-blue-600 to-indigo-600 rounded-xl shadow-lg shadow-blue-500/20 text-white">
            <Plus :size="18" stroke-width="3" />
          </div>
          <span>新增规则</span>
        </h3>
        <p class="text-zinc-500 text-xs mt-2 pl-1">配置新的敏感数据脱敏策略。</p>
      </div>

      <!-- 2. 表单主体：可滚动 (flex-1 overflow-y-auto) -->
      <div class="flex-1 overflow-y-auto px-8 py-2 custom-scroll space-y-5">
        
        <!-- Name Input -->
        <div class="space-y-1.5 group">
          <label class="text-[10px] font-bold text-zinc-500 uppercase tracking-widest px-1">规则名称</label>
          <div class="relative">
            <div class="absolute left-3.5 top-1/2 -translate-y-1/2 text-zinc-600 group-focus-within:text-blue-500 transition-colors">
              <Hash :size="14" />
            </div>
            <input v-model="form.name" 
                   class="w-full bg-black/20 border border-white/10 hover:border-white/20 focus:border-blue-500/50 p-3 pl-10 rounded-xl text-sm outline-none transition-all text-zinc-200 placeholder:text-zinc-700" 
                   placeholder="例如：身份证号" />
          </div>
        </div>

        <!-- Pattern Input -->
        <div class="space-y-1.5 group">
          <label class="text-[10px] font-bold text-zinc-500 uppercase tracking-widest px-1">正则表达式 (RegEx)</label>
          <div class="relative">
             <div class="absolute left-3.5 top-4 text-zinc-600 group-focus-within:text-blue-500 transition-colors">
              <AlignLeft :size="14" />
            </div>
            <textarea v-model="form.pattern" 
                      class="w-full bg-black/20 border border-white/10 hover:border-white/20 focus:border-blue-500/50 p-3 pl-10 rounded-xl text-xs font-mono outline-none transition-all h-28 text-zinc-300 resize-none leading-relaxed" 
                      placeholder="输入匹配模式..." />
          </div>
        </div>

        <!-- Grid Inputs -->
        <div class="grid grid-cols-2 gap-4">
          <div class="space-y-1.5 group">
            <label class="text-[10px] font-bold text-zinc-500 uppercase tracking-widest px-1">替换内容</label>
            <div class="relative">
              <div class="absolute left-3 top-1/2 -translate-y-1/2 text-zinc-600 group-focus-within:text-blue-500 transition-colors">
                <Fingerprint :size="14" />
              </div>
              <input v-model="form.mask" class="w-full bg-black/20 border border-white/10 focus:border-blue-500/50 p-3 pl-9 rounded-xl text-xs font-mono text-blue-400 outline-none transition-all text-center" />
            </div>
          </div>
          
          <div class="space-y-1.5 group">
            <label class="text-[10px] font-bold text-zinc-500 uppercase tracking-widest px-1">优先级</label>
            <div class="relative">
              <div class="absolute left-3 top-1/2 -translate-y-1/2 text-zinc-600 group-focus-within:text-blue-500 transition-colors">
                <ArrowUpDown :size="14" />
              </div>
              <input type="number" v-model="form.priority" class="w-full bg-black/20 border border-white/10 focus:border-blue-500/50 p-3 pl-8 rounded-xl text-xs text-zinc-300 outline-none transition-all text-center" />
            </div>
          </div>
        </div>
        
        <!-- 底部留白，防止滚动到底部太贴边 -->
        <div class="h-2"></div>
      </div>

      <!-- 3. 底部按钮：固定 -->
      <div class="shrink-0 p-6 pt-4 border-t border-white/5 bg-black/20 backdrop-blur-sm z-10">
        <button @click="handleSave" :disabled="isSubmitting"
                class="w-full py-3.5 bg-gradient-to-r from-blue-600 to-indigo-600 hover:from-blue-500 hover:to-indigo-500 hover:shadow-lg hover:shadow-blue-500/25 active:scale-[0.98] disabled:opacity-50 disabled:cursor-not-allowed text-white rounded-xl font-bold transition-all flex items-center justify-center gap-2 group">
          <span v-if="!isSubmitting" class="flex items-center gap-2">
            <Save :size="16" class="group-hover:-translate-y-0.5 transition-transform" /> 
            编译并注入
          </span>
          <Loader2 v-else class="animate-spin" :size="18" />
        </button>

        <div class="h-6 flex items-center justify-center mt-2">
          <p v-if="message" 
             :class="message.includes('❌') ? 'text-red-400' : 'text-emerald-400'" 
             class="text-[11px] font-medium flex items-center gap-1.5 animate-in fade-in slide-in-from-bottom-2">
            {{ message }}
          </p>
        </div>
      </div>

    </div>
  </div>
</template>

<style scoped>
/* 玻璃拟态基础样式 - 如果你有全局类可以删除这个 */
.glass {
  background: rgba(22, 22, 24, 0.6);
  backdrop-filter: blur(20px);
  -webkit-backdrop-filter: blur(20px);
}

/* 自定义滚动条 */
.custom-scroll::-webkit-scrollbar {
  width: 4px;
}
.custom-scroll::-webkit-scrollbar-thumb {
  background: rgba(255, 255, 255, 0.1);
  border-radius: 10px;
}
.custom-scroll::-webkit-scrollbar-thumb:hover {
  background: rgba(255, 255, 255, 0.2);
}
.custom-scroll::-webkit-scrollbar-track {
  background: transparent;
}

/* 隐藏数字输入框箭头 */
input[type="number"]::-webkit-inner-spin-button,
input[type="number"]::-webkit-outer-spin-button {
  -webkit-appearance: none;
  margin: 0;
}
</style>