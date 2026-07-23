import { cn } from "@/lib/utils";
import type { InputHTMLAttributes, TextareaHTMLAttributes } from "react";

interface InputProps extends InputHTMLAttributes<HTMLInputElement> {
  className?: string;
}

/** Styled text input matching the original .input-wrapper design */
export function Input({ className, ...props }: InputProps) {
  return (
    <div
      className={cn(
        "relative rounded-2xl border transition-all duration-300 shadow-inner",
        "hover:border-white/[0.2]",
        "focus-within:border-amber-500/40",
        "focus-within:shadow-input-glow",
      )}
      style={{
        backgroundColor: "var(--bg-input)",
        borderColor: "var(--border-default)",
      }}
    >
      <input
        className={cn(
          "w-full bg-transparent border-none outline-none p-4 text-sm text-amber-50/90",
          "placeholder:text-zinc-700 font-medium transition-all",
          className,
        )}
        {...props}
      />
    </div>
  );
}

interface TextareaProps
  extends TextareaHTMLAttributes<HTMLTextAreaElement> {
  className?: string;
}

/** Styled textarea matching the original .input-wrapper design */
export function Textarea({ className, ...props }: TextareaProps) {
  return (
    <div
      className={cn(
        "relative rounded-2xl border transition-all duration-300 shadow-inner",
        "hover:border-white/[0.2]",
        "focus-within:border-amber-500/40",
        "focus-within:shadow-input-glow",
      )}
      style={{
        backgroundColor: "var(--bg-input)",
        borderColor: "var(--border-default)",
      }}
    >
      <textarea
        className={cn(
          "w-full bg-transparent border-none outline-none p-4 text-sm text-amber-50/90",
          "placeholder:text-zinc-700 font-medium transition-all resize-none",
          className,
        )}
        {...props}
      />
    </div>
  );
}
