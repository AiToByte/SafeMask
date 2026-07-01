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
    "bg-zinc-800 text-zinc-500 border-white/5",
  "mode-shadow":
    "bg-blue-500/10 text-blue-400 border-blue-500/20 shadow-[0_0_15px_rgba(59,130,246,0.1)]",
  "mode-sentry":
    "bg-amber-500/10 text-amber-500 border-amber-500/20 shadow-[0_0_15px_rgba(245,158,11,0.1)]",
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
        "inline-flex items-center px-2.5 py-1 rounded text-[8px] font-black uppercase border",
        variantClasses[variant],
        className,
      )}
    >
      {children}
    </span>
  );
}
