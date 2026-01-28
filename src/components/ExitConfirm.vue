<script setup lang="ts">
import { ref, onMounted, onUnmounted } from 'vue';
import { getCurrentWindow } from '@tauri-apps/api/window';
import { listen, type UnlistenFn} from '@tauri-apps/api/event';
import { isPermissionGranted, requestPermission, sendNotification } from '@tauri-apps/plugin-notification';

const show = ref(false);
const remember = ref(false);
const appWindow = getCurrentWindow();
let unlisten: UnlistenFn;

const handleAction = async (action: 'quit' | 'tray') => {
    if (remember.value) {
        localStorage.setItem('close-behavior', action);
    }
    
    if (action === 'quit') {
        await appWindow.destroy(); // 真正关闭
    } else {
        show.value = false;
        await appWindow.hide(); // 隐藏到后台
    }

    // 发送系统通知告知用户位置
    let permission = await isPermissionGranted();
    if (!permission) {
        permission = await requestPermission() === 'granted';
    }
    if (permission) {
        sendNotification({ title: 'SafeMask', body: '程序已最小化到系统托盘，继续为您守护隐私。' });
    }
};

onMounted(async () => {
  // 监听 Rust 发来的关闭请求
  unlisten = await listen('request-close', (event) => {
    console.log("📥 收到来自 Rust 的关闭信号:", event.payload);
    // 检查本地存储的用户偏好
    const savedAction = localStorage.getItem('close-behavior');

    if (savedAction === 'quit' || savedAction === 'tray') {
      handleAction(savedAction as 'quit' | 'tray');
    } else {
      show.value = true;
    }
  });
});

onUnmounted(() => {
  if (unlisten) unlisten();
});
</script>

<template>
  <div v-if="show" class="fixed inset-0 z-[100] flex items-center justify-center bg-black/60 backdrop-blur-sm animate-in fade-in duration-300">
    <div class="glass w-[400px] p-8 rounded-[2.5rem] border border-white/10 shadow-2xl scale-in-center">
      <h3 class="text-xl font-bold mb-4">退出 SafeMask</h3>
      <p class="text-zinc-400 text-sm mb-8 leading-relaxed">
        您希望直接关闭程序，还是让它在后台继续保护您的剪贴板隐私？
      </p>
      
      <div class="space-y-3 mb-8">
        <button @click="handleAction('tray')" class="w-full py-3 bg-blue-600 hover:bg-blue-500 rounded-xl font-bold transition-all">
          最小化到系统托盘
        </button>
        <button @click="handleAction('quit')" class="w-full py-3 bg-zinc-800 hover:bg-zinc-700 rounded-xl font-bold transition-all text-zinc-400">
          彻底退出程序
        </button>
      </div>

      <div class="flex items-center gap-2 cursor-pointer" @click="remember = !remember">
        <div class="w-4 h-4 border border-zinc-600 rounded flex items-center justify-center transition-colors" :class="{'bg-blue-600 border-blue-600': remember}">
          <span v-if="remember" class="text-[10px]">✓</span>
        </div>
        <span class="text-xs text-zinc-500">记住我的选择，下次不再提示</span>
      </div>
    </div>
  </div>
</template>

<style scoped>
.scale-up { animation: scaleUp 0.2s ease-out forwards; }
@keyframes scaleUp {
  0% { transform: scale(0.95); opacity: 0; }
  100% { transform: scale(1); opacity: 1; }
}
</style>