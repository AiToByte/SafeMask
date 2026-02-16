<script setup lang="ts">
import { ref, onMounted, computed, watch } from 'vue';
import { useAppStore } from '../stores/useAppStore';
import { MaskAPI } from '../services/api';
import { Plus, Layers, Trash2, ShieldCheck, Search, Edit3, X, Beaker, Check, AlertTriangle, Save, Loader2 } from 'lucide-vue-next';
import { confirm } from '@tauri-apps/plugin-dialog';

const store = useAppStore();
const isEditing = ref(false);
const isSubmitting = ref(false);
const searchQuery = ref("");
const form = ref({ name: '', pattern: '', mask: '<LABEL>', priority: 10, is_custom: true, enabled: true });
const testInput = ref("在这里输入测试文本...");
const testOutput = ref("");
const testError = ref("");

watch([() => form.value.pattern, () => form.value.mask, testInput], async () => {
  if (!form.value.pattern) { testOutput.value = ""; return; }
  try {
    testOutput.value = await MaskAPI.testRule(form.value.pattern, form.value.mask, testInput.value);
    testError.value = "";
  } catch (e: any) { testError.value = e.toString(); }
}, { immediate: true });

onMounted(() => store.fetchAllRules());

const handleSave = async () => {
  isSubmitting.value = true;
  try {
    await MaskAPI.saveRule({ ...form.value });
    await store.fetchAllRules();
    await store.fetchStats();
    isEditing.value = false;
    form.value = { name: '', pattern: '', mask: '<LABEL>', priority: 10, is_custom: true, enabled: true };
  } catch (e) { alert(e); } finally { isSubmitting.value = false; }
};

const sortedRules = computed(() => {
  let f = store.allRulesList;
  if (searchQuery.value) {
    const q = searchQuery.value.toLowerCase();
    f = f.filter(r => r.name.toLowerCase().includes(q) || r.pattern.toLowerCase().includes(q));
  }
  return [...f].sort((a, b) => (a.is_custom === b.is_custom ? b.priority - a.priority : a.is_custom ? -1 : 1));
});
</script>

<template>
  <div class="flex items-stretch gap-6 h-[calc(100vh-180px)] animate-in fade-in duration-500">
    <div class="flex-1 min-w-0 flex flex-col glass rounded-[2.5rem] border border-white/5 overflow-hidden">
      <div class="px-8 py-6 border-b border-white/5 flex items-center justify-between">
        <div class="flex items-center gap-3">
          <div class="p-2 bg-blue-500/10 rounded-xl"><Layers :size="20" class="text-blue-400" /></div>
          <div><h3 class="font-bold text-zinc-100">脱敏规则库</h3></div>
        </div>
        <div class="relative"><Search class="absolute left-3 top-1/2 -translate-y-1/2 text-zinc-500" :size="14" />
          <input v-model="searchQuery" placeholder="搜索规则..." class="bg-black/40 border border-white/5 rounded-xl py-2 pl-9 pr-4 text-xs outline-none focus:border-blue-500/30 w-64" />
        </div>
      </div>
      <div class="flex-1 overflow-y-auto p-6 space-y-3 custom-scroll">
        <div v-for="rule in sortedRules" :key="rule.name" @click="form = {...rule}; isEditing=true" class="flex items-center p-4 rounded-2xl bg-white/[0.02] border border-white/5 hover:bg-white/[0.05] transition-all cursor-pointer">
          <div class="flex-1 min-w-0"><div class="flex items-center gap-2 mb-1"><span class="text-sm font-bold text-zinc-200">{{ rule.name }}</span><span :class="rule.is_custom ? 'tag-custom' : 'tag-system'">{{ rule.is_custom ? 'Custom' : 'System' }}</span></div><code class="text-[10px] font-mono text-zinc-500 truncate block">{{ rule.pattern }}</code></div>
          <span class="text-[10px] font-mono font-bold text-emerald-400 bg-emerald-500/5 px-2 py-1 rounded-lg border border-emerald-500/10">{{ rule.mask }}</span>
        </div>
      </div>
    </div>

    <div class="w-[400px] flex flex-col gap-6">
      <div class="glass p-8 rounded-[2.5rem] border border-white/5 space-y-5">
        <h3 class="text-lg font-bold flex items-center gap-2"><component :is="isEditing ? Edit3 : Plus" :size="20" class="text-blue-400" /> {{ isEditing ? '编辑规则' : '新增规则' }}</h3>
        <div class="space-y-4">
          <div class="field"><label>规则名称</label><input v-model="form.name" :disabled="isEditing && !form.is_custom" /></div>
          <div class="field"><label>正则表达式</label><textarea v-model="form.pattern" class="h-20 font-mono text-xs" /></div>
          <div class="grid grid-cols-2 gap-4"><div class="field"><label>掩码</label><input v-model="form.mask" class="text-center text-blue-400 font-bold" /></div><div class="field"><label>优先级</label><input type="number" v-model.number="form.priority" class="text-center" /></div></div>
        </div>
        <button @click="handleSave" class="w-full py-4 bg-blue-600 rounded-2xl font-bold flex items-center justify-center gap-2 hover:bg-blue-500 transition-all"><Save :size="18" /> {{ isEditing ? '更新并应用' : '注入引擎' }}</button>
      </div>

      <div class="glass p-8 rounded-[2.5rem] border border-emerald-500/10 flex-1 flex flex-col">
        <div class="flex items-center gap-2 mb-4"><Beaker :size="18" class="text-emerald-400" /><h3 class="font-bold text-zinc-200">调试沙盒</h3></div>
        <div class="flex-1 flex flex-col gap-4">
          <div class="flex-1 relative"><p class="abs-label">测试原文</p><textarea v-model="testInput" class="sandbox input" /></div>
          <div class="flex-1 relative"><p class="abs-label">预览结果</p><div class="sandbox output" :class="{'err': testError}">{{ testError || testOutput }}<Check v-if="!testError && testOutput !== testInput" class="absolute right-4 bottom-4 text-emerald-500" :size="16" /></div></div>
        </div>
      </div>
    </div>
  </div>
</template>

<style scoped>
.tag-custom { @apply text-[9px] bg-indigo-500/20 text-indigo-300 px-1.5 py-0.5 rounded font-black uppercase tracking-tighter; }
.tag-system { @apply text-[9px] bg-zinc-800 text-zinc-500 px-1.5 py-0.5 rounded font-black uppercase tracking-tighter; }
.field { @apply flex flex-col gap-1.5; }
.field label { @apply text-[10px] font-black text-zinc-600 uppercase tracking-widest px-1; }
.field input, .field textarea { @apply bg-black/40 border border-white/5 rounded-xl p-3 text-sm text-zinc-200 outline-none focus:border-blue-500/30 transition-all; }
.abs-label { @apply text-[9px] font-black text-zinc-700 uppercase absolute -top-2 left-4 px-2 bg-[#09090b] z-10; }
.sandbox { @apply w-full h-full min-h-[80px] bg-black/40 border border-white/5 rounded-2xl p-4 text-xs font-mono outline-none break-all overflow-y-auto custom-scroll; }
.sandbox.output { @apply bg-emerald-500/[0.02] text-emerald-200/80; }
.sandbox.err { @apply border-red-500/20 text-red-400; }
</style>