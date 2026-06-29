<script setup lang="ts">
import { ref, computed, reactive, onMounted, onUnmounted, watch } from 'vue';
import { useAppStore } from '../stores/useAppStore';
import { MaskAPI } from '../services/api';
import {
  Shield, Keyboard, Bell, Timer, RotateCcw,
  Save, Trash2, Monitor, Cpu, Volume2, Eye, AlertTriangle,
  User, Mail, Github, Globe, Info, ExternalLink, Copyright,
  Copy, Check, Brain, Zap, Power, PowerOff, Loader2
} from 'lucide-vue-next';
import { message } from '@tauri-apps/plugin-dialog';

const store = useAppStore();
const isRecording = ref(false);
const showKeyWarning = ref(false);
const emailCopied = ref(false);

// AI 模型选择状态
const selectedModels = reactive(new Set<string>());
const aiEnabled = computed(() => store.aiEngineStatus?.state === 'ready' || store.aiEngineStatus?.state === 'loading');

// 加载计时
const loadingStartTime = ref<number | null>(null);
const loadingElapsed = ref(0);
let loadingTimer: number | null = null;

// 监听 loading 状态变化，启动/停止计时
const startLoadingTimer = () => {
  loadingStartTime.value = Date.now();
  loadingElapsed.value = 0;
  loadingTimer = window.setInterval(() => {
    if (loadingStartTime.value) {
      loadingElapsed.value = Math.floor((Date.now() - loadingStartTime.value) / 1000);
    }
  }, 1000);
};

const stopLoadingTimer = () => {
  if (loadingTimer) {
    clearInterval(loadingTimer);
    loadingTimer = null;
  }
  loadingStartTime.value = null;
};

const loadingElapsedText = computed(() => {
  const mins = Math.floor(loadingElapsed.value / 60);
  const secs = loadingElapsed.value % 60;
  if (mins > 0) {
    return `已用时 ${mins} 分 ${secs} 秒`;
  }
  return `已用时 ${secs} 秒`;
});

// 🚀 开发者基本信息
const developerInfo = {
  name: "XiaoSheng",
  email: "xiaosheng.tech@outlook.com",
  github: "https://github.com/AiToByte/SafeMask",
  website: "https://safemask.hub", 
  version: "v1.2.4"
};



const openLink = async (url: string) => {
  const { openUrl } = await import('@tauri-apps/plugin-opener');
  await openUrl(url);
};

const formatRecognizer = (name: string) => {
  const map: Record<string, string> = {
    'aho_corasick_engine': '字典匹配',
    'regex_engine': '正则匹配',
    'ner_engine': 'AI 识别',
    'context_enhancer': '上下文增强',
    'checksum_recognizer': '校验位验证',
  };
  return map[name] || name;
};

const getRecognizerColor = (name: string) => {
  const map: Record<string, string> = {
    'aho_corasick_engine': 'bg-emerald-500',
    'regex_engine': 'bg-blue-500',
    'ner_engine': 'bg-purple-500',
    'context_enhancer': 'bg-amber-500',
    'checksum_recognizer': 'bg-cyan-500',
  };
  return map[name] || 'bg-zinc-500';
};

const formatEntityType = (type: string) => {
  const map: Record<string, string> = {
    'person': '人名',
    'email': '邮箱',
    'phone': '电话',
    'address': '地址',
    'account_number': '账号',
    'date': '日期',
    'url': '链接',
    'secret': '密钥',
  };
  return map[type] || type;
};

const aiStatusColor = computed(() => {
  switch (store.aiEngineStatus?.state) {
    case 'ready': return 'bg-emerald-500 shadow-[0_0_8px_rgba(16,185,129,0.5)]';
    case 'loading': return 'bg-amber-500 animate-pulse shadow-[0_0_8px_rgba(245,158,11,0.5)]';
    case 'error': return 'bg-red-500 shadow-[0_0_8px_rgba(239,68,68,0.5)]';
    default: return 'bg-zinc-600';
  }
});

const aiStatusText = computed(() => {
  switch (store.aiEngineStatus?.state) {
    case 'ready': return '模型已就绪，AI 识别可用';
    case 'loading': return '模型加载中，首次加载约需 1-3 分钟';
    case 'error': return `加载失败: ${store.aiEngineStatus.error || '未知错误'}`;
    case 'not_loaded': return '模型未加载，复制文本时将自动触发';
    case 'not_available': return 'AI 引擎不可用';
    default: return '未知状态';
  }
});

// 可用模型列表
const availableModels = computed(() => {
  const models = [];
  if (store.aiEngineStatus?.available_count > 0) {
    // 从 aiEngineStatus 获取模型信息
    if (store.aiEngineStatus.model) {
      models.push({
        name: store.aiEngineStatus.model.name,
        description: `OpenAI 隐私过滤模型 · ${store.aiEngineStatus.model.entity_types.length} 种实体`,
        size_mb: store.aiEngineStatus.model.size_mb,
        loaded: store.aiEngineStatus.state === 'ready',
      });
    }
    // 如果 available_count > 1，添加占位模型（未来扩展）
    if (store.aiEngineStatus.available_count > 1 && !store.aiEngineStatus.model) {
      for (let i = 0; i < store.aiEngineStatus.available_count; i++) {
        models.push({
          name: `model-${i + 1}`,
          description: '待加载模型',
          size_mb: 0,
          loaded: false,
        });
      }
    }
  }
  // 默认添加 privacy-filter 如果没有其他模型
  if (models.length === 0 && store.aiEngineStatus?.state !== 'not_available') {
    models.push({
      name: 'privacy-filter',
      description: 'OpenAI 隐私过滤模型',
      size_mb: 874,
      loaded: store.aiEngineStatus?.state === 'ready',
    });
  }
  return models;
});

// 切换模型选择
const toggleModelSelection = (modelName: string) => {
  if (selectedModels.has(modelName)) {
    selectedModels.delete(modelName);
  } else {
    selectedModels.add(modelName);
  }
};

// 切换 AI 引擎启用/停用
const handleToggleAi = async (event: Event) => {
  const enabled = (event.target as HTMLInputElement).checked;
  await store.toggleAiEngine(enabled);
};

// 刷新 AI 状态
const refreshAiStatus = async () => {
  await store.fetchAiStatus();
  await store.fetchEngineInfo();
  // 管理计时器
  if (store.aiEngineStatus?.state === 'loading') {
    if (!loadingTimer) startLoadingTimer();
  } else {
    stopLoadingTimer();
  }
};

// 重试加载
const handleRetryLoad = async () => {
  await store.toggleAiEngine(true);
  startLoadingTimer();
  // 定期刷新状态
  const checkInterval = setInterval(async () => {
    await store.fetchAiStatus();
    if (store.aiEngineStatus?.state !== 'loading') {
      clearInterval(checkInterval);
      stopLoadingTimer();
    }
  }, 5000);
};

// 生命周期：启动时检查状态，启动计时器
onMounted(async () => {
  await store.fetchAiStatus();
  if (store.aiEngineStatus?.state === 'loading') {
    startLoadingTimer();
  }
});

// 生命周期：卸载时清理计时器
onUnmounted(() => {
  stopLoadingTimer();
});

/**
 * 复制邮箱逻辑
 */
const copyEmail = async () => {
  try {
    await navigator.clipboard.writeText(developerInfo.email);
    emailCopied.value = true;
    
    // 触发精密机械音效
    store.playFeedbackSound('CLICK');
    
    // 2秒后重置状态
    setTimeout(() => {
      emailCopied.value = false;
    }, 2000);
  } catch (err) {
    console.error("复制失败", err);
  }
};

const startRecording = async () => {
  isRecording.value = true;
  showKeyWarning.value = false;
  await MaskAPI.setRecordingMode(true);
};

const stopRecording = async () => {
  isRecording.value = false;
  await MaskAPI.setRecordingMode(false);
};

const handleKeyDown = (e: KeyboardEvent) => {
  if (!isRecording.value) return;
  e.preventDefault(); e.stopPropagation();
  const mods: string[] = [];
  const isCtrl = e.ctrlKey || e.getModifierState('Control');
  const isAlt = e.altKey || e.getModifierState('Alt');
  const isShift = e.shiftKey || e.getModifierState('Shift');
  const isMeta = e.metaKey || e.getModifierState('Meta');

  if (isCtrl) mods.push("Ctrl");
  if (isAlt) mods.push("Alt");
  if (isShift) mods.push("Shift");
  if (isMeta) mods.push("Super");

  let key = e.key.toUpperCase();
  if (!["CONTROL", "ALT", "SHIFT", "META"].includes(key)) {
    if (key === " ") key = "SPACE";
    const finalShortcut = [...mods, key].join("+");
    if (finalShortcut.toLowerCase() === "alt+m") {
      showKeyWarning.value = true;
      store.playFeedbackSound('ERROR');
      setTimeout(() => showKeyWarning.value = false, 2500);
      return;
    }
    store.settings.magic_paste_shortcut = finalShortcut;
    stopRecording();
    store.playFeedbackSound('RECORD');
  }
};

const handleSave = async () => {
  await MaskAPI.updateSettings(store.settings);
  store.playFeedbackSound('ASCEND');
  await message("系统配置已实时同步至脱敏内核", { title: "同步成功", kind: "info" });
};

const sliderProgress = computed(() => ((store.settings.paste_delay_ms - 50) / (800 - 50)) * 100);
</script>

<template>
  <div class="max-w-5xl mx-auto space-y-8 animate-in fade-in duration-700 pb-16">
    
    <!-- 头部 -->
    <div class="flex items-center gap-6 mb-10 px-2">
      <div class="w-14 h-14 rounded-2xl bg-[#141210] border border-amber-500/10 flex items-center justify-center shadow-2xl">
        <Monitor class="text-amber-400/80 w-6 h-6" />
      </div>
      <div>
        <h2 class="text-3xl font-bold text-amber-50/90 tracking-tight">控制台偏好设置</h2>
        <p class="text-[10px] text-zinc-600 font-black uppercase tracking-[0.4em] mt-1.5 opacity-60">System Configuration & Developer Info</p>
      </div>
    </div>

    <div class="grid grid-cols-1 lg:grid-cols-2 gap-8">
      <!-- 模块 1: 引擎行为 -->
      <section class="config-card">
        <div class="card-header"><Cpu :size="16" class="text-blue-500" /><span>脱敏内核行为 (Kernel)</span></div>
        <div class="space-y-8 mt-8">
          <div class="setting-row">
            <div class="info"><span class="lbl">启用影子宇宙模式</span><span class="dsc">数据流在内存中脱敏，物理剪贴板保留原文</span></div>
            <label class="sw-wrapper"><input type="checkbox" v-model="store.settings.shadow_mode_enabled" @change="store.playFeedbackSound(store.settings.shadow_mode_enabled ? 'ASCEND' : 'DESCEND')"><div class="sw-slider"></div></label>
          </div>
          <div class="relative p-7 bg-black/40 rounded-[2rem] border border-white/[0.03] shadow-inner group">
            <div class="flex justify-between items-center mb-5 px-1"><span class="text-[10px] font-black text-zinc-600 uppercase tracking-widest">安全粘贴快捷键</span><Keyboard :size="14" class="text-zinc-800" /></div>
            <div class="relative">
              <input readonly :value="isRecording ? '正在监听按键组合...' : store.settings.magic_paste_shortcut" @keydown="handleKeyDown" @focus="startRecording" @blur="stopRecording" class="shortcut-input" :class="{'recording': isRecording, 'error-shake': showKeyWarning}" />
              <transition name="slide-fade"><div v-if="showKeyWarning" class="absolute -bottom-7 left-0 right-0 flex justify-center"><span class="text-[9px] text-red-500 font-bold uppercase tracking-widest flex items-center gap-1.5 bg-[#0c0b0a] px-3 py-1 rounded-full border border-red-500/20"><AlertTriangle :size="10" /> Alt+M 为系统内置切换键</span></div></transition>
            </div>
          </div>
        </div>
      </section>

      <!-- 模块 2: AI 引擎 -->
      <section class="config-card lg:col-span-2">
        <div class="card-header"><Brain :size="16" class="text-purple-500" /><span>AI 智能识别 (AI Engine)</span></div>
        <div class="space-y-6 mt-8">

          <!-- 引擎总开关 -->
          <div class="flex items-center justify-between p-4 bg-black/30 rounded-xl border border-white/[0.03]">
            <div class="flex items-center gap-3">
              <div :class="['w-3 h-3 rounded-full transition-colors', aiStatusColor]" />
              <div>
                <span class="text-[11px] text-zinc-300 font-medium">AI 引擎状态</span>
                <p class="text-[9px] text-zinc-600 mt-0.5">{{ aiStatusText }}</p>
              </div>
            </div>
            <div class="flex items-center gap-3">
              <button @click="refreshAiStatus" class="p-1.5 rounded-lg hover:bg-white/5 transition-colors" title="刷新状态">
                <RotateCcw :size="12" class="text-zinc-600" />
              </button>
              <!-- 启用/停用开关 -->
              <label class="sw-wrapper sm" :title="aiEnabled ? '点击停用 AI' : '点击启用 AI'">
                <input type="checkbox" :checked="aiEnabled" @change="handleToggleAi" :disabled="store.aiEngineStatus?.state === 'loading'">
                <div class="sw-slider sm"></div>
              </label>
            </div>
          </div>

          <!-- 加载状态 (仅 loading 状态显示) -->
          <div v-if="store.aiEngineStatus?.state === 'loading'" class="p-4 bg-amber-500/10 rounded-xl border border-amber-500/20 space-y-3">
            <div class="flex items-center gap-3">
              <div class="relative w-4 h-4">
                <div class="absolute inset-0 rounded-full bg-amber-500 animate-ping opacity-75"></div>
                <div class="relative w-4 h-4 rounded-full bg-amber-500"></div>
              </div>
              <div>
                <span class="text-[11px] text-amber-400 font-medium">模型加载中</span>
                <p class="text-[9px] text-amber-400/60 mt-0.5">{{ loadingElapsedText }}</p>
              </div>
            </div>
            <div class="flex items-center gap-2">
              <Loader2 :size="12" class="text-amber-400 animate-spin" />
              <span class="text-[10px] text-amber-400/80">正在加载 874MB 模型文件...</span>
            </div>
            <p class="text-[9px] text-zinc-600">首次加载约需 3-10 分钟，后续启动将使用缓存</p>
          </div>

          <!-- 错误信息 -->
          <div v-if="store.aiEngineStatus?.state === 'error'" class="p-4 bg-red-500/10 rounded-xl border border-red-500/20 space-y-3">
            <div class="flex items-center gap-2">
              <AlertTriangle :size="14" class="text-red-400" />
              <span class="text-[11px] text-red-400 font-medium">加载失败</span>
            </div>
            <p class="text-[10px] text-red-400/70 break-all">{{ store.aiEngineStatus.error }}</p>
            <button @click="handleRetryLoad" class="flex items-center gap-1.5 px-3 py-1.5 bg-red-500/20 hover:bg-red-500/30 rounded-lg text-[10px] text-red-400 transition-colors">
              <RotateCcw :size="10" />
              <span>重新加载</span>
            </button>
          </div>

          <!-- 已加载模型信息 -->
          <div v-if="store.aiEngineStatus?.model" class="space-y-3">
            <div class="flex items-center gap-2 mb-2">
              <Zap :size="12" class="text-amber-500" />
              <span class="text-[10px] font-black text-zinc-500 uppercase tracking-widest">已加载模型</span>
            </div>

            <div class="p-4 bg-black/30 rounded-xl border border-white/[0.03] space-y-3">
              <!-- 模型名称和大小 -->
              <div class="flex items-center justify-between">
                <div class="flex items-center gap-2">
                  <div class="w-6 h-6 rounded-lg bg-purple-500/20 flex items-center justify-center">
                    <Brain :size="12" class="text-purple-400" />
                  </div>
                  <div>
                    <p class="text-[11px] text-zinc-300 font-medium">{{ store.aiEngineStatus.model.name }}</p>
                    <p class="text-[9px] text-zinc-600">v{{ store.aiEngineStatus.model.version }}</p>
                  </div>
                </div>
                <span class="text-[10px] text-zinc-500 font-mono">{{ store.aiEngineStatus.model.size_mb.toFixed(0) }} MB</span>
              </div>

              <!-- 支持的实体类型 -->
              <div class="pt-2 border-t border-white/[0.03]">
                <span class="text-[9px] text-zinc-600 uppercase tracking-widest">支持识别</span>
                <div class="flex flex-wrap gap-1.5 mt-2">
                  <span v-for="type in store.aiEngineStatus.model.entity_types" :key="type"
                    class="px-2 py-0.5 bg-purple-500/10 rounded text-[9px] text-purple-400 border border-purple-500/20">
                    {{ formatEntityType(type) }}
                  </span>
                </div>
              </div>
            </div>
          </div>

          <!-- 可用模型列表 (多选) -->
          <div v-if="store.aiEngineStatus?.available_count > 0" class="space-y-2">
            <div class="flex items-center justify-between">
              <div class="flex items-center gap-2">
                <Cpu :size="12" class="text-blue-500" />
                <span class="text-[10px] font-black text-zinc-500 uppercase tracking-widest">可用模型</span>
              </div>
              <span class="text-[10px] text-zinc-600">{{ selectedModels.size }}/{{ store.aiEngineStatus.available_count }} 已选</span>
            </div>

            <div class="space-y-2">
              <!-- 模型列表项 -->
              <div v-for="model in availableModels" :key="model.name"
                @click="toggleModelSelection(model.name)"
                class="p-3 bg-black/20 rounded-xl border border-white/[0.03] hover:border-purple-500/30 transition-colors cursor-pointer"
                :class="{ 'border-purple-500/50 bg-purple-500/5': selectedModels.has(model.name) }">
                <div class="flex items-center justify-between">
                  <div class="flex items-center gap-2">
                    <div class="w-4 h-4 rounded border flex items-center justify-center transition-colors"
                      :class="selectedModels.has(model.name) ? 'bg-purple-500 border-purple-500' : 'border-zinc-600'">
                      <Check v-if="selectedModels.has(model.name)" :size="10" class="text-white" />
                    </div>
                    <div>
                      <p class="text-[11px] text-zinc-300">{{ model.name }}</p>
                      <p class="text-[9px] text-zinc-600">{{ model.description }}</p>
                    </div>
                  </div>
                  <div class="flex items-center gap-2">
                    <span class="text-[10px] text-zinc-500 font-mono">{{ model.size_mb.toFixed(0) }} MB</span>
                    <div :class="['w-2 h-2 rounded-full', model.loaded ? 'bg-emerald-500' : 'bg-zinc-600']" :title="model.loaded ? '已加载' : '未加载'" />
                  </div>
                </div>
              </div>
            </div>
          </div>

          <!-- 无模型提示 -->
          <div v-else-if="store.aiEngineStatus?.state !== 'loading'" class="p-6 bg-black/20 rounded-xl border border-dashed border-white/[0.05] text-center space-y-2">
            <Brain :size="24" class="text-zinc-700 mx-auto" />
            <p class="text-[11px] text-zinc-500">未发现 AI 模型</p>
            <p class="text-[10px] text-zinc-600">将 ONNX 模型放置在 <code class="px-1.5 py-0.5 bg-black/30 rounded text-zinc-500">models/</code> 目录</p>
          </div>

          <!-- 识别器列表 -->
          <div v-if="store.engineInfo?.recognizers" class="pt-4 border-t border-white/[0.03]">
            <div class="flex items-center gap-2 mb-3">
              <Cpu :size="12" class="text-blue-500" />
              <span class="text-[10px] font-black text-zinc-500 uppercase tracking-widest">已注册识别器</span>
            </div>
            <div class="grid grid-cols-2 gap-2">
              <div v-for="name in store.engineInfo.recognizers" :key="name"
                class="flex items-center gap-2 p-2.5 bg-black/20 rounded-lg border border-white/[0.03]">
                <div :class="['w-1.5 h-1.5 rounded-full', getRecognizerColor(name)]" />
                <span class="text-[10px] text-zinc-400">{{ formatRecognizer(name) }}</span>
              </div>
            </div>
          </div>
        </div>
      </section>

      <!-- 模块 3: 交互感知 -->
      <section class="config-card">
        <div class="card-header"><Volume2 :size="16" class="text-amber-500" /><span>实时感官反馈 (Feedback)</span></div>
        <div class="space-y-7 mt-8">
          <div class="setting-row-sm"><div class="flex items-center gap-3"><Eye :size="14" class="text-zinc-700"/><span class="lbl-sm">开启蓝盾视觉气泡</span></div>
            <label class="sw-wrapper sm"><input type="checkbox" v-model="store.settings.enable_visual_feedback"><div class="sw-slider sm"></div></label>
          </div>
          <div class="setting-row-sm"><div class="flex items-center gap-3"><Volume2 :size="14" class="text-zinc-700"/><span class="lbl-sm">开启物理机械音效</span></div>
            <label class="sw-wrapper sm"><input type="checkbox" v-model="store.settings.enable_audio_feedback"><div class="sw-slider sm"></div></label>
          </div>
          <div class="pt-8 border-t border-white/[0.03] space-y-6">
            <div class="flex justify-between items-end">
              <div class="flex flex-col gap-1"><span class="lbl-sm flex items-center gap-2 font-bold text-zinc-300"><Timer :size="14" class="text-amber-500/60" /> 粘贴注入延迟</span><span class="text-[8px] text-zinc-700 uppercase font-black tracking-widest">Injection Latency Buffer</span></div>
              <span class="font-mono text-amber-200 text-sm font-bold bg-amber-500/10 px-3 py-1 rounded-lg border border-amber-500/20 shadow-[0_0_15px_rgba(245,158,11,0.1)]">{{ store.settings.paste_delay_ms }}ms</span>
            </div>
            <div class="range-container">
              <input type="range" v-model.number="store.settings.paste_delay_ms" min="50" max="800" step="50" class="precision-range" :style="{ backgroundSize: `${sliderProgress}% 100%` }" />
              <div class="range-ticks"><span v-for="n in 8" :key="n" :class="{ 'active': (n-1)*100 + 50 <= store.settings.paste_delay_ms }"></span></div>
            </div>
          </div>
        </div>
      </section>

      <!-- 🚀 模块 3: 关于与开发者 -->
      <section class="config-card lg:col-span-2">
        <div class="card-header"><Info :size="16" class="text-emerald-500" /><span>关于与支持 (About & Support)</span></div>
        
        <div class="grid grid-cols-1 md:grid-cols-3 gap-10 mt-8">
          <!-- 开发者 -->
          <div class="flex flex-col gap-4">
            <div class="flex items-center gap-3">
              <div class="p-2 rounded-lg bg-white/5 border border-white/5"><User :size="14" class="text-zinc-400"/></div>
              <span class="text-[11px] font-black text-zinc-500 uppercase tracking-widest">Author</span>
            </div>
            <div class="pl-2">
              <p class="text-lg font-bold text-amber-50/90 tracking-tight">{{ developerInfo.name }}</p>
              <div class="mt-4 space-y-3">
                <!-- 🚀 优化后的邮箱复制条 -->
                <!-- 优化后的邮箱复制条 -->
                <div class="group flex items-center justify-between bg-white/[0.02] border border-white/5 p-3.5 rounded-xl hover:border-amber-500/30 transition-all duration-500">
                  <div class="flex items-center gap-3 overflow-hidden">
                    <div class="p-1.5 rounded-lg bg-black/20">
                      <Mail :size="14" class="text-zinc-600 shrink-0" />
                    </div>
                    <span class="text-[11px] text-zinc-400 font-mono truncate tracking-tight">{{ developerInfo.email }}</span>
                  </div>

                  <!-- 🚀 动效核心按钮 -->
                  <button 
                    @click="copyEmail" 
                    class="relative flex items-center justify-center w-8 h-8 rounded-lg transition-all duration-300"
                    :class="emailCopied ? 'bg-emerald-500/10' : 'hover:bg-amber-500/10'"
                  >
                    <Transition name="icon-pop" mode="out-in">
                      <div :key="emailCopied ? 'check' : 'copy'" class="flex items-center justify-center">
                        <Check v-if="emailCopied" :size="14" class="text-emerald-400" />
                        <Copy v-else :size="14" class="text-zinc-600 group-hover:text-amber-500/80 transition-colors" />
                      </div>
                    </Transition>

                    <!-- 复制成功的扩散微光 -->
                    <div v-if="emailCopied" class="absolute inset-0 rounded-lg shadow-[0_0_15px_rgba(16,185,129,0.2)] animate-pulse"></div>
                  </button>
                </div>
              </div>
            </div>
          </div>

          <!-- 资源中心 -->
          <div class="flex flex-col gap-4">
            <div class="flex items-center gap-3">
              <div class="p-2 rounded-lg bg-white/5 border border-white/5"><Github :size="14" class="text-zinc-400"/></div>
              <span class="text-[11px] font-black text-zinc-500 uppercase tracking-widest">Connect</span>
            </div>
            <div class="pl-2 space-y-3">
              <div @click="openLink(developerInfo.github)" class="group flex items-center justify-between bg-white/[0.02] border border-white/5 p-3 rounded-xl hover:border-amber-500/30 transition-all cursor-pointer">
                <div class="flex items-center gap-3"><Github :size="16" class="text-zinc-400 group-hover:text-white" /><span class="text-xs font-bold text-zinc-300 group-hover:text-white">GitHub Repository</span></div>
                <ExternalLink :size="12" class="text-zinc-700" />
              </div>
              <!-- 🚀 SafeMask Hub 暂时隐藏：改为 v-if="false" -->
              <div v-if="false" @click="openLink(developerInfo.website)" class="group flex items-center justify-between bg-white/[0.02] border border-white/5 p-3 rounded-xl hover:border-amber-500/30 transition-all cursor-pointer">
                <div class="flex items-center gap-3"><Globe :size="16" class="text-amber-500/60 group-hover:text-amber-400" /><span class="text-xs font-bold text-zinc-300 group-hover:text-white">SafeMask Hub</span></div>
                <div class="flex items-center gap-2"><span class="text-[8px] bg-amber-500/10 text-amber-500 px-1.5 py-0.5 rounded uppercase font-black">Coming Soon</span><ExternalLink :size="12" class="text-zinc-700" /></div>
              </div>
            </div>
          </div>

          <!-- 声明 -->
          <div class="flex flex-col gap-4">
            <div class="flex items-center gap-3">
              <div class="p-2 rounded-lg bg-white/5 border border-white/5"><Copyright :size="14" class="text-zinc-400"/></div>
              <span class="text-[11px] font-black text-zinc-500 uppercase tracking-widest">Project Info</span>
            </div>
            <div class="pl-2 space-y-2">
               <div class="flex justify-between items-center text-xs"><span class="text-zinc-600 font-bold">构建版本</span><span class="text-amber-200/80 font-mono">{{ developerInfo.version }}</span></div>
               <div class="flex justify-between items-center text-xs"><span class="text-zinc-600 font-bold">开源协议</span><span class="text-zinc-400 font-mono">MIT License</span></div>
               <div class="mt-4 p-3 bg-emerald-500/5 border border-emerald-500/10 rounded-xl">
                 <p class="text-[10px] text-emerald-500/80 font-medium leading-relaxed italic">
                   "SafeMask 核心脱敏逻辑完全离线运行，绝不上传任何原始敏感数据。"
                 </p>
               </div>
            </div>
          </div>
        </div>
      </section>
    </div>

    <!-- 底部操作 -->
    <div class="flex justify-between items-center pt-10 border-t border-white/[0.03]">
      <button @click="store.clearHistory" class="purge-btn group"><Trash2 :size="14" class="group-hover:text-red-500 transition-colors" /><span>销毁所有审计痕迹</span></button>
      <button @click="handleSave" class="save-btn-jewelry"><Save :size="18" /><span>保存配置并重载内核</span></button>
    </div>
  </div>
</template>

<style scoped>
.config-card { @apply bg-[#0d0d0f]/80 border border-white/[0.04] rounded-[2.5rem] p-8 shadow-2xl; }
.card-header { @apply flex items-center gap-3 text-[11px] font-black text-amber-50/50 uppercase tracking-[0.3em]; }
.setting-row { @apply flex justify-between items-center; }
.setting-row-sm { @apply flex justify-between items-center py-1; }
.info { @apply flex flex-col gap-1; }
.lbl { @apply text-[15px] font-bold text-amber-50/80; }
.lbl-sm { @apply text-[13px] font-bold text-zinc-400; }
.dsc { @apply text-[10px] text-zinc-600 font-bold uppercase tracking-widest; }

.shortcut-input { @apply w-full bg-[#08080a] border border-white/[0.08] rounded-2xl py-5 text-base font-mono text-amber-200 text-center outline-none transition-all cursor-pointer shadow-inner; }
.shortcut-input.recording { @apply border-amber-500/50 bg-amber-500/[0.03] text-amber-400 shadow-[0_0_30px_rgba(245,158,11,0.1)]; }
.error-shake { animation: shake 0.4s ease-in-out; @apply border-red-500/50 text-red-500 !important; }

.range-container { @apply relative py-2; }
.precision-range { @apply w-full h-2.5 bg-zinc-900 rounded-full appearance-none cursor-pointer outline-none border border-white/[0.05] shadow-inner; background-image: linear-gradient(#f59e0b, #f59e0b); background-repeat: no-repeat; }
.precision-range::-webkit-slider-thumb { @apply appearance-none w-6 h-6 bg-amber-500 rounded-full shadow-[0_0_20px_rgba(245,158,11,0.6)] border-[4px] border-[#0c0b0a] transition-all active:scale-125; margin-top: -7px; }
.range-ticks { @apply absolute top-10 left-1 right-1 flex justify-between px-0.5 pointer-events-none; }
.range-ticks span { @apply w-[2px] h-1.5 bg-zinc-800 rounded-full transition-colors duration-500; }
.range-ticks span.active { @apply bg-amber-500/40; }

.save-btn-jewelry { @apply flex items-center gap-4 px-12 py-4 bg-amber-500/10 border border-amber-500/20 text-amber-500 rounded-2xl text-[11px] font-black uppercase tracking-[0.2em] transition-all duration-500 hover:bg-amber-500 hover:text-black hover:shadow-[0_0_40px_rgba(245,158,11,0.25)] active:scale-95; }
.purge-btn { @apply flex items-center gap-3 text-zinc-700 hover:text-red-400 transition-all font-black text-[10px] uppercase tracking-[0.2em] px-5 py-2.5 rounded-xl hover:bg-red-500/5; }

.sw-wrapper { @apply relative w-12 h-6 cursor-pointer; }
.sw-wrapper input { @apply opacity-0 w-0 h-0; }
.sw-slider { @apply absolute inset-0 bg-zinc-800 rounded-full transition-all duration-500 border border-white/[0.05]; }
.sw-slider::before { content: ""; @apply absolute h-4 w-4 left-1 bottom-1 bg-zinc-500 rounded-full transition-all duration-500 shadow-lg; }
input:checked + .sw-slider { @apply bg-blue-600/80 border-blue-400/20; }
input:checked + .sw-slider::before { @apply translate-x-6 bg-white shadow-[0_0_15px_white]; }

.sw-wrapper.sm { @apply w-9 h-5; }
.sw-slider.sm::before { @apply h-3 w-3 left-1 bottom-1; }
input:checked + .sw-slider.sm::before { @apply translate-x-4; }

@keyframes shake { 0%, 100% { transform: translateX(0); } 25% { transform: translateX(-6px); } 75% { transform: translateX(6px); } }
</style>