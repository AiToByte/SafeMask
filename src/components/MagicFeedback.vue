<script setup lang="ts">
import { useAppStore } from '../stores/useAppStore';
import { ShieldCheck, Ghost, ShieldAlert, Clipboard, Loader2 } from 'lucide-vue-next';
const store = useAppStore();
</script>

<template>
  <Transition name="slide-up">
    <div v-if="store.activeFeedback" class="fixed top-8 left-1/2 -translate-x-1/2 z-[999] pointer-events-none">
      <div class="glass flex items-center gap-4 px-6 py-3 rounded-full border shadow-2xl shadow-blue-500/10">
        <!-- 模式切换反馈 -->
        <template v-if="store.activeFeedback.type === 'MODE_CHANGE'">
          <div class="p-1.5 rounded-full" :class="store.activeFeedback.mode === 'SHADOW' ? 'bg-blue-600' : 'bg-amber-600'">
            <component :is="store.activeFeedback.mode === 'SHADOW' ? Ghost : ShieldAlert" :size="16" class="text-white" />
          </div>
          <div class="flex flex-col">
            <span class="text-xs font-bold text-white">{{ store.activeFeedback.mode === 'SHADOW' ? '影子宇宙已激活' : '哨兵宇宙已激活' }}</span>
            <span class="text-[9px] text-zinc-400 font-medium uppercase tracking-widest">{{ store.activeFeedback.mode === 'SHADOW' ? 'ALT+V 安全粘贴' : '实时强制拦截' }}</span>
          </div>
        </template>

        <!-- 粘贴成功反馈 -->
        <template v-else-if="store.activeFeedback.type === 'SUCCESS'">
          <div class="p-1.5 bg-emerald-600 rounded-full shadow-lg shadow-emerald-500/20">
            <ShieldCheck :size="16" class="text-white" />
          </div>
          <span class="text-xs font-bold text-white">安全宇宙副本已注入 ({{ store.activeFeedback.duration_ms }}ms)</span>
        </template>

        <!-- 常规反馈 -->
        <template v-else-if="store.activeFeedback.type === 'NORMAL'">
          <Clipboard :size="16" class="text-zinc-500" />
          <span class="text-xs font-medium text-zinc-400">常规粘贴 (未发现敏感信息)</span>
        </template>
      </div>
    </div>
  </Transition>
</template>

<style scoped>
.glass { background: rgba(15, 15, 20, 0.9); backdrop-filter: blur(20px); border-color: rgba(255, 255, 255, 0.1); }
.slide-up-enter-active { transition: all 0.4s cubic-bezier(0.16, 1, 0.3, 1); }
.slide-up-leave-active { transition: all 0.3s ease-in; }
.slide-up-enter-from { opacity: 0; transform: translate(-50%, -20px) scale(0.9); }
.slide-up-leave-to { opacity: 0; transform: translate(-50%, -10px) scale(0.9); }
</style>