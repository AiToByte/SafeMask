import { useEffect } from "react";
import { useModelDownloader } from "@/hooks/useModelDownloader";
import { useAudioFeedback } from "@/hooks/useAudioFeedback";
import { useAppStore } from "@/hooks/useAppStore";
import { Download, X, CheckCircle2, AlertTriangle, Loader2, RefreshCw } from "lucide-react";
import { cn } from "@/lib/utils";

export default function ModelDownloadCard() {
  const { status, progress, errorMessage, etaSeconds, checkStatus, startDownload, cancelDownload } = useModelDownloader();
  const settings = useAppStore((s) => s.settings);
  const { play } = useAudioFeedback(settings.enable_audio_feedback);

  useEffect(() => {
    checkStatus();
  }, []);

  useEffect(() => {
    if (status === "READY") {
      play("ASCEND");
    }
  }, [status]);

  const formatBytes = (bytes: number) => {
    if (bytes === 0) return "0 B";
    const sizes = ["B", "KB", "MB", "GB"];
    const i = Math.floor(Math.log(bytes) / Math.log(1024));
    return parseFloat((bytes / Math.pow(1024, i)).toFixed(1)) + " " + sizes[i];
  };

  return (
    <div className="bg-[#141210]/40 border border-white/[0.03] rounded-3xl p-6 relative overflow-hidden transition-all duration-500">
      <div className="flex items-start justify-between">
        <div className="flex items-center gap-3">
          <div className="w-10 h-10 rounded-xl bg-purple-500/10 border border-purple-500/20 flex items-center justify-center shrink-0">
            <Download size={16} className="text-purple-400" />
          </div>
          <div>
            <h4 className="text-sm font-bold text-zinc-300">本地 AI NER 专属模型</h4>
            <p className="text-[10px] text-zinc-600 font-bold uppercase tracking-wider mt-0.5">
              量化版: model_q4 · 离线实体提取
            </p>
          </div>
        </div>

        <div className="flex items-center gap-2">
          {status === "CHECKING" && <Loader2 size={12} className="text-zinc-500 animate-spin" />}
          <span
            className={cn(
              "text-[10px] border px-2.5 py-1 rounded-lg font-black uppercase tracking-wider transition-all",
              status === "READY" && "bg-emerald-500/10 text-emerald-400 border-emerald-500/20 shadow-[0_0_10px_rgba(16,185,129,0.15)]",
              status === "DOWNLOADING" && "bg-amber-500/10 text-amber-400 border-amber-500/20 animate-pulse",
              status === "EXTRACTING" && "bg-blue-500/10 text-blue-400 border-blue-500/20 animate-pulse",
              status === "MISSING" && "bg-red-500/10 text-red-400 border-red-500/20",
              status === "ERROR" && "bg-red-500/15 text-red-300 border-red-500/30"
            )}
          >
            {status === "CHECKING" && "检测中"}
            {status === "READY" && "已装载"}
            {status === "DOWNLOADING" && "正在下载"}
            {status === "EXTRACTING" && "解压校验中"}
            {status === "MISSING" && "文件缺失"}
            {status === "ERROR" && "下载失败"}
          </span>
        </div>
      </div>

      <div className="mt-6">
        {status === "READY" && (
          <div className="flex items-start gap-3 p-4 rounded-2xl bg-emerald-500/[0.01] border border-emerald-500/10">
            <CheckCircle2 size={16} className="text-emerald-400 shrink-0 mt-0.5" />
            <p className="text-xs text-zinc-500 leading-relaxed">
              模型已100%成功装载至本地隔离沙盒。AI 实体识别已启用，所有地址、人名及组织名称分析均脱网处理。
            </p>
          </div>
        )}

        {status === "MISSING" && (
          <div className="space-y-4">
            <div className="flex items-start gap-3 p-4 rounded-2xl bg-amber-500/[0.01] border border-amber-500/10">
              <AlertTriangle size={16} className="text-amber-400 shrink-0 mt-0.5" />
              <p className="text-xs text-zinc-500 leading-relaxed">
                本地 AI 引擎依赖项不完整。要解锁 AI-NER 敏感信息提取功能，请连接网络进行一键自动初始化（约 550MB 压缩包）。
              </p>
            </div>
            <button
              type="button"
              disabled={status !== "MISSING"}
              onClick={() => { startDownload(settings.model_download_urls); play("CLICK"); }}
              className="w-full py-4 bg-amber-500/10 border border-amber-500/20 hover:bg-amber-500 hover:text-black text-amber-500 font-bold rounded-2xl text-xs uppercase tracking-widest transition-all disabled:opacity-30 disabled:cursor-not-allowed disabled:hover:bg-amber-500/10 disabled:hover:text-amber-500"
            >
              一键下载并配置 (550 MB)
            </button>
          </div>
        )}

        {status === "DOWNLOADING" && (
          <div className="space-y-4">
            <div className="space-y-2">
              <div className="flex justify-between text-xs font-mono">
                <span className="text-zinc-600 font-bold">
                  {formatBytes(progress.downloaded_bytes)} / {formatBytes(progress.total_bytes)}
                </span>
                <span className="text-amber-400 font-extrabold">{progress.percentage.toFixed(1)}%</span>
              </div>
              <div className="relative w-full h-2 bg-black/60 rounded-full border border-white/[0.03] shadow-inner overflow-hidden">
                <div
                  className="h-full bg-gradient-to-r from-amber-500 to-amber-400 rounded-full shadow-[0_0_12px_#f59e0b] transition-all duration-150"
                  style={{ width: `${progress.percentage}%` }}
                />
              </div>
            </div>
            <div className="flex justify-between items-center text-xs font-mono text-zinc-500">
              <div className="flex gap-4">
                <span>速度: <strong className="text-zinc-300 font-bold">{progress.speed_mbps.toFixed(1)} MB/s</strong></span>
                <span>预计剩余: <strong className="text-zinc-300 font-bold">{etaSeconds < 0 ? "估算中..." : `${etaSeconds}s`}</strong></span>
              </div>
              <button
                type="button"
                onClick={() => { cancelDownload(); play("CLICK"); }}
                className="flex items-center gap-1.5 px-3 py-1.5 rounded-lg border border-red-500/20 bg-red-500/5 text-red-400 hover:bg-red-500/10 transition-all text-[10px]"
              >
                <X size={10} /> 中断
              </button>
            </div>
          </div>
        )}

        {status === "EXTRACTING" && (
          <div className="flex flex-col items-center justify-center py-6 gap-3 animate-pulse">
            <Loader2 size={24} className="text-blue-400 animate-spin" />
            <div className="text-center">
              <p className="text-xs font-bold text-zinc-300">正在执行原子解压与完整性校验...</p>
              <p className="text-[10px] text-zinc-600 mt-1">这大约需要 5 - 15 秒，请不要关闭程序</p>
            </div>
          </div>
        )}

        {status === "ERROR" && (
          <div className="space-y-4">
            <div className="flex items-start gap-3 p-4 rounded-2xl bg-red-500/[0.02] border border-red-500/15">
              <AlertTriangle size={16} className="text-red-400 shrink-0 mt-0.5" />
              <div>
                <p className="text-xs text-red-400 font-bold">加载任务中断</p>
                <p className="text-[11px] text-zinc-600 mt-1 leading-relaxed">
                  原因: {errorMessage || "请求超时，请检查网络或代理配置。"}
                </p>
              </div>
            </div>
            <button
              type="button"
              disabled={status !== "ERROR"}
              onClick={() => { startDownload(settings.model_download_urls); play("CLICK"); }}
              className="w-full py-4 bg-zinc-900 border border-white/5 text-zinc-400 hover:text-amber-200 hover:border-amber-500/20 font-bold rounded-2xl text-xs uppercase tracking-widest transition-all disabled:opacity-30 disabled:cursor-not-allowed disabled:hover:text-zinc-400 disabled:hover:border-white/5"
            >
              <RefreshCw size={12} className="inline mr-2" /> 重新尝试连接
            </button>
          </div>
        )}
      </div>
    </div>
  );
}
