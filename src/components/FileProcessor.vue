<script setup lang="ts">
import { listen } from "@tauri-apps/api/event";
import { onMounted, onUnmounted } from "vue";
import { useAppStore } from "../stores/useAppStore";
import { MaskAPI } from "../services/api";
import { message, ask } from '@tauri-apps/plugin-dialog';

const store = useAppStore();

// 处理文件脱敏核心逻辑
const startProcessing = async (path: string) => {
  if (!path) return;
  store.isProcessing = true;
  store.currentFileName = path.split(/[\\/]/).pop() || "";
  
  try {
    // 1. 调用脱敏接口
    const result = await MaskAPI.processFile(path);
    
    // 2. 交互式反馈：询问是否打开文件夹
    const shouldOpen = await ask(
      `${result.message}\n\n` +
      `处理耗时: ${result.duration}\n` +
      `引擎吞吐: ${result.throughput}\n\n` +
      `文件已保存至：\n${result.output_path}\n\n` +
      `是否立即打开所在文件夹？`,
      { 
        title: '🛡️ SafeMask 脱敏成功',
        kind: 'info',
        okLabel: '查看文件',
        cancelLabel: '知道了'
      }
    );

    if (shouldOpen) {
      await MaskAPI.openFolder(result.output_path);
    }

  } catch (err) {
    await message(`处理失败: ${err}`, { title: '错误', kind: 'error' });
  } finally {
    // 延迟重置进度条，给用户一点视觉缓冲
    setTimeout(() => { 
      store.isProcessing = false; 
      store.progress = 0; 
    }, 800);
  }
};


// 点击上传
const handleBrowse = async () => {
  const selected = await MaskAPI.selectFile();
  if (selected && typeof selected === 'string') {
    await startProcessing(selected);
  }
};

let unlistenDrag: any;

onMounted(async () => {
  // 监听 Tauri 拖拽事件
  unlistenDrag = await listen<{ paths: string[] }>("tauri://drag-drop", (event) => {
    const path = event.payload.paths[0];
    startProcessing(path);
  });
});

onUnmounted(() => { if (unlistenDrag) unlistenDrag(); });
</script>

<template>
  <div 
    @click="handleBrowse"
    class="flex-1 border-2 border-dashed border-zinc-800 rounded-[3rem] flex flex-col items-center justify-center transition-all duration-300 group hover:border-blue-500/50 cursor-pointer"
    :class="{ 'bg-blue-500/5 border-blue-500/50': store.isProcessing }"
  >
    <div v-if="!store.isProcessing" class="text-center group-hover:scale-105 transition-transform">
      <div class="text-6xl mb-6">📂</div>
      <h3 class="text-xl font-bold mb-2 text-zinc-200">拖拽文件或点击上传</h3>
      <p class="text-zinc-500 text-sm">支持多 GB 级文件，保持行序 100% 一致</p>
    </div>

    <div v-else class="w-3/4 space-y-4 animate-in fade-in zoom-in duration-300">
      <div class="flex justify-between text-sm font-bold">
        <span class="text-blue-400 truncate max-w-xs">{{ store.currentFileName }}</span>
        <span class="font-mono">{{ Math.round(store.progress) }}%</span>
      </div>
      <div class="w-full bg-zinc-900 h-3 rounded-full overflow-hidden border border-zinc-800 p-[2px]">
        <div 
          class="bg-gradient-to-r from-blue-600 to-indigo-500 h-full rounded-full transition-all duration-300"
          :style="{ width: `${store.progress}%` }"
        ></div>
      </div>
      <p class="text-center text-xs text-zinc-500 animate-pulse">正在调用多核 Rust 引擎加速处理...</p>
    </div>
  </div>
</template>

