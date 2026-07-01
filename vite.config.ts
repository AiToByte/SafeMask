import { defineConfig } from "vite";
import react from "@vitejs/plugin-react";
import path from "path";

// https://vitejs.dev/config/
export default defineConfig(async () => ({
  plugins: [react()],

  resolve: {
    alias: {
      "@": path.resolve(__dirname, "./src"),
    },
  },

  // Vite options tuned for Tauri development
  clearScreen: false,
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