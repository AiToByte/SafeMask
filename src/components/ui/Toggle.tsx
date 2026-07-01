import { cn } from "@/lib/utils";
import { motion } from "framer-motion";
import { springSnappy } from "@/lib/animations";

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
        isSm ? "w-9 h-5" : "w-12 h-6",
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
      <motion.div
        className={cn(
          "absolute inset-0 rounded-full border border-white/[0.05] transition-colors duration-300",
          checked ? "bg-blue-600/80 border-blue-400/20" : "bg-zinc-800",
        )}
        animate={checked ? "checked" : "unchecked"}
      >
        <motion.div
          className={cn(
            "absolute rounded-full shadow-lg",
            isSm
              ? "h-3 w-3 left-1 bottom-1"
              : "h-4 w-4 left-1 bottom-1",
          )}
          variants={{
            unchecked: { x: 0, backgroundColor: "#71717a" },
            checked: {
              x: isSm ? 16 : 24,
              backgroundColor: "#ffffff",
              boxShadow: "0 0 15px rgba(255,255,255,0.5)",
            },
          }}
          transition={springSnappy}
        />
      </motion.div>
    </label>
  );
}
