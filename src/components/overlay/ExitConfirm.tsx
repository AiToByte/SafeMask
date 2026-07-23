import { useState, useEffect } from "react";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { cn } from "@/lib/utils";

const STORAGE_KEY = "exit-preference";

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

  if (!visible) return null;

  return (
    <div
      className="fixed inset-0 z-[1000] flex items-center justify-center backdrop-blur-md modal-backdrop"
      style={{ backgroundColor: "color-mix(in srgb, var(--bg-root) 55%, rgba(0,0,0,0.45))" }}
      onClick={() => setVisible(false)}
    >
      <div
        className="p-12 rounded-4xl w-full max-w-md shadow-2xl text-center space-y-8 modal-panel border"
        style={{
          backgroundColor: "color-mix(in srgb, var(--bg-card) 97%, transparent)",
          borderColor: "var(--border-default)",
          boxShadow: "0 24px 64px rgba(61, 57, 41, 0.18)",
        }}
        onClick={(e) => e.stopPropagation()}
      >
        <div className="space-y-2">
          <h2
            className="text-2xl font-bold tracking-tight"
            style={{
              color: "var(--text-primary)",
              textShadow: "0 0 18px rgba(var(--accent-rgb), 0.22)",
            }}
          >
            确认退出程序？
          </h2>
          <p
            className="text-base leading-relaxed"
            style={{ color: "var(--text-muted)" }}
          >
            建议最小化到系统托盘
            <br />
            程序将在后台持续守护您的隐私数据
          </p>
        </div>

        <div className="space-y-3">
          {/* 主操作：最小化到托盘 */}
          <button
            type="button"
            onClick={() => handleExit("minimize")}
            className={cn(
              "w-full py-5 rounded-2xl font-bold text-sm active:scale-95 transition-all duration-300",
              "border border-transparent",
              "hover:brightness-105 hover:shadow-[0_8px_28px_rgba(var(--accent-rgb),0.28)]",
            )}
            style={{
              backgroundColor: "var(--accent)",
              color: "#FFFCF5",
              boxShadow: "0 8px 24px rgba(var(--accent-rgb), 0.18)",
            }}
          >
            最小化到系统托盘
          </button>

          {/* 次操作：彻底关闭 */}
          <button
            type="button"
            onClick={() => handleExit("quit")}
            className={cn(
              "w-full py-5 rounded-2xl font-bold text-sm transition-all duration-300 active:scale-95 border",
              "hover:border-[color:var(--accent-border)]",
            )}
            style={{
              backgroundColor: "color-mix(in srgb, var(--bg-elevated) 90%, transparent)",
              borderColor: "var(--border-default)",
              color: "var(--text-secondary)",
            }}
            onMouseEnter={(e) => {
              e.currentTarget.style.backgroundColor = "color-mix(in srgb, var(--accent) 10%, var(--bg-elevated))";
              e.currentTarget.style.color = "var(--text-primary)";
              e.currentTarget.style.boxShadow = "0 0 16px rgba(var(--accent-rgb), 0.14)";
            }}
            onMouseLeave={(e) => {
              e.currentTarget.style.backgroundColor = "color-mix(in srgb, var(--bg-elevated) 90%, transparent)";
              e.currentTarget.style.color = "var(--text-secondary)";
              e.currentTarget.style.boxShadow = "none";
            }}
          >
            彻底关闭程序
          </button>
        </div>

        <label className="flex items-center justify-center gap-2.5 cursor-pointer select-none group">
          <input
            type="checkbox"
            checked={rememberChoice}
            onChange={(e) => setRememberChoice(e.target.checked)}
            className="w-4 h-4 rounded cursor-pointer accent-[var(--accent)]"
            style={{
              borderColor: "var(--border-strong)",
              backgroundColor: "var(--bg-input)",
            }}
          />
          <span
            className="text-sm transition-colors duration-300 group-hover:opacity-100"
            style={{ color: "var(--text-muted)" }}
          >
            记住我的选择
          </span>
        </label>
      </div>
    </div>
  );
}
