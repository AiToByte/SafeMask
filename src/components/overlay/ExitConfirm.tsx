import { useState, useEffect } from "react";
import { motion, AnimatePresence } from "framer-motion";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { modalBackdrop, modalContent } from "@/lib/animations";

const STORAGE_KEY = "exit-preference";

/** Exit confirmation modal — intercepts window close and asks user to minimize or quit */
export default function ExitConfirm() {
  const [visible, setVisible] = useState(false);
  const [rememberChoice, setRememberChoice] = useState(false);

  useEffect(() => {
    const appWindow = getCurrentWindow();

    const unlisten = appWindow.onCloseRequested(async (event) => {
      const preference = localStorage.getItem(STORAGE_KEY);

      if (preference === "minimize") {
        await appWindow.hide();
        return;
      }

      if (preference === "quit") {
        appWindow.destroy();
        return;
      }

      // No saved preference — show the dialog
      event.preventDefault();
      setVisible(true);
    });

    return () => {
      unlisten.then((fn) => fn());
    };
  }, []);

  const handleExit = async (action: "minimize" | "quit") => {
    const appWindow = getCurrentWindow();

    if (rememberChoice) {
      localStorage.setItem(STORAGE_KEY, action);
    }

    setVisible(false);

    if (action === "minimize") {
      await appWindow.hide();
    } else {
      appWindow.destroy();
    }
  };

  return (
    <AnimatePresence>
      {visible && (
        <motion.div
          key="exit-backdrop"
          variants={modalBackdrop}
          initial="initial"
          animate="animate"
          exit="exit"
          className="fixed inset-0 z-[1000] flex items-center justify-center bg-black/80 backdrop-blur-md"
          onClick={() => setVisible(false)}
        >
          <motion.div
            key="exit-panel"
            variants={modalContent}
            initial="initial"
            animate="animate"
            exit="exit"
            className="p-12 rounded-4xl border border-white/10 w-full max-w-md shadow-2xl text-center space-y-8 bg-[#0f0f14]/95"
            onClick={(e) => e.stopPropagation()}
          >
            {/* Title */}
            <div className="space-y-2">
              <h2 className="text-2xl font-bold text-white">确认退出程序？</h2>
              <p className="text-base text-zinc-400 leading-relaxed">
                建议最小化到系统托盘
                <br />
                程序将在后台持续守护您的隐私数据
              </p>
            </div>

            {/* Buttons */}
            <div className="space-y-3">
              <button
                type="button"
                onClick={() => handleExit("minimize")}
                className="w-full py-5 bg-amber-500 text-black rounded-2xl font-bold text-sm hover:bg-amber-400 active:scale-95 transition-all shadow-lg shadow-amber-500/10"
              >
                最小化到系统托盘
              </button>

              <button
                type="button"
                onClick={() => handleExit("quit")}
                className="w-full py-5 bg-white/5 border border-white/10 text-zinc-400 rounded-2xl font-bold text-sm hover:bg-white/10 hover:text-white transition-all"
              >
                彻底关闭程序
              </button>
            </div>

            {/* Remember choice checkbox */}
            <label className="flex items-center justify-center gap-2 cursor-pointer select-none">
              <input
                type="checkbox"
                checked={rememberChoice}
                onChange={(e) => setRememberChoice(e.target.checked)}
                className="w-4 h-4 rounded border-zinc-600 bg-zinc-800 text-amber-500 focus:ring-amber-500/30 focus:ring-offset-0 cursor-pointer"
              />
              <span className="text-sm text-zinc-500">记住我的选择</span>
            </label>
          </motion.div>
        </motion.div>
      )}
    </AnimatePresence>
  );
}
