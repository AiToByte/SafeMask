将前端升级到 **Vue 3 (Composition API)** 是非常明智的选择。它不仅能让 UI 逻辑更加清晰，还能利用 Vue 的响应式系统完美对接 Rust 后端的异步事件（如进度回传）。

我们将采用 **Vite + Vue 3 + Tailwind CSS + Pinia** 的现代技术栈，并保持严格的**服务层（Service）- 状态层（Store）- 视图层（Component）**分层架构。

---

### 1. 推荐的目录结构
```text
safemask/src/
├── main.ts             # 入口文件
├── App.vue             # 根组件（处理全局系统监听）
├── style.css           # 全局样式（Tailwind 指令）
├── services/           # 【服务层】封装与 Rust 端的 invoke 通信
│   └── api.ts
├── stores/             # 【状态层】Pinia 管理全局状态（开关、规则数）
│   └── useAppStore.ts
└── components/         # 【组件层】可复用的 UI 单元
    ├── Sidebar.vue
    ├── StatCard.vue
    └── FileProcessor.vue
```

---

### 2. 代码实现

#### A. 服务层：`services/api.ts`
封装后端指令，增加类型提示。

```typescript
import { invoke } from "@tauri-apps/api/core";
import { open } from "@tauri-apps/plugin-dialog";

export interface RuleStats {
  rule_count: number;
  group_count: number;
}

export const MaskAPI = {
  // 获取规则统计
  async getStats(): Promise<RuleStats> {
    return await invoke("get_rules_stats");
  },

  // 切换监控开关
  async toggleMonitor(enabled: boolean): Promise<void> {
    await invoke("toggle_monitor", { enabled });
  },

  // 文件脱敏
  async processFile(inputPath: string, outputPath: string): Promise<string> {
    return await invoke("process_file_gui", { inputPath, outputPath });
  },

  // 选择文件
  async selectFile() {
    return await open({
      multiple: false,
      filters: [{ name: 'Log/Text', extensions: ['log', 'txt', 'csv', 'json'] }]
    });
  }
};
```

#### B. 状态层：`stores/useAppStore.ts`
使用 Pinia 管理全局状态，响应式处理脱敏进度。

```typescript
import { defineStore } from 'pinia';
import { ref } from 'vue';
import { MaskAPI } from '../services/api';

export const useAppStore = defineStore('app', () => {
  const isMonitorOn = ref(true);
  const ruleCount = ref(0);
  const isProcessing = ref(false);
  const progress = ref(0);
  const currentFileName = ref("");

  // 初始化统计
  const fetchStats = async () => {
    const stats = await MaskAPI.getStats();
    ruleCount.ref = stats.rule_count;
  };

  // 切换监控
  const toggleMonitor = async () => {
    isMonitorOn.value = !isMonitorOn.value;
    await MaskAPI.toggleMonitor(isMonitorOn.value);
  };

  return { 
    isMonitorOn, ruleCount, isProcessing, 
    progress, currentFileName, fetchStats, toggleMonitor 
  };
});
```

#### C. 组件层：`components/FileProcessor.vue`
处理核心的拖拽与点击上传逻辑。

```vue
<script setup lang="ts">
import { listen } from "@tauri-apps/api/event";
import { onMounted, onUnmounted } from "vue";
import { useAppStore } from "../stores/useAppStore";
import { MaskAPI } from "../services/api";

const store = useAppStore();

// 处理文件脱敏核心逻辑
const startProcessing = async (path: string) => {
  if (!path) return;
  store.isProcessing = true;
  store.currentFileName = path.split(/[\\/]/).pop() || "";
  
  try {
    const outPath = `${path}.masked.log`;
    await MaskAPI.processFile(path, outPath);
  } catch (err) {
    console.error("处理失败:", err);
  } finally {
    setTimeout(() => { store.isProcessing = false; store.progress = 0; }, 1000);
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
```

#### D. 根组件：`App.vue`
负责全局布局与系统级事件监听。

```vue
<script setup lang="ts">
import { onMounted } from 'vue';
import { listen } from "@tauri-apps/api/event";
import { useAppStore } from './stores/useAppStore';
import Sidebar from './components/Sidebar.vue';
import StatCard from './components/StatCard.vue';
import FileProcessor from './components/FileProcessor.vue';

const store = useAppStore();

onMounted(async () => {
  // 1. 初始化统计信息
  await store.fetchStats();

  // 2. 监听后端回传的进度事件
  await listen<{ percentage: number }>("file-progress", (event) => {
    store.progress = event.payload.percentage;
  });

  // 3. 监听脱敏通知
  await listen("masked-event", (event) => {
    console.info("🛡️ SafeMask:", event.payload);
    // 这里可以集成更高级的 Toast 组件
  });
});
</script>

<template>
  <div class="flex h-screen bg-[#09090b] text-zinc-100 select-none overflow-hidden">
    <Sidebar />

    <main class="flex-1 p-12 flex flex-col max-w-6xl mx-auto w-full">
      <header class="flex justify-between items-center mb-12">
        <div>
          <h1 class="text-4xl font-extrabold tracking-tight bg-clip-text text-transparent bg-gradient-to-b from-white to-zinc-500">
            SafeMask 控制台
          </h1>
          <p class="text-zinc-500 mt-2 text-lg">实时保护剪贴板与大规模日志隐私</p>
        </div>

        <!-- 监控开关 -->
        <div class="glass px-6 py-4 rounded-[2rem] flex items-center gap-4">
          <span class="text-sm font-semibold">自动保护</span>
          <button 
            @click="store.toggleMonitor"
            class="w-12 h-6 rounded-full relative transition-colors duration-300"
            :class="store.isMonitorOn ? 'bg-blue-600' : 'bg-zinc-700'"
          >
            <div 
              class="absolute top-1 left-1 bg-white w-4 h-4 rounded-full transition-transform duration-300"
              :class="{ 'translate-x-6': store.isMonitorOn }"
            ></div>
          </button>
        </div>
      </header>

      <!-- 统计栏 -->
      <div class="grid grid-cols-3 gap-8 mb-12">
        <StatCard title="已加载规则" :value="store.ruleCount" />
        <StatCard title="引擎架构" value="HYBRID" color="text-blue-400" />
        <StatCard title="内存占用" value="LOW (MMAP)" color="text-emerald-400" />
      </div>

      <FileProcessor />
    </main>
  </div>
</template>

<style>
.glass {
  background: rgba(24, 24, 27, 0.8);
  backdrop-filter: blur(12px);
  border: 1px solid rgba(255, 255, 255, 0.08);
}
</style>
```

---

### 3. 该方案的升级点（面试/简历加分项）：

1.  **响应式状态流**：利用 Pinia 集中管理 UI 状态。当 Rust 后端通过 `emit` 发送 `file-progress` 时，Store 会自动捕获并更新进度条，实现了**数据驱动 UI** 的最佳实践。
2.  **异步通信增强**：所有后端调用都封装在 `MaskAPI` 服务中，支持 Promise 异步处理，避免了 UI 线程阻塞。
3.  **类型安全 (TypeScript)**：在 Vue 组件与 Rust 交互间定义了明确的 `interface`，大幅降低了由于后端字段变更导致的运行时错误。
4.  **组件化解耦**：侧边栏、统计卡片、文件处理器各司其职，后期如果增加“规则配置页面”，只需在 `views/` 下新建页面并配置路由即可。

### 4. 操作建议：
1.  **初始化 Vite**：如果你之前没装 Vue，在 `safemask` 目录下运行 `npm install vue pinia lucide-vue-next`。
2.  **配置 Tailwind**：确保 `tailwind.config.js` 包含新创建的 `.vue` 文件路径。
3.  **运行**：`npm run tauri dev`。

这套 Vue 3 架构将让你的 **SafeMask** 从一个简单的工具转变为一个具有**互联网大厂质感**的桌面桌面应用。