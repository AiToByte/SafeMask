import { useState, useEffect, useMemo, useRef } from "react";
import {
  Monitor, Cpu, Volume2, Eye, AlertTriangle,
  User, Mail, Github, Globe, Info, ExternalLink, Copyright,
  Copy, Check, Brain, Zap, Loader2, Lock,
  SwitchCamera, Trash2, RotateCcw, Timer, Keyboard, FileText
} from "lucide-react";
import { useAppStore } from "@/hooks/useAppStore";
import { useAudioFeedback } from "@/hooks/useAudioFeedback";
import { MaskAPI } from "@/services/api";
import { Toggle } from "@/components/ui/Toggle";
import { SettingToggle } from "@/components/ui/SettingToggle";
import { Card, CardHeader } from "@/components/ui/Card";
import ModelDownloadCard from "@/components/settings/ModelDownloadCard";
import ThemePicker from "@/components/settings/ThemePicker";
import { cn } from "@/lib/utils";
import { message, confirm } from "@tauri-apps/plugin-dialog";

// ── Format helpers ──

const formatRecognizer = (name: string) => {
  const map: Record<string, string> = {
    aho_corasick_engine: "字典匹配",
    regex_engine: "正则匹配",
    ner_engine: "AI 识别",
    context_enhancer: "上下文增强",
    checksum_recognizer: "校验位验证",
  };
  return map[name] || name;
};

const formatSize = (mb: number): string => {
  if (mb >= 1) return `${mb.toFixed(1)} MB`;
  return `${Math.round(mb * 1024)} KB`;
};

const getRecognizerColor = (name: string) => {
  const map: Record<string, string> = {
    aho_corasick_engine: "bg-emerald-500",
    regex_engine: "bg-blue-500",
    ner_engine: "bg-purple-500",
    context_enhancer: "bg-amber-500",
    checksum_recognizer: "bg-cyan-500",
  };
  return map[name] || "bg-zinc-500";
};

const formatEntityType = (type: string) => {
  const map: Record<string, string> = {
    person: "人名",
    email: "邮箱",
    phone: "电话",
    address: "地址",
    account_number: "账号",
    date: "日期",
    url: "链接",
    secret: "密钥",
  };
  return map[type] || type;
};


export default function SettingsPage() {
  const store = useAppStore();
  const [isRecording, setRecording] = useState(false);
  const recordingRef = useRef(false);
  const [showKeyWarn, setShowWarn] = useState(false);
  const [elapsed, setElapsed] = useState(0);
  const [emailCopied, setEmail] = useState(false);
  const [selectedModel, setSelectedModel] = useState<string | null>(null);
  const [modelUnselectLock, setModelUnselectLock] = useState(false);
  const [aiToggling, setAiToggling] = useState(false);
  const [recordsDir, setRecordsDir] = useState("");
  const [dirExists, setDirExists] = useState(false);
  const [dirLoading, setDirLoading] = useState(false);
  const { play } = useAudioFeedback(store.settings.enable_audio_feedback);

  // Auto-select first model when available models change
  useEffect(() => {
    if (store.aiEngineStatus?.available_count && store.aiEngineStatus?.available_count > 0) {
      setSelectedModel(store.aiEngineStatus.model?.name || "privacy-filter");
    }
  }, [store.aiEngineStatus?.available_count, store.aiEngineStatus?.model?.name]);

  // ── Effects ──

  useEffect(() => {
    store.fetchAiStatus();
    store.fetchEngineInfo();
  }, []);

  useEffect(() => {
    if (store.aiEngineStatus?.state === "loading") {
      const start = Date.now();
      const id = setInterval(
        () => setElapsed(Math.floor((Date.now() - start) / 1000)),
        1000,
      );
      return () => clearInterval(id);
    }
    setElapsed(0);
  }, [store.aiEngineStatus?.state]);

  useEffect(() => {
    let cancelled = false;
    setDirLoading(true);
    MaskAPI.getRecordsDirInfo().then((info) => {
      if (cancelled) return;
      setRecordsDir(info.dir);
      setDirExists(info.exists);
      setDirLoading(false);
    }).catch(() => {
      if (cancelled) return;
      setDirLoading(false);
    });
    return () => { cancelled = true; };
  }, [store.settings.record_writer_enabled]);

  // ── Handlers ──

  const handleAiToggle = async (enabled: boolean) => {
    setAiToggling(true);
    try {
      await store.toggleAiEngine(enabled);
      if (enabled) {
        play("ASCEND");
        await message("AI 引擎已启动，正在加载模型...", { title: "AI 引擎", kind: "info" });
      } else {
        play("DESCEND");
        await message("AI 识别已关闭，将使用规则引擎进行脱敏", { title: "AI 引擎", kind: "info" });
      }
    } catch (e) {
      await message("切换 AI 引擎失败: " + e, { title: "错误", kind: "error" });
    } finally {
      setAiToggling(false);
    }
  };

  const copyEmail = async () => {
    await navigator.clipboard.writeText("xiaosheng.tech@outlook.com");
    setEmail(true);
    play("CLICK");
    setTimeout(() => setEmail(false), 2000);
  };

  const handleKeyDown = async (e: React.KeyboardEvent<HTMLInputElement>) => {
    if (!recordingRef.current) return;
    e.preventDefault();
    e.stopPropagation();
    const mods: string[] = [];
    if (e.ctrlKey) mods.push("Ctrl");
    if (e.altKey) mods.push("Alt");
    if (e.shiftKey) mods.push("Shift");
    if (e.metaKey) mods.push("Super");
    let key = e.key.toUpperCase();
    if (!["CONTROL", "ALT", "SHIFT", "META"].includes(key)) {
      if (key === " ") key = "SPACE";
      const fs = [...mods, key].join("+");
      if (fs.toLowerCase() === "alt+m") {
        setShowWarn(true);
        play("ERROR");
        setTimeout(() => setShowWarn(false), 2500);
        return;
      }
      const s = { ...store.settings, magic_paste_shortcut: fs };
      store.updateSettings(s);
      setRecording(false);
      play("RECORD");
      try { await MaskAPI.updateSettings(s); }
      catch (err) { await message(`快捷键同步失败: ${err}`, { title: "错误", kind: "error" }); }
    }
  };

  // ── Derived ──

  const sliderProgress = ((store.settings.paste_delay_ms - 50) / 750) * 100;

  const aiDot =
    store.aiEngineStatus?.state === "ready"
      ? "bg-emerald-500 shadow-[0_0_8px_rgba(16,185,129,0.5)]"
      : store.aiEngineStatus?.state === "loading"
        ? "bg-amber-500 animate-pulse shadow-[0_0_8px_rgba(245,158,11,0.5)]"
        : store.aiEngineStatus?.state === "error"
          ? "bg-red-500 shadow-[0_0_8px_rgba(239,68,68,0.5)]"
          : "bg-zinc-600";

  const aiStatusText = (() => {
    switch (store.aiEngineStatus?.state) {
      case "ready":
        return "模型已就绪，AI 识别可用";
      case "loading":
        return "模型加载中，首次加载约需 1-3 分钟";
      case "error":
        return "加载失败: " + (store.aiEngineStatus?.error || "");
      case "not_loaded":
        return "模型未加载，复制文本时将自动触发";
      case "not_available":
        return "AI 引擎不可用";
      default:
        return "未知状态";
    }
  })();


  const aiActive = useMemo(() => {
    if (!store.aiEngineStatus) return false;
    return store.aiEngineStatus.state === "ready"
        || store.aiEngineStatus.state === "loading"
        || store.aiEngineStatus.state === "not_loaded";
  }, [store.aiEngineStatus]);
  // Build models list from available_count + model info
  const preparedModels = useMemo(() => {
    const models: { name: string; size_mb: number; loaded: boolean; description: string }[] = [];
    if (store.aiEngineStatus?.model) {
      models.push({
        name: store.aiEngineStatus.model.name,
        size_mb: store.aiEngineStatus.model.size_mb,
        loaded: store.aiEngineStatus.state === "ready",
        description: `OpenAI privacy filter · ${store.aiEngineStatus.model.entity_types.length} entities`,
      });
    }
    // Fill remaining from available_count
    const existing = models.length;
    for (let i = existing; i < Math.max(store.aiEngineStatus?.available_count || 0, 1); i++) {
      models.push({
        name: `model-${i + 1}`,
        size_mb: 0,
        loaded: false,
        description: "待加载模型",
      });
    }
    if (models.length === 0) {
      models.push({
        name: "privacy-filter",
        size_mb: 874,
        loaded: store.aiEngineStatus?.state === "ready",
        description: "OpenAI 隐私过滤模型",
      });
    }
    return models;
  }, [store.aiEngineStatus]);


  return (
    <div className="max-w-5xl mx-auto space-y-10 pb-16 page-active">
      {/* ════════════════ HEADER ════════════════ */}
      <div className="flex items-center gap-6 mb-10 px-2">
        <div
          className="w-14 h-14 rounded-2xl border border-amber-500/10 flex items-center justify-center shadow-2xl"
          style={{ backgroundColor: "var(--bg-elevated)" }}
        >
          <Monitor className="text-amber-400/80 w-6 h-6" />
        </div>
        <div>
          <h2 className="text-3xl font-bold text-amber-50/90 tracking-tight">
            控制台偏好设置
          </h2>
           <p className="text-xs text-zinc-600 font-black uppercase tracking-[0.4em] mt-1.5">
            System Configuration &amp; Developer Info
          </p>
        </div>
      </div>

      {/* ════════════════ THEME PICKER ════════════════ */}
      <ThemePicker />

      {/* ════════════════ GRID ════════════════ */}
      <div className="grid grid-cols-1 lg:grid-cols-2 gap-8 items-start">
        
        {/* ── Left Column: Kernel Behavior & shortcuts (Tall) ── */}
        <div className="space-y-8">
          <Card className="space-y-8">
            <CardHeader>
              <Cpu size={18} className="text-blue-500/70" />
              <span>内核脱敏行为 (Kernel)</span>
            </CardHeader>

            <div className="space-y-8">
              <SettingToggle
                title="启用影子宇宙模式"
                description="数据流在内存中脱敏，物理剪贴板保留原文"
                checked={store.settings.shadow_mode_enabled}
                onChange={async (checked) => {
                  const s = { ...store.settings, shadow_mode_enabled: checked };
                  store.updateSettings(s);
                  try { await MaskAPI.updateSettings(s); }
                  catch (err) { await message(`同步失败: ${err}`, { title: "错误", kind: "error" }); }
                }}
              />

              <SettingToggle
                icon={FileText}
                iconColor="emerald"
                title="历史记录持久化"
                description="将脱敏映射记录写入 .md 文件，用于 AI 训练分析"
                checked={store.settings.record_writer_enabled}
                onChange={async (checked) => {
                  const newSettings = { ...store.settings, record_writer_enabled: checked };
                  store.updateSettings(newSettings);
                  try {
                    const result = await MaskAPI.updateSettings(newSettings);
                    if (checked) {
                      setRecordsDir(result.records_dir);
                      setDirExists(result.records_dir_exists);
                    }
                    await message(`记录持久化${checked ? "已启用" : "已关闭"}`, { title: "同步成功", kind: "info" });
                  } catch (err) {
                    await message(`同步失败: ${err}`, { title: "错误", kind: "error" });
                  }
                }}
              />

              {store.settings.record_writer_enabled && (
                <div
                  className="p-4 rounded-2xl border border-white/[0.04]"
                  style={{ backgroundColor: "color-mix(in srgb, var(--bg-input) 92%, transparent)" }}
                >
                  <div className="text-[11px] font-black text-zinc-600 uppercase tracking-widest mb-2">
                    记录目录
                  </div>
                  <div className="font-mono text-[13px] text-zinc-300 break-all leading-relaxed">
                    {dirLoading ? (
                      <span className="text-zinc-600">加载中...</span>
                    ) : (
                      <>
                        <span className="text-amber-400/80">{recordsDir}</span>
                        <span className={dirExists ? "text-emerald-500 ml-2" : "text-zinc-600 ml-2"}>
                          {dirExists ? "✓ 已创建" : "尚未创建"}
                        </span>
                      </>
                    )}
                  </div>
                </div>
              )}

              <div
                className="p-5 rounded-[2rem] border border-white/[0.03]"
                style={{ backgroundColor: "color-mix(in srgb, var(--bg-input) 92%, transparent)" }}
              >
                <div className="text-xs font-black text-zinc-600 uppercase tracking-widest mb-4">
                  脱敏标签包裹样式
                </div>
                <div className="flex gap-3">
                  <button
                    type="button"
                    onClick={async () => {
                      const s = { ...store.settings, mask_wrapper_style: "angle" };
                      store.updateSettings(s);
                      try { await MaskAPI.updateSettings(s); }
                      catch (err) { await message(`同步失败: ${err}`, { title: "错误", kind: "error" }); }
                    }}
                    className={cn(
                      "flex-1 py-4 px-5 rounded-2xl border transition-all text-center",
                      store.settings.mask_wrapper_style === "angle"
                        ? "bg-amber-500/10 border-amber-500/30"
                        : "bg-white/[0.02] border-white/[0.04] hover:bg-white/[0.04]",
                    )}
                  >
                    <span className="text-lg font-bold font-mono">{"<EMAIL>"}</span>
                    <p className="text-[10px] text-zinc-600 mt-2">尖括号</p>
                  </button>
                  <button
                    type="button"
                    onClick={async () => {
                      const s = { ...store.settings, mask_wrapper_style: "square" };
                      store.updateSettings(s);
                      try { await MaskAPI.updateSettings(s); }
                      catch (err) { await message(`同步失败: ${err}`, { title: "错误", kind: "error" }); }
                    }}
                    className={cn(
                      "flex-1 py-4 px-5 rounded-2xl border transition-all text-center",
                      store.settings.mask_wrapper_style === "square"
                        ? "bg-amber-500/10 border-amber-500/30"
                        : "bg-white/[0.02] border-white/[0.04] hover:bg-white/[0.04]",
                    )}
                  >
                    <span className="text-lg font-bold font-mono">{"[EMAIL]"}</span>
                    <p className="text-[10px] text-zinc-600 mt-2">方括号</p>
                  </button>
                </div>
                <p className="text-[10px] text-zinc-700 mt-3 leading-relaxed">
                  此设置影响所有新建规则的默认标签格式，已有规则不受影响。
                </p>
              </div>

              <div
                className="p-7 rounded-[2rem] border border-white/[0.03] shadow-inner"
                style={{ backgroundColor: "color-mix(in srgb, var(--bg-input) 92%, transparent)" }}
              >
                <div className="text-xs font-black text-zinc-600 uppercase tracking-widest mb-5">Paste Shortcut</div>
                <div className="relative">
                  <input
                    readOnly
                    value={isRecording ? "正在监听按键组合..." : store.settings.magic_paste_shortcut}
                    onKeyDown={handleKeyDown}
                    onFocus={() => { recordingRef.current = true; setRecording(true); MaskAPI.setRecordingMode(true); }}
                    onBlur={() => { recordingRef.current = false; setRecording(false); MaskAPI.setRecordingMode(false); }}
                    className={cn(
                      "w-full border rounded-2xl py-5 text-base font-mono text-amber-200 text-center outline-none transition-all cursor-pointer shadow-inner",
                      isRecording
                        ? "border-amber-500/50 bg-amber-500/[0.03] text-amber-400 shadow-[0_0_30px_rgba(245,158,11,0.1)]"
                        : "border-white/[0.08]"
                    )}
                    style={isRecording ? undefined : { backgroundColor: "var(--bg-input)" }}
                  />
                  {showKeyWarn && (
                    <div className="absolute -bottom-7 left-0 right-0 flex justify-center">
                      <span
                        className="text-[10px] text-red-500 font-bold uppercase px-3 py-1 rounded-full border border-red-500/20"
                        style={{ backgroundColor: "var(--bg-root)" }}
                      >
                        Alt+M is reserved
                      </span>
                    </div>
                  )}
                </div>
              </div>
            </div>
          </Card>

          {/* Shortcut Guide */}
          <Card className="p-8 space-y-6">
            <CardHeader>
              <Keyboard size={16} className="text-amber-500/70" />
              <span>键盘快捷键 (Keyboard Shortcuts)</span>
            </CardHeader>
            <div className="space-y-3">
              <div className="p-5 bg-white/[0.02] rounded-2xl border border-white/[0.03] hover:border-amber-500/20 transition-colors">
                <div className="flex items-start gap-4">
                  <div className="w-10 h-10 rounded-xl bg-amber-500/10 border border-amber-500/20 flex items-center justify-center shrink-0 mt-0.5">
                    <SwitchCamera size={16} className="text-amber-400" />
                  </div>
                  <div className="min-w-0">
                    <div className="flex items-center gap-2.5 flex-wrap">
                      <code className="px-2.5 py-0.5 bg-amber-500/15 text-amber-300 text-xs font-mono font-bold rounded-lg border border-amber-500/20 shrink-0">Alt+M</code>
                      <span className="text-sm font-bold text-zinc-300">切换运行模式</span>
                    </div>
                    <div className="mt-3 space-y-2 text-[11px] text-zinc-500 leading-relaxed">
                      <div>
                        <span className="text-amber-200/80 font-semibold">影子宇宙</span> — 复制不脱敏，剪贴板保留原文，按 <code className="px-1.5 py-0.5 bg-white/[0.04] text-zinc-400 text-[10px] font-mono rounded">{store.settings.magic_paste_shortcut}</code> 粘贴脱敏副本
                      </div>
                      <div>
                        <span className="text-blue-400/80 font-semibold">哨兵宇宙</span> — 复制即脱敏，系统自动洗白剪贴板内容
                      </div>
                    </div>
                  </div>
                </div>
              </div>

              <div className="p-5 bg-white/[0.02] rounded-2xl border border-white/[0.03] hover:border-indigo-500/20 transition-colors">
                <div className="flex items-start gap-4">
                  <div className="w-10 h-10 rounded-xl bg-indigo-500/10 border border-indigo-500/20 flex items-center justify-center shrink-0 mt-0.5">
                    <Zap size={16} className="text-indigo-400" />
                  </div>
                  <div className="min-w-0">
                    <div className="flex items-center gap-2.5 flex-wrap">
                      <code className="px-2.5 py-0.5 bg-indigo-500/15 text-indigo-300 text-xs font-mono font-bold rounded-lg border border-indigo-500/20 shrink-0">{store.settings.magic_paste_shortcut}</code>
                      <span className="text-sm font-bold text-zinc-300">安全粘贴</span>
                    </div>
                    <p className="text-[11px] text-zinc-500 leading-relaxed mt-2">
                      将影子宇宙模式中暂存的脱敏副本注入到当前输入框。
                    </p>
                  </div>
                </div>
              </div>
            </div>
          </Card>
        </div>

        {/* ── Right Column: Feedback & Diagnostics (Stacked to Symmetrize Heights!) ── */}
        <div className="space-y-8">
          {/* Feedback section */}
          <Card className="space-y-8">
            <CardHeader>
              <Volume2 size={18} className="text-amber-500/70" />
              <span>实时感官反馈 (Feedback)</span>
            </CardHeader>
            
            <div className="space-y-6">
              <SettingToggle
                icon={Eye}
                iconColor="blue"
                title="蓝盾视觉气泡"
                description="桌面叠加层实时反馈"
                checked={store.settings.enable_visual_feedback}
                onChange={async (checked) => {
                  const s = { ...store.settings, enable_visual_feedback: checked };
                  store.updateSettings(s);
                  try { await MaskAPI.updateSettings(s); }
                  catch (err) { await message(`同步失败: ${err}`, { title: "错误", kind: "error" }); }
                }}
              />

              <SettingToggle
                icon={Volume2}
                iconColor="amber"
                title="物理机械音效"
                description="系统声音反馈"
                checked={store.settings.enable_audio_feedback}
                onChange={async (checked) => {
                  const s = { ...store.settings, enable_audio_feedback: checked };
                  store.updateSettings(s);
                  try { await MaskAPI.updateSettings(s); }
                  catch (err) { await message(`同步失败: ${err}`, { title: "错误", kind: "error" }); }
                }}
              />

              <div className="pt-8 border-t border-white/[0.03] space-y-5">
                <div className="flex justify-between items-end">
                  <div className="flex items-center gap-3">
                    <div className="w-9 h-9 rounded-xl bg-amber-500/10 border border-amber-500/20 flex items-center justify-center">
                      <Timer size={16} className="text-amber-400/80" />
                    </div>
                    <div>
                      <div className="text-sm font-bold text-zinc-300">粘贴注入延迟</div>
                      <div className="text-[10px] text-zinc-600 font-bold uppercase tracking-widest mt-0.5">快捷键注入后延迟毫秒数</div>
                    </div>
                  </div>
                  <span className="font-mono text-amber-300 text-sm font-bold bg-amber-500/10 px-3 py-1.5 rounded-lg border border-amber-500/20 shadow-[0_0_10px_rgba(245,158,11,0.08)]">
                    {store.settings.paste_delay_ms} ms
                  </span>
                </div>
                <div className="relative py-2">
                  <input
                    type="range"
                    min="50"
                    max="800"
                    step="50"
                    value={store.settings.paste_delay_ms}
                    onChange={(e) => store.updateSettings({ ...store.settings, paste_delay_ms: parseInt(e.target.value) })}
                    onMouseUp={async (e) => {
                      const s = { ...store.settings, paste_delay_ms: parseInt(e.currentTarget.value) };
                      try { await MaskAPI.updateSettings(s); }
                      catch (err) { await message(`同步失败: ${err}`, { title: "错误", kind: "error" }); }
                    }}
                    className="w-full h-3.5 rounded-full appearance-none cursor-pointer outline-none border border-white/[0.05] shadow-inner slider-amber-glow"
                    style={{
                      backgroundColor: "var(--bg-input)",
                      backgroundImage: "linear-gradient(var(--accent), var(--accent))",
                      backgroundSize: sliderProgress + "% 100%",
                      backgroundRepeat: "no-repeat"
                    }}
                  />
                  <div className="flex justify-between px-0.5 mt-2">
                    {[50, 200, 400, 600, 800].map((ms) => (
                      <span
                        key={ms}
                        className={cn(
                          "text-[8px] font-mono transition-colors",
                          store.settings.paste_delay_ms === ms ? "text-amber-500/60" : "text-zinc-800"
                        )}
                      >
                        {ms}
                      </span>
                    ))}
                  </div>
                </div>
              </div>
            </div>
          </Card>

          {/* Engine running diagnostics card — Fills the previous empty gap with symmetric height! */}
          <Card className="space-y-6">
            <CardHeader>
              <Zap size={18} className="text-emerald-400" />
              <span>引擎运行特征监视器 (Diagnostics &amp; Metrics)</span>
            </CardHeader>

            <div className="space-y-5">
              {/* Defensive State Row */}
              <div className="flex justify-between items-center py-3.5 px-5 rounded-2xl border border-white/[0.02]" style={{ backgroundColor: "color-mix(in srgb, var(--bg-input) 90%, transparent)" }}>
                <div>
                  <div className="text-sm font-bold text-zinc-300">主动防御监控守护</div>
                  <div className="text-[10px] text-zinc-600 font-bold uppercase tracking-widest mt-0.5">Core Active Sentry</div>
                </div>
                <div className="flex items-center gap-2 bg-emerald-500/10 px-3.5 py-1.5 rounded-xl border border-emerald-500/20">
                  <span className="w-2 h-2 rounded-full bg-emerald-500 animate-pulse shadow-[0_0_8px_#10b981]" />
                  <span className="text-xs font-black text-emerald-400 uppercase tracking-wider">运行中</span>
                </div>
              </div>

              {/* Internal Engines load */}
              <div className="p-5 rounded-[2rem] border border-white/[0.03] space-y-4" style={{ backgroundColor: "color-mix(in srgb, var(--bg-input) 92%, transparent)" }}>
                <div className="text-xs font-black text-zinc-600 uppercase tracking-widest">内置识别器集群 (Registered Recognizers)</div>
                <div className="grid grid-cols-1 sm:grid-cols-2 gap-2.5">
                  <div className="flex items-center gap-2.5 py-2.5 px-3.5 rounded-xl bg-white/[0.02] border border-white/[0.03]">
                    <div className="w-1.5 h-1.5 rounded-full bg-emerald-500" />
                    <span className="text-xs font-bold text-zinc-400">Aho-Corasick ({store.ruleCount} 模式)</span>
                  </div>
                  <div className="flex items-center gap-2.5 py-2.5 px-3.5 rounded-xl bg-white/[0.02] border border-white/[0.03]">
                    <div className="w-1.5 h-1.5 rounded-full bg-blue-500" />
                    <span className="text-xs font-bold text-zinc-400">Regex 字节级覆盖</span>
                  </div>
                  <div className="flex items-center gap-2.5 py-2.5 px-3.5 rounded-xl bg-white/[0.02] border border-white/[0.03]">
                    <div className="w-1.5 h-1.5 rounded-full bg-purple-500" />
                    <span className="text-xs font-bold text-zinc-400">ONNX AI ({store.aiEngineStatus?.state === "ready" ? "高能就绪" : "待机/未加载"})</span>
                  </div>
                  <div className="flex items-center gap-2.5 py-2.5 px-3.5 rounded-xl bg-white/[0.02] border border-white/[0.03]">
                    <div className="w-1.5 h-1.5 rounded-full bg-cyan-500" />
                    <span className="text-xs font-bold text-zinc-400">Checksum 校验位验证</span>
                  </div>
                </div>
              </div>

              {/* Core limits / telemetry */}
              <div className="p-5 rounded-[2rem] border border-white/[0.03] space-y-3.5 text-xs" style={{ backgroundColor: "color-mix(in srgb, var(--bg-input) 92%, transparent)" }}>
                <div className="flex justify-between">
                  <span className="text-zinc-500 font-bold">线程调度策略</span>
                  <span className="text-zinc-400 font-mono font-medium">Rayon 限制 2 线程</span>
                </div>
                <div className="flex justify-between">
                  <span className="text-zinc-500 font-bold">系统高速分配器</span>
                  <span className="text-emerald-400 font-mono font-medium">Mimalloc 已激活</span>
                </div>
                <div className="flex justify-between">
                  <span className="text-zinc-500 font-bold">进程虚拟内存管线</span>
                  <span className="text-zinc-400 font-mono font-medium">I/O 零拷贝就绪 (Mmap)</span>
                </div>
                <div className="flex justify-between">
                  <span className="text-zinc-500 font-bold">核心常驻静态内存</span>
                  <span className="text-zinc-400 font-mono font-medium">约 40 MB</span>
                </div>
              </div>
            </div>
          </Card>
        </div>

        {/* ── AI Engine (span-2) ── */}
        <Card className="lg:col-span-2 space-y-6">
          <CardHeader className="mb-4">
            <Brain size={18} className="text-purple-500/70" />
            <span>AI Engine</span>
          </CardHeader>

          <div className="flex items-center justify-between p-5 rounded-2xl border border-white/[0.03]" style={{ backgroundColor: "color-mix(in srgb, var(--bg-input) 90%, transparent)" }}>
            <div className="flex items-center gap-4">
              <div className={cn("w-3.5 h-3.5 rounded-full transition-colors", aiDot)} />
              <div>
                <div className="text-sm font-bold text-zinc-300">AI NER 实体引擎分析器</div>
                <p className="text-xs text-zinc-600 mt-1">{aiStatusText}</p>
              </div>
            </div>
            <div className="flex items-center gap-4">
              <button
                type="button"
                onClick={() => { store.fetchAiStatus(); store.fetchEngineInfo(); }}
                className="p-2 rounded-xl hover:bg-white/5 text-zinc-600 hover:text-zinc-300 transition-colors"
              >
                <RotateCcw size={14} />
              </button>
              <Toggle
                size="sm"
                checked={aiActive || aiToggling}
                disabled={aiToggling || store.aiEngineStatus?.state === "loading"}
                onChange={(checked) => handleAiToggle(checked)}
              />
            </div>
          </div>

          <ModelDownloadCard />

          {store.aiEngineStatus?.state === "loading" && (
            <div className="p-5 bg-amber-500/[0.06] rounded-xl border border-amber-500/20 space-y-3">
              <div className="flex items-center gap-3">
                <div className="flex gap-1.5">
                  <span className="w-2.5 h-2.5 rounded-full bg-amber-500 animate-ping" />
                  <span className="w-2.5 h-2.5 rounded-full bg-amber-500 animate-ping" style={{ animationDelay: "0.15s" }} />
                  <span className="w-2.5 h-2.5 rounded-full bg-amber-500 animate-ping" style={{ animationDelay: "0.3s" }} />
                </div>
                <span className="text-xs text-amber-400 font-bold flex items-center gap-2">
                  <Loader2 size={16} className="animate-spin" />
                  正在加载 874MB 模型文件...
                </span>
              </div>
              <p className="text-xs text-zinc-600 font-mono pl-10">
                已用时 {Math.floor(elapsed / 60)} 分 {elapsed % 60} 秒
              </p>
            </div>
          )}

          {store.aiEngineStatus?.state === "error" && (
            <div className="p-5 bg-red-500/[0.06] rounded-xl border border-red-500/20 space-y-3">
              <div className="flex items-center gap-3">
                <AlertTriangle size={16} className="text-red-400 shrink-0" />
                <span className="text-xs text-red-400 font-medium">{aiStatusText}</span>
              </div>
              <button
                type="button"
                onClick={() => { store.fetchAiStatus(); store.fetchEngineInfo(); }}
                className="flex items-center gap-2 text-xs text-red-300/70 hover:text-red-300 transition-colors font-bold uppercase tracking-wider ml-7"
              >
                <RotateCcw size={12} /> Retry
              </button>
            </div>
          )}

          {store.aiEngineStatus?.state === "ready" && store.aiEngineStatus?.model && (
            <div className="p-5 rounded-xl border border-white/[0.03] space-y-4" style={{ backgroundColor: "color-mix(in srgb, var(--bg-input) 90%, transparent)" }}>
              <div className="flex items-center justify-between">
                <div className="flex items-center gap-3">
                  <Brain size={14} className="text-purple-400" />
                  <span className="text-sm font-bold text-zinc-200">{store.aiEngineStatus.model.name}</span>
                </div>
                <span className="text-xs font-mono text-zinc-500 bg-white/[0.03] px-3 py-1 rounded-full">
                  v{store.aiEngineStatus.model.version}
                </span>
              </div>
              <div className="flex items-center justify-between text-xs text-zinc-600 border-t border-white/[0.03] pt-3 flex-wrap gap-4">
                <span className="font-mono">{formatSize(store.aiEngineStatus.model.size_mb)}</span>
                <div className="flex flex-wrap gap-1.5">
                  {store.aiEngineStatus.model.entity_types?.map((et) => (
                    <span key={et} className="px-2.5 py-0.5 rounded-full bg-purple-500/15 text-purple-300 text-[10px] font-bold uppercase tracking-wider">
                      {formatEntityType(et)}
                    </span>
                  ))}
                </div>
              </div>
            </div>
          )}

          <div className="p-5 rounded-xl border border-white/[0.03]" style={{ backgroundColor: "color-mix(in srgb, var(--bg-input) 90%, transparent)" }}>
            <p className="text-xs font-black text-zinc-600 uppercase tracking-widest mb-4">
              已载入模型 ({store.aiEngineStatus?.available_count || 0})
            </p>
            <div className="space-y-3">
              {preparedModels.map((model) => {
                const isActive = selectedModel === model.name;
                const isOnly = preparedModels.length <= 1;
                return (
                  <div
                    key={model.name}
                    onClick={async () => {
                      if (isActive && isOnly) {
                        if (!modelUnselectLock) {
                          setModelUnselectLock(true);
                          await message("当前仅有一个可用模型，至少需要选择一个模型才能运行 AI 识别", { title: "模型选择", kind: "info" });
                          setModelUnselectLock(false);
                        }
                        return;
                      }
                      setSelectedModel(model.name);
                    }}
                    className={cn(
                      "flex items-center gap-4 p-4 rounded-xl border transition-all duration-300 cursor-pointer",
                      isActive
                        ? "bg-purple-500/10 border-purple-500/30 shadow-[0_0_15px_rgba(168,85,247,0.08)]"
                        : "bg-white/[0.01] border-white/[0.04] hover:bg-white/[0.03] hover:border-white/[0.08]",
                    )}
                  >
                    <div
                      className={cn(
                        "w-5 h-5 rounded-full border-2 flex items-center justify-center transition-all shrink-0",
                        isActive ? "border-purple-400" : "border-zinc-700",
                      )}
                    >
                      {isActive && <div className="w-2.5 h-2.5 rounded-full bg-purple-400 shadow-[0_0_8px_rgba(168,85,247,0.5)]" />}
                    </div>
                    <div className="flex-1 min-w-0">
                      <div className="flex items-center gap-2">
                        <span className="text-xs font-bold text-zinc-300 truncate">{model.name}</span>
                        {isOnly && <Lock size={10} className="text-purple-400/60 shrink-0" />}
                      </div>
                      <p className="text-[10px] text-zinc-600 mt-0.5 truncate">{model.description}</p>
                    </div>
                    <span className="text-[10px] font-mono text-zinc-600 bg-white/[0.03] px-2.5 py-1 rounded-full">
                      {formatSize(model.size_mb)}
                    </span>
                    <div
                      className={cn(
                        "w-2 h-2 rounded-full shrink-0",
                        model.loaded || (isActive && store.aiEngineStatus?.state === 'ready')
                          ? "bg-emerald-500 shadow-[0_0_6px_rgba(16,185,129,0.5)]"
                          : "bg-zinc-700",
                      )}
                    />
                  </div>
                );
              })}
            </div>
          </div>

          {store.engineInfo?.recognizers && store.engineInfo.recognizers.length > 0 && (
            <div className="p-5 rounded-xl border border-white/[0.03]" style={{ backgroundColor: "color-mix(in srgb, var(--bg-input) 90%, transparent)" }}>
              <p className="text-xs font-black text-zinc-600 uppercase tracking-widest mb-4">已注册识别器</p>
              <div className="grid grid-cols-2 gap-2">
                {store.engineInfo.recognizers.map((rec) => (
                  <div key={rec} className="flex items-center gap-2.5 py-3 px-4 rounded-xl bg-white/[0.02] border border-white/[0.03]">
                    <div className={cn("w-2 h-2 rounded-full shrink-0", getRecognizerColor(rec))} />
                    <span className="text-xs font-bold text-zinc-400">{formatRecognizer(rec)}</span>
                  </div>
                ))}
              </div>
            </div>
          )}
        </Card>

        {/* ── About ── */}
        <Card className="lg:col-span-2 space-y-6">
          <CardHeader className="mb-8">
            <Info size={18} className="text-emerald-500/70" />
            <span>About</span>
          </CardHeader>

          <div className="grid grid-cols-1 md:grid-cols-3 gap-8">
            <div className="space-y-4">
              <div className="flex items-center gap-3 text-xs font-bold text-zinc-600 uppercase tracking-widest mb-4">
                <User size={16} className="text-emerald-500/60" />
                <span>Author</span>
              </div>
              <p className="text-lg font-bold text-amber-50/90">XiaoSheng</p>
              <div className="flex items-center gap-2">
                <span className="text-xs text-zinc-500">xiaosheng.tech@outlook.com</span>
                <button
                  type="button"
                  onClick={copyEmail}
                  className={cn("p-1.5 rounded-lg transition-all", emailCopied ? "bg-emerald-500/20 text-emerald-400" : "hover:bg-amber-500/10 text-zinc-600")}
                >
                  {emailCopied ? <Check size={14} className="text-emerald-400" /> : <Copy size={14} />}
                </button>
              </div>
            </div>

            <div className="space-y-4">
              <div className="flex items-center gap-3 text-xs font-bold text-zinc-600 uppercase tracking-widest mb-4">
                <Globe size={16} className="text-blue-500/60" />
                <span>Connect</span>
              </div>
              <a
                href="https://github.com/AiToByte/SafeMask"
                target="_blank"
                rel="noopener noreferrer"
                className="inline-flex items-center gap-3 bg-white/[0.02] hover:bg-white/[0.05] transition-colors p-3.5 rounded-xl border border-white/[0.04]"
              >
                <Github size={16} className="text-zinc-400" />
                <span className="text-sm text-zinc-300 font-medium">GitHub</span>
                <ExternalLink size={14} className="text-zinc-600" />
              </a>
            </div>

            <div className="space-y-4">
              <div className="flex items-center gap-3 text-xs font-bold text-zinc-600 uppercase tracking-widest mb-4">
                <Copyright size={16} className="text-amber-500/60" />
                <span>Project Info</span>
              </div>
              <div className="flex flex-wrap items-center gap-3">
                <span className="text-sm font-mono text-zinc-400 bg-white/[0.03] px-3 py-1 rounded-full">v{__APP_VERSION__}</span>
                <span className="text-xs text-zinc-500">MIT License</span>
              </div>
              <blockquote className="border-l-2 border-emerald-500/40 pl-4 py-2 bg-emerald-500/[0.03] rounded-r-xl">
                <p className="text-xs text-emerald-300/80 leading-relaxed">
                  SafeMask 核心脱敏逻辑完全离线运行，绝不上传任何原始敏感数据。
                </p>
              </blockquote>
            </div>
          </div>

          <div className="mt-8 pt-6 border-t border-white/[0.04]">
            <div className="p-5 rounded-2xl bg-red-500/[0.03] border border-red-500/10">
              <div className="flex items-center justify-between flex-wrap gap-4">
                <div className="flex items-center gap-3">
                  <Trash2 size={16} className="text-red-400/70" />
                  <div>
                    <div className="text-sm font-bold text-red-300/80">危险操作</div>
                    <div className="text-[10px] text-zinc-600 font-bold uppercase tracking-widest mt-0.5">清空所有审计记录，不可恢复</div>
                  </div>
                </div>
                <button
                  type="button"
                  onClick={async () => {
                    const confirmed = await confirm("此操作将永久删除所有审计记录，且不可恢复。确定要继续吗？", { title: "危险操作", kind: "warning" });
                    if (confirmed) { store.clearHistory(); play("CLICK"); }
                  }}
                  className="flex items-center gap-2 px-5 py-2.5 bg-red-500/10 border border-red-500/20 text-red-400 rounded-xl text-xs font-bold uppercase tracking-wider hover:bg-red-500/20 transition-all active:scale-95"
                >
                  <Trash2 size={14} />
                  销毁审计痕迹
                </button>
              </div>
            </div>
          </div>
        </Card>

      </div>


    </div>
  );
}

