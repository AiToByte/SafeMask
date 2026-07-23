import { cn } from "@/lib/utils";
import type { ReactNode } from "react";

type BadgeVariant = "custom" | "system" | "mode-shadow" | "mode-sentry" | "default";

interface BadgeProps {
  children: ReactNode;
  variant?: BadgeVariant;
  className?: string;
}

const variantClasses: Record<BadgeVariant, string> = {
  custom:
    "bg-blue-500/10 text-blue-400 border-blue-500/20",
  system:
    "bg-zinc-800 text-zinc-500 border-white/5 bg-[color:var(--bg-elevated)]",
  "mode-shadow":
    "bg-blue-500/10 text-blue-400 border-blue-500/20 shadow-blue-glow",
  "mode-sentry":
    "bg-amber-500/10 text-amber-500 border-amber-500/20 shadow-amber-glow",
  default:
    "bg-white/5 text-zinc-400 border-white/10",
};

export function Badge({
  children,
  variant = "default",
  className,
}: BadgeProps) {
  return (
    <span
      className={cn(
        "inline-flex items-center px-3 py-1.5 rounded text-[10px] font-black uppercase border",
        variantClasses[variant],
        className,
      )}
    >
      {children}
    </span>
  );
}
