import { create } from "zustand";
import { invoke } from "@tauri-apps/api/core";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";

export type DownloadStatus = "IDLE" | "CHECKING" | "MISSING" | "DOWNLOADING" | "EXTRACTING" | "READY" | "ERROR";

interface ProgressData {
  percentage: number;
  downloaded_bytes: number;
  total_bytes: number;
  speed_mbps: number;
}

interface ModelDownloaderState {
  status: DownloadStatus;
  progress: ProgressData;
  errorMessage: string | null;
  etaSeconds: number;
  currentUrlIndex: number;
  checkStatus: () => Promise<DownloadStatus>;
  startDownload: (urls: string[]) => Promise<void>;
  cancelDownload: () => Promise<void>;
}

export const useModelDownloader = create<ModelDownloaderState>((set, get) => {
  let unlistenProgress: UnlistenFn | null = null;
  let unlistenStatus: UnlistenFn | null = null;

  const cleanup = () => {
    if (unlistenProgress) { unlistenProgress(); unlistenProgress = null; }
    if (unlistenStatus) { unlistenStatus(); unlistenStatus = null; }
  };

  return {
    status: "IDLE",
    progress: { percentage: 0, downloaded_bytes: 0, total_bytes: 0, speed_mbps: 0 },
    errorMessage: null,
    etaSeconds: -1,
    currentUrlIndex: 0,

    checkStatus: async () => {
      console.log("🔍 [SafeMask AI] 开始检查本地模型完整性...");
      set({ status: "CHECKING" });
      try {
        const res = await invoke<string>("check_model_file");
        const isReady = res === "READY" || res === "READY_PORTABLE";
        console.log(`📡 [SafeMask AI] 模型状态响应: ${res} (就绪: ${isReady})`);
        set({ status: isReady ? "READY" : "MISSING", errorMessage: null });
        return isReady ? "READY" : "MISSING";
      } catch (e: any) {
        console.error("❌ [SafeMask AI] 检查模型失败:", e);
        set({ status: "ERROR", errorMessage: String(e) });
        return "ERROR";
      }
    },

    startDownload: async (urls: string[]) => {
      if (get().status === "DOWNLOADING" || get().status === "EXTRACTING") {
        console.warn("⚠️ [SafeMask AI] 已有下载/解压任务进行中，忽略重复触发");
        return;
      }

      // 🚀 核心看守：若后端因序列化漏传 urls，采用高可用静态 CDN 链接池进行强力兜底
      const staticFallbacks = [
        "https://obs.behource.com:9004/gxzh/2026/07/06/privacy-filter.zip",
        "https://950544b1401caf10f82ba1e82b03f89a.r2.cloudflarestorage.com/safemask-ai-model/privacy-filter/privacy-filter.zip",
        "https://github.com/AiToByte/SafeMask/releases/download/v1.2.4/privacy-filter.zip"
      ];

      const actualUrls = urls && urls.length > 0 ? urls : staticFallbacks;
      const currentIdx = get().currentUrlIndex;
      const url = actualUrls[currentIdx];

      if (!url) {
        console.error("❌ [SafeMask AI] 无可用下载源，地址列表为空");
        set({ status: "ERROR", errorMessage: "所有可用下载地址均已失效" });
        return;
      }

      console.log(`🚀 [SafeMask AI] 准备启动下载线。选定通道 [${currentIdx}]: ${url}`);
      cleanup();

      set({
        status: "DOWNLOADING",
        progress: { percentage: 0, downloaded_bytes: 0, total_bytes: 0, speed_mbps: 0 },
        errorMessage: null,
        etaSeconds: -1,
      });

      // ─── 鉴权代理健康度预检 (1.5秒极致超时判定) ───
      if (url.includes('/download?token=')) {
        const healthUrl = url.substring(0, url.indexOf('/download')) + '/health';
        console.log(`🩺 [SafeMask AI] 正在执行鉴权代理健康预检: ${healthUrl}`);
        try {
          const controller = new AbortController();
          const timer = setTimeout(() => controller.abort(), 1500);
          const resp = await fetch(healthUrl, { signal: controller.signal });
          clearTimeout(timer);
          if (!resp.ok) throw new Error(`HTTP ${resp.status}`);
          console.log("💚 [SafeMask AI] 鉴权代理在线，连接成功");
        } catch (err) {
          console.warn("💔 [SafeMask AI] 鉴权代理预检失败或超时，自动切换至降级直连链路...", err);
          cleanup();
          const nextIdx = currentIdx + 1;
          if (nextIdx < actualUrls.length) {
            set({ currentUrlIndex: nextIdx, status: "IDLE" });
            get().startDownload(actualUrls);
          } else {
            set({ status: "ERROR", errorMessage: "鉴权代理服务器不可达，且无备用下载地址" });
          }
          return;
        }
      }

      // ─── 注册 Rust 主动推送监听 ───
      unlistenProgress = await listen<ProgressData>("model-download-progress", (e) => {
        const remaining = e.payload.total_bytes - e.payload.downloaded_bytes;
        const speed = e.payload.speed_mbps * 1024 * 1024;
        const eta = speed > 0 ? Math.ceil(remaining / speed) : -1;
        set({ progress: e.payload, etaSeconds: eta });
      });

      unlistenStatus = await listen<string>("model-download-status", (e) => {
        const signal = e.payload;
        console.log(`📡 [SafeMask AI] 收到内核管道信号: ${signal}`);
        if (signal === "READY") {
          set({ status: "READY", errorMessage: null, currentUrlIndex: 0 });
          cleanup();
        } else if (signal === "EXTRACTING") {
          set({ status: "EXTRACTING" });
        } else if (signal.startsWith("ERROR:")) {
          const rawErr = signal.replace("ERROR:", "").trim();
          const isCancelled = rawErr === "DOWNLOAD_CANCELLED";
          if (isCancelled) {
            set({ status: "MISSING", errorMessage: null });
            cleanup();
            return;
          }
          // 🚀 时钟偏置/鉴权失败 403 → 自动跳过当前 Worker URL，用直连 OSS
          if (rawErr.includes("403")) {
            cleanup();
            const nextIdx = get().currentUrlIndex + 1;
            if (nextIdx < actualUrls.length) {
              console.log(`🔄 [SafeMask AI] Worker 鉴权拒绝(403)，切换至备用通道 ${nextIdx}`);
              set({ currentUrlIndex: nextIdx, status: "IDLE" });
              get().startDownload(actualUrls);
              return;
            }
          }
          set({ status: "ERROR", errorMessage: rawErr });
          cleanup();
        }
      });

      // ─── 触发 Rust 二进制后台异步下载流 ───
      try {
        await invoke("start_model_download", { url });
      } catch (e: any) {
        const errMsg = typeof e === "string" ? e : e?.message ?? String(e);
        console.error("❌ [SafeMask AI] 调用内核下载任务失败:", errMsg);

        if (errMsg === "DISK_SPACE_LOW") {
          set({ status: "ERROR", errorMessage: "用户本地磁盘空间不足 (至少需要 2.2GB 剩余空间)" });
          cleanup();
          return;
        }

        // 非空间性物理故障：触发多链路自愈降级
        cleanup();
        const nextIdx = currentIdx + 1;
        if (nextIdx < actualUrls.length) {
          console.log(`🔄 [SafeMask AI] 通道 ${currentIdx} 握手失败，正尝试无缝重连通道 ${nextIdx}`);
          set({ currentUrlIndex: nextIdx, status: "IDLE" });
          get().startDownload(actualUrls);
        } else {
          set({ status: "ERROR", errorMessage: `所有下载通道尝试完毕，最后一次报错: ${errMsg}` });
          cleanup();
        }
      }
    },

    cancelDownload: async () => {
      console.log("🛑 [SafeMask AI] 用户执行了中止下载操作");
      try {
        await invoke("cancel_model_download");
      } catch (e: any) {
        set({ errorMessage: `内核中断指令发送失败: ${e}` });
      }
    },
  };
});
