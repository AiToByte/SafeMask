<script setup lang="ts">
import { ref, computed } from 'vue';
import { useAppStore } from '../stores/useAppStore';
import { MaskAPI } from '../services/api';
import { 
  Shield, Keyboard, Bell, Timer, RotateCcw, 
  Save, Trash2, Monitor, Cpu, Volume2, Eye, AlertTriangle
} from 'lucide-vue-next';
import { message } from '@tauri-apps/plugin-dialog';

const store = useAppStore();
const isRecording = ref(false);
const showKeyWarning = ref(false);

/**
 * 🚀 录制保护：进入录制时，告诉后端暂停所有“魔术粘贴”模拟，防止干扰
 */
const startRecording = async () => {
  isRecording.value = true;
  showKeyWarning.value = false;
  // 核心：锁定后端模拟按键
  await MaskAPI.setRecordingMode(true);
};

const stopRecording = async () => {
  isRecording.value = false;
  // 核心：解锁后端模拟按键
  await MaskAPI.setRecordingMode(false);
};

/**
 * 核心录制逻辑：采用 getModifierState 确保 Alt 键被精准捕获
 */
const handleKeyDown = (e: KeyboardEvent) => {
  if (!isRecording.value) return;
  
  // 阻止 Alt 激活系统菜单
  e.preventDefault();
  e.stopPropagation();

  const mods: string[] = [];
  
  // 这里的精准判定解决了 Alt+V 变 Ctrl+V 的问题
  const isCtrl = e.ctrlKey || e.getModifierState('Control');
  const isAlt = e.altKey || e.getModifierState('Alt');
  const isShift = e.shiftKey || e.getModifierState('Shift');
  const isMeta = e.metaKey || e.getModifierState('Meta') || e.getModifierState('OS');

  if (isCtrl) mods.push("Ctrl");
  if (isAlt) mods.push("Alt");
  if (isShift) mods.push("Shift");
  if (isMeta) mods.push("Super");

  let key = e.key.toUpperCase();
  
  // 排除单纯按下修饰键的情况
  const isModifierOnly = ["CONTROL", "ALT", "SHIFT", "META"].includes(key);
  
  if (!isModifierOnly) {
    if (key === " ") key = "SPACE";

    const finalShortcut = [...mods, key].join("+");

    // 校验：禁止占用系统切换键 Alt+M
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

// 动态计算进度条背景宽度
const sliderProgress = computed(() => {
  return ((store.settings.paste_delay_ms - 50) / (800 - 50)) * 100;
});
</script>

<template>
  <div class="max-w-5xl mx-auto space-y-10 animate-in fade-in duration-700 pb-16">
    
    <!-- 顶栏介绍 -->
    <div class="flex items-center gap-6 mb-12 px-2">
      <div class="w-14 h-14 rounded-2xl bg-[#141210] border border-amber-500/10 flex items-center justify-center shadow-2xl shadow-black">
        <Monitor class="text-amber-400/80 w-6 h-6" />
      </div>
      <div>
        <h2 class="text-3xl font-bold text-amber-50/90 tracking-tight">控制台偏好设置</h2>
        <p class="text-[10px] text-zinc-600 font-black uppercase tracking-[0.4em] mt-1.5 opacity-60">System Configuration & Interface Feedback</p>
      </div>
    </div>

    <div class="grid grid-cols-1 lg:grid-cols-2 gap-8">
      
      <!-- 1. 引擎行为模块 -->
      <section class="config-card">
        <div class="card-header">
          <Cpu :size="16" class="text-blue-500" />
          <span>脱敏内核行为</span>
        </div>

        <div class="space-y-8 mt-8">
          <div class="setting-row">
            <div class="info">
              <span class="lbl">启用影子宇宙模式</span>
              <span class="dsc">粘贴时选择宇宙，物理剪贴板始终保留明文</span>
            </div>
            <label class="sw-wrapper">
              <input type="checkbox" v-model="store.settings.shadow_mode_enabled" @change="store.playFeedbackSound(store.settings.shadow_mode_enabled ? 'ASCEND' : 'DESCEND')">
              <div class="sw-slider"></div>
            </label>
          </div>

          <!-- 快捷键录制 -->
          <div class="relative p-7 bg-black/40 rounded-[2rem] border border-white/[0.03] shadow-inner group">
            <div class="flex justify-between items-center mb-5 px-1">
              <span class="text-[10px] font-black text-zinc-600 uppercase tracking-widest">安全粘贴快捷键</span>
              <Keyboard :size="14" class="text-zinc-800" />
            </div>
            
            <div class="relative">
              <input 
                readonly 
                :value="isRecording ? '正在等待按键...' : store.settings.magic_paste_shortcut" 
                @keydown="handleKeyDown" 
                @focus="startRecording" 
                @blur="stopRecording" 
                class="shortcut-input"
                :class="{'recording': isRecording, 'error-shake': showKeyWarning}"
              />
              
              <transition name="slide-fade">
                <div v-if="showKeyWarning" class="absolute -bottom-7 left-0 right-0 flex justify-center">
                  <span class="text-[9px] text-red-500 font-bold uppercase tracking-widest flex items-center gap-1.5 bg-[#0c0b0a] px-3 py-1 rounded-full border border-red-500/20">
                    <AlertTriangle :size="10" /> Alt+M 为系统内置切换键
                  </span>
                </div>
              </transition>
            </div>
          </div>
        </div>
      </section>

      <!-- 2. 交互感知模块 -->
      <section class="config-card">
        <div class="card-header">
          <Volume2 :size="16" class="text-amber-500" />
          <span>实时感官反馈</span>
        </div>

        <div class="space-y-7 mt-8">
          <div class="setting-row-sm">
             <div class="flex items-center gap-3">
               <Eye :size="14" class="text-zinc-700"/>
               <span class="lbl-sm">开启蓝盾视觉气泡</span>
             </div>
             <label class="sw-wrapper sm">
                <input type="checkbox" v-model="store.settings.enable_visual_feedback">
                <div class="sw-slider sm"></div>
             </label>
          </div>

          <div class="setting-row-sm">
             <div class="flex items-center gap-3">
               <Volume2 :size="14" class="text-zinc-700"/>
               <span class="lbl-sm">开启物理机械音效</span>
             </div>
             <label class="sw-wrapper sm">
                <input type="checkbox" v-model="store.settings.enable_audio_feedback">
                <div class="sw-slider sm"></div>
             </label>
          </div>

          <!-- 🚀 优化后的延迟滑动条：高辨识度轴线 -->
          <div class="pt-8 border-t border-white/[0.03] space-y-6">
            <div class="flex justify-between items-end">
              <div class="flex flex-col gap-1">
                <span class="lbl-sm flex items-center gap-2 font-bold text-zinc-300">
                  <Timer :size="14" class="text-amber-500/60" /> 
                  粘贴注入延迟
                </span>
                <span class="text-[8px] text-zinc-700 uppercase font-black tracking-widest">Injection Latency Buffer</span>
              </div>
              <span class="font-mono text-amber-200 text-sm font-bold bg-amber-500/10 px-3 py-1 rounded-lg border border-amber-500/20 shadow-[0_0_15px_rgba(245,158,11,0.1)]">
                {{ store.settings.paste_delay_ms }}ms
              </span>
            </div>
            
            <div class="range-container">
              <input 
                type="range" 
                v-model.number="store.settings.paste_delay_ms" 
                min="50" max="800" step="50" 
                class="precision-range"
                :style="{ backgroundSize: `${sliderProgress}% 100%` }"
              />
              <!-- 刻度线 -->
              <div class="range-ticks">
                <span v-for="n in 8" :key="n" :class="{ 'active': (n-1)*100 + 50 <= store.settings.paste_delay_ms }"></span>
              </div>
            </div>
          </div>
        </div>
      </section>
    </div>

    <!-- 底部操作区 -->
    <div class="flex justify-between items-center pt-10 border-t border-white/[0.03]">
      <button @click="store.clearHistory" class="purge-btn group">
        <Trash2 :size="14" class="group-hover:text-red-500 transition-colors" /> 
        <span>销毁所有审计痕迹</span>
      </button>
      
      <button @click="handleSave" class="save-btn-jewelry">
        <Save :size="18" />
        <span>同步配置并重载内核</span>
      </button>
    </div>
  </div>
</template>

<style scoped>
.config-card {
  @apply bg-[#0d0d0f]/80 border border-white/[0.04] rounded-[2.5rem] p-8 shadow-2xl;
}
.card-header {
  @apply flex items-center gap-3 text-[11px] font-black text-amber-50/50 uppercase tracking-[0.3em];
}

.setting-row { @apply flex justify-between items-center; }
.setting-row-sm { @apply flex justify-between items-center py-1; }

.info { @apply flex flex-col gap-1; }
.lbl { @apply text-[15px] font-bold text-amber-50/80; }
.lbl-sm { @apply text-[13px] font-bold text-zinc-400; }
.dsc { @apply text-[10px] text-zinc-600 font-bold uppercase tracking-widest; }

/* 🚀 快捷键输入框 */
.shortcut-input {
  @apply w-full bg-[#08080a] border border-white/[0.08] rounded-2xl py-5 text-base font-mono text-amber-200 text-center outline-none transition-all cursor-pointer shadow-inner;
}
.shortcut-input.recording {
  @apply border-amber-500/50 bg-amber-500/[0.03] text-amber-400 shadow-[0_0_30px_rgba(245,158,11,0.1)];
}
.error-shake { animation: shake 0.4s ease-in-out; @apply border-red-500/50 text-red-500 !important; }

/* 🚀 高辨识度滑动条 (Slider) */
.range-container {
  @apply relative py-2;
}

.precision-range {
  @apply w-full h-2.5 bg-zinc-900 rounded-full appearance-none cursor-pointer outline-none border border-white/[0.05] shadow-inner;
  /* 进度条底色 */
  background-image: linear-gradient(#f59e0b, #f59e0b);
  background-repeat: no-repeat;
}

.precision-range::-webkit-slider-runnable-track {
  @apply h-2.5 rounded-full;
}

.precision-range::-webkit-slider-thumb {
  @apply appearance-none w-6 h-6 bg-amber-500 rounded-full shadow-[0_0_20px_rgba(245,158,11,0.6)] border-[4px] border-[#0c0b0a] transition-all active:scale-125 active:bg-white;
  margin-top: -7px;
}

.range-ticks {
  @apply absolute top-10 left-1 right-1 flex justify-between px-0.5 pointer-events-none;
}
.range-ticks span {
  @apply w-[2px] h-1.5 bg-zinc-800 rounded-full transition-colors duration-500;
}
.range-ticks span.active {
  @apply bg-amber-500/40;
}

/* 🚀 珠宝化按钮 */
.save-btn-jewelry {
  @apply flex items-center gap-4 px-12 py-4 bg-amber-500/10 border border-amber-500/20 text-amber-500 rounded-2xl text-[11px] font-black uppercase tracking-[0.2em] transition-all duration-500 hover:bg-amber-500 hover:text-black hover:shadow-[0_0_40px_rgba(245,158,11,0.25)] active:scale-95;
}

.purge-btn {
  @apply flex items-center gap-3 text-zinc-700 hover:text-red-400 transition-all font-black text-[10px] uppercase tracking-[0.2em] px-5 py-2.5 rounded-xl hover:bg-red-500/5;
}

/* 🚀 自定义开关 */
.sw-wrapper { @apply relative w-12 h-6 cursor-pointer; }
.sw-wrapper input { @apply opacity-0 w-0 h-0; }
.sw-slider { @apply absolute inset-0 bg-zinc-800 rounded-full transition-all duration-500 border border-white/[0.05]; }
.sw-slider::before { content: ""; @apply absolute h-4 w-4 left-1 bottom-1 bg-zinc-500 rounded-full transition-all duration-500 shadow-lg; }
input:checked + .sw-slider { @apply bg-blue-600/80 border-blue-400/20; }
input:checked + .sw-slider::before { @apply translate-x-6 bg-white shadow-[0_0_15px_white]; }

.sw-wrapper.sm { @apply w-9 h-5; }
.sw-slider.sm::before { @apply h-3 w-3 left-1 bottom-1; }
input:checked + .sw-slider.sm::before { @apply translate-x-4; }

@keyframes shake {
  0%, 100% { transform: translateX(0); }
  25% { transform: translateX(-6px); }
  75% { transform: translateX(6px); }
}

.slide-fade-enter-active { transition: all 0.3s ease-out; }
.slide-fade-enter-from { opacity: 0; transform: translateY(-10px); }
</style>