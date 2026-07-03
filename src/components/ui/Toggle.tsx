import { cn } from "@/lib/utils";

interface ToggleProps {
  checked: boolean;
  onChange: (checked: boolean) => void;
  disabled?: boolean;
  size?: "default" | "sm";
  className?: string;
}

export function Toggle({
  checked,
  onChange,
  disabled = false,
  size = "default",
  className,
}: ToggleProps) {
  const isSm = size === "sm";

  return (
    <label
        className={cn(
          "relative inline-block cursor-pointer",
          isSm ? "w-11 h-6" : "w-14 h-7",
          disabled && "opacity-50 cursor-not-allowed",
          className,
        )}
    >
      <input
        type="checkbox"
        checked={checked}
        onChange={(e) => onChange(e.target.checked)}
        disabled={disabled}
        className="opacity-0 w-0 h-0 absolute"
      />
      <div
        className={cn(
          "absolute inset-0 rounded-full border border-white/[0.05] transition-colors duration-300",
          checked ? "bg-blue-600/80 border-blue-400/20" : "bg-zinc-800",
        )}
      >
        <div
          className={cn(
            "absolute rounded-full shadow-lg transition-all duration-300",
            isSm
              ? "h-4 w-4 top-1"
              : "h-5 w-5 top-1",
            checked
              ? (isSm ? "left-[24px] bg-white shadow-[0_0_15px_rgba(255,255,255,0.5)]" : "left-[32px] bg-white shadow-[0_0_15px_rgba(255,255,255,0.5)]")
              : "left-1 bg-zinc-400",
          )}
        />
      </div>
    </label>
  );
}
