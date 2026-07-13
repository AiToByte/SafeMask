export interface EntityColor {
  badge: string;
  hlBg: string;
  hlBorder: string;
  chinese: string;
}

export const ENTITY_COLORS: Record<string, EntityColor> = {
  EMAIL:    { badge: "bg-amber-500/15 text-amber-300 border-amber-500/30",         hlBg: "bg-amber-500/8",    hlBorder: "border-b-amber-500/40",  chinese: "邮箱" },
  PHONE:    { badge: "bg-blue-500/15 text-blue-300 border-blue-500/30",           hlBg: "bg-blue-500/8",     hlBorder: "border-b-blue-500/40",   chinese: "电话" },
  PERSON:   { badge: "bg-emerald-500/15 text-emerald-300 border-emerald-500/30",  hlBg: "bg-emerald-500/8",  hlBorder: "border-b-emerald-500/40", chinese: "人名" },
  ADDRESS:  { badge: "bg-violet-500/15 text-violet-300 border-violet-500/30",     hlBg: "bg-violet-500/8",   hlBorder: "border-b-violet-500/40",  chinese: "地址" },
  ID_CARD:  { badge: "bg-rose-500/15 text-rose-300 border-rose-500/30",           hlBg: "bg-rose-500/8",     hlBorder: "border-b-rose-500/40",    chinese: "证件号" },
  BANK_CARD:{ badge: "bg-orange-500/15 text-orange-300 border-orange-500/30",     hlBg: "bg-orange-500/8",   hlBorder: "border-b-orange-500/40",  chinese: "银行卡" },
  API_KEY:  { badge: "bg-red-500/15 text-red-300 border-red-500/30",              hlBg: "bg-red-500/8",      hlBorder: "border-b-red-500/40",     chinese: "密钥" },
  PASSWORD: { badge: "bg-pink-500/15 text-pink-300 border-pink-500/30",           hlBg: "bg-pink-500/8",     hlBorder: "border-b-pink-500/40",    chinese: "密码" },
  IP:       { badge: "bg-cyan-500/15 text-cyan-300 border-cyan-500/30",           hlBg: "bg-cyan-500/8",     hlBorder: "border-b-cyan-500/40",    chinese: "IP地址" },
  URL:      { badge: "bg-indigo-500/15 text-indigo-300 border-indigo-500/30",     hlBg: "bg-indigo-500/8",   hlBorder: "border-b-indigo-500/40",  chinese: "链接" },
  DATE:     { badge: "bg-teal-500/15 text-teal-300 border-teal-500/30",           hlBg: "bg-teal-500/8",     hlBorder: "border-b-teal-500/40",    chinese: "日期" },
};

export function getEntityColor(type: string): EntityColor {
  return ENTITY_COLORS[type.toUpperCase()] ?? {
    badge: "bg-zinc-500/15 text-zinc-300 border-zinc-500/30",
    hlBg: "bg-zinc-500/8",
    hlBorder: "border-b-zinc-500/40",
    chinese: type,
  };
}
