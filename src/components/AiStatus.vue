<template>
  <div class="bg-surface rounded-xl p-4 border border-border">
    <div class="flex items-center justify-between mb-3">
      <h3 class="text-sm font-semibold text-text">AI 引擎</h3>
      <button
        @click="refreshStatus"
        class="p-1 rounded hover:bg-hover transition-colors"
        title="刷新状态"
      >
        <RefreshCw :size="14" class="text-muted" />
      </button>
    </div>

    <!-- 状态指示器 -->
    <div class="flex items-center gap-2 mb-3">
      <div
        :class="[
          'w-2 h-2 rounded-full',
          statusColor,
        ]"
      />
      <span class="text-xs text-muted">{{ statusText }}</span>
    </div>

    <!-- 模型信息 -->
    <div v-if="aiStatus?.model" class="space-y-2">
      <div class="flex items-center justify-between text-xs">
        <span class="text-muted">模型</span>
        <span class="text-text font-mono">{{ aiStatus.model.name }}</span>
      </div>
      <div class="flex items-center justify-between text-xs">
        <span class="text-muted">大小</span>
        <span class="text-text">{{ aiStatus.model.size_mb.toFixed(1) }} MB</span>
      </div>
      <div class="flex items-center justify-between text-xs">
        <span class="text-muted">实体类型</span>
        <span class="text-text">{{ aiStatus.model.entity_types.length }}</span>
      </div>
    </div>

    <!-- 可用模型数 -->
    <div v-else-if="aiStatus?.available_count" class="text-xs text-muted">
      发现 {{ aiStatus.available_count }} 个模型
    </div>

    <!-- 错误信息 -->
    <div v-if="aiStatus?.error" class="mt-2 p-2 bg-error/10 rounded text-xs text-error">
      {{ aiStatus.error }}
    </div>

    <!-- 识别器列表 -->
    <div v-if="engineInfo?.recognizers" class="mt-3 pt-3 border-t border-border">
      <div class="text-xs text-muted mb-2">已注册识别器</div>
      <div class="flex flex-wrap gap-1">
        <span
          v-for="name in engineInfo.recognizers"
          :key="name"
          class="px-2 py-0.5 bg-hover rounded text-xs text-text"
        >
          {{ formatRecognizerName(name) }}
        </span>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed, onMounted } from 'vue';
import { RefreshCw } from 'lucide-vue-next';
import { useAppStore } from '../stores/useAppStore';

const store = useAppStore();

const aiStatus = computed(() => store.aiEngineStatus);
const engineInfo = computed(() => store.engineInfo);

const statusColor = computed(() => {
  switch (aiStatus.value?.state) {
    case 'ready': return 'bg-success';
    case 'loading': return 'bg-warning animate-pulse';
    case 'error': return 'bg-error';
    default: return 'bg-muted';
  }
});

const statusText = computed(() => {
  switch (aiStatus.value?.state) {
    case 'ready': return '就绪';
    case 'loading': return '加载中...';
    case 'error': return '错误';
    case 'not_loaded': return '未加载';
    case 'not_available': return '不可用';
    default: return '未知';
  }
});

const formatRecognizerName = (name: string) => {
  const map: Record<string, string> = {
    'aho_corasick_engine': '字典',
    'regex_engine': '正则',
    'ner_engine': 'AI',
    'context_enhancer': '上下文',
    'checksum_recognizer': '校验',
  };
  return map[name] || name;
};

const refreshStatus = async () => {
  await Promise.all([
    store.fetchAiStatus(),
    store.fetchEngineInfo(),
  ]);
};

onMounted(() => {
  if (!aiStatus.value) {
    refreshStatus();
  }
});
</script>
