<script setup lang="ts">
import { ref, onMounted, computed } from 'vue';
import { useAppStore } from '../stores/useAppStore';
import { MaskAPI } from '../services/api';
import { Plus, Info, Layers, Trash2, ShieldCheck, Search, Hash } from 'lucide-vue-next';
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
    } catch (e) {
      console.error("删除失败:", e);
    }
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
  <div class="flex items-stretch gap-8 h-full max-h-[680px] animate-in fade-in duration-500 font-sans">
    
    <!-- 左侧列表 -->
    <div class="flex-1 flex flex-col glass rounded-[2.5rem] border-white/5 overflow-hidden">
      <!-- 列表头部 -->
      <div class="px-8 py-6 border-b border-white/5 flex items-center justify-between bg-white/[0.01]">
        <div class="flex items-center gap-3">
          <div class="p-2 bg-blue-500/20 rounded-lg">
            <Layers :size="18" class="text-blue-400" />
          </div>
          <h3 class="font-bold text-zinc-200 tracking-tight">规则库列表</h3>
        </div>
        <div class="relative">
          <Search class="absolute left-3 top-1/2 -translate-y-1/2 text-zinc-600" :size="14" />
          <input v-model="searchQuery" placeholder="搜索规则名称..." 
                 class="bg-black/40 border border-zinc-800 rounded-xl py-2 pl-10 pr-4 text-xs outline-none focus:border-blue-500/50 w-48 transition-all focus:w-64 text-zinc-300" />
        </div>
      </div>
      
      <!-- 列表滚动区 -->
      <div class="flex-1 overflow-y-auto p-6 space-y-3 custom-scroll">
        <div v-for="rule in sortedRules" :key="rule.name" 
             class="group p-4 rounded-2xl border transition-all flex items-center bg-white/[0.02] border-white/5 hover:bg-white/[0.04] hover:border-white/10">
          
          <!-- 1. 核心信息区：使用 min-w-0 允许 flex 项收缩以触发布断 -->
          <div class="flex-1 min-w-0 flex flex-col gap-1.5">
            <div class="flex items-center gap-2">
              <span class="text-sm font-bold truncate text-zinc-200" :title="rule.name">
                {{ rule.name }}
              </span>
              <span v-if="rule.is_custom" class="shrink-0 text-[8px] bg-blue-600/20 text-blue-400 px-1.5 py-0.5 rounded border border-blue-500/30 font-black uppercase italic">
                Custom
              </span>
              <span v-else class="shrink-0 text-[8px] bg-zinc-800 text-zinc-500 px-1.5 py-0.5 rounded font-black uppercase">System</span>
            </div>
            
            <!-- 模式展示：增加背景代码块，强制单行截断 -->
            <div class="flex items-center gap-2">
               <code class="text-[10px] font-mono text-zinc-500 bg-black/30 px-2 py-0.5 rounded border border-white/5 truncate max-w-[90%]" :title="rule.pattern">
                 {{ rule.pattern }}
               </code>
            </div>
          </div>

          <!-- 2. 右侧对齐 Meta 区：固定宽度确保整齐 -->
          <div class="flex items-center gap-4 shrink-0 ml-4">
            <!-- 脱敏标签：固定最小宽度 -->
            <div class="hidden sm:flex items-center justify-end min-w-[100px]">
              <span class="text-[10px] font-mono font-bold text-blue-400/80 bg-blue-500/5 px-2.5 py-1 rounded-lg border border-blue-500/10">
                {{ rule.mask }}
              </span>
            </div>

            <!-- 操作按钮列 -->
            <div class="w-10 flex justify-center">
              <button v-if="rule.is_custom" 
                      @click.stop="handleDelete(rule.name)"
                      class="p-2 rounded-lg text-zinc-600 hover:text-red-400 hover:bg-red-500/10 transition-all opacity-0 group-hover:opacity-100"
                      title="删除自定义规则">
                <Trash2 :size="16" />
              </button>
              <div v-else class="text-zinc-800 opacity-40" title="系统内置规则不可删除">
                <ShieldCheck :size="16" />
              </div>
            </div>
          </div>
        </div>
      </div>
    </div>

    <!-- 右侧：配置面板 -->
    <div class="w-[380px] flex flex-col gap-6">
      <div class="glass p-8 rounded-[2.5rem] border-blue-500/20 flex-1 relative overflow-hidden">
        <!-- 装饰背景 -->
        <div class="absolute -right-10 -top-10 w-32 h-32 bg-blue-600/5 blur-3xl rounded-full"></div>
        
        <h3 class="text-xl font-bold mb-8 flex items-center gap-3 text-white">
           <div class="p-2 bg-blue-600 rounded-xl shadow-lg shadow-blue-500/20">
             <Plus :size="18" class="text-white" />
           </div>
           新增脱敏规则
        </h3>

        <div class="space-y-5 relative z-10">
          <div class="space-y-2">
            <label class="text-[10px] font-black text-zinc-500 uppercase tracking-widest flex items-center gap-2">
              <Hash :size="10" /> 规则名称
            </label>
            <input v-model="form.name" 
                   class="w-full bg-black/40 border border-white/10 p-3.5 rounded-xl text-sm focus:border-blue-500/50 outline-none transition-all text-zinc-200 placeholder:text-zinc-700" 
                   placeholder="例如：华为工号" />
          </div>

          <div class="space-y-2">
            <label class="text-[10px] font-black text-zinc-500 uppercase tracking-widest">匹配模式 (正则)</label>
            <textarea v-model="form.pattern" 
                      class="w-full bg-black/40 border border-white/10 p-3.5 rounded-xl text-xs font-mono focus:border-blue-500/50 outline-none transition-all h-28 text-zinc-300 resize-none" 
                      placeholder="\bHW-[0-9]{5}\b" />
          </div>

          <div class="grid grid-cols-2 gap-4">
            <div class="space-y-2">
              <label class="text-[10px] font-black text-zinc-500 uppercase tracking-widest">替换标签</label>
              <input v-model="form.mask" class="w-full bg-black/40 border border-white/10 p-3 rounded-xl text-xs font-mono text-blue-400 outline-none focus:border-blue-500/50" />
            </div>
            <div class="space-y-2">
              <label class="text-[10px] font-black text-zinc-500 uppercase tracking-widest">优先级</label>
              <input type="number" v-model="form.priority" class="w-full bg-black/40 border border-white/10 p-3 rounded-xl text-xs text-zinc-300 outline-none focus:border-blue-500/50" />
            </div>
          </div>

          <button @click="handleSave" :disabled="isSubmitting"
                  class="w-full py-4 bg-blue-600 hover:bg-blue-500 disabled:bg-zinc-800 disabled:text-zinc-500 text-white rounded-2xl font-bold transition-all mt-4 shadow-lg shadow-blue-600/10 flex items-center justify-center gap-2">
            <span v-if="!isSubmitting">编译并保存规则</span>
            <span v-else class="flex items-center gap-2">
              <div class="w-4 h-4 border-2 border-white/30 border-t-white rounded-full animate-spin"></div>
              引擎正在重新编译...
            </span>
          </button>
          
          <p v-if="message" :class="message.includes('❌') ? 'text-red-400' : 'text-emerald-400'" class="text-center text-[11px] font-bold animate-pulse">
            {{ message }}
          </p>
        </div>
      </div>
      
      <!-- 温馨提示 -->
      <div class="glass p-5 rounded-3xl bg-blue-500/[0.02] border-blue-500/10 flex gap-4">
        <Info class="text-blue-500/50 shrink-0" :size="18" />
        <p class="text-[10px] text-zinc-500 leading-relaxed italic">
          <strong>提示：</strong> 为了保证性能，请尽量使用 <code class="text-zinc-400">\b</code> 词边界。正则表达式错误将导致脱敏引擎自动回滚到上一个稳定版本。
        </p>
      </div>
    </div>
  </div>
</template>

<style scoped>
.custom-scroll::-webkit-scrollbar {
  width: 4px;
}
.custom-scroll::-webkit-scrollbar-thumb {
  background: rgba(255, 255, 255, 0.05);
  border-radius: 10px;
}
.custom-scroll::-webkit-scrollbar-track {
  background: transparent;
}
</style>