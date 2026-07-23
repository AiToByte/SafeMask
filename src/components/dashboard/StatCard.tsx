import { ArrowUpRight } from "lucide-react";
import { cn } from "@/lib/utils";

interface StatCardProps {
  title: string;
  value: string | number;
  color?: string;
  unit?: string;
  clickable?: boolean;
  type?: "amber" | "blue" | "emerald";
  onClick?: () => void;
}

const typeStyles: Record<string, string> = {
  amber:
    "border-amber-500/20 bg-amber-500/[0.015] shadow-amber-900/10",
  blue:
    "border-blue-500/20 bg-blue-500/[0.015] shadow-blue-900/10",
  emerald:
    "border-emerald-500/20 bg-emerald-500/[0.015] shadow-emerald-900/10",
};

const glowColors: Record<string, string> = {
  amber: "bg-amber-500",
  blue: "bg-blue-500",
  emerald: "bg-emerald-500",
};

const barColors: Record<string, string> = {
  amber: "bg-amber-400",
  blue: "bg-blue-400",
  emerald: "bg-emerald-400",
};

export default function StatCard({
  title,
  value,
  color = "text-zinc-100",
  unit,
  clickable = false,
  type = "blue",
  onClick,
}: StatCardProps) {
  const Component = clickable ? "button" : "div";

  return (
    <Component
      {...(onClick ? { onClick } : {})}
      className={cn(
        "relative group px-8 py-7 rounded-3xl border transition-all duration-700 overflow-hidden h-40 text-left",
        typeStyles[type],
        clickable &&
          "cursor-pointer hover:border-white/30 hover:bg-white/[0.05] hover:-translate-y-1",
        clickable && "active:scale-[0.98]",
      )}
    >
      <div
        className={cn(
          "absolute -right-8 -bottom-8 w-32 h-32 blur-3xl opacity-20 pointer-events-none",
          glowColors[type],
        )}
      />

      <div className="relative z-10 flex flex-col justify-between h-full">
        <div className="flex justify-between items-start">
          <p
            className={cn(
              "text-xs font-bold tracking-[0.25em] uppercase transition-all duration-500",
              "text-zinc-500 group-hover:text-[color:var(--text-primary)]",
              "group-hover:[text-shadow:0_0_12px_rgba(var(--accent-rgb),0.35),0_1px_0_rgba(255,255,255,0.35)]",
            )}
          >
            {title}
          </p>
          {clickable && (
            <div className="opacity-20 group-hover:opacity-100 transition-all group-hover:translate-x-0.5 group-hover:-translate-y-0.5">
              <ArrowUpRight
                size={16}
                className="text-zinc-400 group-hover:text-[color:var(--accent)] transition-colors duration-500"
              />
            </div>
          )}
        </div>

        <div className="flex items-baseline gap-3">
          <p
            className="text-4xl font-mono font-bold tracking-tighter tabular-nums leading-none transition-all duration-500 group-hover:[text-shadow:0_0_18px_rgba(var(--accent-rgb),0.28)]"
            key={String(value)}
          >
            <span className={color}>{value}</span>
          </p>
          {unit && (
            <span className="text-[11px] font-black text-zinc-600 uppercase tracking-widest mb-1 transition-colors duration-500 group-hover:text-[color:var(--text-muted)]">
              {unit}
            </span>
          )}
        </div>
      </div>

      <div
        className={cn(
          "absolute left-0 top-0 bottom-0 w-[1.5px] opacity-40 group-hover:opacity-100 transition-opacity",
          barColors[type],
        )}
      />
    </Component>
  );
}
