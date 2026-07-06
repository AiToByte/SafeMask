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
      console.log("[ModelDownload] checkStatus");
      set({ status: "CHECKING" });
      try {
        const res = await invoke<string>("check_model_file");
        const isReady = res === "READY" || res === "READY_PORTABLE";
        console.log("[ModelDownload] checkStatus result:", res, "isReady:", isReady);
        set({ status: isReady ? "READY" : "MISSING", errorMessage: null });
        return isReady ? "READY" : "MISSING";
      } catch (e: any) {
        console.error("[ModelDownload] checkStatus error:", e);
        set({ status: "ERROR", errorMessage: e });
        return "ERROR";
      }
    },

    startDownload: async (urls: string[]) => {
      if (get().status === "DOWNLOADING" || get().status === "EXTRACTING") {
        console.log("[ModelDownload] startDownload skipped — already downloading/extracting");
        return;
      }
      const currentIdx = get().currentUrlIndex;
      const url = urls[currentIdx];
      if (!url) {
        console.error("[ModelDownload] no URL at index", currentIdx, "urls:", urls);
        set({ status: "ERROR", errorMessage: "所有下载地址均已失效" });
        return;
      }
      console.log("[ModelDownload] startDownload url[" + currentIdx + "]=" + url);

      cleanup();

      set({
        status: "DOWNLOADING",
        progress: { percentage: 0, downloaded_bytes: 0, total_bytes: 0, speed_mbps: 0 },
        errorMessage: null,
        etaSeconds: -1,
      });

      unlistenProgress = await listen<ProgressData>("model-download-progress", (e) => {
        const remaining = e.payload.total_bytes - e.payload.downloaded_bytes;
        const speed = e.payload.speed_mbps * 1024 * 1024;
        const eta = speed > 0 ? Math.ceil(remaining / speed) : -1;
        set({ progress: e.payload, etaSeconds: eta });
      });

      unlistenStatus = await listen<string>("model-download-status", (e) => {
        const signal = e.payload;
        console.log("[ModelDownload] event:", signal);
        if (signal === "READY") {
          set({ status: "READY", errorMessage: null, currentUrlIndex: 0 });
          cleanup();
        } else if (signal === "EXTRACTING") {
          set({ status: "EXTRACTING" });
        } else if (signal.startsWith("ERROR:")) {
          const rawErr = signal.replace("ERROR:", "").trim();
          const isCancelled = rawErr === "DOWNLOAD_CANCELLED";
          set({
            status: isCancelled ? "MISSING" : "ERROR",
            errorMessage: isCancelled ? null : rawErr,
          });
          cleanup();
        }
      });

      try {
        await invoke("start_model_download", { url });
      } catch (e: any) {
        const errMsg = typeof e === "string" ? e : e?.message ?? String(e);
        console.error("[ModelDownload] invoke error:", errMsg);
        if (errMsg === "DISK_SPACE_LOW") {
          set({ status: "ERROR", errorMessage: "用户本地磁盘空间不足 (至少需要 2.2GB 剩余空间)" });
          cleanup();
          return;
        }
        // 非磁盘错误：尝试下一个 URL
        const nextIdx = currentIdx + 1;
        if (nextIdx < urls.length) {
          console.log("[ModelDownload] retrying with next URL index", nextIdx);
          set({ currentUrlIndex: nextIdx });
          // 重新调用自身以使用下一个 URL
          get().startDownload(urls);
        } else {
          console.error("[ModelDownload] all URLs exhausted");
          set({ status: "ERROR", errorMessage: `所有下载地址均失败: ${errMsg}` });
          cleanup();
        }
      }
    },

    cancelDownload: async () => {
      try {
        await invoke("cancel_model_download");
      } catch (e: any) {
        set({ errorMessage: `中断失败: ${e}` });
      }
    },
  };
});
