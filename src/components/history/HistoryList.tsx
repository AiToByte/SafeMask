import { useState, useMemo, useCallback } from "react";
import {
  ClipboardCopy,
  ClipboardCheck,
  CornerDownRight,
  Clock,
  Ghost,
  ShieldAlert,
  Trash2,
  Search,
  X,
} from "lucide-react";
import { useAppStore } from "@/hooks/useAppStore";
import { MaskAPI, type HistoryItem } from "@/services/api";
import { cn } from "@/lib/utils";

export default function HistoryList() {
  const historyList = useAppStore((s) => s.historyList);
  const clearHistory = useAppStore((s) => s.clearHistory);
  const [copiedId, setCopiedId] = useState("");
  const [searchQuery, setSearchQuery] = useState("");

  const handleCopy = useCallback(async (id: string, text: string, type: "org" | "msk") => {
    if (type === "org") await MaskAPI.copyOriginal(text);
    else await navigator.clipboard.writeText(text);
    setCopiedId(`${id}_${type}`);
    setTimeout(() => setCopiedId(""), 2000);
  }, []);

  const filteredHistory = useMemo(() => {
    if (!searchQuery) return historyList;
    const q = searchQuery.toLowerCase().trim();
    return historyList.filter((item) => {
      const displayId = item.id.split("-")[0].toLowerCase();
      return (
        item.original.toLowerCase().includes(q) ||
        item.masked.toLowerCase().includes(q) ||
        displayId.includes(q) ||
        item.timestamp.includes(q)
      );
    });
  }, [historyList, searchQuery]);

  return (
    <div className="flex flex-col gap-8 pb-20">
      {/* Header */}
      <div className="flex flex-col gap-6 px-2">
        <div className="flex justify-between items-end">
          <div className="space-y-1 relative">
            <div className="absolute -left-6 top-0 w-1.5 h-8 bg-gradient-to-b from-rose-500/60 to-rose-500/0 rounded-full" />
            <h2 className="text-2xl font-bold text-amber-50/80 tracking-tight">审计账本</h2>
            <p className="text-xs text-zinc-600 font-bold uppercase tracking-[0.3em]">
              Historical Audit Trail
            </p>
          </div>
          <button
            type="button"
            onClick={clearHistory}
            className="flex items-center gap-2 text-xs font-black text-zinc-600 hover:text-red-400 transition-all uppercase tracking-widest py-2 px-4 rounded-xl border border-white/5 hover:bg-red-500/5"
          >
            <Trash2 size={14} />
            <span>销毁审计记录</span>
          </button>
        </div>

        {/* Search bar */}
        <div className="relative w-full max-w-2xl mx-auto group/search">
          <div className="absolute -inset-2 bg-amber-500/[0.03] rounded-[2rem] blur-2xl opacity-0 group-focus-within/search:opacity-100 transition-opacity duration-700" />
          <div className="relative flex items-center h-16 px-5 rounded-3xl transition-all duration-500 border border-amber-500/25 bg-[#141210]/90 shadow-[0_4px_20px_-2px_rgba(0,0,0,0.5)] focus-within:border-amber-500/60 focus-within:bg-black focus-within:shadow-[0_0_25px_rgba(245,158,11,0.08)]">
            <Search size={18} className="text-amber-500/60 group-focus-within/search:text-amber-400 transition-colors" />
            <input
              value={searchQuery}
              onChange={(e) => setSearchQuery(e.target.value)}
              placeholder="搜索原文、脱敏结果或 Audit-ID..."
              className="flex-1 bg-transparent border-none outline-none px-4 text-amber-50 font-medium text-sm placeholder:text-zinc-600"
            />
            {searchQuery && (
              <button type="button" onClick={() => setSearchQuery("")} className="p-2 rounded-lg text-zinc-600 hover:text-amber-200 hover:bg-white/5 transition-all">
                <X size={14} />
              </button>
            )}
          </div>
        </div>
      </div>

      {/* Empty states */}
      {filteredHistory.length === 0 && !searchQuery && (
        <div className="flex flex-col items-center justify-center py-24 opacity-20">
          <Search size={48} className="mb-4" />
          <p className="text-base font-bold tracking-widest uppercase">暂无脱敏记录</p>
        </div>
      )}

      {filteredHistory.length === 0 && searchQuery && (
        <div className="flex flex-col items-center justify-center py-24">
          <div className="relative mb-6">
            <div className="absolute inset-0 bg-amber-500/10 blur-3xl rounded-full" />
            <Search size={48} className="text-zinc-800 relative z-10" />
          </div>
          <h3 className="text-amber-50/60 font-bold tracking-widest uppercase text-xs">No Audit Matches</h3>
          <p className="text-xs text-zinc-600 mt-2">未发现包含 "{searchQuery}" 的审计项</p>
        </div>
      )}

      {/* History cards */}
      <div>
        {filteredHistory.map((item) => (
          <HistoryCard key={item.id} item={item} copiedId={copiedId} onCopy={handleCopy} />
        ))}
      </div>
    </div>
  );
}

function HistoryCard({
  item,
  copiedId,
  onCopy,
}: {
  item: HistoryItem;
  copiedId: string;
  onCopy: (id: string, text: string, type: "org" | "msk") => void;
}) {
  return (
    <div
      className={cn(
        "group/card p-10 rounded-4xl border border-white/[0.03] bg-[#0c0b0a]/40 hover:bg-[#110f0e]/60 transition-all duration-700 mb-6 relative overflow-hidden",
        item.mode === "SHADOW"
          ? "before:content-[''] before:absolute before:top-0 before:left-8 before:right-8 before:h-[2px] before:bg-gradient-to-r before:from-amber-500/0 before:via-amber-400/60 before:to-amber-500/0"
          : "before:content-[''] before:absolute before:top-0 before:left-8 before:right-8 before:h-[2px] before:bg-gradient-to-r before:from-blue-500/0 before:via-blue-400/60 before:to-blue-500/0",
      )}
    >
      {/* Meta row */}
      <div className="flex justify-between items-center mb-6">
        <div className="flex items-center gap-4">
          <span className="flex items-center gap-2 text-zinc-500 text-xs font-mono font-bold bg-black/40 px-3 py-1.5 rounded-lg border border-white/[0.02]">
            <Clock size={14} /> {item.timestamp}
          </span>

          {item.mode === "SHADOW" ? (
            <span className="flex items-center gap-2 px-3 py-1.5 rounded-lg text-xs font-black uppercase border bg-amber-500/10 text-amber-500 border-amber-500/20">
              <span className="w-1.5 h-1.5 rounded-full bg-cyan-400/80 shadow-[0_0_6px_rgba(34,211,238,0.3)]" />
              <Ghost size={13} /> 影子宇宙侦测
            </span>
          ) : (
            <span className="flex items-center gap-2 px-3 py-1.5 rounded-lg text-xs font-black uppercase border bg-blue-500/10 text-blue-400 border-blue-500/20">
              <span className="w-1.5 h-1.5 rounded-full bg-cyan-400/80 shadow-[0_0_6px_rgba(34,211,238,0.3)]" />
              <ShieldAlert size={13} /> 哨兵宇宙拦截
            </span>
          )}
        </div>

        <span className="text-[10px] font-mono text-zinc-600 uppercase tracking-widest group-hover/card:text-zinc-400 transition-colors">
          Audit-ID: {item.id.split("-")[0]}
        </span>
      </div>

      {/* Comparison grid */}
      <div className="grid grid-cols-1 lg:grid-cols-2 gap-8 relative">
        {/* Original */}
        <div className="space-y-3">
          <div className="flex justify-between items-center px-1">
            <p className="text-xs font-black uppercase tracking-[0.2em] text-zinc-600">
              原始数据流 (Raw)
            </p>
            <CopyButton
              id={item.id}
              type="org"
              text={item.original}
              copiedId={copiedId}
              onCopy={onCopy}
              label="复制原文"
              labelDone="已复制"
            />
          </div>
          <div className="p-8 rounded-[1.5rem] text-sm font-mono leading-relaxed break-all bg-black/40 text-zinc-500 border-white/[0.03] border h-48 overflow-y-auto custom-scroll">
            {item.original}
          </div>
        </div>

        {/* Arrow divider */}
        <div className="absolute left-1/2 top-1/2 -translate-x-1/2 -translate-y-1/2 text-zinc-800 opacity-20 hidden lg:block">
          <CornerDownRight size={24} />
        </div>

        {/* Masked */}
        <div className="space-y-3">
          <div className="flex justify-between items-center px-1">
            <p className={cn(
              "text-xs font-black uppercase tracking-[0.2em]",
              item.mode === "SHADOW" ? "text-blue-500/80" : "text-amber-500/80"
            )}>
              脱敏副本 (Masked)
            </p>
            <CopyButton
              id={item.id}
              type="msk"
              text={item.masked}
              copiedId={copiedId}
              onCopy={onCopy}
              label="复制副本"
              labelDone="已复制副本"
              isMasked
            />
          </div>
          <div className="p-8 rounded-[1.5rem] text-sm font-mono leading-relaxed break-all bg-white/[0.01] text-zinc-200 border-white/[0.03] border shadow-inner h-48 overflow-y-auto custom-scroll">
            {item.masked}
          </div>
        </div>
      </div>
    </div>
  );
}

// ── Copy button micro-component ──

function CopyButton({
  id, type, text, copiedId, onCopy, label, labelDone, isMasked,
}: {
  id: string;
  type: "org" | "msk";
  text: string;
  copiedId: string;
  onCopy: (id: string, text: string, type: "org" | "msk") => void;
  label: string;
  labelDone: string;
  isMasked?: boolean;
}) {
  const isCopied = copiedId === `${id}_${type}`;
  return (
    <button
      type="button"
      onClick={() => onCopy(id, text, type)}
      className={cn(
        "flex items-center gap-2 text-[10px] font-bold transition-all px-3 py-2 rounded-lg bg-white/[0.02] border border-white/[0.05] active:scale-95",
        isCopied
          ? "text-emerald-400 bg-emerald-500/10 border-emerald-500/20"
          : isMasked
            ? "text-blue-500/60 hover:text-blue-400"
            : "text-zinc-600 hover:text-amber-100",
      )}
    >
      {isCopied ? <ClipboardCheck size={12} /> : <ClipboardCopy size={12} />}
      {isCopied ? labelDone : label}
    </button>
  );
}
