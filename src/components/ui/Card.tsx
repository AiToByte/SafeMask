import { cn } from "@/lib/utils";
import type { ReactNode } from "react";

interface CardProps {
  children: ReactNode;
  className?: string;
}

/**
 * Config-card surface — used in Settings page section wrappers.
 * Mirrors the original .config-card style.
 */
export function Card({ children, className }: CardProps) {
  return (
    <section
      className={cn(
        "border border-white/[0.04] rounded-4xl p-10 shadow-2xl",
        className,
      )}
      style={{
        backgroundColor: "color-mix(in srgb, var(--bg-card) 96%, transparent)",
        borderColor: "var(--border-subtle)",
      }}
    >
      {children}
    </section>
  );
}

/** Card header with icon + title — mirrors .card-header */
export function CardHeader({
  children,
  className,
}: {
  children: ReactNode;
  className?: string;
}) {
  return (
    <div
      className={cn(
        "flex items-center gap-3 text-xs font-black text-amber-50/50 uppercase tracking-[0.3em]",
        className,
      )}
    >
      {children}
    </div>
  );
}
