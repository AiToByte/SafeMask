import { cn } from "@/lib/utils";
import type { ReactNode } from "react";

interface GlassPanelProps {
  children: ReactNode;
  className?: string;
}

/** Frosted glass container — base surface for cards and panels */
export function GlassPanel({ children, className }: GlassPanelProps) {
  return (
    <div
      className={cn(
        "bg-[#0d0d0f]/80 border border-white/[0.04] rounded-4xl shadow-2xl",
        className,
      )}
    >
      {children}
    </div>
  );
}
