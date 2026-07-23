import { cn } from "@/lib/utils";
import { Toggle } from "./Toggle";
import type { ComponentType } from "react";

interface SettingToggleProps {
  title: string;
  description: string;
  checked: boolean;
  onChange: (checked: boolean) => void;
  icon?: ComponentType<{ size?: number; className?: string }>;
  iconColor?: keyof typeof iconColorMap;
  disabled?: boolean;
  size?: "default" | "sm";
}

const iconColorMap = {
  blue: { bg: "bg-blue-500/10 border-blue-500/20", icon: "text-blue-400/80" },
  amber: { bg: "bg-amber-500/10 border-amber-500/20", icon: "text-amber-400/80" },
  indigo: { bg: "bg-indigo-500/10 border-indigo-500/20", icon: "text-indigo-400/80" },
  emerald: { bg: "bg-emerald-500/10 border-emerald-500/20", icon: "text-emerald-400/80" },
  purple: { bg: "bg-purple-500/10 border-purple-500/20", icon: "text-purple-400/80" },
};

export function SettingToggle({
  title,
  description,
  checked,
  onChange,
  icon: Icon,
  iconColor = "blue",
  disabled = false,
  size = "default",
}: SettingToggleProps) {
  const colors = iconColorMap[iconColor];

  if (Icon) {
    return (
      <div
        className="flex justify-between items-center py-4 px-5 rounded-2xl border border-white/[0.02] transition-colors"
        style={{ backgroundColor: "color-mix(in srgb, var(--bg-input) 90%, transparent)" }}
      >
        <div className="flex items-center gap-4">
          <div className={cn("w-9 h-9 rounded-xl border flex items-center justify-center", colors.bg)}>
            <Icon size={16} className={colors.icon} />
          </div>
          <div>
            <div className="text-sm font-bold text-zinc-300">{title}</div>
            <div className="text-[10px] text-zinc-600 font-bold uppercase tracking-widest mt-0.5">{description}</div>
          </div>
        </div>
        <Toggle checked={checked} onChange={onChange} disabled={disabled} size={size} />
      </div>
    );
  }

  return (
    <div
      className="flex justify-between items-center p-5 rounded-2xl border border-white/[0.02]"
      style={{ backgroundColor: "color-mix(in srgb, var(--bg-input) 90%, transparent)" }}
    >
      <div>
        <div className="text-base font-bold text-amber-50/80">{title}</div>
        <div className="text-xs text-zinc-600 font-bold uppercase tracking-widest mt-1">{description}</div>
      </div>
      <Toggle checked={checked} onChange={onChange} disabled={disabled} size={size} />
    </div>
  );
}
