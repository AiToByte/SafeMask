<script setup lang="ts">
import { onMounted, ref } from 'vue';
import { useAppStore } from '../stores/useAppStore';
import { 
  Settings, Shield, Trash2, Github, Info, 
  Monitor, ExternalLink, CheckCircle2 
} from 'lucide-vue-next';
import { confirm } from '@tauri-apps/plugin-dialog';

// 🚀 修复点 1: 正确导入方式 (适配 Tauri v2)
// 如果之前报错 'open' 不存在，请尝试 revealItemInDir 或检查插件注册
import { openUrl } from '@tauri-apps/plugin-opener';

const store = useAppStore();
const showSuccess = ref(false);

// 🚀 添加初始化标识，防止重复请求
const isInitializing = ref(false);

onMounted(async () => {
  console.log("⚙️ Settings Component Mounted"); // 调试日志
  isInitializing.value = true;
  try {
    // 确保 store 里有这个方法
    if (typeof store.fetchAppInfo === 'function') {
      await store.fetchAppInfo();
      console.log("✅ App Info Fetched:", store.appInfo);
    } else {
      console.error("❌ store.fetchAppInfo is not a function");
    }
  } catch (err) {
    console.error("❌ Failed to load settings:", err);
  } finally {
    isInitializing.value = false;
  }
});

const handleClearHistory = async () => {
  const confirmed = await confirm(
    '确定要清除所有脱敏拦截记录吗？此操作无法恢复。',
    { title: '安全提醒', kind: 'warning' }
  );
  
  if (confirmed) {
    await store.clearHistory();
    showSuccess.value = true;
    setTimeout(() => showSuccess.value = false, 2000);
  }
};

const openGithub = async () => {
  if (store.appInfo?.github) {
    try {
      await openUrl(store.appInfo.github);
    } catch (e) {
      console.error("无法打开链接:", e);
      // 降级方案
      window.open(store.appInfo.github, '_blank');
    }
  }
};
</script>

<template>
  <!-- 外层容器添加加载状态遮罩(可选) -->
  <div v-if="isInitializing" class="h-full flex items-center justify-center">
    <div class="animate-spin rounded-full h-8 w-8 border-b-2 border-blue-500"></div>
  </div>

  <div v-else class="max-w-4xl mx-auto space-y-8 animate-in fade-in slide-in-from-bottom-4 duration-500 pb-20">
    <!-- 头部 -->
    <div class="flex items-center gap-4 mb-10">
      <div class="p-3 bg-blue-600/20 rounded-2xl">
        <Settings class="text-blue-500 w-6 h-6" />
      </div>
      <div>
        <h2 class="text-2xl font-bold text-white">系统设置</h2>
        <p class="text-zinc-500 text-sm font-medium">管理引擎行为与应用偏好</p>
      </div>
    </div>

    <div class="grid grid-cols-1 md:grid-cols-2 gap-6">
      <!-- 引擎控制卡片 -->
      <section class="glass p-8 rounded-[2.5rem] space-y-6 border border-white/5">
        <div class="flex items-center gap-3 mb-2">
          <Monitor :size="18" class="text-blue-400" />
          <h3 class="font-bold text-zinc-200">引擎核心</h3>
        </div>
        
        <div class="flex justify-between items-center group">
          <div>
            <p class="text-sm font-bold">剪贴板自动保护</p>
            <p class="text-xs text-zinc-500">开启后将静默脱敏剪贴板敏感信息</p>
          </div>
          <button 
            @click="store.toggleMonitor"
            class="w-12 h-6 rounded-full relative transition-all duration-300 shadow-inner"
            :class="store.isMonitorOn ? 'bg-blue-600' : 'bg-zinc-800'"
          >
            <div class="absolute top-1 left-1 bg-white w-4 h-4 rounded-full transition-transform shadow-sm"
                 :class="{ 'translate-x-6': store.isMonitorOn }"></div>
          </button>
        </div>

        <div class="pt-4 border-t border-white/5">
          <button 
            @click="handleClearHistory"
            class="flex items-center gap-2 text-xs font-bold text-red-400 hover:text-red-300 transition-colors"
          >
            <Trash2 :size="14" />
            清除脱敏拦截历史
          </button>
          <p v-if="showSuccess" class="text-[10px] text-emerald-400 mt-2 flex items-center gap-1">
            <CheckCircle2 :size="10" /> 记录已安全抹除
          </p>
        </div>
      </section>

      <!-- 关于卡片 -->
      <section class="glass p-8 rounded-[2.5rem] space-y-6 border border-white/5">
        <div class="flex items-center gap-3 mb-2">
          <Info :size="18" class="text-blue-400" />
          <h3 class="font-bold text-zinc-200">关于 SafeMask</h3>
        </div>

        <div class="space-y-4">
          <div class="flex justify-between items-baseline">
            <span class="text-xs text-zinc-500 font-bold uppercase tracking-widest">版本号</span>
            <span class="text-sm font-mono text-blue-400">{{ store.appInfo?.version || '1.0.0' }}</span>
          </div>
          <div class="flex justify-between items-baseline">
            <span class="text-xs text-zinc-500 font-bold uppercase tracking-widest">架构</span>
            <span class="text-sm font-mono text-zinc-300">Rust / Tauri 2.0</span>
          </div>
          
          <div class="pt-4 border-t border-white/5">
            <button 
              @click="openGithub"
              class="w-full flex items-center justify-between p-4 bg-white/[0.03] hover:bg-white/[0.06] rounded-2xl transition-all group"
            >
              <div class="flex items-center gap-3">
                <Github :size="18" class="text-zinc-400 group-hover:text-white" />
                <span class="text-sm font-bold text-zinc-300 group-hover:text-white">开源仓库</span>
              </div>
              <ExternalLink :size="14" class="text-zinc-600 group-hover:text-zinc-400" />
            </button>
          </div>
        </div>
      </section>

      <!-- 安全说明 -->
      <div class="md:col-span-2 glass p-6 rounded-[2rem] bg-emerald-500/5 border border-emerald-500/10 flex gap-4">
        <Shield class="text-emerald-500 shrink-0" :size="24" />
        <div class="space-y-1">
          <p class="text-sm font-bold text-emerald-200">100% 隐私保证</p>
          <p class="text-xs text-emerald-200/50 leading-relaxed italic">
            SafeMask 脱敏工作不会连接互联网，所有的正则表达式匹配和脱敏计算均在您的计算机本地完成。
            我们不收集、不上传任何原始记录。
          </p>
        </div>
      </div>
    </div>
  </div>
</template>