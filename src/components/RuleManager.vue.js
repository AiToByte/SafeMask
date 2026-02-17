/// <reference types="../../node_modules/.vue-global-types/vue_3.5_0_0_0.d.ts" />
import { ref, onMounted, computed, watch } from 'vue';
import { useAppStore } from '../stores/useAppStore';
import { MaskAPI } from '../services/api';
import { Plus, Layers, Trash2, ShieldCheck, Search, Edit3, Beaker, Check, Save, CopyPlus, Lock, Info } from 'lucide-vue-next';
import { confirm, message } from '@tauri-apps/plugin-dialog';
const store = useAppStore();
// --- 状态管理 ---
const isSubmitting = ref(false);
const searchQuery = ref("");
const selectedRuleName = ref(""); // 标记当前选中的原始规则名
// --- 表单模型 ---
const initialForm = { name: '', pattern: '', mask: '<LABEL>', priority: 10, is_custom: true, enabled: true };
const form = ref({ ...initialForm });
// --- 实时验证逻辑 ---
const nameDuplicateError = computed(() => {
    if (!form.value.name)
        return "";
    const exists = store.allRulesList.find(r => r.name === form.value.name);
    if (exists) {
        if (selectedRuleName.value === exists.name && exists.is_custom)
            return "";
        return `库中已存在名为 [${form.value.name}] 的${exists.is_custom ? '自定义' : '系统'}规则`;
    }
    return "";
});
const patternDuplicateError = computed(() => {
    if (!form.value.pattern)
        return "";
    const exists = store.allRulesList.find(r => r.pattern === form.value.pattern);
    if (exists) {
        if (selectedRuleName.value === exists.name)
            return "";
        return `该表达式与规则 [${exists.name}] 重复`;
    }
    return "";
});
// --- 调试沙盒状态 ---
const testInput = ref("");
const testOutput = ref("");
const testError = ref("");
// 监听变动实时测试
watch([() => form.value.pattern, () => form.value.mask, testInput], async () => {
    if (!form.value.pattern || !testInput.value) {
        testOutput.value = "";
        testError.value = "";
        return;
    }
    try {
        testOutput.value = await MaskAPI.testRule(form.value.pattern, form.value.mask, testInput.value);
        testError.value = "";
    }
    catch (e) {
        testError.value = e.toString();
    }
}, { immediate: true });
onMounted(() => store.fetchAllRules());
// --- 核心交互方法 ---
const selectRule = (rule) => {
    selectedRuleName.value = rule.name;
    form.value = { ...rule };
};
const clearForm = () => {
    if (!selectedRuleName.value && form.value.name === "")
        return;
    selectedRuleName.value = "";
    form.value = { ...initialForm };
};
const handleSave = async (asNew = false) => {
    if (!form.value.name || !form.value.pattern) {
        await message("请填写完整的规则名称和正则表达式", { title: "数据缺失", kind: "warning" });
        return;
    }
    if (asNew) {
        const isDuplicate = store.allRulesList.some(r => r.name === form.value.name || r.pattern === form.value.pattern);
        if (isDuplicate) {
            await message("检测到规则库中已存在相同名称或表达式的记录，请修改后再保存。", { title: "拒绝重复添加", kind: "error" });
            return;
        }
    }
    isSubmitting.value = true;
    try {
        const payload = { ...form.value };
        if (asNew)
            payload.is_custom = true;
        await MaskAPI.saveRule(payload);
        await store.fetchAllRules();
        await store.fetchStats();
        if (asNew)
            selectedRuleName.value = payload.name;
        await message(asNew ? "规则已作为新模式存入库中" : "规则修改已即时应用至脱敏引擎", { title: "注入成功", kind: "info" });
    }
    catch (e) {
        await message("引擎注入失败: " + e, { kind: "error" });
    }
    finally {
        isSubmitting.value = false;
    }
};
const handleDelete = async (name) => {
    const ok = await confirm(`此操作将永久删除自定义规则 [${name}]。确定继续吗？`, { title: '销毁确认', kind: 'warning' });
    if (ok) {
        await MaskAPI.deleteRule(name);
        await store.fetchAllRules();
        await store.fetchStats();
        if (selectedRuleName.value === name)
            clearForm();
    }
};
const sortedRules = computed(() => {
    let f = store.allRulesList;
    if (searchQuery.value) {
        const q = searchQuery.value.toLowerCase();
        f = f.filter(r => r.name.toLowerCase().includes(q) || r.pattern.toLowerCase().includes(q));
    }
    return [...f].sort((a, b) => (a.is_custom === b.is_custom ? b.priority - a.priority : a.is_custom ? -1 : 1));
});
debugger; /* PartiallyEnd: #3632/scriptSetup.vue */
const __VLS_ctx = {};
let __VLS_components;
let __VLS_directives;
/** @type {__VLS_StyleScopedClasses['rule-item']} */ ;
/** @type {__VLS_StyleScopedClasses['rule-item']} */ ;
/** @type {__VLS_StyleScopedClasses['rule-item']} */ ;
/** @type {__VLS_StyleScopedClasses['active']} */ ;
/** @type {__VLS_StyleScopedClasses['label-header']} */ ;
/** @type {__VLS_StyleScopedClasses['input-wrapper']} */ ;
/** @type {__VLS_StyleScopedClasses['input-wrapper']} */ ;
/** @type {__VLS_StyleScopedClasses['input-wrapper']} */ ;
/** @type {__VLS_StyleScopedClasses['input-wrapper']} */ ;
/** @type {__VLS_StyleScopedClasses['sandbox-area']} */ ;
/** @type {__VLS_StyleScopedClasses['sandbox-area']} */ ;
/** @type {__VLS_StyleScopedClasses['custom-scroll']} */ ;
// CSS variable injection 
// CSS variable injection end 
__VLS_asFunctionalElement(__VLS_intrinsicElements.div, __VLS_intrinsicElements.div)({
    ...{ onClick: (__VLS_ctx.clearForm) },
    ...{ class: "flex items-stretch gap-6 h-full overflow-hidden animate-in fade-in duration-700" },
});
__VLS_asFunctionalElement(__VLS_intrinsicElements.div, __VLS_intrinsicElements.div)({
    ...{ onClick: () => { } },
    ...{ class: "flex-1 min-w-0 flex flex-col bg-[#0d0d0f]/60 border border-white/[0.04] rounded-[2rem] overflow-hidden" },
});
__VLS_asFunctionalElement(__VLS_intrinsicElements.div, __VLS_intrinsicElements.div)({
    ...{ class: "px-8 py-5 border-b border-white/[0.04] flex items-center justify-between" },
});
__VLS_asFunctionalElement(__VLS_intrinsicElements.div, __VLS_intrinsicElements.div)({
    ...{ class: "flex items-center gap-3" },
});
const __VLS_0 = {}.Layers;
/** @type {[typeof __VLS_components.Layers, ]} */ ;
// @ts-ignore
const __VLS_1 = __VLS_asFunctionalComponent(__VLS_0, new __VLS_0({
    size: (18),
    ...{ class: "text-amber-500/50" },
}));
const __VLS_2 = __VLS_1({
    size: (18),
    ...{ class: "text-amber-500/50" },
}, ...__VLS_functionalComponentArgsRest(__VLS_1));
__VLS_asFunctionalElement(__VLS_intrinsicElements.h3, __VLS_intrinsicElements.h3)({
    ...{ class: "font-bold text-amber-50/80 text-sm tracking-widest uppercase" },
});
__VLS_asFunctionalElement(__VLS_intrinsicElements.div, __VLS_intrinsicElements.div)({
    ...{ class: "relative group" },
});
const __VLS_4 = {}.Search;
/** @type {[typeof __VLS_components.Search, ]} */ ;
// @ts-ignore
const __VLS_5 = __VLS_asFunctionalComponent(__VLS_4, new __VLS_4({
    ...{ class: "absolute left-3 top-1/2 -translate-y-1/2 text-zinc-600 group-focus-within:text-amber-500/40 transition-colors" },
    size: (14),
}));
const __VLS_6 = __VLS_5({
    ...{ class: "absolute left-3 top-1/2 -translate-y-1/2 text-zinc-600 group-focus-within:text-amber-500/40 transition-colors" },
    size: (14),
}, ...__VLS_functionalComponentArgsRest(__VLS_5));
__VLS_asFunctionalElement(__VLS_intrinsicElements.input)({
    placeholder: "快速检索模式...",
    ...{ class: "search-bar" },
});
(__VLS_ctx.searchQuery);
__VLS_asFunctionalElement(__VLS_intrinsicElements.div, __VLS_intrinsicElements.div)({
    ...{ onClick: (__VLS_ctx.clearForm) },
    ...{ class: "flex-1 overflow-y-auto p-5 space-y-2 custom-scroll" },
});
for (const [rule] of __VLS_getVForSourceType((__VLS_ctx.sortedRules))) {
    __VLS_asFunctionalElement(__VLS_intrinsicElements.div, __VLS_intrinsicElements.div)({
        ...{ onClick: (...[$event]) => {
                __VLS_ctx.selectRule(rule);
            } },
        key: (rule.name),
        ...{ class: "rule-item group" },
        ...{ class: ({ 'active': __VLS_ctx.selectedRuleName === rule.name }) },
    });
    __VLS_asFunctionalElement(__VLS_intrinsicElements.div, __VLS_intrinsicElements.div)({
        ...{ class: "flex-1 min-w-0 pr-4" },
    });
    __VLS_asFunctionalElement(__VLS_intrinsicElements.div, __VLS_intrinsicElements.div)({
        ...{ class: "flex items-center gap-2 mb-0.5" },
    });
    __VLS_asFunctionalElement(__VLS_intrinsicElements.span, __VLS_intrinsicElements.span)({
        ...{ class: "text-sm font-bold truncate text-zinc-200" },
    });
    (rule.name);
    __VLS_asFunctionalElement(__VLS_intrinsicElements.span, __VLS_intrinsicElements.span)({
        ...{ class: (rule.is_custom ? 'tag-custom' : 'tag-system') },
    });
    (rule.is_custom ? 'Custom' : 'System');
    __VLS_asFunctionalElement(__VLS_intrinsicElements.code, __VLS_intrinsicElements.code)({
        ...{ class: "pattern-text" },
    });
    (rule.pattern);
    __VLS_asFunctionalElement(__VLS_intrinsicElements.div, __VLS_intrinsicElements.div)({
        ...{ class: "flex items-center gap-3 shrink-0" },
    });
    __VLS_asFunctionalElement(__VLS_intrinsicElements.span, __VLS_intrinsicElements.span)({
        ...{ class: "mask-label" },
    });
    (rule.mask);
    __VLS_asFunctionalElement(__VLS_intrinsicElements.div, __VLS_intrinsicElements.div)({
        ...{ class: "w-6 flex justify-center" },
    });
    if (rule.is_custom) {
        __VLS_asFunctionalElement(__VLS_intrinsicElements.button, __VLS_intrinsicElements.button)({
            ...{ onClick: (...[$event]) => {
                    if (!(rule.is_custom))
                        return;
                    __VLS_ctx.handleDelete(rule.name);
                } },
            ...{ class: "delete-trigger" },
        });
        const __VLS_8 = {}.Trash2;
        /** @type {[typeof __VLS_components.Trash2, ]} */ ;
        // @ts-ignore
        const __VLS_9 = __VLS_asFunctionalComponent(__VLS_8, new __VLS_8({
            size: (14),
        }));
        const __VLS_10 = __VLS_9({
            size: (14),
        }, ...__VLS_functionalComponentArgsRest(__VLS_9));
    }
    else {
        const __VLS_12 = {}.ShieldCheck;
        /** @type {[typeof __VLS_components.ShieldCheck, ]} */ ;
        // @ts-ignore
        const __VLS_13 = __VLS_asFunctionalComponent(__VLS_12, new __VLS_12({
            size: (14),
            ...{ class: "text-zinc-800" },
        }));
        const __VLS_14 = __VLS_13({
            size: (14),
            ...{ class: "text-zinc-800" },
        }, ...__VLS_functionalComponentArgsRest(__VLS_13));
    }
}
__VLS_asFunctionalElement(__VLS_intrinsicElements.div, __VLS_intrinsicElements.div)({
    ...{ onClick: (__VLS_ctx.clearForm) },
    ...{ class: "flex-1 min-h-[120px]" },
});
__VLS_asFunctionalElement(__VLS_intrinsicElements.div, __VLS_intrinsicElements.div)({
    ...{ onClick: () => { } },
    ...{ class: "w-[420px] flex flex-col gap-5 overflow-y-auto custom-scroll pr-1" },
});
__VLS_asFunctionalElement(__VLS_intrinsicElements.div, __VLS_intrinsicElements.div)({
    ...{ class: "glass-panel p-8 space-y-7 shadow-2xl" },
});
__VLS_asFunctionalElement(__VLS_intrinsicElements.div, __VLS_intrinsicElements.div)({
    ...{ class: "flex justify-between items-center" },
});
__VLS_asFunctionalElement(__VLS_intrinsicElements.div, __VLS_intrinsicElements.div)({
    ...{ class: "flex items-center gap-4" },
});
__VLS_asFunctionalElement(__VLS_intrinsicElements.div, __VLS_intrinsicElements.div)({
    ...{ class: "w-10 h-10 rounded-xl bg-white/[0.03] border border-white/10 flex items-center justify-center" },
});
if (__VLS_ctx.selectedRuleName) {
    const __VLS_16 = {}.Edit3;
    /** @type {[typeof __VLS_components.Edit3, ]} */ ;
    // @ts-ignore
    const __VLS_17 = __VLS_asFunctionalComponent(__VLS_16, new __VLS_16({
        size: (18),
        ...{ class: "text-amber-400" },
    }));
    const __VLS_18 = __VLS_17({
        size: (18),
        ...{ class: "text-amber-400" },
    }, ...__VLS_functionalComponentArgsRest(__VLS_17));
}
else {
    const __VLS_20 = {}.Plus;
    /** @type {[typeof __VLS_components.Plus, ]} */ ;
    // @ts-ignore
    const __VLS_21 = __VLS_asFunctionalComponent(__VLS_20, new __VLS_20({
        size: (18),
        ...{ class: "text-blue-500" },
    }));
    const __VLS_22 = __VLS_21({
        size: (18),
        ...{ class: "text-blue-500" },
    }, ...__VLS_functionalComponentArgsRest(__VLS_21));
}
__VLS_asFunctionalElement(__VLS_intrinsicElements.h3, __VLS_intrinsicElements.h3)({
    ...{ class: "font-bold text-amber-50/90 tracking-tight" },
});
(__VLS_ctx.selectedRuleName ? '配置既有模式' : '创建新脱敏模式');
if (__VLS_ctx.selectedRuleName) {
    __VLS_asFunctionalElement(__VLS_intrinsicElements.button, __VLS_intrinsicElements.button)({
        ...{ onClick: (__VLS_ctx.clearForm) },
        ...{ class: "text-zinc-600 hover:text-white transition-all" },
        title: "切换回新建模式",
    });
    const __VLS_24 = {}.Plus;
    /** @type {[typeof __VLS_components.Plus, ]} */ ;
    // @ts-ignore
    const __VLS_25 = __VLS_asFunctionalComponent(__VLS_24, new __VLS_24({
        size: (18),
    }));
    const __VLS_26 = __VLS_25({
        size: (18),
    }, ...__VLS_functionalComponentArgsRest(__VLS_25));
}
if (!__VLS_ctx.form.is_custom && __VLS_ctx.selectedRuleName) {
    __VLS_asFunctionalElement(__VLS_intrinsicElements.div, __VLS_intrinsicElements.div)({
        ...{ class: "bg-amber-900/10 border border-amber-500/20 p-4 rounded-2xl flex gap-4 animate-in slide-in-from-top-2" },
    });
    const __VLS_28 = {}.Lock;
    /** @type {[typeof __VLS_components.Lock, ]} */ ;
    // @ts-ignore
    const __VLS_29 = __VLS_asFunctionalComponent(__VLS_28, new __VLS_28({
        size: (20),
        ...{ class: "text-amber-500 shrink-0 mt-0.5" },
    }));
    const __VLS_30 = __VLS_29({
        size: (20),
        ...{ class: "text-amber-500 shrink-0 mt-0.5" },
    }, ...__VLS_functionalComponentArgsRest(__VLS_29));
    __VLS_asFunctionalElement(__VLS_intrinsicElements.p, __VLS_intrinsicElements.p)({
        ...{ class: "text-[11px] text-amber-200/50 leading-relaxed font-medium" },
    });
}
__VLS_asFunctionalElement(__VLS_intrinsicElements.div, __VLS_intrinsicElements.div)({
    ...{ class: "space-y-6" },
    ...{ class: ({ 'opacity-30 pointer-events-none filter grayscale': !__VLS_ctx.form.is_custom && __VLS_ctx.selectedRuleName }) },
});
__VLS_asFunctionalElement(__VLS_intrinsicElements.div, __VLS_intrinsicElements.div)({
    ...{ class: "input-group" },
});
__VLS_asFunctionalElement(__VLS_intrinsicElements.div, __VLS_intrinsicElements.div)({
    ...{ class: "label-header" },
});
__VLS_asFunctionalElement(__VLS_intrinsicElements.label, __VLS_intrinsicElements.label)({});
__VLS_asFunctionalElement(__VLS_intrinsicElements.span, __VLS_intrinsicElements.span)({
    ...{ class: "required-dot" },
});
__VLS_asFunctionalElement(__VLS_intrinsicElements.div, __VLS_intrinsicElements.div)({
    ...{ class: "input-wrapper" },
});
__VLS_asFunctionalElement(__VLS_intrinsicElements.input)({
    placeholder: "例如：用户隐私手机号",
});
(__VLS_ctx.form.name);
if (__VLS_ctx.nameDuplicateError) {
    __VLS_asFunctionalElement(__VLS_intrinsicElements.p, __VLS_intrinsicElements.p)({
        ...{ class: "validation-msg" },
    });
    const __VLS_32 = {}.Info;
    /** @type {[typeof __VLS_components.Info, ]} */ ;
    // @ts-ignore
    const __VLS_33 = __VLS_asFunctionalComponent(__VLS_32, new __VLS_32({
        size: (10),
    }));
    const __VLS_34 = __VLS_33({
        size: (10),
    }, ...__VLS_functionalComponentArgsRest(__VLS_33));
    (__VLS_ctx.nameDuplicateError);
}
__VLS_asFunctionalElement(__VLS_intrinsicElements.div, __VLS_intrinsicElements.div)({
    ...{ class: "input-group" },
});
__VLS_asFunctionalElement(__VLS_intrinsicElements.div, __VLS_intrinsicElements.div)({
    ...{ class: "label-header" },
});
__VLS_asFunctionalElement(__VLS_intrinsicElements.label, __VLS_intrinsicElements.label)({});
__VLS_asFunctionalElement(__VLS_intrinsicElements.span, __VLS_intrinsicElements.span)({
    ...{ class: "required-dot" },
});
__VLS_asFunctionalElement(__VLS_intrinsicElements.div, __VLS_intrinsicElements.div)({
    ...{ class: "input-wrapper" },
});
__VLS_asFunctionalElement(__VLS_intrinsicElements.textarea)({
    value: (__VLS_ctx.form.pattern),
    ...{ class: "h-28 font-mono text-[12px]" },
    placeholder: "输入匹配模式...",
});
if (__VLS_ctx.patternDuplicateError) {
    __VLS_asFunctionalElement(__VLS_intrinsicElements.p, __VLS_intrinsicElements.p)({
        ...{ class: "validation-msg" },
    });
    const __VLS_36 = {}.Info;
    /** @type {[typeof __VLS_components.Info, ]} */ ;
    // @ts-ignore
    const __VLS_37 = __VLS_asFunctionalComponent(__VLS_36, new __VLS_36({
        size: (10),
    }));
    const __VLS_38 = __VLS_37({
        size: (10),
    }, ...__VLS_functionalComponentArgsRest(__VLS_37));
    (__VLS_ctx.patternDuplicateError);
}
__VLS_asFunctionalElement(__VLS_intrinsicElements.div, __VLS_intrinsicElements.div)({
    ...{ class: "grid grid-cols-2 gap-5" },
});
__VLS_asFunctionalElement(__VLS_intrinsicElements.div, __VLS_intrinsicElements.div)({
    ...{ class: "input-group" },
});
__VLS_asFunctionalElement(__VLS_intrinsicElements.div, __VLS_intrinsicElements.div)({
    ...{ class: "label-header" },
});
__VLS_asFunctionalElement(__VLS_intrinsicElements.label, __VLS_intrinsicElements.label)({});
__VLS_asFunctionalElement(__VLS_intrinsicElements.div, __VLS_intrinsicElements.div)({
    ...{ class: "input-wrapper" },
});
__VLS_asFunctionalElement(__VLS_intrinsicElements.input)({
    ...{ class: "text-center text-blue-400 font-mono font-bold tracking-widest" },
});
(__VLS_ctx.form.mask);
__VLS_asFunctionalElement(__VLS_intrinsicElements.div, __VLS_intrinsicElements.div)({
    ...{ class: "input-group" },
});
__VLS_asFunctionalElement(__VLS_intrinsicElements.div, __VLS_intrinsicElements.div)({
    ...{ class: "label-header" },
});
__VLS_asFunctionalElement(__VLS_intrinsicElements.label, __VLS_intrinsicElements.label)({});
__VLS_asFunctionalElement(__VLS_intrinsicElements.div, __VLS_intrinsicElements.div)({
    ...{ class: "input-wrapper" },
});
__VLS_asFunctionalElement(__VLS_intrinsicElements.input)({
    type: "number",
    ...{ class: "text-center font-mono" },
});
(__VLS_ctx.form.priority);
__VLS_asFunctionalElement(__VLS_intrinsicElements.div, __VLS_intrinsicElements.div)({
    ...{ class: "flex flex-col gap-3 pt-4" },
});
if (__VLS_ctx.form.is_custom && __VLS_ctx.selectedRuleName) {
    __VLS_asFunctionalElement(__VLS_intrinsicElements.button, __VLS_intrinsicElements.button)({
        ...{ onClick: (...[$event]) => {
                if (!(__VLS_ctx.form.is_custom && __VLS_ctx.selectedRuleName))
                    return;
                __VLS_ctx.handleSave(false);
            } },
        ...{ class: "btn-primary" },
        disabled: (__VLS_ctx.isSubmitting || !!__VLS_ctx.nameDuplicateError),
    });
    const __VLS_40 = {}.Save;
    /** @type {[typeof __VLS_components.Save, ]} */ ;
    // @ts-ignore
    const __VLS_41 = __VLS_asFunctionalComponent(__VLS_40, new __VLS_40({
        size: (16),
    }));
    const __VLS_42 = __VLS_41({
        size: (16),
    }, ...__VLS_functionalComponentArgsRest(__VLS_41));
}
if (!__VLS_ctx.selectedRuleName) {
    __VLS_asFunctionalElement(__VLS_intrinsicElements.button, __VLS_intrinsicElements.button)({
        ...{ onClick: (...[$event]) => {
                if (!(!__VLS_ctx.selectedRuleName))
                    return;
                __VLS_ctx.handleSave(false);
            } },
        ...{ class: "btn-primary" },
        disabled: (__VLS_ctx.isSubmitting || !!__VLS_ctx.nameDuplicateError),
    });
    const __VLS_44 = {}.Plus;
    /** @type {[typeof __VLS_components.Plus, ]} */ ;
    // @ts-ignore
    const __VLS_45 = __VLS_asFunctionalComponent(__VLS_44, new __VLS_44({
        size: (16),
    }));
    const __VLS_46 = __VLS_45({
        size: (16),
    }, ...__VLS_functionalComponentArgsRest(__VLS_45));
}
if (__VLS_ctx.selectedRuleName) {
    __VLS_asFunctionalElement(__VLS_intrinsicElements.button, __VLS_intrinsicElements.button)({
        ...{ onClick: (...[$event]) => {
                if (!(__VLS_ctx.selectedRuleName))
                    return;
                __VLS_ctx.handleSave(true);
            } },
        ...{ class: "btn-secondary" },
        disabled: (__VLS_ctx.isSubmitting),
    });
    const __VLS_48 = {}.CopyPlus;
    /** @type {[typeof __VLS_components.CopyPlus, ]} */ ;
    // @ts-ignore
    const __VLS_49 = __VLS_asFunctionalComponent(__VLS_48, new __VLS_48({
        size: (16),
    }));
    const __VLS_50 = __VLS_49({
        size: (16),
    }, ...__VLS_functionalComponentArgsRest(__VLS_49));
}
__VLS_asFunctionalElement(__VLS_intrinsicElements.div, __VLS_intrinsicElements.div)({
    ...{ class: "glass-panel p-8 flex-1 border-emerald-500/10" },
});
__VLS_asFunctionalElement(__VLS_intrinsicElements.div, __VLS_intrinsicElements.div)({
    ...{ class: "flex items-center gap-3 mb-6" },
});
__VLS_asFunctionalElement(__VLS_intrinsicElements.div, __VLS_intrinsicElements.div)({
    ...{ class: "p-1.5 bg-emerald-500/10 rounded-lg" },
});
const __VLS_52 = {}.Beaker;
/** @type {[typeof __VLS_components.Beaker, ]} */ ;
// @ts-ignore
const __VLS_53 = __VLS_asFunctionalComponent(__VLS_52, new __VLS_52({
    size: (16),
    ...{ class: "text-emerald-400" },
}));
const __VLS_54 = __VLS_53({
    size: (16),
    ...{ class: "text-emerald-400" },
}, ...__VLS_functionalComponentArgsRest(__VLS_53));
__VLS_asFunctionalElement(__VLS_intrinsicElements.h3, __VLS_intrinsicElements.h3)({
    ...{ class: "font-bold text-amber-50/80" },
});
__VLS_asFunctionalElement(__VLS_intrinsicElements.div, __VLS_intrinsicElements.div)({
    ...{ class: "space-y-4" },
});
__VLS_asFunctionalElement(__VLS_intrinsicElements.div, __VLS_intrinsicElements.div)({
    ...{ class: "relative" },
});
__VLS_asFunctionalElement(__VLS_intrinsicElements.span, __VLS_intrinsicElements.span)({
    ...{ class: "sandbox-label" },
});
__VLS_asFunctionalElement(__VLS_intrinsicElements.textarea)({
    value: (__VLS_ctx.testInput),
    placeholder: "在这里输入原始文本进行测试...",
    ...{ class: "sandbox-area input custom-scroll" },
});
__VLS_asFunctionalElement(__VLS_intrinsicElements.div, __VLS_intrinsicElements.div)({
    ...{ class: "relative" },
});
__VLS_asFunctionalElement(__VLS_intrinsicElements.span, __VLS_intrinsicElements.span)({
    ...{ class: "sandbox-label" },
    ...{ class: (__VLS_ctx.testError ? 'text-red-500' : 'text-emerald-500') },
});
(__VLS_ctx.testError ? '正则语法错误' : '实时预览');
__VLS_asFunctionalElement(__VLS_intrinsicElements.div, __VLS_intrinsicElements.div)({
    ...{ class: "sandbox-area output custom-scroll" },
    ...{ class: ({ 'err': __VLS_ctx.testError }) },
});
if (__VLS_ctx.testError) {
    __VLS_asFunctionalElement(__VLS_intrinsicElements.span, __VLS_intrinsicElements.span)({
        ...{ class: "text-red-400 font-mono text-[10px] leading-tight" },
    });
    (__VLS_ctx.testError);
}
else {
    __VLS_asFunctionalElement(__VLS_intrinsicElements.span, __VLS_intrinsicElements.span)({
        ...{ class: "text-emerald-100/70" },
    });
    (__VLS_ctx.testOutput);
}
if (!__VLS_ctx.testError && __VLS_ctx.testOutput !== __VLS_ctx.testInput && __VLS_ctx.testInput) {
    const __VLS_56 = {}.Check;
    /** @type {[typeof __VLS_components.Check, ]} */ ;
    // @ts-ignore
    const __VLS_57 = __VLS_asFunctionalComponent(__VLS_56, new __VLS_56({
        ...{ class: "absolute right-3 bottom-3 text-emerald-500/40" },
        size: (16),
    }));
    const __VLS_58 = __VLS_57({
        ...{ class: "absolute right-3 bottom-3 text-emerald-500/40" },
        size: (16),
    }, ...__VLS_functionalComponentArgsRest(__VLS_57));
}
/** @type {__VLS_StyleScopedClasses['flex']} */ ;
/** @type {__VLS_StyleScopedClasses['items-stretch']} */ ;
/** @type {__VLS_StyleScopedClasses['gap-6']} */ ;
/** @type {__VLS_StyleScopedClasses['h-full']} */ ;
/** @type {__VLS_StyleScopedClasses['overflow-hidden']} */ ;
/** @type {__VLS_StyleScopedClasses['animate-in']} */ ;
/** @type {__VLS_StyleScopedClasses['fade-in']} */ ;
/** @type {__VLS_StyleScopedClasses['duration-700']} */ ;
/** @type {__VLS_StyleScopedClasses['flex-1']} */ ;
/** @type {__VLS_StyleScopedClasses['min-w-0']} */ ;
/** @type {__VLS_StyleScopedClasses['flex']} */ ;
/** @type {__VLS_StyleScopedClasses['flex-col']} */ ;
/** @type {__VLS_StyleScopedClasses['bg-[#0d0d0f]/60']} */ ;
/** @type {__VLS_StyleScopedClasses['border']} */ ;
/** @type {__VLS_StyleScopedClasses['border-white/[0.04]']} */ ;
/** @type {__VLS_StyleScopedClasses['rounded-[2rem]']} */ ;
/** @type {__VLS_StyleScopedClasses['overflow-hidden']} */ ;
/** @type {__VLS_StyleScopedClasses['px-8']} */ ;
/** @type {__VLS_StyleScopedClasses['py-5']} */ ;
/** @type {__VLS_StyleScopedClasses['border-b']} */ ;
/** @type {__VLS_StyleScopedClasses['border-white/[0.04]']} */ ;
/** @type {__VLS_StyleScopedClasses['flex']} */ ;
/** @type {__VLS_StyleScopedClasses['items-center']} */ ;
/** @type {__VLS_StyleScopedClasses['justify-between']} */ ;
/** @type {__VLS_StyleScopedClasses['flex']} */ ;
/** @type {__VLS_StyleScopedClasses['items-center']} */ ;
/** @type {__VLS_StyleScopedClasses['gap-3']} */ ;
/** @type {__VLS_StyleScopedClasses['text-amber-500/50']} */ ;
/** @type {__VLS_StyleScopedClasses['font-bold']} */ ;
/** @type {__VLS_StyleScopedClasses['text-amber-50/80']} */ ;
/** @type {__VLS_StyleScopedClasses['text-sm']} */ ;
/** @type {__VLS_StyleScopedClasses['tracking-widest']} */ ;
/** @type {__VLS_StyleScopedClasses['uppercase']} */ ;
/** @type {__VLS_StyleScopedClasses['relative']} */ ;
/** @type {__VLS_StyleScopedClasses['group']} */ ;
/** @type {__VLS_StyleScopedClasses['absolute']} */ ;
/** @type {__VLS_StyleScopedClasses['left-3']} */ ;
/** @type {__VLS_StyleScopedClasses['top-1/2']} */ ;
/** @type {__VLS_StyleScopedClasses['-translate-y-1/2']} */ ;
/** @type {__VLS_StyleScopedClasses['text-zinc-600']} */ ;
/** @type {__VLS_StyleScopedClasses['group-focus-within:text-amber-500/40']} */ ;
/** @type {__VLS_StyleScopedClasses['transition-colors']} */ ;
/** @type {__VLS_StyleScopedClasses['search-bar']} */ ;
/** @type {__VLS_StyleScopedClasses['flex-1']} */ ;
/** @type {__VLS_StyleScopedClasses['overflow-y-auto']} */ ;
/** @type {__VLS_StyleScopedClasses['p-5']} */ ;
/** @type {__VLS_StyleScopedClasses['space-y-2']} */ ;
/** @type {__VLS_StyleScopedClasses['custom-scroll']} */ ;
/** @type {__VLS_StyleScopedClasses['rule-item']} */ ;
/** @type {__VLS_StyleScopedClasses['group']} */ ;
/** @type {__VLS_StyleScopedClasses['flex-1']} */ ;
/** @type {__VLS_StyleScopedClasses['min-w-0']} */ ;
/** @type {__VLS_StyleScopedClasses['pr-4']} */ ;
/** @type {__VLS_StyleScopedClasses['flex']} */ ;
/** @type {__VLS_StyleScopedClasses['items-center']} */ ;
/** @type {__VLS_StyleScopedClasses['gap-2']} */ ;
/** @type {__VLS_StyleScopedClasses['mb-0.5']} */ ;
/** @type {__VLS_StyleScopedClasses['text-sm']} */ ;
/** @type {__VLS_StyleScopedClasses['font-bold']} */ ;
/** @type {__VLS_StyleScopedClasses['truncate']} */ ;
/** @type {__VLS_StyleScopedClasses['text-zinc-200']} */ ;
/** @type {__VLS_StyleScopedClasses['pattern-text']} */ ;
/** @type {__VLS_StyleScopedClasses['flex']} */ ;
/** @type {__VLS_StyleScopedClasses['items-center']} */ ;
/** @type {__VLS_StyleScopedClasses['gap-3']} */ ;
/** @type {__VLS_StyleScopedClasses['shrink-0']} */ ;
/** @type {__VLS_StyleScopedClasses['mask-label']} */ ;
/** @type {__VLS_StyleScopedClasses['w-6']} */ ;
/** @type {__VLS_StyleScopedClasses['flex']} */ ;
/** @type {__VLS_StyleScopedClasses['justify-center']} */ ;
/** @type {__VLS_StyleScopedClasses['delete-trigger']} */ ;
/** @type {__VLS_StyleScopedClasses['text-zinc-800']} */ ;
/** @type {__VLS_StyleScopedClasses['flex-1']} */ ;
/** @type {__VLS_StyleScopedClasses['min-h-[120px]']} */ ;
/** @type {__VLS_StyleScopedClasses['w-[420px]']} */ ;
/** @type {__VLS_StyleScopedClasses['flex']} */ ;
/** @type {__VLS_StyleScopedClasses['flex-col']} */ ;
/** @type {__VLS_StyleScopedClasses['gap-5']} */ ;
/** @type {__VLS_StyleScopedClasses['overflow-y-auto']} */ ;
/** @type {__VLS_StyleScopedClasses['custom-scroll']} */ ;
/** @type {__VLS_StyleScopedClasses['pr-1']} */ ;
/** @type {__VLS_StyleScopedClasses['glass-panel']} */ ;
/** @type {__VLS_StyleScopedClasses['p-8']} */ ;
/** @type {__VLS_StyleScopedClasses['space-y-7']} */ ;
/** @type {__VLS_StyleScopedClasses['shadow-2xl']} */ ;
/** @type {__VLS_StyleScopedClasses['flex']} */ ;
/** @type {__VLS_StyleScopedClasses['justify-between']} */ ;
/** @type {__VLS_StyleScopedClasses['items-center']} */ ;
/** @type {__VLS_StyleScopedClasses['flex']} */ ;
/** @type {__VLS_StyleScopedClasses['items-center']} */ ;
/** @type {__VLS_StyleScopedClasses['gap-4']} */ ;
/** @type {__VLS_StyleScopedClasses['w-10']} */ ;
/** @type {__VLS_StyleScopedClasses['h-10']} */ ;
/** @type {__VLS_StyleScopedClasses['rounded-xl']} */ ;
/** @type {__VLS_StyleScopedClasses['bg-white/[0.03]']} */ ;
/** @type {__VLS_StyleScopedClasses['border']} */ ;
/** @type {__VLS_StyleScopedClasses['border-white/10']} */ ;
/** @type {__VLS_StyleScopedClasses['flex']} */ ;
/** @type {__VLS_StyleScopedClasses['items-center']} */ ;
/** @type {__VLS_StyleScopedClasses['justify-center']} */ ;
/** @type {__VLS_StyleScopedClasses['text-amber-400']} */ ;
/** @type {__VLS_StyleScopedClasses['text-blue-500']} */ ;
/** @type {__VLS_StyleScopedClasses['font-bold']} */ ;
/** @type {__VLS_StyleScopedClasses['text-amber-50/90']} */ ;
/** @type {__VLS_StyleScopedClasses['tracking-tight']} */ ;
/** @type {__VLS_StyleScopedClasses['text-zinc-600']} */ ;
/** @type {__VLS_StyleScopedClasses['hover:text-white']} */ ;
/** @type {__VLS_StyleScopedClasses['transition-all']} */ ;
/** @type {__VLS_StyleScopedClasses['bg-amber-900/10']} */ ;
/** @type {__VLS_StyleScopedClasses['border']} */ ;
/** @type {__VLS_StyleScopedClasses['border-amber-500/20']} */ ;
/** @type {__VLS_StyleScopedClasses['p-4']} */ ;
/** @type {__VLS_StyleScopedClasses['rounded-2xl']} */ ;
/** @type {__VLS_StyleScopedClasses['flex']} */ ;
/** @type {__VLS_StyleScopedClasses['gap-4']} */ ;
/** @type {__VLS_StyleScopedClasses['animate-in']} */ ;
/** @type {__VLS_StyleScopedClasses['slide-in-from-top-2']} */ ;
/** @type {__VLS_StyleScopedClasses['text-amber-500']} */ ;
/** @type {__VLS_StyleScopedClasses['shrink-0']} */ ;
/** @type {__VLS_StyleScopedClasses['mt-0.5']} */ ;
/** @type {__VLS_StyleScopedClasses['text-[11px]']} */ ;
/** @type {__VLS_StyleScopedClasses['text-amber-200/50']} */ ;
/** @type {__VLS_StyleScopedClasses['leading-relaxed']} */ ;
/** @type {__VLS_StyleScopedClasses['font-medium']} */ ;
/** @type {__VLS_StyleScopedClasses['space-y-6']} */ ;
/** @type {__VLS_StyleScopedClasses['input-group']} */ ;
/** @type {__VLS_StyleScopedClasses['label-header']} */ ;
/** @type {__VLS_StyleScopedClasses['required-dot']} */ ;
/** @type {__VLS_StyleScopedClasses['input-wrapper']} */ ;
/** @type {__VLS_StyleScopedClasses['validation-msg']} */ ;
/** @type {__VLS_StyleScopedClasses['input-group']} */ ;
/** @type {__VLS_StyleScopedClasses['label-header']} */ ;
/** @type {__VLS_StyleScopedClasses['required-dot']} */ ;
/** @type {__VLS_StyleScopedClasses['input-wrapper']} */ ;
/** @type {__VLS_StyleScopedClasses['h-28']} */ ;
/** @type {__VLS_StyleScopedClasses['font-mono']} */ ;
/** @type {__VLS_StyleScopedClasses['text-[12px]']} */ ;
/** @type {__VLS_StyleScopedClasses['validation-msg']} */ ;
/** @type {__VLS_StyleScopedClasses['grid']} */ ;
/** @type {__VLS_StyleScopedClasses['grid-cols-2']} */ ;
/** @type {__VLS_StyleScopedClasses['gap-5']} */ ;
/** @type {__VLS_StyleScopedClasses['input-group']} */ ;
/** @type {__VLS_StyleScopedClasses['label-header']} */ ;
/** @type {__VLS_StyleScopedClasses['input-wrapper']} */ ;
/** @type {__VLS_StyleScopedClasses['text-center']} */ ;
/** @type {__VLS_StyleScopedClasses['text-blue-400']} */ ;
/** @type {__VLS_StyleScopedClasses['font-mono']} */ ;
/** @type {__VLS_StyleScopedClasses['font-bold']} */ ;
/** @type {__VLS_StyleScopedClasses['tracking-widest']} */ ;
/** @type {__VLS_StyleScopedClasses['input-group']} */ ;
/** @type {__VLS_StyleScopedClasses['label-header']} */ ;
/** @type {__VLS_StyleScopedClasses['input-wrapper']} */ ;
/** @type {__VLS_StyleScopedClasses['text-center']} */ ;
/** @type {__VLS_StyleScopedClasses['font-mono']} */ ;
/** @type {__VLS_StyleScopedClasses['flex']} */ ;
/** @type {__VLS_StyleScopedClasses['flex-col']} */ ;
/** @type {__VLS_StyleScopedClasses['gap-3']} */ ;
/** @type {__VLS_StyleScopedClasses['pt-4']} */ ;
/** @type {__VLS_StyleScopedClasses['btn-primary']} */ ;
/** @type {__VLS_StyleScopedClasses['btn-primary']} */ ;
/** @type {__VLS_StyleScopedClasses['btn-secondary']} */ ;
/** @type {__VLS_StyleScopedClasses['glass-panel']} */ ;
/** @type {__VLS_StyleScopedClasses['p-8']} */ ;
/** @type {__VLS_StyleScopedClasses['flex-1']} */ ;
/** @type {__VLS_StyleScopedClasses['border-emerald-500/10']} */ ;
/** @type {__VLS_StyleScopedClasses['flex']} */ ;
/** @type {__VLS_StyleScopedClasses['items-center']} */ ;
/** @type {__VLS_StyleScopedClasses['gap-3']} */ ;
/** @type {__VLS_StyleScopedClasses['mb-6']} */ ;
/** @type {__VLS_StyleScopedClasses['p-1.5']} */ ;
/** @type {__VLS_StyleScopedClasses['bg-emerald-500/10']} */ ;
/** @type {__VLS_StyleScopedClasses['rounded-lg']} */ ;
/** @type {__VLS_StyleScopedClasses['text-emerald-400']} */ ;
/** @type {__VLS_StyleScopedClasses['font-bold']} */ ;
/** @type {__VLS_StyleScopedClasses['text-amber-50/80']} */ ;
/** @type {__VLS_StyleScopedClasses['space-y-4']} */ ;
/** @type {__VLS_StyleScopedClasses['relative']} */ ;
/** @type {__VLS_StyleScopedClasses['sandbox-label']} */ ;
/** @type {__VLS_StyleScopedClasses['sandbox-area']} */ ;
/** @type {__VLS_StyleScopedClasses['input']} */ ;
/** @type {__VLS_StyleScopedClasses['custom-scroll']} */ ;
/** @type {__VLS_StyleScopedClasses['relative']} */ ;
/** @type {__VLS_StyleScopedClasses['sandbox-label']} */ ;
/** @type {__VLS_StyleScopedClasses['sandbox-area']} */ ;
/** @type {__VLS_StyleScopedClasses['output']} */ ;
/** @type {__VLS_StyleScopedClasses['custom-scroll']} */ ;
/** @type {__VLS_StyleScopedClasses['text-red-400']} */ ;
/** @type {__VLS_StyleScopedClasses['font-mono']} */ ;
/** @type {__VLS_StyleScopedClasses['text-[10px]']} */ ;
/** @type {__VLS_StyleScopedClasses['leading-tight']} */ ;
/** @type {__VLS_StyleScopedClasses['text-emerald-100/70']} */ ;
/** @type {__VLS_StyleScopedClasses['absolute']} */ ;
/** @type {__VLS_StyleScopedClasses['right-3']} */ ;
/** @type {__VLS_StyleScopedClasses['bottom-3']} */ ;
/** @type {__VLS_StyleScopedClasses['text-emerald-500/40']} */ ;
var __VLS_dollars;
const __VLS_self = (await import('vue')).defineComponent({
    setup() {
        return {
            Plus: Plus,
            Layers: Layers,
            Trash2: Trash2,
            ShieldCheck: ShieldCheck,
            Search: Search,
            Edit3: Edit3,
            Beaker: Beaker,
            Check: Check,
            Save: Save,
            CopyPlus: CopyPlus,
            Lock: Lock,
            Info: Info,
            isSubmitting: isSubmitting,
            searchQuery: searchQuery,
            selectedRuleName: selectedRuleName,
            form: form,
            nameDuplicateError: nameDuplicateError,
            patternDuplicateError: patternDuplicateError,
            testInput: testInput,
            testOutput: testOutput,
            testError: testError,
            selectRule: selectRule,
            clearForm: clearForm,
            handleSave: handleSave,
            handleDelete: handleDelete,
            sortedRules: sortedRules,
        };
    },
});
export default (await import('vue')).defineComponent({
    setup() {
        return {};
    },
});
; /* PartiallyEnd: #4569/main.vue */
//# sourceMappingURL=RuleManager.vue.js.map