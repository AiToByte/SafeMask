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

export default function Sidebar() {
  const activeTab = useAppStore((s) => s.activeTab);
  const setActiveTab = useAppStore((s) => s.setActiveTab);

  return (
    <nav className="w-24 flex flex-col items-center py-8 bg-[#0c0c0e] border-r border-zinc-800/50 z-50">
      <div className="w-14 h-14 bg-gradient-to-br from-blue-500 to-indigo-600 rounded-3xl flex items-center justify-center shadow-lg shadow-blue-500/20 mb-12 cursor-pointer transition-transform duration-200 hover:scale-105 active:scale-95 hover:shadow-[0_0_30px_rgba(59,130,246,0.4)]">
        <ShieldCheck className="text-white w-8 h-8" />
      </div>

      <div className="flex flex-col gap-8 flex-1">
        {menuItems.map((item) => {
          const isActive = activeTab === item.id;
          const Icon = item.icon;

          return (
            <button
              key={item.id}
              type="button"
              onClick={() => setActiveTab(item.id)}
              title={item.label}
              className={cn(
                "sidebar-item relative group overflow-hidden",
                isActive
                  ? "text-blue-400 shadow-[inset_0_0_12px_rgba(59,130,246,0.1)]"
                  : "text-zinc-600 hover:text-zinc-200",
              )}
            >
              {isActive && (
                <div className="absolute inset-0 bg-gradient-to-r from-blue-600/20 via-indigo-500/20 to-blue-600/20 animate-flow" />
              )}

              <Icon
                size={28}
                className="relative z-10 transition-all duration-300"
                strokeWidth={isActive ? 2.5 : 2}
              />

              {isActive && (
                <div className="absolute -left-4 w-1 h-6 bg-gradient-to-b from-blue-500 to-indigo-500 rounded-r-full shadow-blue-glow transition-all duration-300" />
              )}

              <span className="absolute left-full ml-4 px-4 py-2 bg-zinc-800/90 text-white text-sm font-medium rounded-lg border border-blue-500/20 shadow-lg opacity-0 group-hover:opacity-100 transition-all duration-300 translate-x-2 group-hover:translate-x-0 pointer-events-none z-50 whitespace-nowrap">
                {item.label}
              </span>
            </button>
          );
        })}
      </div>

      <button
        type="button"
        onClick={() => setActiveTab("settings")}
        className={cn(
          "sidebar-item relative group overflow-hidden mt-auto transition-transform duration-200 hover:scale-105 active:scale-95",
          activeTab === "settings" ? "text-blue-400" : "text-zinc-600 hover:text-zinc-200",
        )}
      >
        {activeTab === "settings" && (
          <div className="absolute inset-0 bg-gradient-to-r from-blue-600/20 via-indigo-500/20 to-blue-600/20 animate-flow" />
        )}
        <Settings size={28} className="relative z-10" />
      </button>
    </nav>
  );
}
