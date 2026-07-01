import { cn } from "@/lib/utils";
import { type HTMLMotionProps, motion } from "framer-motion";
import type { ReactNode } from "react";

interface GlassPanelProps extends HTMLMotionProps<"div"> {
  children: ReactNode;
  className?: string;
}

/** Frosted glass container — base surface for cards and panels */
export function GlassPanel({ children, className, ...props }: GlassPanelProps) {
  return (
    <motion.div
      className={cn(
        "bg-[#0d0d0f]/80 border border-white/[0.04] rounded-[2.5rem] shadow-2xl",
        className,
      )}
      {...props}
    >
      {children}
    </motion.div>
  );
}
