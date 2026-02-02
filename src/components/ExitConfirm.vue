<!-- src/components/ExitConfirm.vue -->
<script setup lang="ts">
import { ref, onMounted, onUnmounted } from 'vue'
import { getCurrentWindow } from '@tauri-apps/api/window'
import { listen } from '@tauri-apps/api/event'
import { isPermissionGranted, requestPermission, sendNotification } from '@tauri-apps/plugin-notification'
import { emit } from '@tauri-apps/api/event' // 如果需要通知其他组件

const appWindow = getCurrentWindow()

// 是否显示对话框
const showConfirm = ref(false)
// 是否记住本次选择
const rememberChoice = ref(false)
// 当前选中的行为：'minimize' | 'quit' | null
const selectedAction = ref<'minimize' | 'quit' | null>(null)

// 监听后端发来的关闭请求
let unlistenClose: (() => void) | null = null

onMounted(async () => {
  // 读取用户之前的选择（如果有）
  const saved = localStorage.getItem('exit-preference')
  if (saved === 'minimize' || saved === 'quit') {
    selectedAction.value = saved as 'minimize' | 'quit'
  }

  unlistenClose = await listen<string>('request-close', () => {
    // 如果用户已经记住选择，直接执行
    if (selectedAction.value) {
      handleExit(selectedAction.value)
    } else {
      showConfirm.value = true
    }
  })
})

onUnmounted(() => {
  if (unlistenClose) unlistenClose()
})

/**
 * 执行退出/最小化逻辑
 * @param action 'minimize' 或 'quit'
 */
const handleExit = async (action: 'minimize' | 'quit') => {
  // 是否需要记住选择
  if (rememberChoice.value) {
    localStorage.setItem('exit-preference', action)
  }

  if (action === 'minimize') {
    // 最小化到托盘
    await appWindow.hide()

    // 可选：发送系统通知（提升体验）
    try {
      let permission = await isPermissionGranted()
      if (!permission) {
        permission = (await requestPermission()) === 'granted'
      }
      if (permission) {
        await sendNotification({
          title: 'SafeMask',
          body: '程序已最小化到系统托盘，继续为您保护剪贴板隐私',
          icon: 'icons/128x128.png' // 可选：使用应用图标
        })
      }
    } catch (err) {
      console.warn('发送托盘通知失败:', err)
    }
  } else {
    // 彻底退出
    await appWindow.destroy() // 或 close()，destroy 更彻底
  }

  // 关闭对话框
  showConfirm.value = false
}
</script>

<template>
  <div
    v-if="showConfirm"
    class="fixed inset-0 z-[9999] flex items-center justify-center bg-black/60 backdrop-blur-sm"
  >
    <div
      class="
        glass-card
        w-full max-w-md mx-4 p-8 rounded-3xl
        border border-zinc-700/50
        shadow-2xl shadow-black/60
        transform transition-all duration-300 scale-100
      "
    >
      <h2 class="text-2xl font-bold text-white mb-3">退出 SafeMask？</h2>
      
      <p class="text-zinc-400 mb-8 leading-relaxed">
        程序将继续在后台保护您的剪贴板隐私，您可以随时从系统托盘重新打开。
      </p>

      <div class="space-y-4">
        <!-- 最小化选项 -->
        <button
          @click="handleExit('minimize')"
          class="
            w-full py-4 px-6 rounded-2xl font-medium text-lg
            bg-gradient-to-r from-blue-600 to-indigo-600
            hover:from-blue-500 hover:to-indigo-500
            transition-all duration-300 transform hover:scale-[1.02]
            focus:outline-none focus:ring-2 focus:ring-blue-500/40
          "
        >
          最小化到系统托盘（推荐）
        </button>

        <!-- 彻底退出 -->
        <button
          @click="handleExit('quit')"
          class="
            w-full py-4 px-6 rounded-2xl font-medium text-lg
            bg-zinc-800 hover:bg-zinc-700
            border border-zinc-600 hover:border-zinc-500
            transition-all duration-300
            focus:outline-none focus:ring-2 focus:ring-zinc-500/40
          "
        >
          彻底退出程序
        </button>

        <!-- 记住选择 -->
        <label class="flex items-center mt-6 cursor-pointer select-none">
          <input
            v-model="rememberChoice"
            type="checkbox"
            class="
              w-5 h-5 rounded border-zinc-600 bg-zinc-800
              text-blue-500 focus:ring-blue-500/30
            "
          />
          <span class="ml-3 text-sm text-zinc-400">
            记住我的选择，下次不再询问
          </span>
        </label>
      </div>
    </div>
  </div>
</template>

<style scoped>
.glass-card {
  background: rgba(24, 24, 27, 0.85);
  backdrop-filter: blur(16px);
  -webkit-backdrop-filter: blur(16px);
}
</style>