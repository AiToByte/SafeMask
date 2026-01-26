<script setup lang="ts">
import { ref, onMounted, computed } from 'vue';
import { useAppStore } from '../stores/useAppStore';
import { MaskAPI } from '../services/api';
// 🚀 修复：补全所有用到的图标导入
import { Plus, Info, Layers, Trash2, ShieldCheck, UserCog, Search } from 'lucide-vue-next';
import { confirm } from '@tauri-apps/plugin-dialog'; // 🚀 引入 Tauri 原生确认框

const store = useAppStore();
const form = ref({ name: '', pattern: '', mask: '<LABEL>', priority: 10, is_custom: true });
const isSubmitting = ref(false);
const message = ref("");
const searchQuery = ref(""); // 🚀 独立搜索变量

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

/**
 * 优雅的删除逻辑处理
 * @param name 规则名称
 */
const handleDelete = async (name: string) => {
  // 使用 Tauri 原生对话框替代浏览器 window.confirm
  // 这将提供更原生的 UI 体验（支持自定义标题和图标）
  const confirmation = await confirm(
    `您确定要永久删除自定义规则 [${name}] 吗？\n此操作不可撤销。`, 
    { 
        title: 'SafeMask 规则管理', 
        kind: 'warning',
        okLabel: '确定删除',
        cancelLabel: '取消'
    }
  );

  if (confirmation) {
    try {
      console.log(`正在请求删除规则: ${name}`);
      await MaskAPI.deleteRule(name);
      
      // 删除成功后，刷新 UI 数据
      await store.fetchAllRules();
      await store.fetchStats();
      
      // 这里的逻辑已经闭环：
      // 1. Rust 删除了 custom/user_rules.yaml 中的对应条目
      // 2. 前端重新获取了最新的规则列表
      // 3. 仪表盘统计数字同步更新
    } catch (e) {
      console.error("删除失败:", e);
    }
  }
};

// 🚀 排序逻辑：自定义置顶 + 优先级降序
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
    <div class="flex items-stretch gap-8 h-full max-h-[640px] animate-in fade-in duration-500 font-sans">
    
    <!-- 左侧列表 -->
    <div class="flex-1 flex flex-col glass rounded-[3rem] border-white/5 overflow-hidden">
      <div class="p-8 border-b border-white/5 flex items-center justify-between bg-white/[0.01]">
        <div class="flex items-center gap-3">
          <Layers :size="20" class="text-blue-500" />
          <h3 class="font-bold text-zinc-200">规则引擎库</h3>
        </div>
        <div class="relative">
          <Search class="absolute left-3 top-1/2 -translate-y-1/2 text-zinc-600" :size="12" />
          <input v-model="searchQuery" placeholder="搜索规则..." 
                 class="bg-zinc-900/50 border border-zinc-800 rounded-full py-1.5 pl-9 pr-4 text-xs outline-none focus:border-blue-500/50 w-40 transition-all focus:w-56" />
        </div>
      </div>
      
      <div class="flex-1 overflow-y-auto p-8 space-y-4 custom-scroll">
        <!-- 🚀 核心修复：添加 group 类 -->
        <div v-for="rule in sortedRules" :key="rule.name" 
             class="group p-5 rounded-[2rem] border transition-all flex justify-between items-center"
             :class="rule.is_custom ? 'bg-blue-600/[0.03] border-blue-500/20 shadow-lg shadow-blue-500/5' : 'bg-white/[0.02] border-white/5'">
          
          <div class="min-w-0 flex-1 mr-6">
            <div class="flex items-center gap-2 mb-2">
              <span class="text-sm font-bold tracking-tight" :class="rule.is_custom ? 'text-blue-400' : 'text-zinc-300'">
                {{ rule.name }}
              </span>
              <span v-if="rule.is_custom" class="text-[9px] bg-blue-600 text-white px-2 py-0.5 rounded-full font-black uppercase italic">
                Custom
              </span>
              <span v-else class="text-[9px] bg-zinc-800 text-zinc-500 px-2 py-0.5 rounded-full font-black uppercase">System</span>
            </div>
            <p class="text-[11px] font-mono text-zinc-500 truncate opacity-80">{{ rule.pattern }}</p>
          </div>

          <div class="flex items-center gap-6 shrink-0">
            <code class="text-[10px] font-mono font-bold text-zinc-400 bg-zinc-900/80 px-3 py-1.5 rounded-xl border border-white/5">
              {{ rule.mask }}
            </code>
            
            <!-- 🚀 修复后的操作按钮 -->
            <div class="w-8 flex justify-center">
              <button v-if="rule.is_custom" 
                      @click.stop="handleDelete(rule.name)"
                      class="p-2.5 rounded-xl bg-red-500/10 text-red-500/40 hover:text-red-500 hover:bg-red-500/20 transition-all opacity-0 group-hover:opacity-100 transform scale-90 group-hover:scale-100"
                      title="删除规则">
                <Trash2 :size="16" />
              </button>
              <div v-else class="text-zinc-800" title="系统规则锁定">
                <ShieldCheck :size="16" />
              </div>
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