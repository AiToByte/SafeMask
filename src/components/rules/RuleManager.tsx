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
        // Confirm dialog
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
    // Debounce: only run when the user has actually typed something
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

  // ── Render helpers ──

  const renderSearchBar = () => (
    <div className="relative">
      <Search
        size={14}
          className="absolute left-3.5 top-1/2 -translate-y-1/2 text-indigo-400/40 pointer-events-none"
      />
      <input
        value={searchQuery}
        onChange={(e) => setSearchQuery(e.target.value)}
        placeholder="快速检索..."
        className="w-full bg-[#08080a] border border-amber-500/10 rounded-2xl py-3 pl-10 pr-8 text-xs text-amber-50/80 outline-none transition-all duration-500 shadow-inner hover:border-white/20 focus:border-amber-500/40 focus:shadow-[0_0_15px_rgba(245,158,11,0.03),inset_0_2px_8px_rgba(0,0,0,0.6)]"
      />
    </div>
  );

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
      <div className="relative">
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

  // ── Main render ──

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
