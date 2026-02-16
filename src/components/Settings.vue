<script setup lang="ts">
import { ref } from 'vue';
import { useAppStore } from '../stores/useAppStore';
import { MaskAPI } from '../services/api';
import { Settings as SettingsIcon, Shield, Keyboard, Bell, Timer, RotateCcw, Save, Trash2, Github, Info, ExternalLink } from 'lucide-vue-next';
import { confirm } from '@tauri-apps/plugin-dialog';

const store = useAppStore();
const isRecording = ref(false);

const handleRecord = (e: KeyboardEvent) => {
  if (!isRecording.value) return;
  e.preventDefault();
  const mods: string[] = [];
  if (e.ctrlKey) mods.push("Ctrl");
  if (e.altKey) mods.push("Alt");
  if (e.shiftKey) mods.push("Shift");
  if (e.metaKey) mods.push("Super");
  const key = e.key.toUpperCase();
  if (!["CONTROL", "ALT", "SHIFT", "META"].includes(key)) {
    store.settings.magic_paste_shortcut = [...mods, key].join("+");
    isRecording.value = false;
  }
};

const handleSave = async () => {
  await MaskAPI.updateSettings(store.settings);
  alert("✅ 设置已保存并重载");
};
</script>

<template>
  <div class="max-w-4xl mx-auto space-y-8 animate-in fade-in slide-in-from-bottom-4 duration-500 pb-20">
    <div class="flex items-center gap-4 mb-10"><div class="p-3 bg-blue-600/20 rounded-2xl"><SettingsIcon class="text-blue-500 w-6 h-6" /></div>
      <div><h2 class="text-2xl font-bold text-white">系统设置</h2><p class="text-zinc-500 text-sm font-medium">管理宇宙行为与交互偏好</p></div>
    </div>

    <div class="grid grid-cols-1 md:grid-cols-2 gap-8">
      <section class="glass p-8 rounded-[2.5rem] space-y-6 border border-white/5">
        <div class="flex items-center gap-3"><Shield :size="18" class="text-blue-400" /><h3 class="font-bold text-zinc-200">引擎核心</h3></div>
        <div class="space-y-6">
          <div class="row"><div class="info"><span class="lbl">启用影子模式</span><span class="dsc">复制不拦截，Alt+V 安全粘贴</span></div>
            <button @click="store.settings.shadow_mode_enabled = !store.settings.shadow_mode_enabled" :class="store.settings.shadow_mode_enabled ? 'bg-blue-600' : 'bg-zinc-800'" class="sw"><div class="dot" :class="{'act': store.settings.shadow_mode_enabled}"></div></button>
          </div>
          <div class="p-4 bg-white/5 rounded-2xl border border-white/5 space-y-3">
            <div class="flex justify-between items-center"><span class="lbl">安全粘贴快捷键</span><Keyboard :size="14" class="text-zinc-600" /></div>
            <input readonly :value="isRecording ? '录制中...' : store.settings.magic_paste_shortcut" @keydown="handleRecord" @focus="isRecording=true" @blur="isRecording=false" :class="{'rec': isRecording}" class="shortcut-box" />
          </div>
        </div>
      </section>

      <section class="glass p-8 rounded-[2.5rem] space-y-6 border border-white/5">
        <div class="flex items-center gap-3"><Bell :size="18" class="text-blue-400" /><h3 class="font-bold text-zinc-200">交互反馈</h3></div>
        <div class="space-y-4">
          <div class="row"><span class="lbl-sm">显示蓝盾气泡提示</span><button @click="store.settings.enable_visual_feedback = !store.settings.enable_visual_feedback" :class="store.settings.enable_visual_feedback ? 'bg-blue-600' : 'bg-zinc-800'" class="sw-sm"><div class="dot-sm" :class="{'act': store.settings.enable_visual_feedback}"></div></button></div>
          <div class="row"><span class="lbl-sm">启用机械音效反馈</span><button @click="store.settings.enable_audio_feedback = !store.settings.enable_audio_feedback" :class="store.settings.enable_audio_feedback ? 'bg-blue-600' : 'bg-zinc-800'" class="sw-sm"><div class="dot-sm" :class="{'act': store.settings.enable_audio_feedback}"></div></button></div>
          <div class="pt-4 border-t border-white/5 space-y-3">
            <div class="flex justify-between"><div class="flex items-center gap-2 text-zinc-500"><Timer :size="14" /><span class="lbl-sm">注入延迟</span></div><span class="font-mono text-blue-400 text-xs">{{ store.settings.paste_delay_ms }}ms</span></div>
            <input type="range" v-model.number="store.settings.paste_delay_ms" min="50" max="800" step="50" class="w-full h-1 bg-zinc-800 rounded-lg appearance-none accent-blue-500" />
          </div>
        </div>
      </section>
    </div>

    <div class="flex justify-between pt-8 border-t border-white/5">
      <button @click="store.clearHistory" class="flex items-center gap-2 text-zinc-500 hover:text-red-400 transition-all font-bold text-sm"><Trash2 :size="16" /> 清除审计历史</button>
      <button @click="handleSave" class="px-10 py-3 bg-blue-600 rounded-2xl font-bold flex items-center gap-2 hover:bg-blue-500 transition-all shadow-lg shadow-blue-600/20"><Save :size="18" /> 保存配置并重载</button>
    </div>
  </div>
</template>

<style scoped>
.glass { background: rgba(18, 18, 22, 0.6); backdrop-filter: blur(20px); }
.row { @apply flex justify-between items-center; }
.lbl { @apply text-sm font-bold text-zinc-200; }
.lbl-sm { @apply text-xs font-semibold text-zinc-400; }
.dsc { @apply text-[10px] text-zinc-500 block mt-0.5 uppercase tracking-wider; }
.sw { @apply w-11 h-6 rounded-full relative transition-colors duration-300; }
.dot { @apply absolute top-1 left-1 bg-white w-4 h-4 rounded-full transition-transform duration-300; }
.dot.act { @apply translate-x-5; }
.sw-sm { @apply w-8 h-[1.125rem] rounded-full relative transition-colors duration-300; }
.dot-sm { @apply absolute top-0.5 left-0.5 bg-white w-3.5 h-3.5 rounded-full transition-transform duration-300; }
.dot-sm.act { @apply translate-x-3.5; }
.shortcut-box { @apply w-full bg-black/40 border border-white/5 rounded-xl py-3 text-sm font-mono text-blue-400 text-center outline-none cursor-pointer hover:border-blue-500/30; }
.shortcut-box.rec { @apply border-blue-600 bg-blue-600/10 animate-pulse; }
</style>