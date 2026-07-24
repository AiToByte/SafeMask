import { useState, useEffect, useMemo, useCallback } from "react";
import {
  Plus, Layers, Trash2, ShieldCheck, Search, Edit3, X,
  Beaker, Check, Save, CopyPlus, Lock, Info, Fingerprint,
  Upload, Download, FileDown,
} from "lucide-react";
import { open, save, message } from "@tauri-apps/plugin-dialog";
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
  mask: "",
  priority: 10,
  is_custom: true,
  enabled: true,
};

/** Convert a Rule into the form state shape (strip bracket wrappers for editing). */
function ruleToForm(r: Rule): FormState {
  return {
    name: r.name,
    pattern: r.pattern,
    mask: r.mask.replace(/[<>\[\]]/g, ""),
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

/** Wrap bare mask label with configured brackets */
const wrapMask = (bare: string, style: string) => {
  return style === "square" ? `[${bare}]` : `<${bare}>`;
};

// ───────────────────────────────────────────────────────────────────────────────
// Component
// ───────────────────────────────────────────────────────────────────────────────

export default function RuleManager() {
  // --- Store ---
  const allRules = useAppStore((s) => s.allRulesList);
  const fetchAllRules = useAppStore((s) => s.fetchAllRules);
  const fetchStats = useAppStore((s) => s.fetchStats);
  const settings = useAppStore((s) => s.settings);

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
        const ruleToSave = formToRule(form, isCustom);
        ruleToSave.mask = wrapMask(ruleToSave.mask, settings.mask_wrapper_style);
        await MaskAPI.saveRule(ruleToSave);
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
      mask: selectedRule.mask.replace(/[<>\[\]]/g, ""),
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
        const testMask = wrapMask(form.mask, settings.mask_wrapper_style);
        const result = await MaskAPI.testRule(
          form.pattern,
          testMask,
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

  const renderSearchBar = () => {
    return (
      <div className="relative w-full min-w-0 group/search">
        <div
          className={cn(
            "relative flex items-center gap-2.5 rounded-[14px] border px-3.5",
            "transition-all duration-300 ease-out",
            "shadow-[inset_0_1px_0_rgba(255,255,255,0.04)]",
            "group-hover/search:border-[color:var(--border-default)]",
            "group-focus-within/search:border-[color:var(--accent-border)]",
            "group-focus-within/search:shadow-[0_0_0_3px_rgba(var(--accent-rgb),0.12),inset_0_1px_0_rgba(255,255,255,0.06)]",
          )}
          style={{
            backgroundColor: "var(--bg-input)",
            borderColor: "var(--border-subtle)",
          }}
        >
          <Search
            size={15}
            className="shrink-0 text-zinc-500 transition-colors duration-300 group-focus-within/search:text-[color:var(--accent)]"
          />
          <input
            value={searchQuery}
            onChange={(e) => setSearchQuery(e.target.value)}
            placeholder="搜索规则名称或正则 pattern…"
            className="min-w-0 flex-1 bg-transparent border-none outline-none py-[11px] text-[13px] tracking-tight text-[color:var(--text-primary)] placeholder:text-zinc-600 focus-visible:ring-0 focus-visible:ring-offset-0"
          />
          {searchQuery ? (
            <button
              type="button"
              onClick={() => setSearchQuery("")}
              className="shrink-0 w-6 h-6 rounded-full flex items-center justify-center text-zinc-500 hover:text-[color:var(--text-primary)] transition-all duration-200"
              style={{ backgroundColor: "color-mix(in srgb, var(--text-primary) 8%, transparent)" }}
              title="清空"
            >
              <X size={12} strokeWidth={2.5} />
            </button>
          ) : (
            <kbd
              className="hidden md:inline-flex items-center h-5 px-1.5 rounded-md text-[10px] font-semibold text-zinc-600 border"
              style={{
                borderColor: "var(--border-subtle)",
                backgroundColor: "color-mix(in srgb, var(--bg-elevated) 80%, transparent)",
              }}
            >
              ⌕
            </kbd>
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
          "group flex items-center gap-3 px-4 py-3.5 rounded-2xl border transition-all duration-200 cursor-pointer relative overflow-hidden",
          "hover:bg-white/[0.03] hover:border-white/[0.08]",
          active
            ? "border-[color:var(--accent-border)] bg-[color:var(--accent-dim)] shadow-[0_8px_24px_-16px_rgba(var(--accent-rgb),0.55)]"
            : "bg-white/[0.01] border-white/[0.03]",
          active &&
            "before:content-[''] before:absolute before:left-0 before:top-2.5 before:bottom-2.5 before:w-[2.5px] before:rounded-full before:bg-[color:var(--accent)]",
        )}
      >
        {/* Left icon */}
        <div
          className={cn(
            "w-8 h-8 rounded-xl flex items-center justify-center shrink-0 border",
            rule.is_custom
              ? "bg-indigo-500/10 border-indigo-500/15 text-indigo-400"
              : "bg-white/[0.02] border-white/[0.04] text-zinc-600",
          )}
        >
          {rule.is_custom ? <Edit3 size={13} /> : <Lock size={13} />}
        </div>

        {/* Info */}
        <div className="flex-1 min-w-0">
          <div className="flex items-center gap-2 mb-0.5">
            <span className="text-[13px] font-semibold text-[color:var(--text-primary)] truncate tracking-tight">
              {rule.name}
            </span>
            <span
              className={cn(
                "text-[9px] border px-1.5 py-0.5 rounded-md font-bold uppercase tracking-wider shrink-0",
                rule.is_custom
                  ? "bg-indigo-500/10 text-indigo-400 border-indigo-500/20"
                  : "text-zinc-500 border-white/5 bg-[color:var(--bg-elevated)]",
              )}
            >
              {rule.is_custom ? "自定义" : "系统"}
            </span>
          </div>
          <span className="text-[11px] font-mono text-zinc-600 truncate block">
            {rule.pattern}
          </span>
        </div>

        {/* Mask label */}
        <div className="shrink-0 max-w-[36%]">
          <span className="inline-block max-w-full truncate text-[10px] font-mono font-semibold text-emerald-400/80 bg-emerald-500/[0.06] px-2.5 py-1 rounded-lg border border-emerald-500/10">
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
    labelSuffix?: React.ReactNode,
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
                ? settings.mask_wrapper_style === "square" ? "[LABEL]" : "<LABEL>"
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
          placeholder={
            field === "name"
              ? "脱敏规则名称"
              : field === "mask"
                ? settings.mask_wrapper_style === "square" ? "[LABEL]" : "<LABEL>"
                : ""
          }
        />
      );

    return (
      <div className="flex flex-col gap-2">
        <label className="text-xs font-bold text-amber-100/80 uppercase tracking-[0.12em] flex items-center gap-2">
          {label}
          {labelSuffix && (
            <span className="text-[10px] font-mono text-zinc-600 bg-white/[0.04] px-2 py-0.5 rounded-full">
              {labelSuffix}
            </span>
          )}
        </label>
        <div
          className={cn(
            "relative rounded-2xl border transition-all duration-300 shadow-inner",
            readOnly
              ? "border-white/[0.05]"
              : "border-white/[0.12] hover:border-white/[0.2] focus-within:border-amber-500/40 focus-within:shadow-input-glow",
            extraClass,
          )}
          style={{ backgroundColor: "var(--bg-input)" }}
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
      <div
        className="w-full min-w-0 border rounded-3xl overflow-hidden"
        style={{
          backgroundColor: "color-mix(in srgb, var(--bg-card) 96%, transparent)",
          borderColor: "var(--border-subtle)",
          boxShadow: "0 1px 0 rgba(255,255,255,0.04) inset",
        }}
      >
        {/* In-card header — no absolute label overflow */}
        <div
          className="flex items-center justify-between gap-3 px-5 py-3.5 border-b"
          style={{ borderColor: "var(--border-subtle)" }}
        >
          <div className="flex items-center gap-2.5 min-w-0">
            <div
              className="w-8 h-8 rounded-xl border flex items-center justify-center shrink-0"
              style={{
                backgroundColor: "color-mix(in srgb, var(--accent) 12%, transparent)",
                borderColor: "var(--accent-border)",
              }}
            >
              <Beaker size={14} className="text-[color:var(--accent)]" />
            </div>
            <div className="min-w-0">
              <h4 className="text-[13px] font-semibold tracking-tight text-[color:var(--text-primary)] truncate">
                调试沙盒
              </h4>
              <p className="text-[10px] text-zinc-600 mt-0.5 truncate">
                实时验证正则与脱敏结果
              </p>
            </div>
          </div>
          <span
            className={cn(
              "shrink-0 text-[10px] font-bold uppercase tracking-wider px-2 py-1 rounded-md border",
              showOutput && "text-emerald-400 border-emerald-500/20 bg-emerald-500/10",
              testError && "text-red-400 border-red-500/20 bg-red-500/10",
              showStandby && !testError && "text-zinc-500 border-white/[0.06] bg-white/[0.02]",
            )}
          >
            {testError ? "错误" : showOutput ? "就绪" : "待机"}
          </span>
        </div>

        <div className="p-4 space-y-3">
          {/* Input */}
          <div className="space-y-1.5">
            <label className="text-[10px] font-bold uppercase tracking-[0.14em] text-zinc-600 px-0.5">
              测试文本
            </label>
            <textarea
              value={testInput}
              onChange={(e) => setTestInput(e.target.value)}
              placeholder="粘贴或输入待脱敏样例…"
              className="w-full border rounded-2xl px-4 py-3.5 text-[13px] font-mono leading-relaxed outline-none transition-all duration-200 resize-none min-h-[88px] text-[color:var(--text-primary)] placeholder:text-zinc-600 focus:border-[color:var(--accent-border)] focus:shadow-[0_0_0_3px_rgba(var(--accent-rgb),0.10)]"
              style={{
                backgroundColor: "var(--bg-input)",
                borderColor: "var(--border-default)",
              }}
            />
          </div>

          {/* Output */}
          <div className="space-y-1.5">
            <label className="text-[10px] font-bold uppercase tracking-[0.14em] text-zinc-600 px-0.5">
              输出预览
            </label>
            <div
              className={cn(
                "w-full border rounded-2xl px-4 py-3.5 text-[13px] font-mono leading-relaxed min-h-[104px] transition-all duration-200",
                showOutput && "border-emerald-500/20 bg-emerald-500/[0.04] text-emerald-300/90",
                showStandby && !testError && "border-dashed border-white/[0.06] text-zinc-600",
                testError && "border-red-500/25 bg-red-500/[0.04] text-red-400",
              )}
              style={
                showOutput || testError
                  ? undefined
                  : {
                      backgroundColor: "color-mix(in srgb, var(--bg-input) 92%, transparent)",
                      borderColor: "var(--border-subtle)",
                    }
              }
            >
              {testError && (
                <div className="flex items-start gap-2">
                  <X size={13} className="mt-0.5 shrink-0 text-red-400" />
                  <span className="whitespace-pre-wrap break-words">{testError}</span>
                </div>
              )}

              {showOutput && (
                <div className="flex items-start gap-2">
                  <Check size={13} className="mt-0.5 shrink-0 text-emerald-400" />
                  <span className="whitespace-pre-wrap break-words">{testOutput}</span>
                </div>
              )}

              {showStandby && !testError && (
                <div className="flex flex-col items-center justify-center py-5 text-center gap-2.5">
                  <div
                    className="w-10 h-10 rounded-xl border flex items-center justify-center"
                    style={{
                      backgroundColor: "color-mix(in srgb, var(--bg-elevated) 90%, transparent)",
                      borderColor: "var(--border-subtle)",
                    }}
                  >
                    <Fingerprint size={18} className="text-zinc-600" />
                  </div>
                  <div>
                    <p className="text-[12px] font-semibold text-zinc-500">等待输入</p>
                    <p className="text-[11px] text-zinc-600 mt-1 leading-relaxed max-w-[240px]">
                      填写规则 pattern 与测试文本后，将自动显示脱敏结果
                    </p>
                  </div>
                </div>
              )}
            </div>
          </div>
        </div>
      </div>
    );
  };


  // ── Import / Export ──

  const handleImportRules = useCallback(async () => {
    try {
      const selected = await open({
        multiple: true,
        filters: [{ name: "YAML Rules", extensions: ["yaml", "yml"] }],
      });
      if (!selected) return;
      const paths = Array.isArray(selected) ? selected : [selected];
      if (paths.length === 0) return;

      const report = await MaskAPI.importCustomRules(paths);
      await Promise.all([fetchAllRules(), fetchStats()]);

      const summary =
        `解析 ${report.total_parsed} 条 · 新增 ${report.imported} · 覆盖 ${report.overwritten} · 跳过 ${report.skipped}` +
        (report.failed ? ` · 失败 ${report.failed}` : "");

      const details = report.items
        .filter((i) => i.status !== "imported")
        .slice(0, 12)
        .map((i) => `• ${i.name}: ${i.message}`)
        .join("\n");

      await message(
        details ? `${summary}\n\n${details}` : summary,
        { title: "规则导入完成", kind: report.failed ? "warning" : "info" },
      );
    } catch (e) {
      await message(`导入失败: ${e}`, { title: "错误", kind: "error" });
    }
  }, [fetchAllRules, fetchStats]);

  const handleExportRules = useCallback(async () => {
    try {
      const yaml = await MaskAPI.exportCustomRulesYaml();
      const path = await save({
        defaultPath: "safemask_custom_rules.yaml",
        filters: [{ name: "YAML", extensions: ["yaml", "yml"] }],
      });
      if (!path) return;
      await MaskAPI.saveTextToPath(path, yaml);
      await message("自定义规则已导出", { title: "导出成功", kind: "info" });
    } catch (e) {
      await message(`导出失败: ${e}`, { title: "错误", kind: "error" });
    }
  }, []);

  const handleDownloadTemplate = useCallback(async () => {
    try {
      const yaml = await MaskAPI.getRulesImportTemplate();
      const path = await save({
        defaultPath: "safemask_rules_template.yaml",
        filters: [{ name: "YAML", extensions: ["yaml", "yml"] }],
      });
      if (!path) return;
      await MaskAPI.saveTextToPath(path, yaml);
      await message("模板已保存", { title: "完成", kind: "info" });
    } catch (e) {
      await message(`保存模板失败: ${e}`, { title: "错误", kind: "error" });
    }
  }, []);


  // ── Main render ──

  return (
    <div className="flex items-stretch gap-8 h-full overflow-hidden">
      {/* ── Left panel: Rule list ── */}
      <div
        className="flex-1 min-w-0 flex flex-col border rounded-4xl overflow-hidden"
        style={{
          backgroundColor: "color-mix(in srgb, var(--bg-card) 94%, transparent)",
          borderColor: "var(--border-subtle)",
        }}
      >
        {/* Header — 精致分层：标题 / 分段操作 / 搜索 */}
        <div
          className="px-5 pt-5 pb-4 border-b space-y-4"
          style={{ borderColor: "var(--border-subtle)" }}
        >
          {/* Title row */}
          <div className="flex items-center gap-3 min-w-0">
            <div
              className="relative w-10 h-10 rounded-2xl border flex items-center justify-center shrink-0 overflow-hidden"
              style={{
                background:
                  "linear-gradient(145deg, color-mix(in srgb, var(--accent) 18%, var(--bg-elevated)), var(--bg-elevated))",
                borderColor: "var(--border-default)",
                boxShadow: "0 1px 0 rgba(255,255,255,0.06) inset, 0 8px 20px -12px rgba(var(--accent-rgb),0.45)",
              }}
            >
              <Layers size={17} className="text-[color:var(--accent)] relative z-10" />
            </div>
            <div className="min-w-0 flex-1">
              <div className="flex items-baseline gap-2">
                <h3 className="text-[15px] font-semibold tracking-tight text-[color:var(--text-primary)]">
                  规则仓库
                </h3>
                <span className="text-[11px] font-medium text-zinc-600 tabular-nums">
                  {sortedRules.length}
                </span>
              </div>
              <p className="text-[11px] text-zinc-600 mt-0.5 tracking-wide">
                管理内置与自定义脱敏模式
              </p>
            </div>
          </div>

          {/* Segmented actions — Apple-style control strip */}
          <div
            className="grid grid-cols-4 gap-0.5 p-1 rounded-2xl border"
            style={{
              backgroundColor: "color-mix(in srgb, var(--bg-input) 92%, transparent)",
              borderColor: "var(--border-subtle)",
              boxShadow: "inset 0 1px 0 rgba(255,255,255,0.03)",
            }}
          >
            {(
              [
                {
                  key: "import",
                  label: "导入",
                  title: "导入规则 YAML",
                  onClick: handleImportRules,
                  Icon: Upload,
                  tone: "text-amber-500",
                },
                {
                  key: "export",
                  label: "导出",
                  title: "导出自定义规则",
                  onClick: handleExportRules,
                  Icon: Download,
                  tone: "text-emerald-400",
                },
                {
                  key: "template",
                  label: "模板",
                  title: "下载导入模板",
                  onClick: handleDownloadTemplate,
                  Icon: FileDown,
                  tone: "text-zinc-400",
                },
                {
                  key: "create",
                  label: "新建",
                  title: "新建规则",
                  onClick: clearForm,
                  Icon: Plus,
                  tone: "text-indigo-300",
                },
              ] as const
            ).map(({ key, label, title, onClick, Icon, tone }) => (
              <button
                key={key}
                type="button"
                onClick={onClick}
                title={title}
                className={cn(
                  "group/btn relative inline-flex items-center justify-center gap-1.5 h-9 rounded-[12px]",
                  "text-[11px] font-semibold tracking-wide",
                  "transition-all duration-200 ease-out",
                  "hover:bg-white/[0.06] active:scale-[0.97]",
                  "focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-[color:var(--ring-color)]",
                  tone,
                )}
              >
                <Icon
                  size={14}
                  strokeWidth={2.25}
                  className="shrink-0 opacity-90 transition-transform duration-200 group-hover/btn:-translate-y-px"
                />
                <span>{label}</span>
              </button>
            ))}
          </div>

          <div className="w-full min-w-0">{renderSearchBar()}</div>
        </div>

        {/* Rule list */}
        <div className="flex-1 overflow-y-auto p-3.5 space-y-1.5 custom-scroll">
          {sortedRules.map(renderRuleItem)}

          {sortedRules.length === 0 && (
            <div className="flex flex-col items-center justify-center py-16 text-center px-6">
              <div
                className="w-14 h-14 rounded-2xl border flex items-center justify-center mb-4"
                style={{
                  backgroundColor: "color-mix(in srgb, var(--bg-elevated) 90%, transparent)",
                  borderColor: "var(--border-subtle)",
                }}
              >
                <Fingerprint size={26} className="text-zinc-600" />
              </div>
              <p className="text-[13px] font-semibold text-[color:var(--text-secondary)]">
                {searchQuery ? "没有匹配的规则" : "还没有规则"}
              </p>
              <p className="text-[11px] text-zinc-600 mt-1.5 leading-relaxed">
                {searchQuery
                  ? "试试其他关键词，或清空搜索"
                  : "点击上方「新建」创建，或「导入」批量添加"}
              </p>
            </div>
          )}
        </div>
      </div>

      {/* ── Right panel: Editor + Sandbox（可滚动，沙盒始终可达） ── */}
      <div className="w-[min(480px,42vw)] min-w-[300px] shrink-0 flex flex-col min-h-0 overflow-hidden">
        <div className="flex-1 min-h-0 overflow-y-auto custom-scroll pr-1 space-y-4 pb-2">
        {/* Form container */}
        <div
          className="border rounded-3xl p-6 space-y-5 shadow-2xl"
          style={{
            backgroundColor: "color-mix(in srgb, var(--bg-card) 96%, transparent)",
            borderColor: "var(--border-subtle)",
          }}
        >
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
          {renderFormField("MASK", "mask", "input", "", isSystemRule ?? false,
            settings.mask_wrapper_style === "square" ? "[  ]" : "<  >",
          )}
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
                      className="w-full py-4 border border-white/5 text-zinc-500 rounded-2xl text-xs font-black uppercase tracking-widest flex items-center justify-center gap-3 hover:text-amber-200 hover:border-amber-500/20 transition-all"
                      style={{ backgroundColor: "var(--bg-elevated)" }}
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

        {/* Sandbox — 同滚动流内紧跟表单，保证进入视口可滚动看到 */}
        <div className="shrink-0 pb-1">
          {renderSandbox()}
        </div>
        </div>
      </div>
    </div>
  );
}
