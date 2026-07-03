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
        "relative rounded-2xl bg-[#08080a] border border-white/[0.12] transition-all duration-300 shadow-inner",
        "hover:border-white/[0.2]",
        "focus-within:border-amber-500/40 focus-within:bg-[#0a0a0c]",
        "focus-within:shadow-input-glow",
      )}
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
        "relative rounded-2xl bg-[#08080a] border border-white/[0.12] transition-all duration-300 shadow-inner",
        "hover:border-white/[0.2]",
        "focus-within:border-amber-500/40 focus-within:bg-[#0a0a0c]",
        "focus-within:shadow-input-glow",
      )}
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
