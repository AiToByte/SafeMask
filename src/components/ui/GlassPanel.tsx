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
        "border rounded-4xl shadow-2xl",
        className,
      )}
      style={{
        backgroundColor: "color-mix(in srgb, var(--bg-card) 96%, transparent)",
        borderColor: "var(--border-subtle)",
      }}
    >
      {children}
    </div>
  );
}
