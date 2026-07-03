import { cn } from "@/lib/utils";
import type { ReactNode } from "react";
import type { LucideIcon } from "lucide-react";

interface EmptyStateProps {
  icon?: LucideIcon;
  title: string;
  description?: string;
  className?: string;
  children?: ReactNode;
}

export function EmptyState({
  icon: Icon,
  title,
  description,
  className,
  children,
}: EmptyStateProps) {
  return (
    <div
      className={cn(
        "flex flex-col items-center justify-center py-24 text-center",
        className,
      )}
    >
      {Icon && (
        <div className="relative mb-6">
          <div className="absolute inset-0 bg-amber-500/10 blur-3xl rounded-full" />
          <Icon size={48} className="text-zinc-800 relative z-10" />
        </div>
      )}
      <h3 className="text-amber-50/60 font-bold tracking-widest uppercase text-sm">
        {title}
      </h3>
      {description && (
        <p className="text-xs text-zinc-600 mt-2 max-w-xs">
          {description}
        </p>
      )}
      {children}
    </div>
  );
}
