<script setup lang="ts">
import { ref, onMounted, computed, watch } from 'vue';
import { useAppStore } from '../stores/useAppStore';
import { MaskAPI } from '../services/api';
import { 
  Plus,Fingerprint, Layers, Trash2, ShieldCheck, Search, Edit3, X, 
  Beaker, Check, AlertTriangle, Save, CopyPlus, Lock, Info
} from 'lucide-vue-next';
import { confirm, message } from '@tauri-apps/plugin-dialog';

const store = useAppStore();

// --- 状态管理 ---
const isSubmitting = ref(false);
const searchQuery = ref("");
const selectedRuleName = ref(""); // 标记当前选中的原始规则名

// --- 表单模型 ---
const initialForm = { name: '', pattern: '', mask: '<LABEL>', priority: 10, is_custom: true, enabled: true };
const form = ref({ ...initialForm });

// --- 实时验证逻辑 ---
const nameDuplicateError = computed(() => {
  if (!form.value.name) return "";
  const exists = store.allRulesList.find(r => r.name === form.value.name);
  if (exists) {
    if (selectedRuleName.value === exists.name && exists.is_custom) return ""; 
    return `库中已存在名为 [${form.value.name}] 的${exists.is_custom ? '自定义' : '系统'}规则`;
  }
  return "";
});

const patternDuplicateError = computed(() => {
  if (!form.value.pattern) return "";
  const exists = store.allRulesList.find(r => r.pattern === form.value.pattern);
  if (exists) {
    if (selectedRuleName.value === exists.name) return ""; 
    return `该表达式与规则 [${exists.name}] 重复`;
  }
  return "";
});

// --- 调试沙盒状态 ---
const testInput = ref(""); 
const testOutput = ref("");
const testError = ref("");

// 监听变动实时测试
watch([() => form.value.pattern, () => form.value.mask, testInput], async () => {
  if (!form.value.pattern || !testInput.value) { 
    testOutput.value = ""; 
    testError.value = "";
    return; 
  }
  try {
    testOutput.value = await MaskAPI.testRule(form.value.pattern, form.value.mask, testInput.value);
    testError.value = "";
  } catch (e: any) {
    testError.value = e.toString();
  }
}, { immediate: true });

onMounted(() => store.fetchAllRules());

// --- 核心交互方法 ---

const selectRule = (rule: any) => {
  selectedRuleName.value = rule.name;
  form.value = { ...rule };
};

const clearForm = () => {
  if (!selectedRuleName.value && form.value.name === "") return;
  selectedRuleName.value = "";
  form.value = { ...initialForm };
};

const handleSave = async (asNew = false) => {
  if (!form.value.name || !form.value.pattern) {
    await message("请填写完整的规则名称和正则表达式", { title: "数据缺失", kind: "warning" });
    return;
  }

  if (asNew) {
    const isDuplicate = store.allRulesList.some(r => r.name === form.value.name || r.pattern === form.value.pattern);
    if (isDuplicate) {
      await message("检测到规则库中已存在相同名称或表达式的记录，请修改后再保存。", { title: "拒绝重复添加", kind: "error" });
      return;
    }
  }

  isSubmitting.value = true;
  try {
    const payload = { ...form.value };
    if (asNew) payload.is_custom = true;

    await MaskAPI.saveRule(payload);
    await store.fetchAllRules();
    await store.fetchStats();
    
    if (asNew) selectedRuleName.value = payload.name;
    await message(asNew ? "规则已作为新模式存入库中" : "规则修改已即时应用至脱敏引擎", { title: "注入成功", kind: "info" });
  } catch (e) {
    await message("引擎注入失败: " + e, { kind: "error" });
  } finally {
    isSubmitting.value = false;
  }
};

const handleDelete = async (name: string) => {
  const ok = await confirm(`此操作将永久删除自定义规则 [${name}]。确定继续吗？`, { title: '销毁确认', kind: 'warning' });
  if (ok) {
    await MaskAPI.deleteRule(name);
    await store.fetchAllRules();
    await store.fetchStats();
    if (selectedRuleName.value === name) clearForm();
  }
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
  <div class="flex items-stretch gap-6 h-full overflow-hidden animate-in fade-in duration-700" @click="clearForm">
    
    <!-- 1. 左侧：规则列表面板 -->
    <div class="flex-1 min-w-0 flex flex-col bg-[#0d0d0f]/60 border border-white/[0.04] rounded-[2rem] overflow-hidden" @click.stop>
      <!-- 🚀 优化后的搜索头部：解决挤压问题 -->
  <div class="px-8 py-6 border-b border-white/[0.04] flex items-center justify-between gap-6">
    
    <!-- 标题组：增加 flex-shrink-0 确保不被挤压 -->
    <div class="flex items-center gap-3 flex-shrink-0">
      <div class="w-8 h-8 rounded-lg bg-amber-500/5 border border-amber-500/10 flex items-center justify-center shadow-inner">
        <Layers :size="16" class="text-amber-500/60" />
      </div>
      <div class="flex flex-col">
        <h3 class="font-black text-amber-50/90 text-[13px] tracking-[0.1em] leading-tight">规则模式库</h3>
        <span class="text-[8px] text-zinc-600 font-bold uppercase tracking-widest mt-0.5">Pattern Repository</span>
      </div>
    </div>

    <!-- 搜索框容器：使用 flex-1 占据剩余空间，并设置最大宽度 -->
    <div class="relative flex-1 max-w-[240px] group" @click.stop>
      <!-- 搜索图标 -->
      <div class="absolute left-3.5 top-1/2 -translate-y-1/2 text-zinc-600 group-focus-within:text-amber-500/60 transition-colors z-10">
        <Search :size="14" />
      </div>
      
      <input 
        v-model="searchQuery" 
        placeholder="快速检索..." 
        class="search-bar-fluid" 
      />
      
      <!-- 装饰用的小圆点，增加精密感 -->
      <div class="absolute right-3 top-1/2 -translate-y-1/2 w-1 h-1 rounded-full bg-zinc-800 group-focus-within:bg-amber-500/40 transition-colors"></div>
    </div>
  </div>
      
      <div class="flex-1 overflow-y-auto p-5 space-y-2 custom-scroll" @click.self="clearForm">
        <div v-for="rule in sortedRules" :key="rule.name" 
             @click.stop="selectRule(rule)"
             class="rule-item group" 
             :class="{ 'active': selectedRuleName === rule.name }">
          
          <div class="flex-1 min-w-0 pr-4">
            <div class="flex items-center gap-2 mb-0.5">
              <span class="text-sm font-bold truncate text-zinc-200">{{ rule.name }}</span>
              <span :class="rule.is_custom ? 'tag-custom' : 'tag-system'">
                {{ rule.is_custom ? 'Custom' : 'System' }}
              </span>
            </div>
            <code class="pattern-text">{{ rule.pattern }}</code>
          </div>

          <div class="flex items-center gap-3 shrink-0">
            <span class="mask-label">{{ rule.mask }}</span>
            <div class="w-6 flex justify-center">
              <button v-if="rule.is_custom" @click.stop="handleDelete(rule.name)" 
                      class="delete-trigger">
                <Trash2 :size="14" />
              </button>
              <ShieldCheck v-else :size="14" class="text-zinc-800" />
            </div>
          </div>
        </div>
        <div class="flex-1 min-h-[120px]" @click.self="clearForm"></div>
      </div>
    </div>

    <!-- 2. 右侧：配置区域 -->
    <div class="w-[420px] flex flex-col gap-5 overflow-y-auto custom-scroll pr-1" @click.stop>
      
      <div class="glass-panel p-8 space-y-7 shadow-2xl">
        <div class="flex justify-between items-center">
          <div class="flex items-center gap-4">
            <div class="w-10 h-10 rounded-xl bg-white/[0.03] border border-white/10 flex items-center justify-center">
              <Edit3 v-if="selectedRuleName" :size="18" class="text-amber-400" />
              <Plus v-else :size="18" class="text-blue-500" />
            </div>
            <h3 class="font-bold text-amber-50/90 tracking-tight">{{ selectedRuleName ? '配置既有模式' : '创建新脱敏模式' }}</h3>
          </div>
          <button v-if="selectedRuleName" @click.stop="clearForm" class="text-zinc-600 hover:text-white transition-all" title="切换回新建模式">
            <Plus :size="18" />
          </button>
        </div>

        <div v-if="!form.is_custom && selectedRuleName" class="bg-amber-900/10 border border-amber-500/20 p-4 rounded-2xl flex gap-4 animate-in slide-in-from-top-2">
           <Lock :size="20" class="text-amber-500 shrink-0 mt-0.5" />
           <p class="text-[11px] text-amber-200/50 leading-relaxed font-medium">
             系统预设模式不可直接覆盖。请修改参数后使用下方“另存为”功能创建副本。
           </p>
        </div>

        <div class="space-y-6" :class="{ 'opacity-30 pointer-events-none filter grayscale': !form.is_custom && selectedRuleName }">
          <div class="input-group">
            <div class="label-header"><label>规则显示名称</label><span class="required-dot"></span></div>
            <div class="input-wrapper">
              <input v-model="form.name" placeholder="例如：用户隐私手机号" />
            </div>
            <p v-if="nameDuplicateError" class="validation-msg"><Info :size="10" /> {{ nameDuplicateError }}</p>
          </div>

          <div class="input-group">
            <div class="label-header"><label>正则表达式 (RUST REGEX)</label><span class="required-dot"></span></div>
            <div class="input-wrapper">
              <textarea v-model="form.pattern" class="h-28 font-mono text-[12px]" placeholder="输入匹配模式..." />
            </div>
            <p v-if="patternDuplicateError" class="validation-msg"><Info :size="10" /> {{ patternDuplicateError }}</p>
          </div>

          <div class="grid grid-cols-2 gap-5">
            <div class="input-group">
              <div class="label-header"><label>脱敏掩码标签</label></div>
              <div class="input-wrapper">
                <input v-model="form.mask" class="text-center text-blue-400 font-mono font-bold tracking-widest" />
              </div>
            </div>
            <div class="input-group">
              <div class="label-header"><label>注入权重 (优先级)</label></div>
              <div class="input-wrapper">
                <input type="number" v-model.number="form.priority" class="text-center font-mono" />
              </div>
            </div>
          </div>
        </div>

        <div class="flex flex-col gap-3 pt-4">
          <button v-if="form.is_custom && selectedRuleName" @click="handleSave(false)" 
                  class="btn-primary" :disabled="isSubmitting || !!nameDuplicateError">
            <Save :size="16" /> 保存修改
          </button>
          <button v-if="!selectedRuleName" @click="handleSave(false)" 
                  class="btn-primary" :disabled="isSubmitting || !!nameDuplicateError">
            <Plus :size="16" /> 注入脱敏引擎
          </button>
          <button v-if="selectedRuleName" @click="handleSave(true)" 
                  class="btn-secondary" :disabled="isSubmitting">
            <CopyPlus :size="16" /> 另存为自定义规则
          </button>
        </div>
      </div>

      <!-- 3. 调试沙盒实验室：增加逻辑引导提示 -->
<div class="glass-panel p-8 flex-1 border-emerald-500/10 shadow-[0_0_50px_rgba(16,185,129,0.02)] relative group/sandbox">
  <div class="flex items-center gap-3 mb-6">
    <div class="p-1.5 bg-emerald-500/10 rounded-lg">
      <Beaker :size="16" class="text-emerald-400" />
    </div>
    <h3 class="font-bold text-amber-50/80">调试沙盒实验室</h3>
    <!-- 状态指示灯 -->
    <div class="ml-auto flex items-center gap-2">
      <span class="text-[8px] font-black uppercase tracking-widest text-zinc-600">Sandbox Status:</span>
      <div class="w-1.5 h-1.5 rounded-full" :class="form.pattern ? 'bg-emerald-500 animate-pulse shadow-[0_0_8px_#10b981]' : 'bg-zinc-800'"></div>
    </div>
  </div>

  <div class="space-y-5">
    <!-- 测试输入区 -->
    <div class="relative">
      <span class="sandbox-label">测试输入 (Test Input)</span>
      <textarea 
        v-model="testInput" 
        @click.stop
        placeholder="在这里输入含有敏感信息的原始文本..." 
        class="sandbox-area input custom-scroll" 
      />
    </div>

    <!-- 实时预览区：增加引导逻辑 -->
    <div class="relative">
      <span class="sandbox-label" :class="testError ? 'text-red-500' : 'text-emerald-500'">
        {{ testError ? '正则编译错误 (Syntax Error)' : '实时脱敏仿真 (Simulation)' }}
      </span>
      
      <div class="sandbox-area output custom-scroll" :class="{'err': testError, 'empty-hint': !form.pattern}">
        <!-- 🚀 优化后的引导提示：增强对比度与清晰度 -->
        <!-- 🚀 优化后的引导提示：使用更具“识别/待机”语义的图标 -->
        <template v-if="!form.pattern">
          <div class="flex flex-col items-center justify-center h-full py-6 text-center space-y-4 animate-in fade-in duration-700">
            
            <!-- 图标：改用指纹符号，代表“特征感应中” -->
            <div class="relative w-12 h-12 flex items-center justify-center">
              <!-- 外圈扩散光晕 -->
              <div class="absolute inset-0 rounded-full bg-amber-500/5 animate-ping opacity-20"></div>
              <div class="relative w-12 h-12 rounded-full border border-amber-500/20 flex items-center justify-center bg-[#0d0d0f] shadow-inner">
                <Fingerprint :size="22" class="text-amber-500/40" />
              </div>
            </div>
            
            <div class="space-y-2">
              <!-- 标题：雅致的琥珀色 -->
              <p class="text-[12px] text-amber-200/70 font-bold tracking-[0.2em] uppercase">
                Engine Standby
              </p>
              <p class="text-[11px] text-zinc-500 font-medium leading-relaxed px-10">
                当前沙盒处于待机状态。请从左侧
                <span class="text-amber-500/60 border-b border-amber-500/20 mx-0.5">选定模式</span> 
                或 
                <span class="text-amber-500/60 border-b border-amber-500/20 mx-0.5">编写正则</span>
                以激活仿真逻辑。
              </p>
            </div>
          </div>
        </template>

        <!-- 正常输出或错误显示 -->
        <template v-else>
          <span v-if="testError" class="text-red-400 font-mono text-[10px] leading-tight">{{ testError }}</span>
          <span v-else class="text-emerald-100/70">{{ testOutput }}</span>
          <Check v-if="!testError && testOutput !== testInput && testInput" 
                 class="absolute right-3 bottom-3 text-emerald-500/40 animate-in zoom-in" :size="16" />
        </template>
      </div>
    </div>
  </div>
</div>

    </div>
  </div>
</template>

<style scoped>
/* --- 列表与通用项 --- */

/* 🚀 弹性化搜索框：具备明显的琥珀色边框和内凹感 */
.search-bar-fluid {
  @apply w-full bg-[#08080a] border border-white/[0.08] rounded-xl py-2.5 pl-10 pr-8 text-[11px] text-amber-50/80 outline-none transition-all duration-500 shadow-inner;
}

/* 默认状态就有较清晰的轮廓 */
.search-bar-fluid {
  border-color: rgba(245, 158, 11, 0.1);
}

.search-bar-fluid:hover {
  @apply border-white/20 bg-[#0a0a0c];
}

.search-bar-fluid:focus {
  @apply border-amber-500/40 bg-[#0c0c0e];
  /* 增加微弱的外发光和更深的内阴影 */
  box-shadow: 
    0 0 15px rgba(245, 158, 11, 0.03), 
    inset 0 2px 8px rgba(0, 0, 0, 0.6);
}

/* 调整标题文字的大小和间距，使其看起来更“稳” */
h3 {
  text-shadow: 0 1px 2px rgba(0, 0, 0, 0.5);
}

/* 占位符颜色调整 */
.search-bar-fluid::placeholder {
  @apply text-zinc-700 font-medium tracking-wide;
}
.rule-item { @apply flex items-center p-4 rounded-2xl bg-white/[0.01] border border-white/[0.03] transition-all cursor-pointer relative overflow-hidden; }
.rule-item:hover { @apply bg-white/[0.03] border-white/[0.08] translate-x-1; }
.rule-item.active { @apply border-amber-500/30 bg-amber-500/[0.04] shadow-[0_10px_30px_rgba(0,0,0,0.4)]; }
.rule-item.active::before { content: ''; @apply absolute left-0 top-3 bottom-3 w-[2px] bg-amber-500 rounded-full; }
.pattern-text { @apply text-[10px] font-mono text-zinc-600 truncate block whitespace-nowrap overflow-hidden; max-width: 280px; }
.tag-custom { @apply text-[8px] bg-blue-500/10 text-blue-400 border border-blue-500/20 px-1.5 py-0.5 rounded font-black uppercase; }
.tag-system { @apply text-[8px] bg-zinc-800 text-zinc-500 border border-white/5 px-1.5 py-0.5 rounded font-black uppercase; }
.mask-label { @apply text-[9px] font-mono font-bold text-emerald-400/70 bg-emerald-500/5 px-2.5 py-1 rounded-lg border border-emerald-500/10; }
.delete-trigger { @apply p-1.5 rounded-lg text-zinc-700 hover:text-red-400 hover:bg-red-500/10 transition-all opacity-0 group-hover:opacity-100; }

/* --- 🚀 输入表单优化核心 --- */
.input-group { @apply flex flex-col gap-2.5 relative; }
.label-header { @apply flex items-center justify-between px-1.5; }
.label-header label {
  /* 🚀 提升 Label 辨识度：微黄象牙白 */
@apply text-[11px] font-bold text-amber-100/80 uppercase tracking-[0.12em];
}
.required-dot { @apply w-1.5 h-1.5 rounded-full bg-amber-500/60 shadow-[0_0_8px_rgba(245,158,11,0.3)]; }

.input-wrapper {
  /* 🚀 修复点：使用方括号语法处理任意不透明度 */
  @apply relative rounded-xl bg-[#08080a] border border-white/[0.12] transition-all duration-300 shadow-inner;
}
.input-wrapper:hover { @apply border-white/[0.2]; }
.input-wrapper:focus-within {
  @apply border-amber-500/40 bg-[#0a0a0c];
  box-shadow: 0 0 20px rgba(245, 158, 11, 0.05), inset 0 2px 10px rgba(0, 0, 0, 0.6);
}
.input-wrapper input, .input-wrapper textarea {
  @apply w-full bg-transparent border-none outline-none p-3.5 text-[13px] text-amber-50/90 placeholder:text-zinc-800 transition-all font-medium;
}

.validation-msg { @apply text-[10px] text-amber-400 font-bold mt-1.5 flex items-center gap-1.5 px-2; }

/* --- 按钮与面板 --- */
.btn-primary { @apply w-full py-4 bg-amber-500/10 border border-amber-500/30 text-amber-500 rounded-2xl font-black uppercase tracking-widest text-[11px] flex items-center justify-center gap-3 hover:bg-amber-500 hover:text-black transition-all active:scale-[0.97]; }
.btn-secondary { @apply w-full py-3.5 bg-zinc-900 border border-white/5 text-zinc-500 rounded-2xl text-[10px] font-black uppercase tracking-widest flex items-center justify-center gap-3 hover:text-amber-200 hover:border-amber-500/20 transition-all; }

.glass-panel { @apply bg-[#0d0d0f]/80 border border-white/[0.04] rounded-[2.5rem]; }
.sandbox-label { @apply text-[9px] font-bold uppercase tracking-widest absolute -top-2.5 left-5 px-2 bg-[#0c0b0a] z-10 text-amber-100/60; }
/* 统一提升沙盒文字清晰度 */
.sandbox-area {
  @apply w-full bg-black/40 border border-white/[0.08] rounded-2xl p-5 text-[13px] font-mono leading-relaxed outline-none transition-all resize-none;
  /* 解决深色背景下文字发虚 */
  -webkit-font-smoothing: subpixel-antialiased;
}

.sandbox-area.input:focus { @apply border-amber-500/30; }
.sandbox-area.output { @apply min-h-[100px] bg-emerald-500/[0.01] border-emerald-500/10; }
/* 🚀 强化输出区域的容器感 */
.sandbox-area.output.empty-hint {
  /* 使用稍明显的虚线边框 */
  @apply border-dashed border-white/[0.06] bg-black/20;
  background-image: radial-gradient(circle at center, rgba(245,158,11,0.02) 0%, transparent 70%);
}
/* 强调文字阴影，使文字更锐利 */
.drop-shadow-sm {
  text-shadow: 0 1px 2px rgba(0, 0, 0, 0.8);
}

.custom-scroll::-webkit-scrollbar { width: 2px; }
.custom-scroll::-webkit-scrollbar-thumb { @apply bg-amber-500/10 rounded-full; }
</style>