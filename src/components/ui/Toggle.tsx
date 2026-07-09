import { cn } from "@/lib/utils";

interface ToggleProps {
  checked: boolean;
  onChange: (checked: boolean) => void;
  disabled?: boolean;
  size?: "sm" | "default" | "lg";
  className?: string;
}

export function Toggle({
  checked,
  onChange,
  disabled = false,
  size = "default",
  className,
}: ToggleProps) {

  // 1. 各尺寸轨道（Track）黄金比例几何规格
  const trackClasses = {
    sm: "w-10 h-6",
    default: "w-14 h-8",
    lg: "w-[4.5rem] h-10",
  };

  // 2. 各尺寸内圈滑块（Thumb）物理圆心初始布局
  const circleClasses = {
    sm: "w-4 h-4 top-1 left-1",
    default: "w-6 h-6 top-1 left-1",
    lg: "w-8 h-8 top-1 left-1",
  };

  // 3. 🚀 关键优化：硬件加速平移偏置（Translate）代替 Layout 渲染重排
  const translateClasses = {
    sm: checked ? "translate-x-4" : "translate-x-0",
    default: checked ? "translate-x-6" : "translate-x-0",
    lg: checked ? "translate-x-8" : "translate-x-0",
  };

  // 4. 不同尺寸的光圈弥散发光感 (Ambient Glow)
  const thumbStyle = {
    sm: checked ? "bg-white shadow-[0_0_8px_rgba(255,255,255,0.7)]" : "bg-zinc-400",
    default: checked ? "bg-white shadow-[0_0_15px_rgba(255,255,255,0.8)]" : "bg-zinc-400",
    lg: checked ? "bg-white shadow-[0_0_20px_rgba(255,255,255,0.9)]" : "bg-zinc-400",
  };

  return (
    <label
      className={cn(
        "relative inline-block cursor-pointer select-none rounded-full",
        disabled && "opacity-40 cursor-not-allowed",
        className
      )}
    >
      <input
        type="checkbox"
        checked={checked}
        onChange={(e) => {
          if (!disabled) onChange(e.target.checked);
        }}
        disabled={disabled}
        className="opacity-0 w-0 h-0 absolute pointer-events-none"
      />
      {/* 轨道 Track */}
      <div
        className={cn(
          "rounded-full border border-white/[0.04] transition-colors duration-300 ease-out shadow-inner",
          trackClasses[size],
          checked
            ? "bg-blue-600/80 border-blue-500/20"
            : "bg-zinc-900 border-white/[0.01]"
        )}
      >
        {/* 滑块 Thumb — 🚀 cubic-bezier 曲线赋予绝妙的阻尼微回弹感 */}
        <div
          className={cn(
            "absolute rounded-full transition-transform duration-300 ease-[cubic-bezier(0.25,0.8,0.25,1.25)]",
            circleClasses[size],
            translateClasses[size],
            thumbStyle[size]
          )}
        />
      </div>
    </label>
  );
}
