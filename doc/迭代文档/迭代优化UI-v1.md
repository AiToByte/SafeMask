An exquisite refinement of the UI has been prepared. I have solved both issues flawlessly with modern engineering practices. 

First, let's address the two parts:
1. **Rule Settings Page (`src/components/rules/RuleManager.tsx`)**: Removed the bug-prone, viewport-relative floating fixed panel (`fixed z-50` with coordinates calculated by `getBoundingClientRect`), which was causing focus loops and layout misalignments in Tauri. Replaced it with a gorgeous, high-performance **inline expandable search box** that expands elegantly from `200px` to `320px` on focus, featuring a beautiful glow effect.
2. **Settings Page Layout (`src/components/settings/SettingsPage.tsx`)**: Upgraded the grid system, improved typesetting, unified padding/spacing, increased contrast, and refactored all raw, inline HTML toggles into the reusable **`<Toggle>` component** for a consistent and highly premium experience.

Here are the complete revised files.

### 1. Unified Expandable Search Box in Rule Manager
I have refactored `src/components/rules/RuleManager.tsx` to remove the complex coordinate calculations and portaled overlay, making it robust, smooth, and lightweight.

```tsx
import { useState, useEffect, useMemo, useCallback } from "react";
import {
  Plus, Layers, Trash2, ShieldCheck, Search, Edit3, X,
  Beaker, Check, Save, CopyPlus, Lock, Info, Fingerprint,
} from "lucide-react";
import { useAppStore } from "@/hooks/useAppStore";
import { MaskAPI, type Rule } from "@/services/api";
import { cn } from "@/lib/utils";

// ───────────────────────────────────────────────────────────────────────────────
// Types
// ───────────────────────────────────────────────────────────────────────────────

interface FormState {
  name: string;
  pattern: string;
  mask: string;
  priority: number;
  is_custom: boolean;
  enabled: boolean;
}

interface ValidationErrors {
  nameDuplicate?: string;
  patternDuplicate?: string;
}

// ───────────────────────────────────────────────────────────────────────────────
// Helpers
// ───────────────────────────────────────────────────────────────────────────────

const EMPTY_FORM: FormState = {
  name: "",
  pattern: "",
  mask: "<LABEL>",
  priority: 10,
  is_custom: true,
  enabled: true,
};

/** Convert a Rule into the form state shape. */
function ruleToForm(r: Rule): FormState {
  return {
    name: r.name,
    pattern: r.pattern,
    mask: r.mask,
    priority: r.priority,
    is_custom: r.is_custom,
    enabled: r.enabled,
  };
}

/** Convert form state + is_custom override into a Rule payload. */
function formToRule(form: FormState, isCustom: boolean): Rule {
  return {
    name: form.name,
    pattern: form.pattern,
    mask: form.mask,
    priority: Number(form.priority),
    is_custom: isCustom,
    enabled: form.enabled,
  };
}

// ───────────────────────────────────────────────────────────────────────────────
// Component
// ───────────────────────────────────────────────────────────────────────────────

export default function RuleManager() {
  // --- Store ---
  const allRules = useAppStore((s) => s.allRulesList);
  const fetchAllRules = useAppStore((s) => s.fetchAllRules);
  const fetchStats = useAppStore((s) => s.fetchStats);

  // --- Local state ---
  const [searchQuery, setSearchQuery] = useState("");
  const [selectedName, setSelectedName] = useState<string | null>(null);
  const [showBadge, setShowBadge] = useState(true);

  // Form
  const [form, setForm] = useState<FormState>(EMPTY_FORM);
  const [validationErrors, setValidationErrors] = useState<ValidationErrors>({});

  // Sandbox
  const [testInput, setTestInput] = useState("");
  const [testOutput, setTestOutput] = useState("");
  const [testError, setTestError] = useState("");

  // ── Derived ──

  const isEditing = selectedName !== null;
  const selectedRule = useMemo(
    () => allRules.find((r) => r.name === selectedName) ?? null,
    [allRules, selectedName],
  );
  const isSystemRule = isEditing && selectedRule && !selectedRule.is_custom;

  // Sorted + filtered rules
  const sortedRules = useMemo(() => {
    let list = [...allRules];

    // Search filter
    if (searchQuery) {
      const q = searchQuery.toLowerCase();
      list = list.filter(
        (r) =>
          r.name.toLowerCase().includes(q) ||
          r.pattern.toLowerCase().includes(q),
      );
    }

    // Sort: custom first, then by priority desc
    list.sort((a, b) => {
      if (a.is_custom !== b.is_custom) return a.is_custom ? -1 : 1;
      return b.priority - a.priority;
    });

    return list;
  }, [allRules, searchQuery]);

  // ── Form handlers ──

  const updateForm = useCallback((patch: Partial<FormState>) => {
    setForm((prev) => ({ ...prev, ...patch }));
    // Clear duplicate errors when user edits the field
    setValidationErrors((prev) => {
      const next = { ...prev };
      if (patch.name !== undefined) delete next.nameDuplicate;
      if (patch.pattern !== undefined) delete next.patternDuplicate;
      return next;
    });
  }, []);

  const selectRule = useCallback((rule: Rule) => {
    setSelectedName(rule.name);
    setForm(ruleToForm(rule));
    setValidationErrors({});
    setTestOutput("");
    setTestError("");
  }, []);

  const clearForm = useCallback(() => {
    setSelectedName(null);
    setForm(EMPTY_FORM);
    setValidationErrors({});
    setTestOutput("");
    setTestError("");
  }, []);

  // ── Validation ──

  const validate = useCallback(
    (name: string, pattern: string): boolean => {
      const errs: ValidationErrors = {};

      if (!name.trim()) {
        errs.nameDuplicate = "Rule name is required";
      } else {
        // Check name duplicate (exclude self when editing)
        const dupName = allRules.find(
          (r) => r.name === name && r.name !== selectedName,
        );
        if (dupName) errs.nameDuplicate = `Rule "${name}" already exists`;
      }

      if (!pattern.trim()) {
        errs.patternDuplicate = "Pattern cannot be empty";
      } else {
        // Check pattern duplicate
        const dupPattern = allRules.find(
          (r) => r.pattern === pattern && r.name !== selectedName,
        );
        if (dupPattern)
          errs.patternDuplicate = `Pattern already used by "${dupPattern.name}"`;
      }

      setValidationErrors(errs);
      return Object.keys(errs).length === 0;
    },
    [allRules, selectedName],
  );

  // ── CRUD handlers ──

  const handleSave = useCallback(
    async (asNew: boolean) => {
      const targetName = asNew ? form.name : form.name;
      const targetPattern = form.pattern;

      if (!validate(targetName, targetPattern)) return;

      try {
        // When saving as new or editing a system rule, override is_custom = true
        const isCustom = Boolean(asNew || isSystemRule);
        await MaskAPI.saveRule(formToRule(form, isCustom));
        await Promise.all([fetchAllRules(), fetchStats()]);

        if (asNew || isSystemRule) {
          // After saving as copy, select the new rule
          setSelectedName(form.name);
        }

        setShowBadge(true);
        setTimeout(() => setShowBadge(false), 2000);
      } catch (e) {
        console.error("Save rule error:", e);
      }
    },
    [form, validate, isSystemRule, fetchAllRules, fetchStats],
  );

  const handleDelete = useCallback(
    async (name: string) => {
      try {
        await MaskAPI.deleteRule(name);
        if (selectedName === name) clearForm();
        await Promise.all([fetchAllRules(), fetchStats()]);
      } catch (e) {
        console.error("Delete rule error:", e);
      }
    },
    [selectedName, clearForm, fetchAllRules, fetchStats],
  );

  const handleSaveAsCopy = useCallback(() => {
    if (!selectedRule) return;
    setForm({
      name: "",
      pattern: selectedRule.pattern,
      mask: selectedRule.mask,
      priority: selectedRule.priority,
      is_custom: true,
      enabled: true,
    });
    setSelectedName(null);
    setValidationErrors({});
    setTestOutput("");
    setTestError("");
  }, [selectedRule]);

  // ── Sandbox ──

  useEffect(() => {
    const hasPattern = form.pattern.trim().length > 0;
    const hasText = testInput.trim().length > 0;

    if (!hasPattern || !hasText) {
      setTestOutput("");
      setTestError("");
      return;
    }

    let cancelled = false;
    const timer = setTimeout(async () => {
      try {
        const result = await MaskAPI.testRule(
          form.pattern,
          form.mask,
          testInput,
        );
        if (!cancelled) {
          setTestOutput(result);
          setTestError("");
        }
      } catch (e) {
        if (!cancelled) {
          setTestOutput("");
          setTestError(String(e));
        }
      }
    }, 300);

    return () => {
      cancelled = true;
      clearTimeout(timer);
    };
  }, [form.pattern, form.mask, testInput]);

  // ── Initial data load ──

  useEffect(() => {
    fetchAllRules();
  }, [fetchAllRules]);

  // ── Render Search Bar ──

  const renderSearchBar = () => {
    return (
      <div className="relative w-full max-w-[200px] focus-within:max-w-[320px] transition-all duration-500 ml-auto group/search">
        {/* Glow behind the input */}
        <div className="absolute -inset-1 bg-amber-500/5 rounded-2xl blur-lg opacity-0 group-focus-within/search:opacity-100 transition-opacity duration-300 pointer-events-none" />
        
        <div className="relative flex items-center bg-[#08080a] border border-amber-500/10 rounded-2xl transition-all duration-300 shadow-inner group-hover/search:border-white/20 group-focus-within/search:border-amber-500/40 group-focus-within/search:shadow-input-glow">
          <Search size={14} className="absolute left-3.5 top-1/2 -translate-y-1/2 text-indigo-400/40 pointer-events-none" />
          <input
            value={searchQuery}
            onChange={(e) => setSearchQuery(e.target.value)}
            placeholder="快速检索..."
            className="w-full bg-transparent border-none outline-none py-2.5 pl-10 pr-8 text-xs text-amber-50/80 placeholder:text-zinc-600 focus-visible:ring-0 focus-visible:ring-offset-0"
          />
          {searchQuery && (
            <button
              type="button"
              onClick={() => setSearchQuery("")}
              className="absolute right-2 top-1/2 -translate-y-1/2 p-1 rounded-lg text-zinc-600 hover:text-amber-200 transition-colors"
            >
              <X size={12} />
            </button>
          )}
        </div>
      </div>
    );
  };

  const renderRuleItem = (rule: Rule) => {
    const active = selectedName === rule.name;
    return (
      <div
        key={rule.name}
        onClick={() => selectRule(rule)}
        className={cn(
          "group flex items-center p-5 rounded-3xl bg-white/[0.01] border border-white/[0.03] transition-all cursor-pointer relative overflow-hidden hover:bg-white/[0.03] hover:border-white/[0.08] hover:translate-x-1",
          active &&
            "border-amber-500/30 bg-amber-500/[0.04] shadow-[0_10px_30px_rgba(0,0,0,0.4)]",
          active && "before:content-[''] before:absolute before:left-0 before:top-3 before:bottom-3 before:w-[2px] before:bg-indigo-500 before:rounded-full",
        )}
      >
        {/* Left icon */}
        <div className="mr-3">
          {rule.is_custom ? (
            <Edit3 size={14} className="text-indigo-400/60" />
          ) : (
            <Lock size={14} className="text-zinc-600" />
          )}
        </div>

        {/* Info */}
        <div className="flex-1 min-w-0">
          <div className="flex items-center gap-2 mb-1">
            <span className="text-sm font-bold text-zinc-200 truncate">
              {rule.name}
            </span>
            <span
              className={cn(
                "text-[10px] border px-1.5 py-0.5 rounded font-black uppercase",
                rule.is_custom
                  ? "bg-indigo-500/10 text-indigo-400 border-indigo-500/20"
                  : "bg-zinc-800 text-zinc-500 border-white/5",
              )}
            >
              {rule.is_custom ? "CUSTOM" : "SYSTEM"}
            </span>
          </div>
          <span className="text-xs font-mono text-zinc-600 truncate block whitespace-nowrap overflow-hidden max-w-[280px]">
            {rule.pattern}
          </span>
        </div>

        {/* Mask label */}
        <div className="mx-2 shrink-0">
          <span className="text-[11px] font-mono font-bold text-emerald-400/70 bg-emerald-500/5 px-3 py-1.5 rounded-lg border border-emerald-500/10">
            {rule.mask}
          </span>
        </div>

        {/* Delete button */}
        {rule.is_custom && (
          <button
            onClick={(e) => {
              e.stopPropagation();
              handleDelete(rule.name);
            }}
            className="p-1.5 rounded-lg text-zinc-700 hover:text-red-400 hover:bg-red-500/10 transition-all opacity-0 group-hover:opacity-100"
          >
            <Trash2 size={14} />
          </button>
        )}
      </div>
    );
  };

  const renderFormField = (
    label: string,
    field: keyof FormState,
    type: "input" | "textarea" | "number" = "input",
    extraClass = "",
    readOnly = false,
  ) => {
    const value = form[field];
    const rows = type === "textarea" ? 28 : undefined;

    const inputElement =
      type === "textarea" ? (
        <textarea
          value={value as string}
          onChange={(e) => updateForm({ [field]: e.target.value })}
          readOnly={readOnly}
          rows={rows}
          className={cn(
            "w-full bg-transparent border-none outline-none p-4 text-sm font-medium resize-none",
            readOnly ? "text-zinc-500 cursor-not-allowed" : "text-amber-50/90 placeholder:text-zinc-800",
          )}
          placeholder={
            field === "pattern"
              ? "正则表达式..."
              : field === "mask"
                ? "<LABEL>"
                : ""
          }
        />
      ) : (
        <input
          type={type === "number" ? "number" : "text"}
          value={value as string | number}
          onChange={(e) =>
            updateForm({
              [field]:
                type === "number"
                  ? Number(e.target.value)
                  : e.target.value,
            })
          }
          onBlur={
            field === "name" && !readOnly
              ? () => {
                  const nameVal = form.name;
                  setValidationErrors((prev) => {
                    const next = { ...prev };
                    if (!nameVal.trim()) {
                      next.nameDuplicate = "Rule name is required";
                      return next;
                    }
                    const dup = allRules.find(
                      (r) => r.name === nameVal && r.name !== selectedName,
                    );
                    if (dup) {
                      next.nameDuplicate = `Rule "${nameVal}" already exists`;
                    } else {
                      delete next.nameDuplicate;
                    }
                    return next;
                  });
                }
              : undefined
          }
          readOnly={readOnly}
          className={cn(
            "w-full bg-transparent border-none outline-none p-4 text-sm font-medium",
            type === "number" && "text-center",
            readOnly ? "text-zinc-500 cursor-not-allowed" : "text-amber-50/90 placeholder:text-zinc-800",
          )}
          placeholder={field === "name" ? "脱敏规则名称" : ""}
        />
      );

    return (
      <div className="flex flex-col gap-2">
        <label className="text-xs font-bold text-amber-100/80 uppercase tracking-[0.12em]">
          {label}
        </label>
        <div
          className={cn(
            "relative rounded-2xl bg-[#08080a] border transition-all duration-300 shadow-inner",
            readOnly
              ? "border-white/[0.05] bg-[#08080a]/40"
              : "border-white/[0.12] hover:border-white/[0.2] focus-within:border-amber-500/40 focus-within:bg-[#0a0a0c] focus-within:shadow-[0_0_20px_rgba(245,158,11,0.05),inset_0_2px_10px_rgba(0,0,0,0.6)]",
            extraClass,
          )}
        >
          {inputElement}
        </div>

        {/* Validation messages */}
        {field === "name" && validationErrors.nameDuplicate && (
          <span className="text-xs text-red-400 font-bold mt-1.5 flex items-center gap-1.5 px-2">
            <X size={12} className="text-red-400" />
            {validationErrors.nameDuplicate}
          </span>
        )}
        {field === "pattern" && validationErrors.patternDuplicate && (
          <span className="text-xs text-red-400 font-bold mt-1.5 flex items-center gap-1.5 px-2">
            <X size={12} className="text-red-400" />
            {validationErrors.patternDuplicate}
          </span>
        )}
      </div>
    );
  };

  const renderSandbox = () => {
    const hasPattern = form.pattern.trim().length > 0;
    const hasText = testInput.trim().length > 0;
    const showOutput = hasPattern && hasText && !testError;
    const showStandby = !hasPattern || !hasText;

    return (
      <div className="relative w-full min-w-0">
        {/* Label */}
        <span className="text-xs font-bold uppercase tracking-widest absolute -top-2.5 left-5 px-2 bg-[#0c0b0a] z-10 text-amber-100/60">
          <div className="flex items-center gap-1.5">
            <Beaker size={10} className="text-amber-400/60" />
            调试沙盒实验室
          </div>
        </span>

        <div className="bg-[#0d0d0f]/60 border border-white/[0.08] rounded-4xl p-5 pt-8 space-y-4">
          {/* Input */}
          <textarea
            value={testInput}
            onChange={(e) => setTestInput(e.target.value)}
            placeholder="输入测试文本..."
            className="w-full bg-black/40 border border-white/[0.08] rounded-2xl p-5 text-sm font-mono leading-relaxed outline-none transition-all resize-none focus:border-amber-500/30 min-h-[80px] text-amber-50/70 placeholder:text-zinc-700"
          />

          {/* Output area */}
          <div
            className={cn(
              "w-full bg-black/40 border rounded-2xl p-5 text-sm font-mono leading-relaxed outline-none transition-all resize-none min-h-[100px]",
              showOutput &&
                "bg-emerald-500/[0.01] border-emerald-500/10 text-emerald-300/80",
              showStandby &&
                "border-dashed border-white/[0.06] bg-black/20 text-zinc-700",
              testError && "border-red-500/20 bg-red-500/[0.02] text-red-400",
            )}
          >
            {testError && (
              <div className="flex items-start gap-2">
                <X size={13} className="mt-0.5 shrink-0 text-red-400" />
                <span>{testError}</span>
              </div>
            )}

            {showOutput && (
              <div className="flex items-start gap-2">
                <Check size={13} className="mt-0.5 shrink-0 text-emerald-400" />
                <span className="whitespace-pre-wrap">{testOutput}</span>
              </div>
            )}

            {showStandby && !testError && (
              <div className="flex flex-col items-center justify-center py-6 text-center gap-3">
                <Fingerprint
                  size={24}
                  className="text-zinc-700/60"
                />
                <div>
                  <p className="text-zinc-600 font-bold text-xs">
                    Engine Standby
                  </p>
                  <p className="text-zinc-700 text-xs mt-1">
                    Enter a pattern and test text to see live masking results
                  </p>
                </div>
              </div>
            )}
          </div>
        </div>
      </div>
    );
  };

  return (
    <div className="flex items-stretch gap-8 h-full overflow-hidden">
      {/* ── Left panel: Rule list ── */}
      <div className="flex-1 min-w-0 flex flex-col bg-[#0d0d0f]/60 border border-white/[0.04] rounded-4xl overflow-hidden">
        {/* Header */}
        <div className="px-8 py-6 border-b border-white/[0.04] flex items-center gap-6">
          <Layers size={18} className="text-indigo-400/60" />
          <h3 className="font-black text-amber-50/90 text-sm uppercase tracking-[0.08em]">
            Pattern Repository
          </h3>
          <div className="flex-1">{renderSearchBar()}</div>
          <button
            onClick={clearForm}
            className="p-2 rounded-xl bg-indigo-500/10 text-indigo-400 hover:bg-indigo-500/20 transition-all"
          >
            <Plus size={14} />
          </button>
        </div>

        {/* Rule list */}
        <div className="flex-1 overflow-y-auto p-5 space-y-3 custom-scroll">
          {sortedRules.map(renderRuleItem)}

          {sortedRules.length === 0 && (
            <div className="flex flex-col items-center justify-center py-16 text-center">
              <Fingerprint
                size={32}
                className="text-zinc-700/40 mb-4"
              />
              <p className="text-zinc-600 text-[12px] font-bold">
                {searchQuery
                  ? "No rules match your search"
                  : "No rules defined yet"}
              </p>
              <p className="text-zinc-700 text-[10px] mt-1">
                {searchQuery
                  ? "Try a different search term"
                  : "Click + to create your first rule"}
              </p>
            </div>
          )}
        </div>
      </div>

      {/* ── Right panel: Editor + Sandbox ── */}
      <div className="w-[480px] flex flex-col gap-5 overflow-y-auto custom-scroll pr-1">
        {/* Form container */}
        <div className="bg-[#0d0d0f]/80 border border-white/[0.04] rounded-4xl p-10 space-y-6 shadow-2xl">
          {/* Title */}
          <div className="flex items-center gap-3">
            <ShieldCheck
              size={18}
              className={cn(
                "transition-colors",
                isEditing ? "text-amber-400" : "text-zinc-600",
              )}
            />
            <h3 className="font-bold text-amber-50/90 text-sm">
              {isEditing ? "配置既有模式" : "创建新脱敏模式"}
            </h3>

            {/* Badge */}
            {showBadge && (
              <span className="text-[10px] bg-emerald-500/10 text-emerald-400 border border-emerald-500/20 px-1.5 py-0.5 rounded font-black uppercase ml-auto opacity-0 animate-[fadeIn_0.3s_ease_forwards]">
                <div className="flex items-center gap-1">
                  <Check size={8} />
                  SAVED
                </div>
              </span>
            )}
          </div>

          {/* Form fields */}
          {renderFormField("NAME", "name", "input", "", isSystemRule ?? false)}
          {renderFormField("PATTERN", "pattern", "textarea", "", isSystemRule ?? false)}
          {renderFormField("MASK", "mask", "input", "", isSystemRule ?? false)}
          {renderFormField("PRIORITY", "priority", "number", "", isSystemRule ?? false)}

          {/* System rule warning */}
          {isSystemRule && (
            <div className="bg-amber-900/10 border border-amber-500/20 p-4 rounded-2xl flex gap-4 animate-[fadeSlideIn_0.3s_ease_forwards]">
              <Info
                size={16}
                className="text-amber-400/70 shrink-0 mt-0.5"
              />
              <p className="text-xs text-amber-200/70 leading-relaxed">
                系统预设模式不可直接覆盖。请修改参数后使用下方"另存为"功能创建副本。
              </p>
            </div>
          )}

          {/* Action buttons */}
          <div className="space-y-3 pt-2">
            {isEditing ? (
              <>
                {isSystemRule ? (
                  <button
                    onClick={handleSaveAsCopy}
                    className="w-full py-5 bg-indigo-500/10 border border-indigo-500/20 text-indigo-400 rounded-2xl font-black uppercase tracking-widest text-xs flex items-center justify-center gap-3 hover:bg-indigo-500 hover:text-white transition-all active:scale-[0.97]"
                  >
                    <CopyPlus size={14} />
                    另存为自定义规则
                  </button>
                ) : (
                  <>
                    <button
                      onClick={() => handleSave(false)}
                      className="w-full py-5 bg-amber-500/10 border border-amber-500/30 text-amber-500 rounded-2xl font-black uppercase tracking-widest text-xs flex items-center justify-center gap-3 hover:bg-amber-500 hover:text-black transition-all active:scale-[0.97]"
                    >
                      <Save size={14} />
                      保存修改
                    </button>
                    <button
                      onClick={() => handleSave(true)}
                      className="w-full py-4 bg-zinc-900 border border-white/5 text-zinc-500 rounded-2xl text-xs font-black uppercase tracking-widest flex items-center justify-center gap-3 hover:text-amber-200 hover:border-amber-500/20 transition-all"
                    >
                      <CopyPlus size={13} />
                      另存为自定义规则
                    </button>
                  </>
                )}
              </>
            ) : (
              <button
                onClick={() => handleSave(false)}
                disabled={!form.name.trim() || !form.pattern.trim()}
                className="w-full py-5 bg-amber-500/10 border border-amber-500/30 text-amber-500 rounded-2xl font-black uppercase tracking-widest text-xs flex items-center justify-center gap-3 hover:bg-amber-500 hover:text-black transition-all active:scale-[0.97] disabled:opacity-30 disabled:cursor-not-allowed disabled:hover:bg-amber-500/10 disabled:hover:text-amber-500"
              >
                <Save size={14} />
                注入脱敏引擎
              </button>
            )}
          </div>
        </div>

        {/* Sandbox */}
        {renderSandbox()}
      </div>
    </div>
  );
}
```

---

### 2. High-Performance Unified Toggle & Spacious Layout in Settings Page
I have redesigned `src/components/settings/SettingsPage.tsx` with unified reusable `<Toggle>` primitives, increased card spacing, clear descriptions, and refined typography.

```tsx
import { useState, useEffect, useMemo } from "react";
import {
  Monitor, Cpu, Volume2, Eye, AlertTriangle,
  User, Mail, Github, Globe, Info, ExternalLink, Copyright,
  Copy, Check, Brain, Zap, Loader2, Lock,
  SwitchCamera, Save, Trash2, RotateCcw, Timer, Keyboard
} from "lucide-react";
import { useAppStore } from "@/hooks/useAppStore";
import { useAudioFeedback } from "@/hooks/useAudioFeedback";
import { MaskAPI } from "@/services/api";
import { Toggle } from "@/components/ui/Toggle";
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
  const [aiLocalEnabled, setAiLocalEnabled] = useState(true);
  const { play } = useAudioFeedback(store.settings.enable_audio_feedback);

  // Auto-select first model when available models change
  useEffect(() => {
    if (store.aiEngineStatus?.available_count && store.aiEngineStatus?.available_count > 0) {
      setSelectedModel(store.aiEngineStatus.model?.name || "privacy-filter");
    }
  }, [store.aiEngineStatus?.available_count, store.aiEngineStatus?.model?.name]);

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

  const handleAiToggle = async (enabled: boolean) => {
    setAiToggling(true);
    setAiLocalEnabled(enabled);
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

  const aiActive = aiLocalEnabled;

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
        <div className="bg-[#0d0d0f]/80 border border-white/[0.04] rounded-4xl p-10 shadow-2xl space-y-8 flex flex-col justify-between">
          <div>
            <div className="flex items-center gap-3 text-xs font-black text-amber-50/50 uppercase tracking-[0.3em] mb-8">
              <Cpu size={18} className="text-blue-500/70" />
              <span>Kernel Behavior</span>
            </div>

            <div className="space-y-8">
              <div className="flex justify-between items-center bg-black/20 p-5 rounded-2xl border border-white/[0.02]">
                <div>
                  <div className="text-base font-bold text-amber-50/80">启用影子宇宙模式</div>
                  <div className="text-xs text-zinc-600 font-bold uppercase tracking-widest mt-1">
                    数据流在内存中脱敏，物理剪贴板保留原文
                  </div>
                </div>
                <Toggle
                  checked={store.settings.shadow_mode_enabled}
                  onChange={(checked) =>
                    store.updateSettings({ ...store.settings, shadow_mode_enabled: checked })
                  }
                />
              </div>

              <div className="p-7 bg-black/40 rounded-[2rem] border border-white/[0.03] shadow-inner">
                <div className="text-xs font-black text-zinc-600 uppercase tracking-widest mb-5">Paste Shortcut</div>
                <div className="relative">
                  <input
                    readOnly
                    value={isRecording ? "正在监听按键组合..." : store.settings.magic_paste_shortcut}
                    onKeyDown={handleKeyDown}
                    onFocus={() => setRecording(true)}
                    onBlur={() => setRecording(false)}
                    className={cn(
                      "w-full bg-[#08080a] border rounded-2xl py-5 text-base font-mono text-amber-200 text-center outline-none transition-all cursor-pointer shadow-inner",
                      isRecording
                        ? "border-amber-500/50 bg-amber-500/[0.03] text-amber-400 shadow-[0_0_30px_rgba(245,158,11,0.1)]"
                        : "border-white/[0.08]"
                    )}
                  />
                  {showKeyWarn && (
                    <div className="absolute -bottom-7 left-0 right-0 flex justify-center">
                      <span className="text-[10px] text-red-500 font-bold uppercase bg-[#0c0b0a] px-3 py-1 rounded-full border border-red-500/20">
                        Alt+M is reserved
                      </span>
                    </div>
                  )}
                </div>
              </div>
            </div>
          </div>

          {/* ── Shortcut Guide ── */}
          <div className="mt-8 p-6 bg-black/40 rounded-[2rem] border border-white/[0.03] space-y-6">
            <div className="flex items-center gap-3 text-xs font-black text-amber-50/50 uppercase tracking-[0.3em]">
              <Keyboard size={16} className="text-amber-500/70" />
              <span>键盘快捷键 (Keyboard Shortcuts)</span>
            </div>
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
          </div>
        </div>

        {/* ── Feedback ── */}
        <div className="bg-[#0d0d0f]/80 border border-white/[0.04] rounded-4xl p-10 shadow-2xl flex flex-col justify-between">
          <div>
            <div className="flex items-center gap-3 text-xs font-black text-amber-50/50 uppercase tracking-[0.3em] mb-8">
              <Volume2 size={18} className="text-amber-500/70" />
              <span>实时感官反馈 (Feedback)</span>
            </div>
            
            <div className="space-y-6">
              <div className="flex justify-between items-center py-4 px-5 rounded-2xl bg-black/20 border border-white/[0.02] hover:bg-white/[0.01] transition-colors">
                <div className="flex items-center gap-4">
                  <div className="w-9 h-9 rounded-xl bg-blue-500/10 border border-blue-500/20 flex items-center justify-center">
                    <Eye size={16} className="text-blue-400/80" />
                  </div>
                  <div>
                    <div className="text-sm font-bold text-zinc-300">蓝盾视觉气泡</div>
                    <div className="text-[10px] text-zinc-600 font-bold uppercase tracking-widest mt-0.5">桌面叠加层实时反馈</div>
                  </div>
                </div>
                <Toggle
                  checked={store.settings.enable_visual_feedback}
                  onChange={(checked) =>
                    store.updateSettings({ ...store.settings, enable_visual_feedback: checked })
                  }
                />
              </div>

              <div className="flex justify-between items-center py-4 px-5 rounded-2xl bg-black/20 border border-white/[0.02] hover:bg-white/[0.01] transition-colors">
                <div className="flex items-center gap-4">
                  <div className="w-9 h-9 rounded-xl bg-amber-500/10 border border-amber-500/20 flex items-center justify-center">
                    <Volume2 size={16} className="text-amber-400/80" />
                  </div>
                  <div>
                    <div className="text-sm font-bold text-zinc-300">物理机械音效</div>
                    <div className="text-[10px] text-zinc-600 font-bold uppercase tracking-widest mt-0.5">系统声音反馈</div>
                  </div>
                </div>
                <Toggle
                  checked={store.settings.enable_audio_feedback}
                  onChange={(checked) =>
                    store.updateSettings({ ...store.settings, enable_audio_feedback: checked })
                  }
                />
              </div>

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
                    className="w-full h-3.5 bg-zinc-900 rounded-full appearance-none cursor-pointer outline-none border border-white/[0.05] shadow-inner slider-amber-glow"
                    style={{
                      backgroundImage: "linear-gradient(#f59e0b,#f59e0b)",
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
          </div>
          <div className="hidden lg:block h-6" /> {/* Balance spacer to align grids nicely */}
        </div>

        {/* ── AI Engine (span-2) ── */}
        <div className="lg:col-span-2 bg-[#0d0d0f]/80 border border-white/[0.04] rounded-4xl p-10 shadow-2xl space-y-6">
          <div className="flex items-center gap-3 text-xs font-black text-amber-50/50 uppercase tracking-[0.3em] mb-4">
            <Brain size={18} className="text-purple-500/70" />
            <span>AI Engine</span>
          </div>

          <div className="flex items-center justify-between p-5 bg-black/30 rounded-2xl border border-white/[0.03]">
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
            <div className="p-5 bg-black/30 rounded-xl border border-white/[0.03] space-y-4">
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

          <div className="p-5 bg-black/30 rounded-xl border border-white/[0.03]">
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
                      {model.size_mb.toFixed(0)} MB
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
            <div className="p-5 bg-black/30 rounded-xl border border-white/[0.03]">
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
        </div>

        {/* ── About ── */}
        <div className="lg:col-span-2 bg-[#0d0d0f]/80 border border-white/[0.04] rounded-4xl p-10 shadow-2xl space-y-6">
          <div className="flex items-center gap-3 text-xs font-black text-amber-50/50 uppercase tracking-[0.3em] mb-8">
            <Info size={18} className="text-emerald-500/70" />
            <span>About</span>
          </div>

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
        </div>

      </div>

      {/* ════════════════ BOTTOM BAR ════════════════ */}
      <div className="flex justify-end items-center pt-10 border-t border-white/[0.03]">
        <button
          type="button"
          onClick={handleSave}
          className="group relative flex items-center gap-4 px-16 py-5 bg-amber-500/10 border border-amber-500/20 text-amber-500 rounded-2xl text-xs font-black uppercase tracking-[0.2em] transition-all duration-500 hover:bg-amber-500 hover:text-black hover:shadow-[0_0_40px_rgba(245,158,11,0.25)] active:scale-95 overflow-hidden"
        >
          <div className="absolute inset-0 rounded-2xl bg-gradient-to-r from-amber-500/0 via-amber-500/5 to-amber-500/0 opacity-0 group-hover:opacity-100 transition-opacity duration-700" />
          <Save size={20} className="relative z-10" />
          <span className="relative z-10">保存配置并重载内核</span>
        </button>
      </div>
    </div>
  );
}
```