<script setup lang="ts">
import { ref, onMounted, computed } from 'vue';
import { useAppStore } from '../stores/useAppStore';
import { MaskAPI } from '../services/api';
// 🚀 修复：补全所有用到的图标导入
import { Plus, Info, Layers, Trash2, ShieldCheck, UserCog, Search } from 'lucide-vue-next';

const store = useAppStore();
const form = ref({ name: '', pattern: '', mask: '<LABEL>', priority: 0, is_custom: true });
const isSubmitting = ref(false);
const message = ref("");

onMounted(() => store.fetchAllRules());

const handleSave = async () => {
  if (!form.value.name || !form.value.pattern) return;
  isSubmitting.value = true;
  try {
    await MaskAPI.saveRule({ ...form.value });
    message.value = "✅ 规则保存成功，已生效！";
    await store.fetchAllRules();
    await store.fetchStats(); // 同步更新仪表盘数字
    form.value = { name: '', pattern: '', mask: '<LABEL>', priority: 10, is_custom: true };
  } catch (e) {
    message.value = "❌ 保存失败: " + e;
  } finally {
    isSubmitting.value = false;
    setTimeout(() => message.value = "", 3000);
  }
};

const handleDelete = async (name: string) => {
  if (!confirm(`确定要删除自定义规则 [${name}] 吗？`)) return;
  try {
    await MaskAPI.deleteRule(name);
    await store.fetchAllRules();
    await store.fetchStats();
  } catch (e) {
    alert("删除失败: " + e);
  }
};

// 🚀 排序逻辑：自定义置顶 + 优先级降序
const sortedRules = computed(() => {
  let filtered = store.allRulesList;
  if (message.value) {
    filtered = filtered.filter(r => r.name.toLowerCase().includes(message.value.toLowerCase()));
  }
  return [...filtered].sort((a, b) => {
    if (a.is_custom !== b.is_custom) return a.is_custom ? -1 : 1;
    return b.priority - a.priority;
  });
});


</script>


<template>
  <div class="flex items-stretch gap-8 h-full max-h-[640px] animate-in fade-in duration-500">
    <!-- 左侧列表 -->
    <div class="flex-1 flex flex-col glass rounded-[2.5rem] border-white/5 overflow-hidden font-sans">
      <div class="p-6 border-b border-white/5 flex items-center justify-between">
        <h3 class="font-bold flex items-center gap-2 text-zinc-300">已加载规则引擎</h3>
      </div>
      
      <div class="flex-1 overflow-y-auto p-6 space-y-3 custom-scroll">
        <div v-for="rule in sortedRules" :key="rule.name" 
             class="p-4 rounded-2xl border transition-all flex justify-between items-center"
             :class="rule.is_custom ? 'bg-blue-500/5 border-blue-500/20' : 'bg-white/[0.02] border-white/5'">
          <div class="min-w-0 flex-1 mr-4">
            <div class="flex items-center gap-2 mb-1">
              <span class="text-sm font-bold" :class="rule.is_custom ? 'text-blue-400' : 'text-zinc-300'">{{ rule.name }}</span>
              <!-- 🚀 身份标识 -->
              <span v-if="rule.is_custom" class="flex items-center gap-0.5 text-[8px] bg-blue-500 text-white px-1.5 py-0.5 rounded-full font-black">
                <UserCog :size="8"/> 自定义
              </span>
              <span v-else class="text-[8px] bg-zinc-800 text-zinc-500 px-1.5 py-0.5 rounded-full font-black">SYSTEM</span>
            </div>
            <p class="text-[10px] font-mono text-zinc-500 truncate">{{ rule.pattern }}</p>
          </div>

          <div class="flex items-center gap-4">
            <div class="text-right shrink-0">
               <code class="text-[10px] font-mono text-zinc-400">{{ rule.mask }}</code>
            </div>
            <!-- 🚀 修复后的按钮显示逻辑 -->
            <button v-if="rule.is_custom" 
                    @click.stop="handleDelete(rule.name)"
                    class="p-2 rounded-xl hover:bg-red-500/10 text-zinc-600 hover:text-red-500 transition-all opacity-0 group-hover:opacity-100"
                    title="删除自定义规则">
              <Trash2 :size="14" />
            </button>
            <div v-else class="p-2 text-zinc-800" title="系统规则锁定">
              <ShieldCheck :size="14" />
            </div>
          </div>
        </div>
      </div>
    </div>

    <!-- 右侧：配置面板 -->
    <div class="w-[400px] flex flex-col gap-6 font-sans">
      <div class="glass p-8 rounded-[2.5rem] border-blue-500/20 flex-1 ">
        <h3 class="text-xl font-bold mb-8 flex items-center gap-2">
           <Plus class="text-blue-500" /> 配置新规则
        </h3>
        <div class="space-y-5">
          <div class="space-y-2">
            <label class="text-[10px] font-black text-zinc-500 uppercase tracking-tighter">规则唯一名称</label>
            <input v-model="form.name" class="w-full bg-black/40 border border-white/5 p-4 rounded-2xl text-sm focus:border-blue-500/50 transition-all" placeholder="规则名称" />
          </div>
          <div class="space-y-2">
            <label class="text-[10px] font-black text-zinc-500 uppercase tracking-tighter">匹配模式 (正则或关键字)</label>
            <textarea v-model="form.pattern" class="w-full bg-black/40 border border-white/5 p-4 rounded-2xl text-xs font-mono focus:border-blue-500/50 transition-all h-24" placeholder="关键字或正则表达式" />
          </div>
          <div class="grid grid-cols-2 gap-4">
            <div class="space-y-2">
              <label class="text-[10px] font-black text-zinc-500 uppercase tracking-tighter">脱敏标签</label>
              <input v-model="form.mask" class="w-full bg-black/40 border border-white/5 p-4 rounded-2xl text-xs font-mono" />
            </div>
            <div class="space-y-2">
              <label class="text-[10px] font-black text-zinc-500 uppercase tracking-tighter">优先级 (数字越大越先匹配)</label>
              <input type="number" v-model="form.priority" class="w-full bg-black/40 border border-white/5 p-4 rounded-2xl text-xs" />
            </div>
          </div>
          <button @click="handleSave" :disabled="isSubmitting"
                  class="w-full py-4 bg-blue-600 hover:bg-blue-500 rounded-2xl font-bold transition-all mt-4 disabled:opacity-50">
            {{ isSubmitting ? '正在编译引擎...' : '保存并应用' }}
          </button>
        </div>
      </div>
      
      <!-- 底部提示信息保持紧凑 -->
      <div class="glass p-5 rounded-3xl bg-amber-500/5 border-amber-500/10 flex gap-3">
        <Info class="text-amber-600 shrink-0" :size="16" />
        <p class="text-[10px] text-amber-200/50 leading-relaxed italic">
          注：自定义规则默认优先级较高。若正则语法错误将导致引擎加载失败。
        </p>
      </div>
    </div>
  </div>
</template>