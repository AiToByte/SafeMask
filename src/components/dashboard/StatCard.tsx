import { motion } from "framer-motion";
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
  const Component = clickable ? motion.button : motion.div;

  return (
    <Component
      onClick={onClick}
      whileHover={clickable ? { y: -2 } : undefined}
      className={cn(
        "relative group px-7 py-6 rounded-[1.25rem] border transition-all duration-700 overflow-hidden h-36 text-left",
        typeStyles[type],
        clickable &&
          "cursor-pointer hover:border-white/20 hover:bg-white/[0.03]",
      )}
    >
      {/* Ambient glow */}
      <div
        className={cn(
          "absolute -right-8 -bottom-8 w-32 h-32 blur-3xl opacity-20 pointer-events-none",
          glowColors[type],
        )}
      />

      <div className="relative z-10 flex flex-col justify-between h-full">
        {/* Title row */}
        <div className="flex justify-between items-start">
          <p className="text-[10px] font-bold text-zinc-500 group-hover:text-amber-100/60 transition-colors tracking-[0.25em] uppercase">
            {title}
          </p>
          {clickable && (
            <motion.div
              className="opacity-20 group-hover:opacity-100 transition-all"
              whileHover={{ x: 2, y: -2 }}
            >
              <ArrowUpRight size={14} className="text-zinc-400" />
            </motion.div>
          )}
        </div>

        {/* Value row */}
        <div className="flex items-baseline gap-3">
          <motion.p
            className="text-4xl font-mono font-medium tracking-tighter tabular-nums leading-none"
            style={{ color: undefined }}
            key={String(value)}
            initial={{ opacity: 0, y: 8 }}
            animate={{ opacity: 1, y: 0 }}
            transition={{ type: "spring", stiffness: 300, damping: 30 }}
          >
            <span className={color}>{value}</span>
          </motion.p>
          {unit && (
            <span className="text-[9px] font-black text-zinc-600 uppercase tracking-widest mb-1">
              {unit}
            </span>
          )}
        </div>
      </div>

      {/* Left glowing indicator bar */}
      <div
        className={cn(
          "absolute left-0 top-0 bottom-0 w-[1.5px] opacity-40 group-hover:opacity-100 transition-opacity",
          barColors[type],
        )}
      />
    </Component>
  );
}
