import { cn } from "@/lib/utils";
import type { ButtonHTMLAttributes, ReactNode } from "react";

type ButtonVariant = "primary" | "secondary" | "danger" | "ghost";

interface ButtonProps extends ButtonHTMLAttributes<HTMLButtonElement> {
  children: ReactNode;
  variant?: ButtonVariant;
  className?: string;
}

const variantClasses: Record<ButtonVariant, string> = {
  primary:
    "bg-amber-500/10 border border-amber-500/30 text-amber-500 hover:bg-amber-500 hover:text-black hover:shadow-[0_0_40px_rgba(245,158,11,0.25)]",
  secondary:
    "bg-zinc-900 border border-white/5 text-zinc-500 hover:text-amber-200 hover:border-amber-500/20",
  danger:
    "text-zinc-700 hover:text-red-400 hover:bg-red-500/5 border border-white/5",
  ghost:
    "text-zinc-600 hover:text-zinc-300 border border-transparent hover:bg-white/5",
};

export function Button({
  children,
  variant = "primary",
  className,
  ...props
}: ButtonProps) {
  return (
    <button
      className={cn(
        "flex items-center justify-center gap-2 rounded-2xl font-black uppercase tracking-widest text-[11px] transition-all duration-300 active:scale-[0.97]",
        variant === "primary" || variant === "secondary"
          ? "py-4 px-6 w-full"
          : "py-2.5 px-5",
        variantClasses[variant],
        className,
      )}
      {...props}
    >
      {children}
    </button>
  );
}
