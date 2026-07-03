import { useState, useEffect, useMemo, useRef, useCallback } from "react";
import { motion } from "framer-motion";
import {
  Shield, Keyboard, Bell, Timer, RotateCcw,
  Save, Trash2, Monitor, Cpu, Volume2, Eye, AlertTriangle,
  User, Mail, Github, Globe, Info, ExternalLink, Copyright,
  Copy, Check, Brain, Zap, Power, PowerOff, Loader2, Lock,
} from "lucide-react";
import { useAppStore } from "@/hooks/useAppStore";
import { useAudioFeedback } from "@/hooks/useAudioFeedback";
import { MaskAPI } from "@/services/api";
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
  const [showKeyWarn, setShowWarn] = useState(false);
  const [elapsed, setElapsed] = useState(0);
  const [emailCopied, setEmail] = useState(false);
  const [selectedModel, setSelectedModel] = useState<string | null>(null);
  const [modelUnselectLock, setModelUnselectLock] = useState(false);
  const [aiToggling, setAiToggling] = useState(false);
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

  // ── Handlers ──

  const handleAiToggle = async (enabled: boolean) => {
    setAiToggling(true);
    // Update local state immediately for responsive UI
    setAiLocalEnabled(enabled);
    try {
      const result = await store.toggleAiEngine(enabled);
      if (enabled) {
        play("ASCEND");
        await message("AI 引擎已启动，正在加载模型...", { title: "AI 引擎", kind: "info" });
      } else {
        play("DESCEND");
        await message("AI 识别已关闭，将使用规则引擎进行脱敏", { title: "AI 引擎", kind: "info" });
      }
    } catch (e) {
      // Revert local state on error
      setAiLocalEnabled(!enabled);
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

  const handleSave = async () => {
    await MaskAPI.updateSettings(store.settings);
    play("ASCEND");
  };

  const handleKeyDown = (e: React.KeyboardEvent<HTMLInputElement>) => {
    if (!isRecording) return;
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
      store.updateSettings({ ...store.settings, magic_paste_shortcut: fs });
      setRecording(false);
      play("RECORD");
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

  // Track AI enabled/disabled locally — the backend's ai_enabled flag is
  // separate from model loading state (ai_status stays "ready" even when off).
  // Default to true (enabled) on first render.
  const [aiLocalEnabled, setAiLocalEnabled] = useState(true);
  
  const aiActive = aiLocalEnabled;
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
    <motion.div
      initial={{ opacity: 0, y: 20 }}
      animate={{ opacity: 1, y: 0 }}
      transition={{ duration: 0.5, ease: [0.16, 1, 0.3, 1] }}
      className="max-w-5xl mx-auto space-y-10 pb-16"
    >
      {/* ════════════════ HEADER ════════════════ */}
      <div className="flex items-center gap-6 mb-10 px-2">
        <div className="w-14 h-14 rounded-2xl bg-[#141210] border border-amber-500/10 flex items-center justify-center shadow-2xl">
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

      {/* ════════════════ GRID ════════════════ */}
      <div className="grid grid-cols-1 lg:grid-cols-2 gap-8">

        {/* ── Kernel Behaviour ── */}
<div className="bg-[#0d0d0f]/80 border border-white/[0.04] rounded-4xl p-10 shadow-2xl">
  <div className="flex items-center gap-3 text-xs font-black text-amber-50/50 uppercase tracking-[0.3em] mb-8">
    <Cpu size={18} className="text-blue-500/70" />
    <span>Kernel Behavior</span>
  </div>
  <div className="space-y-8">
    <div className="flex justify-between items-center">
      <div>
        <div className="text-base font-bold text-amber-50/80">启用影子宇宙模式</div>
        <div className="text-xs text-zinc-600 font-bold uppercase tracking-widest">数据流在内存中脱敏，物理剪贴板保留原文</div>
      </div>
      <label className="relative w-14 h-7 cursor-pointer">
        <input
          type="checkbox"
          checked={store.settings.shadow_mode_enabled}
          onChange={() => store.updateSettings({ ...store.settings, shadow_mode_enabled: !store.settings.shadow_mode_enabled })}
          className="opacity-0 w-0 h-0 absolute"
        />
        <div className={cn("absolute inset-0 rounded-full transition-all duration-500 border border-white/[0.05]", store.settings.shadow_mode_enabled ? "bg-blue-600/80 border-blue-400/20" : "bg-zinc-800")}>
          <div className={cn("absolute h-5 w-5 left-1 bottom-1 bg-white rounded-full shadow-lg transition-all duration-500", store.settings.shadow_mode_enabled && "translate-x-7")} />
        </div>
      </label>
    </div>
    <div className="p-7 bg-black/40 rounded-[2rem] border border-white/[0.03] shadow-inner">
      <div className="text-xs font-black text-zinc-600 uppercase tracking-widest mb-5">Paste Shortcut</div>
      <div className="relative">
        <input
          readOnly
          value={isRecording ? "正在监听按键组合..." : store.settings.magic_paste_shortcut}
          onKeyDown={handleKeyDown}
          onFocus={() => { setRecording(true); }}
          onBlur={() => setRecording(false)}
          className={cn("w-full bg-[#08080a] border rounded-2xl py-5 text-base font-mono text-amber-200 text-center outline-none transition-all cursor-pointer shadow-inner", isRecording ? "border-amber-500/50 bg-amber-500/[0.03] text-amber-400 shadow-[0_0_30px_rgba(245,158,11,0.1)]" : "border-white/[0.08]")}
        />
        {showKeyWarn && (
          <div className="absolute -bottom-7 left-0 right-0 flex justify-center">
            <span className="text-[10px] text-red-500 font-bold uppercase bg-[#0c0b0a] px-3 py-1 rounded-full border border-red-500/20">Alt+M is reserved</span>
          </div>
        )}
      </div>
    </div>
  </div>
</div>


        {/* ── Feedback ── */}
<div className="bg-[#0d0d0f]/80 border border-white/[0.04] rounded-4xl p-10 shadow-2xl">
  <div className="flex items-center gap-3 text-xs font-black text-amber-50/50 uppercase tracking-[0.3em] mb-8">
    <Volume2 size={18} className="text-amber-500/70" />
    <span>实时感官反馈 (Feedback)</span>
  </div>
  <div className="space-y-6">
    <div className="flex justify-between items-center py-2 px-1 rounded-xl hover:bg-white/[0.01] transition-colors">
      <div className="flex items-center gap-4">
        <div className="w-9 h-9 rounded-xl bg-blue-500/10 border border-blue-500/20 flex items-center justify-center">
          <Eye size={16} className="text-blue-400/80" />
        </div>
        <div>
          <div className="text-sm font-bold text-zinc-300">蓝盾视觉气泡</div>
          <div className="text-[10px] text-zinc-600 font-bold uppercase tracking-widest mt-0.5">桌面叠加层实时反馈</div>
        </div>
      </div>
      <label className="relative w-11 h-6 cursor-pointer">
        <input
          type="checkbox"
          checked={store.settings.enable_visual_feedback}
          onChange={() => store.updateSettings({ ...store.settings, enable_visual_feedback: !store.settings.enable_visual_feedback })}
          className="opacity-0 w-0 h-0 absolute"
        />
        <div className={cn("absolute inset-0 rounded-full transition-all duration-500 border border-white/[0.05]", store.settings.enable_visual_feedback ? "bg-blue-600/80 border-blue-400/20 shadow-[0_0_8px_rgba(59,130,246,0.2)]" : "bg-zinc-800")}>
          <div className={cn("absolute h-4 w-4 left-1 bottom-1 bg-white rounded-full shadow-lg transition-all duration-500", store.settings.enable_visual_feedback && "translate-x-5")} />
        </div>
      </label>
    </div>
    <div className="flex justify-between items-center py-2 px-1 rounded-xl hover:bg-white/[0.01] transition-colors">
      <div className="flex items-center gap-4">
        <div className="w-9 h-9 rounded-xl bg-amber-500/10 border border-amber-500/20 flex items-center justify-center">
          <Volume2 size={16} className="text-amber-400/80" />
        </div>
        <div>
          <div className="text-sm font-bold text-zinc-300">物理机械音效</div>
          <div className="text-[10px] text-zinc-600 font-bold uppercase tracking-widest mt-0.5">系统声音反馈</div>
        </div>
      </div>
      <label className="relative w-11 h-6 cursor-pointer">
        <input
          type="checkbox"
          checked={store.settings.enable_audio_feedback}
          onChange={() => store.updateSettings({ ...store.settings, enable_audio_feedback: !store.settings.enable_audio_feedback })}
          className="opacity-0 w-0 h-0 absolute"
        />
        <div className={cn("absolute inset-0 rounded-full transition-all duration-500 border border-white/[0.05]", store.settings.enable_audio_feedback ? "bg-amber-600/80 border-amber-400/20 shadow-[0_0_8px_rgba(245,158,11,0.2)]" : "bg-zinc-800")}>
          <div className={cn("absolute h-4 w-4 left-1 bottom-1 bg-white rounded-full shadow-lg transition-all duration-500", store.settings.enable_audio_feedback && "translate-x-5")} />
        </div>
      </label>
    </div>
    <div className="pt-6 border-t border-white/[0.03] space-y-5">
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
          className="w-full h-3.5 bg-zinc-900 rounded-full appearance-none cursor-pointer outline-none border border-white/[0.05] shadow-inner slider-amber-glow"
          style={{ backgroundImage: "linear-gradient(#f59e0b,#f59e0b)", backgroundSize: sliderProgress + "% 100%", backgroundRepeat: "no-repeat" }}
        />
        {/* Tick marks */}
        <div className="flex justify-between px-0.5 mt-2">
          {[50, 200, 400, 600, 800].map((ms) => (
            <span
              key={ms}
              className={cn(
                "text-[8px] font-mono transition-colors",
                store.settings.paste_delay_ms === ms ? "text-amber-500/60" : "text-zinc-800",
              )}
            >
              {ms}
            </span>
          ))}
        </div>
      </div>
    </div>
  </div>
</div>


        {/* ── AI Engine (span-2) ── */}
<div className="lg:col-span-2 bg-[#0d0d0f]/80 border border-white/[0.04] rounded-4xl p-10 shadow-2xl">
  <div className="flex items-center gap-3 text-xs font-black text-amber-50/50 uppercase tracking-[0.3em] mb-8">
    <Brain size={18} className="text-purple-500/70" />
    <span>AI Engine</span>
  </div>

  {/* Status row */}
  <div className="flex items-center justify-between p-4 bg-black/30 rounded-xl border border-white/[0.03]">
    <div className="flex items-center gap-3">
      <div className={cn("w-3 h-3 rounded-full transition-colors", aiDot)} />
      <div>
        <div className="text-xs text-zinc-300 font-medium">Status</div>
        <p className="text-[10px] text-zinc-600 mt-0.5">{aiStatusText}</p>
      </div>
    </div>
    <div className="flex items-center gap-3">
      <button
        onClick={() => { store.fetchAiStatus(); store.fetchEngineInfo(); }}
        className="p-1.5 rounded-lg hover:bg-white/5 transition-colors"
      >
        <RotateCcw size={12} className="text-zinc-600" />
      </button>
      <div
        className={cn(
          "relative w-11 h-6 cursor-pointer select-none rounded-full transition-all duration-500 border",
          (aiActive || aiToggling) ? "bg-blue-600/80 border-blue-400/20" : "bg-zinc-800 border-white/[0.05]",
        )}
        onClick={() => {
          if (!aiToggling && store.aiEngineStatus?.state !== "loading") {
            handleAiToggle(!aiActive);
          }
        }}
      >
        <div className={cn("absolute h-4 w-4 left-1 bottom-1 bg-white rounded-full shadow-lg transition-all duration-500", (aiActive || aiToggling) && "translate-x-5")} />
      </div>
    </div>
  </div>

  {/* Loading state */}
  {store.aiEngineStatus?.state === "loading" && (
    <div className="mt-4 p-5 bg-amber-500/[0.06] rounded-xl border border-amber-500/20 space-y-3">
      <div className="flex items-center gap-3">
        <div className="flex gap-1">
          <span className="w-2 h-2 rounded-full bg-amber-500 animate-ping" />
          <span className="w-2 h-2 rounded-full bg-amber-500 animate-ping" style={{ animationDelay: "0.15s" }} />
          <span className="w-2 h-2 rounded-full bg-amber-500 animate-ping" style={{ animationDelay: "0.3s" }} />
        </div>
        <span className="text-xs text-amber-400 font-medium flex items-center gap-2">
          <Loader2 size={16} className="animate-spin" />
          正在加载 874MB 模型文件...
        </span>
      </div>
      <p className="text-xs text-zinc-600 font-mono pl-8">
        已用时 {Math.floor(elapsed / 60)} 分 {elapsed % 60} 秒
      </p>
    </div>
  )}

  {/* Error state */}
  {store.aiEngineStatus?.state === "error" && (
    <div className="mt-4 p-5 bg-red-500/[0.06] rounded-xl border border-red-500/20 space-y-3">
      <div className="flex items-center gap-3">
        <AlertTriangle size={16} className="text-red-400 shrink-0" />
        <span className="text-xs text-red-400 font-medium">{aiStatusText}</span>
      </div>
      <button
        onClick={() => { store.fetchAiStatus(); store.fetchEngineInfo(); }}
        className="flex items-center gap-2 text-xs text-red-300/70 hover:text-red-300 transition-colors font-bold uppercase tracking-wider ml-7"
      >
        <RotateCcw size={12} /> Retry
      </button>
    </div>
  )}

  {/* Loaded model info */}
  {store.aiEngineStatus?.state === "ready" && store.aiEngineStatus?.model && (
    <div className="mt-4 p-5 bg-black/30 rounded-xl border border-white/[0.03] space-y-4">
      <div className="flex items-center justify-between">
        <div className="flex items-center gap-3">
          <Brain size={14} className="text-purple-400" />
          <span className="text-sm font-bold text-zinc-200">{store.aiEngineStatus.model.name}</span>
        </div>
        <span className="text-xs font-mono text-zinc-500 bg-white/[0.03] px-3 py-1 rounded-full">
          v{store.aiEngineStatus.model.version}
        </span>
      </div>
      <div className="flex items-center justify-between text-xs text-zinc-600 border-t border-white/[0.03] pt-3">
        <span className="font-mono">{store.aiEngineStatus.model.size_mb.toFixed(0)} MB</span>
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

  {/* Available models list */}
  <div className="mt-4 p-5 bg-black/30 rounded-xl border border-white/[0.03]">
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
              "flex items-center gap-4 p-3.5 rounded-xl border transition-all duration-300 cursor-pointer",
              isActive
                ? "bg-purple-500/10 border-purple-500/30 shadow-[0_0_15px_rgba(168,85,247,0.08)]"
                : "bg-white/[0.01] border-white/[0.04] hover:bg-white/[0.03] hover:border-white/[0.08]",
            )}
          >
            {/* Radio indicator */}
            <div
              className={cn(
                "w-5 h-5 rounded-full border-2 flex items-center justify-center transition-all shrink-0",
                isActive ? "border-purple-400" : "border-zinc-700",
              )}
            >
              {isActive && <div className="w-2.5 h-2.5 rounded-full bg-purple-400 shadow-[0_0_8px_rgba(168,85,247,0.5)]" />}
            </div>
            {/* Info */}
            <div className="flex-1 min-w-0">
              <div className="flex items-center gap-2">
                <span className="text-xs font-bold text-zinc-300 truncate">{model.name}</span>
                {isOnly && (
                  <Lock size={10} className="text-purple-400/60 shrink-0" />
                )}
              </div>
              <p className="text-[10px] text-zinc-600 mt-0.5 truncate">{model.description}</p>
            </div>
            {/* Size */}
            <span className="text-[10px] font-mono text-zinc-600 bg-white/[0.03] px-2.5 py-1 rounded-full">
              {model.size_mb.toFixed(0)} MB
            </span>
            {/* Status dot */}
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

  {/* Recognizer grid */}
  {store.engineInfo?.recognizers && store.engineInfo.recognizers.length > 0 && (
    <div className="mt-4 p-5 bg-black/30 rounded-xl border border-white/[0.03]">
      <p className="text-xs font-black text-zinc-600 uppercase tracking-widest mb-4">已注册识别器</p>
      <div className="grid grid-cols-2 gap-2">
        {store.engineInfo.recognizers.map((rec) => (
          <div key={rec} className="flex items-center gap-2.5 py-2 px-3 rounded-xl bg-white/[0.02] border border-white/[0.03]">
            <div className={cn("w-2 h-2 rounded-full shrink-0", getRecognizerColor(rec))} />
            <span className="text-xs font-bold text-zinc-400">{formatRecognizer(rec)}</span>
          </div>
        ))}
      </div>
    </div>
  )}
</div>


        {/* ── About (span-2) ── */}
<div className="lg:col-span-2 bg-[#0d0d0f]/80 border border-white/[0.04] rounded-4xl p-10 shadow-2xl">
  <div className="flex items-center gap-3 text-xs font-black text-amber-50/50 uppercase tracking-[0.3em] mb-8">
    <Info size={18} className="text-emerald-500/70" />
    <span>About</span>
  </div>

  {/* 3-column grid: Author, Connect, Project Info */}
  <div className="grid grid-cols-1 md:grid-cols-3 gap-8">
    {/* Author */}
    <div className="space-y-4">
      <div className="flex items-center gap-3 text-xs font-bold text-zinc-600 uppercase tracking-widest mb-4">
        <User size={16} className="text-emerald-500/60" />
        <span>Author</span>
      </div>
      <div>
        <p className="text-lg font-bold text-amber-50/90">XiaoSheng</p>
      </div>
      <div className="flex items-center gap-2">
        <span className="text-xs text-zinc-500">xiaosheng.tech@outlook.com</span>
        <button
          onClick={copyEmail}
          className={cn("p-1.5 rounded-lg transition-all", emailCopied ? "bg-emerald-500/20 text-emerald-400" : "hover:bg-amber-500/10 text-zinc-600")}
        >
          {emailCopied ? <Check size={14} className="text-emerald-400" /> : <Copy size={14} />}
        </button>
      </div>
    </div>

    {/* Connect */}
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

    {/* Project Info */}
    <div className="space-y-4">
      <div className="flex items-center gap-3 text-xs font-bold text-zinc-600 uppercase tracking-widest mb-4">
        <Copyright size={16} className="text-amber-500/60" />
        <span>Project Info</span>
      </div>
      <div className="flex flex-wrap items-center gap-3">
        <span className="text-sm font-mono text-zinc-400 bg-white/[0.03] px-3 py-1 rounded-full">v1.2.4</span>
        <span className="text-xs text-zinc-500">MIT License</span>
      </div>
      <blockquote className="border-l-2 border-emerald-500/40 pl-4 py-2 bg-emerald-500/[0.03] rounded-r-xl">
        <p className="text-xs text-emerald-300/80 leading-relaxed">
          SafeMask 核心脱敏逻辑完全离线运行，绝不上传任何原始敏感数据。
        </p>
      </blockquote>
    </div>
  </div>

  {/* Danger Zone */}
  <div className="mt-8 pt-6 border-t border-white/[0.04]">
    <div className="p-5 rounded-2xl bg-red-500/[0.03] border border-red-500/10">
      <div className="flex items-center justify-between">
        <div className="flex items-center gap-3">
          <Trash2 size={16} className="text-red-400/70" />
          <div>
            <div className="text-sm font-bold text-red-300/80">危险操作</div>
            <div className="text-[10px] text-zinc-600 font-bold uppercase tracking-widest mt-0.5">清空所有审计记录，不可恢复</div>
          </div>
        </div>
        <button
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
</div>


      </div>

      {/* ════════════════ BOTTOM BAR ════════════════ */}
<div className="flex justify-end items-center pt-10 border-t border-white/[0.03]">
  <button
    onClick={handleSave}
    className="group relative flex items-center gap-4 px-16 py-5 bg-amber-500/10 border border-amber-500/20 text-amber-500 rounded-2xl text-xs font-black uppercase tracking-[0.2em] transition-all duration-500 hover:bg-amber-500 hover:text-black hover:shadow-[0_0_40px_rgba(245,158,11,0.25)] active:scale-95 overflow-hidden"
  >
    <div className="absolute inset-0 rounded-2xl bg-gradient-to-r from-amber-500/0 via-amber-500/5 to-amber-500/0 opacity-0 group-hover:opacity-100 transition-opacity duration-700" />
    <Save size={20} className="relative z-10" />
    <span className="relative z-10">保存配置并重载内核</span>
  </button>
</div>


    </motion.div>
  );
}

