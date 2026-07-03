import { useRef, useCallback, useState, useEffect } from "react";
import { motion, AnimatePresence } from "framer-motion";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import { useAppStore } from "@/hooks/useAppStore";
import { MaskAPI } from "@/services/api";
import { cn } from "@/lib/utils";
import { scaleIn } from "@/lib/animations";
import { message, ask } from "@tauri-apps/plugin-dialog";

interface FileProcessorProps {
  className?: string;
}

export default function FileProcessor({ className }: FileProcessorProps) {
  const isProcessing = useAppStore((s) => s.isProcessing);
  const progress = useAppStore((s) => s.progress);
  const currentFileName = useAppStore((s) => s.currentFileName);
  const setProcessing = useAppStore((s) => s.setProcessing);
  const setProgress = useAppStore((s) => s.setProgress);
  const setCurrentFileName = useAppStore((s) => s.setCurrentFileName);

  const [isDragOver, setIsDragOver] = useState(false);
  const unlistenRef = useRef<UnlistenFn | null>(null);

  // ── Drag-drop event listener ──
  useEffect(() => {
    listen<{ paths: string[] }>("tauri://drag-drop", (event) => {
      const path = event.payload.paths[0];
      startProcessing(path);
    }).then((fn) => {
      unlistenRef.current = fn;
    });
    return () => {
      if (unlistenRef.current) unlistenRef.current();
    };
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, []);

  // ── Processing logic ──
  const startProcessing = useCallback(
    async (path: string) => {
      if (!path) return;
      setProcessing(true);
      setCurrentFileName(path.split(/[\\/]/).pop() || "");

      try {
        const result = await MaskAPI.processFile(path);

        const shouldOpen = await ask(
          `${result.message}\n\n处理耗时: ${result.duration}\n引擎吞吐: ${result.throughput}\n\n文件已保存至：\n${result.output_path}\n\n是否立即打开所在文件夹？`,
          {
            title: "🛡️ SafeMask 脱敏成功",
            kind: "info",
            okLabel: "查看文件",
            cancelLabel: "知道了",
          },
        );

        if (shouldOpen) {
          await MaskAPI.openFolder(result.output_path);
        }
      } catch (err) {
        await message(`处理失败: ${err}`, {
          title: "错误",
          kind: "error",
        });
      } finally {
        setTimeout(() => {
          setProcessing(false);
          setProgress(0);
        }, 800);
      }
    },
    [setProcessing, setCurrentFileName, setProgress],
  );

  // ── Click to browse ──
  const handleBrowse = async () => {
    if (isProcessing) return;
    const selected = await MaskAPI.selectFile();
    if (selected && typeof selected === "string") {
      await startProcessing(selected);
    }
  };

  return (
    <motion.div
      onClick={handleBrowse}
      onDragEnter={() => setIsDragOver(true)}
      onDragLeave={() => setIsDragOver(false)}
      onDragOver={(e) => e.preventDefault()}
      onDrop={() => setIsDragOver(false)}
      whileHover={{ scale: 1.002 }}
      className={cn(
        "flex-1 border-2 border-dashed rounded-5xl flex flex-col items-center justify-center transition-all duration-300 cursor-pointer",
        isDragOver
          ? "border-amber-500/50 bg-amber-500/5"
          : isProcessing
            ? "bg-blue-500/5 border-blue-500/50"
            : "border-zinc-800 hover:border-blue-500/50",
        className,
      )}
    >
      <AnimatePresence mode="wait">
        {!isProcessing ? (
          <motion.div
            key="idle"
            variants={scaleIn}
            initial="initial"
            animate="animate"
            exit="exit"
            className="text-center"
          >
            <motion.div
              className="text-6xl mb-6"
              whileHover={{ scale: 1.1, rotate: [0, -5, 5, 0] }}
              transition={{ duration: 0.4 }}
            >
              📂
            </motion.div>
            <h3 className="text-2xl font-bold mb-2 text-zinc-200">
              拖拽文件或点击上传
            </h3>
            <p className="text-zinc-500 text-base">
              支持多 GB 级文件，保持行序 100% 一致
            </p>
          </motion.div>
        ) : (
          <motion.div
            key="processing"
            variants={scaleIn}
            initial="initial"
            animate="animate"
            exit="exit"
            className="w-3/4 space-y-4"
          >
            {/* File name + percentage */}
            <div className="flex justify-between text-base font-bold">
              <span className="text-blue-400 truncate max-w-xs">
                {currentFileName}
              </span>
              <motion.span
                className="font-mono text-lg"
                key={Math.round(progress)}
                initial={{ opacity: 0, y: -4 }}
                animate={{ opacity: 1, y: 0 }}
              >
                {Math.round(progress)}%
              </motion.span>
            </div>

            {/* Progress bar */}
            <div className="w-full bg-zinc-900 h-4 rounded-full overflow-hidden border border-zinc-800 p-[2px]">
              <motion.div
                className="bg-gradient-to-r from-blue-600 to-indigo-500 h-full rounded-full"
                animate={{ width: `${progress}%` }}
                transition={{ type: "spring", stiffness: 100, damping: 20 }}
              >
                {/* Shimmer overlay */}
                <div className="absolute inset-0 bg-gradient-to-r from-transparent via-white/20 to-transparent animate-shimmer" />
              </motion.div>
            </div>

            <p className="text-center text-sm text-zinc-500 animate-pulse">
              正在调用多核 Rust 引擎加速处理...
            </p>
          </motion.div>
        )}
      </AnimatePresence>
    </motion.div>
  );
}
