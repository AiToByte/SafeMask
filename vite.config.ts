import { defineConfig } from "vite";
import react from "@vitejs/plugin-react";
import path from "path";

// https://vitejs.dev/config/
export default defineConfig(async () => ({
  plugins: [react()],

  define: {
    __APP_VERSION__: JSON.stringify(process.env.npm_package_version),
  },

  resolve: {
    alias: {
      "@": path.resolve(__dirname, "./src"),
    },
  },

  // Vite options tuned for Tauri development
  clearScreen: false,

  build: {
    target: "esnext",
    rollupOptions: {
      output: {
        manualChunks: {
          "vendor-react": ["react", "react-dom", "react/jsx-runtime"],
          "vendor-lucide": ["lucide-react"],
          "vendor-tauri": [
            "@tauri-apps/api",
            "@tauri-apps/plugin-dialog",
            "@tauri-apps/plugin-notification",
            "@tauri-apps/plugin-global-shortcut",
            "@tauri-apps/plugin-opener",
          ],
          "vendor-state": ["zustand"],
        },
      },
    },
  },

  server: {
    host: "127.0.0.1",
    port: 18924,
    strictPort: true,
    watch: {
      // Tell Vite to ignore watching src-tauri
      ignored: ["**/src-tauri/**"],
    },
  },
}));