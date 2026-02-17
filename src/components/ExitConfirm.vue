<script setup lang="ts">
import { ref, onMounted, onUnmounted } from 'vue'
import { getCurrentWindow } from '@tauri-apps/api/window'
import { listen, type UnlistenFn } from '@tauri-apps/api/event'

const appWindow = getCurrentWindow()
const showConfirm = ref(false)
const rememberChoice = ref(false)
let unlistenClose: UnlistenFn;

onMounted(async () => {
  // 🚀 监听全局事件
  unlistenClose = await listen<string>('request-close', (event) => {
    console.log("收到关闭信号:", event.payload);
    
    // 检查用户是否有“记住选择”
    const saved = localStorage.getItem('exit-preference');
    if (saved === 'minimize') {
      appWindow.hide();
    } else if (saved === 'quit') {
      appWindow.destroy();
    } else {
      showConfirm.value = true;
    }
  });
});

onUnmounted(() => {
  if (unlistenClose) unlistenClose();
});

const handleExit = async (action: 'minimize' | 'quit') => {
  if (rememberChoice.value) {
    localStorage.setItem('exit-preference', action);
  }

  if (action === 'minimize') {
    await appWindow.hide();
  } else {
    await appWindow.destroy(); // 彻底销毁进程
  }
  showConfirm.value = false;
};
</script>

<template>
  <Transition name="fade">
    <div v-if="showConfirm" class="fixed inset-0 z-[1000] flex items-center justify-center bg-black/80 backdrop-blur-md">
      <div class="glass-panel p-10 rounded-[2.5rem] border border-white/10 w-full max-w-sm shadow-2xl text-center space-y-8">
        <div class="space-y-2">
          <h2 class="text-xl font-bold text-amber-50">确认退出程序？</h2>
          <p class="text-xs text-zinc-500 leading-relaxed px-4">
            SafeMask 可以在后台继续保护您的剪贴板隐私。建议选择“最小化至托盘”。
          </p>
        </div>

        <div class="flex flex-col gap-3">
          <button @click="handleExit('minimize')" class="btn-exit-primary">
            最小化到系统托盘
          </button>
          <button @click="handleExit('quit')" class="btn-exit-secondary">
            彻底关闭程序
          </button>
        </div>

        <label class="flex items-center justify-center gap-2 cursor-pointer group">
          <input type="checkbox" v-model="rememberChoice" class="opacity-0 absolute">
          <div class="w-4 h-4 rounded border border-white/10 flex items-center justify-center transition-colors" :class="rememberChoice ? 'bg-amber-500 border-amber-500' : 'bg-black/40'">
             <div v-if="rememberChoice" class="w-2 h-2 bg-black rounded-sm"></div>
          </div>
          <span class="text-[10px] font-bold text-zinc-600 group-hover:text-zinc-400 transition-colors">记住我的选择</span>
        </label>
      </div>
    </div>
  </Transition>
</template>

<style scoped>
.glass-panel { background: rgba(15, 15, 20, 0.95); }
.btn-exit-primary { @apply w-full py-4 bg-amber-500 text-black rounded-2xl font-bold text-xs hover:bg-amber-400 active:scale-95 transition-all shadow-lg shadow-amber-500/10; }
.btn-exit-secondary { @apply w-full py-4 bg-white/5 border border-white/10 text-zinc-400 rounded-2xl font-bold text-xs hover:bg-white/10 hover:text-white transition-all; }
.fade-enter-active, .fade-leave-active { transition: opacity 0.3s ease; }
.fade-enter-from, .fade-leave-to { opacity: 0; }
</style>