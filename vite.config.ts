import { defineConfig } from "vite";
import vue from "@vitejs/plugin-vue";

// https://vitejs.dev/config/
export default defineConfig(async () => ({
  plugins: [vue()],

  // Vite 选项定制，优化 Tauri 开发体验
  clearScreen: false,
  server: {
    host: "127.0.0.1", // 强制使用 IPv4 地址
    port: 5173,
    strictPort: true,
    watch: {
      // 告诉 Vite 忽略监听 src-tauri 文件夹，避免循环触发重绘
      ignored: ["**/src-tauri/**"],
    },
  },
}));