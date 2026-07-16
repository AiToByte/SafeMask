import { useEffect, useState, useCallback, useMemo } from "react";
import {
  X,
  Clock,
  Ghost,
  ShieldAlert,
  FileText,
  ShieldCheck,
  ClipboardCopy,
  ClipboardCheck,
} from "lucide-react";
import { MaskAPI, type HistoryItem, type EntitySpanBrief } from "@/services/api";
import { cn } from "@/lib/utils";
import { getEntityColor } from "@/lib/maskColors";

interface DocumentPreviewProps {
  item: HistoryItem | null;
  onClose: () => void;
}

function PreviewCopyButton({
  text,
  isOriginal,
}: {
  text: string;
  isOriginal: boolean;
}) {
  const [copied, setCopied] = useState(false);

  const handleCopy = useCallback(async () => {
    try {
      if (isOriginal) await MaskAPI.copyOriginal(text);
      else await navigator.clipboard.writeText(text);
      setCopied(true);
      setTimeout(() => setCopied(false), 2000);
    } catch {
      /* ignore */
    }
  }, [text, isOriginal]);

  return (
    <button
      type="button"
      onClick={handleCopy}
      className={cn(
        "flex items-center gap-2 text-[10px] font-bold transition-all px-3 py-2 rounded-lg border active:scale-95",
        copied
          ? "text-emerald-400 bg-emerald-500/10 border-emerald-500/20"
          : "text-zinc-600 border-white/5 bg-white/[0.02] hover:text-amber-100 hover:border-amber-500/20",
      )}
    >
      {copied ? <ClipboardCheck size={12} /> : <ClipboardCopy size={12} />}
      {copied ? (isOriginal ? "已复制原文" : "已复制") : "复制"}
    </button>
  );
}

// ── 将 UTF-8 字节偏移映射为 JS 字符串索引 ──
function buildByteToCharMap(text: string): Uint16Array {
  const encoder = new TextEncoder();
  const bytes = encoder.encode(text);
  const map = new Uint16Array(bytes.length);
  let charIdx = 0;
  let byteIdx = 0;
  while (byteIdx < bytes.length) {
    const cp = text.codePointAt(charIdx)!;
    const utf8Len = cp <= 0x7f ? 1 : cp <= 0x7ff ? 2 : cp <= 0xffff ? 3 : 4;
    for (let j = 0; j < utf8Len; j++) {
      map[byteIdx + j] = charIdx;
    }
    byteIdx += utf8Len;
    charIdx += cp > 0xffff ? 2 : 1;
  }
  return map;
}

// ── 原文高亮渲染 ──
function OriginalHighlighter({
  text,
  entities,
  hoveredIdx,
  onHover,
}: {
  text: string;
  entities: EntitySpanBrief[];
  hoveredIdx: number | null;
  onHover: (i: number | null) => void;
}) {
  const segments = useMemo(() => {
    if (entities.length === 0) return [{ text, entity: null as EntitySpanBrief | null }];

    const byteMap = buildByteToCharMap(text);
    const lastByte = text.length > 0 ? new TextEncoder().encode(text).length - 1 : 0;
    const result: Array<{ text: string; entity: EntitySpanBrief | null }> = [];
    let cursor = 0;

    for (const ent of entities) {
      const segStart = byteMap[ent.start] ?? 0;
      const segEnd = byteMap[Math.min(ent.end, lastByte)] ?? text.length;
      if (segStart > cursor) {
        result.push({ text: text.slice(cursor, segStart), entity: null });
      }
      if (segEnd > segStart) {
        result.push({ text: text.slice(segStart, segEnd), entity: ent });
      }
      cursor = segEnd;
    }
    if (cursor < text.length) {
      result.push({ text: text.slice(cursor), entity: null });
    }
    return result;
  }, [text, entities]);

  return (
    <div className="text-sm font-mono leading-relaxed whitespace-pre-wrap break-words text-zinc-500">
      {segments.map((seg, i) =>
        seg.entity ? (
          <HighlightedSpan
            key={i}
            entity={seg.entity}
            isHovered={hoveredIdx !== null && entities.indexOf(seg.entity) === hoveredIdx}
            entityIdx={entities.indexOf(seg.entity)}
            onHover={onHover}
          >
            {seg.text}
          </HighlightedSpan>
        ) : (
          <span key={i}>{seg.text}</span>
        ),
      )}
    </div>
  );
}

function HighlightedSpan({
  entity,
  isHovered,
  entityIdx,
  onHover,
  children,
}: {
  entity: EntitySpanBrief;
  isHovered: boolean;
  entityIdx: number;
  onHover: (i: number | null) => void;
  children: React.ReactNode;
}) {
  const color = getEntityColor(entity.entity_type);
  return (
    <span
      data-entity-idx={entityIdx}
      className={cn(
        "transition-all duration-200 rounded-sm px-0.5 border-b-2 cursor-default",
        color.hlBg,
        color.hlBorder,
        isHovered ? "ring-1 ring-inset ring-white/10 opacity-100" : "opacity-70",
      )}
      onMouseEnter={() => onHover(entityIdx)}
      onMouseLeave={() => onHover(null)}
    >
      {children}
      <span className={cn("text-[9px] font-bold ml-1 align-middle", color.badge.split(" ")[1])}>
        {color.chinese}
      </span>
    </span>
  );
}

// ── 脱敏副本高亮渲染 ──
function MaskedHighlighter({
  text,
  entities,
  hoveredIdx,
  onHover,
}: {
  text: string;
  entities: EntitySpanBrief[];
  hoveredIdx: number | null;
  onHover: (i: number | null) => void;
}) {
  const segments = useMemo(() => {
    if (entities.length === 0) return [{ text, entityIdx: -1 }];

    const labels = [...new Set(entities.map((e) => e.mask_label))];
    const pattern = labels
      .map((l) => l.replace(/[.*+?^${}()|[\]\\]/g, "\\$&"))
      .join("|");
    const re = new RegExp(`(${pattern})`, "g");
    const tokens = text.split(re);
    let entityCounter = 0;
    return tokens.map((token) => {
      const matchIdx = entities.findIndex(
        (e, ei) => ei >= entityCounter && e.mask_label === token,
      );
      if (matchIdx !== -1) {
        entityCounter = matchIdx + 1;
        return { text: token, entityIdx: matchIdx };
      }
      return { text: token, entityIdx: -1 };
    });
  }, [text, entities]);

  return (
    <div className="text-sm font-mono leading-relaxed whitespace-pre-wrap break-words text-zinc-200">
      {segments.map((seg, i) =>
        seg.entityIdx >= 0 ? (
          <span
            key={i}
            data-entity-idx={seg.entityIdx}
            className={cn(
              "inline-flex items-center gap-1 px-2 py-0.5 rounded-md text-[11px] font-bold uppercase tracking-wider transition-all duration-200 cursor-default",
              getEntityColor(entities[seg.entityIdx].entity_type).badge,
              hoveredIdx === seg.entityIdx && "shadow-[0_0_14px_rgba(255,255,255,0.06)] scale-105",
            )}
            onMouseEnter={() => onHover(seg.entityIdx)}
            onMouseLeave={() => onHover(null)}
          >
            {seg.text.replace(/[<>\[\]]/g, "")}
          </span>
        ) : (
          <span key={i}>{seg.text}</span>
        ),
      )}
    </div>
  );
}

export default function DocumentPreview({ item, onClose }: DocumentPreviewProps) {
  const [hoveredIdx, setHoveredIdx] = useState<number | null>(null);

  useEffect(() => {
    if (!item) return;
    const handler = (e: KeyboardEvent) => {
      if (e.key === "Escape") onClose();
    };
    window.addEventListener("keydown", handler);
    return () => window.removeEventListener("keydown", handler);
  }, [item, onClose]);

  if (!item) return null;

  return (
    <div
      className="fixed inset-0 z-[1000] flex items-center justify-center bg-black/80 backdrop-blur-md modal-backdrop"
      onClick={onClose}
    >
      <div
        className="relative w-[95vw] max-w-6xl max-h-[90vh] flex flex-col rounded-4xl border border-white/10 shadow-2xl bg-[#0c0b0a]/95 modal-panel"
        onClick={(e) => e.stopPropagation()}
      >
        {/* ── Header ── */}
        <div className="flex items-center justify-between px-10 pt-8 pb-6 border-b border-white/[0.03] shrink-0">
          <div className="flex items-center gap-5">
            <div className="flex items-center gap-4">
              <div className="w-1.5 h-8 bg-gradient-to-b from-rose-500/60 to-rose-500/0 rounded-full" />
              <h2 className="text-xl font-bold text-amber-50/80 tracking-tight">
                审计详情
              </h2>
            </div>
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
            <span className="flex items-center gap-2 text-zinc-500 text-xs font-mono font-bold bg-black/40 px-3 py-1.5 rounded-lg border border-white/[0.02]">
              <Clock size={14} /> {item.timestamp}
            </span>
          </div>
          <button
            type="button"
            onClick={onClose}
            className="w-10 h-10 rounded-xl border border-white/5 bg-white/[0.02] flex items-center justify-center text-zinc-500 hover:text-amber-100 hover:border-amber-500/20 hover:bg-amber-500/10 transition-all duration-300 active:scale-90"
          >
            <X size={18} />
          </button>
        </div>

        {/* ── Two-column body ── */}
        <div className="flex-1 grid grid-cols-1 lg:grid-cols-2 min-h-0 overflow-hidden">
          {/* Left: Original */}
          <div className="flex flex-col min-h-0 border-r border-white/[0.03]">
            <div className="flex items-center justify-between px-8 pt-6 pb-3 shrink-0">
              <div className="flex items-center gap-3">
                <FileText size={14} className="text-zinc-600" />
                <span className="text-xs font-black uppercase tracking-[0.2em] text-zinc-600">
                  原始数据流
                </span>
                <span className="text-[10px] font-mono text-zinc-700">
                  {item.original.length} 字符
                </span>
              </div>
              <PreviewCopyButton text={item.original} isOriginal />
            </div>
            <div className="flex-1 overflow-y-auto custom-scroll px-8 pb-8">
              <OriginalHighlighter
                text={item.original}
                entities={item.entities}
                hoveredIdx={hoveredIdx}
                onHover={setHoveredIdx}
              />
            </div>
          </div>

          {/* Right: Masked */}
          <div className="flex flex-col min-h-0">
            <div className="flex items-center justify-between px-8 pt-6 pb-3 shrink-0">
              <div className="flex items-center gap-3">
                <ShieldCheck size={14} className="text-amber-500/60" />
                <span
                  className={cn(
                    "text-xs font-black uppercase tracking-[0.2em]",
                    item.mode === "SHADOW" ? "text-blue-500/80" : "text-amber-500/80",
                  )}
                >
                  脱敏副本
                </span>
                <span className="text-[10px] font-mono text-zinc-700">
                  {item.masked.length} 字符
                </span>
              </div>
              <PreviewCopyButton text={item.masked} isOriginal={false} />
            </div>
            <div className="flex-1 overflow-y-auto custom-scroll px-8 pb-8">
              <MaskedHighlighter
                text={item.masked}
                entities={item.entities}
                hoveredIdx={hoveredIdx}
                onHover={setHoveredIdx}
              />
            </div>
          </div>
        </div>

        {/* ── Footer ── */}
        <div className="flex items-center justify-between px-10 py-4 border-t border-white/[0.03] shrink-0">
          <span className="text-[9px] font-mono uppercase tracking-[0.5em] text-zinc-700">
            Audit-ID: {item.id.split("-")[0]}
          </span>
          <span className="text-[10px] font-mono text-zinc-700">
            SafeMask v{__APP_VERSION__}
          </span>
        </div>
      </div>
    </div>
  );
}
