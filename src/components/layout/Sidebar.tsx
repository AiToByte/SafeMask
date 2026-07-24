import {
  Home,
  ClipboardCopy,
  Library,
  Settings,
  ShieldCheck,
} from "lucide-react";
import { useAppStore, type ActiveTab } from "@/hooks/useAppStore";
import { cn } from "@/lib/utils";

const menuItems: { id: ActiveTab; icon: typeof Home; label: string }[] = [
  { id: "dashboard", icon: Home, label: "仪表盘" },
  { id: "history", icon: ClipboardCopy, label: "记录对比" },
  { id: "rules", icon: Library, label: "规则管理" },
];

function NavButton({
  id,
  icon: Icon,
  label,
  active,
  onClick,
  className,
}: {
  id: string;
  icon: typeof Home;
  label: string;
  active: boolean;
  onClick: () => void;
  className?: string;
}) {
  return (
    <button
      key={id}
      type="button"
      onClick={onClick}
      title={label}
      aria-label={label}
      aria-current={active ? "page" : undefined}
      className={cn(
        "relative group w-12 h-12 flex items-center justify-center",
        "rounded-2xl transition-all duration-300 ease-[cubic-bezier(0.22,1,0.36,1)]",
        "focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-[color:var(--ring-color)] focus-visible:ring-offset-2 focus-visible:ring-offset-[color:var(--bg-sidebar)]",
        active
          ? "text-[color:var(--accent)] scale-[1.02]"
          : "text-[color:var(--text-muted)] hover:text-[color:var(--text-primary)] hover:scale-[1.04] active:scale-[0.97]",
        className,
      )}
    >
      {/* Rounded-square halo (matches rules warehouse icon shell) */}
      <span
        aria-hidden
        className={cn(
          "absolute inset-0 rounded-2xl pointer-events-none transition-all duration-350 ease-[cubic-bezier(0.22,1,0.36,1)] border",
          active ? "opacity-100 scale-100" : "opacity-0 scale-[0.94] group-hover:opacity-100 group-hover:scale-100",
        )}
        style={{
          background: active
            ? "linear-gradient(145deg, color-mix(in srgb, var(--accent) 18%, var(--bg-elevated)), var(--bg-elevated))"
            : "color-mix(in srgb, var(--text-primary) 6%, transparent)",
          borderColor: active ? "var(--border-default)" : "var(--border-subtle)",
          boxShadow: active
            ? "0 1px 0 rgba(255,255,255,0.06) inset, 0 8px 20px -12px rgba(var(--accent-rgb),0.45)"
            : "inset 0 1px 0 rgba(255,255,255,0.03)",
        }}
      />

      <Icon
        size={22}
        strokeWidth={active ? 2.3 : 1.9}
        className={cn(
          "relative z-10 transition-all duration-300 ease-[cubic-bezier(0.22,1,0.36,1)]",
          active
            ? "text-[color:var(--accent)] drop-shadow-[0_0_10px_rgba(var(--accent-rgb),0.4)]"
            : "group-hover:-translate-y-px",
        )}
      />

      {/* Tooltip */}
      <span
        className={cn(
          "absolute left-full ml-3.5 px-3 py-1.5 rounded-xl text-[12px] font-semibold tracking-wide whitespace-nowrap z-50",
          "opacity-0 translate-x-1.5 pointer-events-none",
          "group-hover:opacity-100 group-hover:translate-x-0",
          "transition-all duration-300 ease-[cubic-bezier(0.22,1,0.36,1)]",
          "border backdrop-blur-md",
        )}
        style={{
          backgroundColor: "color-mix(in srgb, var(--bg-elevated) 92%, transparent)",
          color: "var(--text-primary)",
          borderColor: "var(--border-default)",
          boxShadow: "0 12px 32px -14px rgba(0,0,0,0.4)",
        }}
      >
        {label}
      </span>
    </button>
  );
}

export default function Sidebar() {
  const activeTab = useAppStore((s) => s.activeTab);
  const setActiveTab = useAppStore((s) => s.setActiveTab);

  return (
    <nav
      className="w-24 flex flex-col items-center py-7 border-r z-50"
      style={{
        backgroundColor: "var(--bg-sidebar)",
        borderColor: "var(--border-subtle)",
      }}
    >
      {/* Brand mark — same rounded-square language as rules warehouse logo */}
      <div
        className="w-12 h-12 rounded-2xl flex items-center justify-center mb-10 cursor-default transition-transform duration-300 ease-[cubic-bezier(0.22,1,0.36,1)] hover:scale-[1.04] active:scale-95 border"
        style={{
          background:
            "linear-gradient(145deg, color-mix(in srgb, var(--logo-gradient-from) 92%, white), var(--logo-gradient-to))",
          borderColor: "color-mix(in srgb, var(--logo-gradient-from) 35%, transparent)",
          boxShadow:
            "0 1px 0 rgba(255,255,255,0.2) inset, 0 10px 24px -12px var(--logo-glow)",
        }}
      >
        <ShieldCheck className="text-white w-6 h-6 drop-shadow-sm" strokeWidth={2.25} />
      </div>

      <div className="flex flex-col gap-4 flex-1 items-center">
        {menuItems.map((item) => (
          <NavButton
            key={item.id}
            id={item.id}
            icon={item.icon}
            label={item.label}
            active={activeTab === item.id}
            onClick={() => setActiveTab(item.id)}
          />
        ))}
      </div>

      <NavButton
        id="settings"
        icon={Settings}
        label="设置"
        active={activeTab === "settings"}
        onClick={() => setActiveTab("settings")}
        className="mt-auto"
      />
    </nav>
  );
}
