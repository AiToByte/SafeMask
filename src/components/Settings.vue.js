/// <reference types="../../node_modules/.vue-global-types/vue_3.5_0_0_0.d.ts" />
import { ref, computed } from 'vue';
import { useAppStore } from '../stores/useAppStore';
import { MaskAPI } from '../services/api';
import { Keyboard, Timer, Save, Trash2, Monitor, Cpu, Volume2, Eye, AlertTriangle } from 'lucide-vue-next';
import { message } from '@tauri-apps/plugin-dialog';
const store = useAppStore();
const isRecording = ref(false);
const showKeyWarning = ref(false);
/**
 * 🚀 录制保护：进入录制时，告诉后端暂停所有“魔术粘贴”模拟，防止干扰
 */
const startRecording = async () => {
    isRecording.value = true;
    showKeyWarning.value = false;
    // 核心：锁定后端模拟按键
    await MaskAPI.setRecordingMode(true);
};
const stopRecording = async () => {
    isRecording.value = false;
    // 核心：解锁后端模拟按键
    await MaskAPI.setRecordingMode(false);
};
/**
 * 核心录制逻辑：采用 getModifierState 确保 Alt 键被精准捕获
 */
const handleKeyDown = (e) => {
    if (!isRecording.value)
        return;
    // 阻止 Alt 激活系统菜单
    e.preventDefault();
    e.stopPropagation();
    const mods = [];
    // 这里的精准判定解决了 Alt+V 变 Ctrl+V 的问题
    const isCtrl = e.ctrlKey || e.getModifierState('Control');
    const isAlt = e.altKey || e.getModifierState('Alt');
    const isShift = e.shiftKey || e.getModifierState('Shift');
    const isMeta = e.metaKey || e.getModifierState('Meta') || e.getModifierState('OS');
    if (isCtrl)
        mods.push("Ctrl");
    if (isAlt)
        mods.push("Alt");
    if (isShift)
        mods.push("Shift");
    if (isMeta)
        mods.push("Super");
    let key = e.key.toUpperCase();
    // 排除单纯按下修饰键的情况
    const isModifierOnly = ["CONTROL", "ALT", "SHIFT", "META"].includes(key);
    if (!isModifierOnly) {
        if (key === " ")
            key = "SPACE";
        const finalShortcut = [...mods, key].join("+");
        // 校验：禁止占用系统切换键 Alt+M
        if (finalShortcut.toLowerCase() === "alt+m") {
            showKeyWarning.value = true;
            store.playFeedbackSound('ERROR');
            setTimeout(() => showKeyWarning.value = false, 2500);
            return;
        }
        store.settings.magic_paste_shortcut = finalShortcut;
        stopRecording();
        store.playFeedbackSound('RECORD');
    }
};
const handleSave = async () => {
    await MaskAPI.updateSettings(store.settings);
    store.playFeedbackSound('ASCEND');
    await message("系统配置已实时同步至脱敏内核", { title: "同步成功", kind: "info" });
};
// 动态计算进度条背景宽度
const sliderProgress = computed(() => {
    return ((store.settings.paste_delay_ms - 50) / (800 - 50)) * 100;
});
debugger; /* PartiallyEnd: #3632/scriptSetup.vue */
const __VLS_ctx = {};
let __VLS_components;
let __VLS_directives;
/** @type {__VLS_StyleScopedClasses['shortcut-input']} */ ;
/** @type {__VLS_StyleScopedClasses['precision-range']} */ ;
/** @type {__VLS_StyleScopedClasses['precision-range']} */ ;
/** @type {__VLS_StyleScopedClasses['range-ticks']} */ ;
/** @type {__VLS_StyleScopedClasses['range-ticks']} */ ;
/** @type {__VLS_StyleScopedClasses['sw-wrapper']} */ ;
/** @type {__VLS_StyleScopedClasses['sw-slider']} */ ;
/** @type {__VLS_StyleScopedClasses['sw-slider']} */ ;
/** @type {__VLS_StyleScopedClasses['sw-slider']} */ ;
/** @type {__VLS_StyleScopedClasses['sw-wrapper']} */ ;
/** @type {__VLS_StyleScopedClasses['sw-slider']} */ ;
/** @type {__VLS_StyleScopedClasses['sm']} */ ;
/** @type {__VLS_StyleScopedClasses['sw-slider']} */ ;
/** @type {__VLS_StyleScopedClasses['sm']} */ ;
// CSS variable injection 
// CSS variable injection end 
__VLS_asFunctionalElement(__VLS_intrinsicElements.div, __VLS_intrinsicElements.div)({
    ...{ class: "max-w-5xl mx-auto space-y-10 animate-in fade-in duration-700 pb-16" },
});
__VLS_asFunctionalElement(__VLS_intrinsicElements.div, __VLS_intrinsicElements.div)({
    ...{ class: "flex items-center gap-6 mb-12 px-2" },
});
__VLS_asFunctionalElement(__VLS_intrinsicElements.div, __VLS_intrinsicElements.div)({
    ...{ class: "w-14 h-14 rounded-2xl bg-[#141210] border border-amber-500/10 flex items-center justify-center shadow-2xl shadow-black" },
});
const __VLS_0 = {}.Monitor;
/** @type {[typeof __VLS_components.Monitor, ]} */ ;
// @ts-ignore
const __VLS_1 = __VLS_asFunctionalComponent(__VLS_0, new __VLS_0({
    ...{ class: "text-amber-400/80 w-6 h-6" },
}));
const __VLS_2 = __VLS_1({
    ...{ class: "text-amber-400/80 w-6 h-6" },
}, ...__VLS_functionalComponentArgsRest(__VLS_1));
__VLS_asFunctionalElement(__VLS_intrinsicElements.div, __VLS_intrinsicElements.div)({});
__VLS_asFunctionalElement(__VLS_intrinsicElements.h2, __VLS_intrinsicElements.h2)({
    ...{ class: "text-3xl font-bold text-amber-50/90 tracking-tight" },
});
__VLS_asFunctionalElement(__VLS_intrinsicElements.p, __VLS_intrinsicElements.p)({
    ...{ class: "text-[10px] text-zinc-600 font-black uppercase tracking-[0.4em] mt-1.5 opacity-60" },
});
__VLS_asFunctionalElement(__VLS_intrinsicElements.div, __VLS_intrinsicElements.div)({
    ...{ class: "grid grid-cols-1 lg:grid-cols-2 gap-8" },
});
__VLS_asFunctionalElement(__VLS_intrinsicElements.section, __VLS_intrinsicElements.section)({
    ...{ class: "config-card" },
});
__VLS_asFunctionalElement(__VLS_intrinsicElements.div, __VLS_intrinsicElements.div)({
    ...{ class: "card-header" },
});
const __VLS_4 = {}.Cpu;
/** @type {[typeof __VLS_components.Cpu, ]} */ ;
// @ts-ignore
const __VLS_5 = __VLS_asFunctionalComponent(__VLS_4, new __VLS_4({
    size: (16),
    ...{ class: "text-blue-500" },
}));
const __VLS_6 = __VLS_5({
    size: (16),
    ...{ class: "text-blue-500" },
}, ...__VLS_functionalComponentArgsRest(__VLS_5));
__VLS_asFunctionalElement(__VLS_intrinsicElements.span, __VLS_intrinsicElements.span)({});
__VLS_asFunctionalElement(__VLS_intrinsicElements.div, __VLS_intrinsicElements.div)({
    ...{ class: "space-y-8 mt-8" },
});
__VLS_asFunctionalElement(__VLS_intrinsicElements.div, __VLS_intrinsicElements.div)({
    ...{ class: "setting-row" },
});
__VLS_asFunctionalElement(__VLS_intrinsicElements.div, __VLS_intrinsicElements.div)({
    ...{ class: "info" },
});
__VLS_asFunctionalElement(__VLS_intrinsicElements.span, __VLS_intrinsicElements.span)({
    ...{ class: "lbl" },
});
__VLS_asFunctionalElement(__VLS_intrinsicElements.span, __VLS_intrinsicElements.span)({
    ...{ class: "dsc" },
});
__VLS_asFunctionalElement(__VLS_intrinsicElements.label, __VLS_intrinsicElements.label)({
    ...{ class: "sw-wrapper" },
});
__VLS_asFunctionalElement(__VLS_intrinsicElements.input, __VLS_intrinsicElements.input)({
    ...{ onChange: (...[$event]) => {
            __VLS_ctx.store.playFeedbackSound(__VLS_ctx.store.settings.shadow_mode_enabled ? 'ASCEND' : 'DESCEND');
        } },
    type: "checkbox",
});
(__VLS_ctx.store.settings.shadow_mode_enabled);
__VLS_asFunctionalElement(__VLS_intrinsicElements.div, __VLS_intrinsicElements.div)({
    ...{ class: "sw-slider" },
});
__VLS_asFunctionalElement(__VLS_intrinsicElements.div, __VLS_intrinsicElements.div)({
    ...{ class: "relative p-7 bg-black/40 rounded-[2rem] border border-white/[0.03] shadow-inner group" },
});
__VLS_asFunctionalElement(__VLS_intrinsicElements.div, __VLS_intrinsicElements.div)({
    ...{ class: "flex justify-between items-center mb-5 px-1" },
});
__VLS_asFunctionalElement(__VLS_intrinsicElements.span, __VLS_intrinsicElements.span)({
    ...{ class: "text-[10px] font-black text-zinc-600 uppercase tracking-widest" },
});
const __VLS_8 = {}.Keyboard;
/** @type {[typeof __VLS_components.Keyboard, ]} */ ;
// @ts-ignore
const __VLS_9 = __VLS_asFunctionalComponent(__VLS_8, new __VLS_8({
    size: (14),
    ...{ class: "text-zinc-800" },
}));
const __VLS_10 = __VLS_9({
    size: (14),
    ...{ class: "text-zinc-800" },
}, ...__VLS_functionalComponentArgsRest(__VLS_9));
__VLS_asFunctionalElement(__VLS_intrinsicElements.div, __VLS_intrinsicElements.div)({
    ...{ class: "relative" },
});
__VLS_asFunctionalElement(__VLS_intrinsicElements.input)({
    ...{ onKeydown: (__VLS_ctx.handleKeyDown) },
    ...{ onFocus: (__VLS_ctx.startRecording) },
    ...{ onBlur: (__VLS_ctx.stopRecording) },
    readonly: true,
    value: (__VLS_ctx.isRecording ? '正在等待按键...' : __VLS_ctx.store.settings.magic_paste_shortcut),
    ...{ class: "shortcut-input" },
    ...{ class: ({ 'recording': __VLS_ctx.isRecording, 'error-shake': __VLS_ctx.showKeyWarning }) },
});
const __VLS_12 = {}.transition;
/** @type {[typeof __VLS_components.Transition, typeof __VLS_components.transition, typeof __VLS_components.Transition, typeof __VLS_components.transition, ]} */ ;
// @ts-ignore
const __VLS_13 = __VLS_asFunctionalComponent(__VLS_12, new __VLS_12({
    name: "slide-fade",
}));
const __VLS_14 = __VLS_13({
    name: "slide-fade",
}, ...__VLS_functionalComponentArgsRest(__VLS_13));
__VLS_15.slots.default;
if (__VLS_ctx.showKeyWarning) {
    __VLS_asFunctionalElement(__VLS_intrinsicElements.div, __VLS_intrinsicElements.div)({
        ...{ class: "absolute -bottom-7 left-0 right-0 flex justify-center" },
    });
    __VLS_asFunctionalElement(__VLS_intrinsicElements.span, __VLS_intrinsicElements.span)({
        ...{ class: "text-[9px] text-red-500 font-bold uppercase tracking-widest flex items-center gap-1.5 bg-[#0c0b0a] px-3 py-1 rounded-full border border-red-500/20" },
    });
    const __VLS_16 = {}.AlertTriangle;
    /** @type {[typeof __VLS_components.AlertTriangle, ]} */ ;
    // @ts-ignore
    const __VLS_17 = __VLS_asFunctionalComponent(__VLS_16, new __VLS_16({
        size: (10),
    }));
    const __VLS_18 = __VLS_17({
        size: (10),
    }, ...__VLS_functionalComponentArgsRest(__VLS_17));
}
var __VLS_15;
__VLS_asFunctionalElement(__VLS_intrinsicElements.section, __VLS_intrinsicElements.section)({
    ...{ class: "config-card" },
});
__VLS_asFunctionalElement(__VLS_intrinsicElements.div, __VLS_intrinsicElements.div)({
    ...{ class: "card-header" },
});
const __VLS_20 = {}.Volume2;
/** @type {[typeof __VLS_components.Volume2, ]} */ ;
// @ts-ignore
const __VLS_21 = __VLS_asFunctionalComponent(__VLS_20, new __VLS_20({
    size: (16),
    ...{ class: "text-amber-500" },
}));
const __VLS_22 = __VLS_21({
    size: (16),
    ...{ class: "text-amber-500" },
}, ...__VLS_functionalComponentArgsRest(__VLS_21));
__VLS_asFunctionalElement(__VLS_intrinsicElements.span, __VLS_intrinsicElements.span)({});
__VLS_asFunctionalElement(__VLS_intrinsicElements.div, __VLS_intrinsicElements.div)({
    ...{ class: "space-y-7 mt-8" },
});
__VLS_asFunctionalElement(__VLS_intrinsicElements.div, __VLS_intrinsicElements.div)({
    ...{ class: "setting-row-sm" },
});
__VLS_asFunctionalElement(__VLS_intrinsicElements.div, __VLS_intrinsicElements.div)({
    ...{ class: "flex items-center gap-3" },
});
const __VLS_24 = {}.Eye;
/** @type {[typeof __VLS_components.Eye, ]} */ ;
// @ts-ignore
const __VLS_25 = __VLS_asFunctionalComponent(__VLS_24, new __VLS_24({
    size: (14),
    ...{ class: "text-zinc-700" },
}));
const __VLS_26 = __VLS_25({
    size: (14),
    ...{ class: "text-zinc-700" },
}, ...__VLS_functionalComponentArgsRest(__VLS_25));
__VLS_asFunctionalElement(__VLS_intrinsicElements.span, __VLS_intrinsicElements.span)({
    ...{ class: "lbl-sm" },
});
__VLS_asFunctionalElement(__VLS_intrinsicElements.label, __VLS_intrinsicElements.label)({
    ...{ class: "sw-wrapper sm" },
});
__VLS_asFunctionalElement(__VLS_intrinsicElements.input, __VLS_intrinsicElements.input)({
    type: "checkbox",
});
(__VLS_ctx.store.settings.enable_visual_feedback);
__VLS_asFunctionalElement(__VLS_intrinsicElements.div, __VLS_intrinsicElements.div)({
    ...{ class: "sw-slider sm" },
});
__VLS_asFunctionalElement(__VLS_intrinsicElements.div, __VLS_intrinsicElements.div)({
    ...{ class: "setting-row-sm" },
});
__VLS_asFunctionalElement(__VLS_intrinsicElements.div, __VLS_intrinsicElements.div)({
    ...{ class: "flex items-center gap-3" },
});
const __VLS_28 = {}.Volume2;
/** @type {[typeof __VLS_components.Volume2, ]} */ ;
// @ts-ignore
const __VLS_29 = __VLS_asFunctionalComponent(__VLS_28, new __VLS_28({
    size: (14),
    ...{ class: "text-zinc-700" },
}));
const __VLS_30 = __VLS_29({
    size: (14),
    ...{ class: "text-zinc-700" },
}, ...__VLS_functionalComponentArgsRest(__VLS_29));
__VLS_asFunctionalElement(__VLS_intrinsicElements.span, __VLS_intrinsicElements.span)({
    ...{ class: "lbl-sm" },
});
__VLS_asFunctionalElement(__VLS_intrinsicElements.label, __VLS_intrinsicElements.label)({
    ...{ class: "sw-wrapper sm" },
});
__VLS_asFunctionalElement(__VLS_intrinsicElements.input, __VLS_intrinsicElements.input)({
    type: "checkbox",
});
(__VLS_ctx.store.settings.enable_audio_feedback);
__VLS_asFunctionalElement(__VLS_intrinsicElements.div, __VLS_intrinsicElements.div)({
    ...{ class: "sw-slider sm" },
});
__VLS_asFunctionalElement(__VLS_intrinsicElements.div, __VLS_intrinsicElements.div)({
    ...{ class: "pt-8 border-t border-white/[0.03] space-y-6" },
});
__VLS_asFunctionalElement(__VLS_intrinsicElements.div, __VLS_intrinsicElements.div)({
    ...{ class: "flex justify-between items-end" },
});
__VLS_asFunctionalElement(__VLS_intrinsicElements.div, __VLS_intrinsicElements.div)({
    ...{ class: "flex flex-col gap-1" },
});
__VLS_asFunctionalElement(__VLS_intrinsicElements.span, __VLS_intrinsicElements.span)({
    ...{ class: "lbl-sm flex items-center gap-2 font-bold text-zinc-300" },
});
const __VLS_32 = {}.Timer;
/** @type {[typeof __VLS_components.Timer, ]} */ ;
// @ts-ignore
const __VLS_33 = __VLS_asFunctionalComponent(__VLS_32, new __VLS_32({
    size: (14),
    ...{ class: "text-amber-500/60" },
}));
const __VLS_34 = __VLS_33({
    size: (14),
    ...{ class: "text-amber-500/60" },
}, ...__VLS_functionalComponentArgsRest(__VLS_33));
__VLS_asFunctionalElement(__VLS_intrinsicElements.span, __VLS_intrinsicElements.span)({
    ...{ class: "text-[8px] text-zinc-700 uppercase font-black tracking-widest" },
});
__VLS_asFunctionalElement(__VLS_intrinsicElements.span, __VLS_intrinsicElements.span)({
    ...{ class: "font-mono text-amber-200 text-sm font-bold bg-amber-500/10 px-3 py-1 rounded-lg border border-amber-500/20 shadow-[0_0_15px_rgba(245,158,11,0.1)]" },
});
(__VLS_ctx.store.settings.paste_delay_ms);
__VLS_asFunctionalElement(__VLS_intrinsicElements.div, __VLS_intrinsicElements.div)({
    ...{ class: "range-container" },
});
__VLS_asFunctionalElement(__VLS_intrinsicElements.input)({
    type: "range",
    min: "50",
    max: "800",
    step: "50",
    ...{ class: "precision-range" },
    ...{ style: ({ backgroundSize: `${__VLS_ctx.sliderProgress}% 100%` }) },
});
(__VLS_ctx.store.settings.paste_delay_ms);
__VLS_asFunctionalElement(__VLS_intrinsicElements.div, __VLS_intrinsicElements.div)({
    ...{ class: "range-ticks" },
});
for (const [n] of __VLS_getVForSourceType((8))) {
    __VLS_asFunctionalElement(__VLS_intrinsicElements.span, __VLS_intrinsicElements.span)({
        key: (n),
        ...{ class: ({ 'active': (n - 1) * 100 + 50 <= __VLS_ctx.store.settings.paste_delay_ms }) },
    });
}
__VLS_asFunctionalElement(__VLS_intrinsicElements.div, __VLS_intrinsicElements.div)({
    ...{ class: "flex justify-between items-center pt-10 border-t border-white/[0.03]" },
});
__VLS_asFunctionalElement(__VLS_intrinsicElements.button, __VLS_intrinsicElements.button)({
    ...{ onClick: (__VLS_ctx.store.clearHistory) },
    ...{ class: "purge-btn group" },
});
const __VLS_36 = {}.Trash2;
/** @type {[typeof __VLS_components.Trash2, ]} */ ;
// @ts-ignore
const __VLS_37 = __VLS_asFunctionalComponent(__VLS_36, new __VLS_36({
    size: (14),
    ...{ class: "group-hover:text-red-500 transition-colors" },
}));
const __VLS_38 = __VLS_37({
    size: (14),
    ...{ class: "group-hover:text-red-500 transition-colors" },
}, ...__VLS_functionalComponentArgsRest(__VLS_37));
__VLS_asFunctionalElement(__VLS_intrinsicElements.span, __VLS_intrinsicElements.span)({});
__VLS_asFunctionalElement(__VLS_intrinsicElements.button, __VLS_intrinsicElements.button)({
    ...{ onClick: (__VLS_ctx.handleSave) },
    ...{ class: "save-btn-jewelry" },
});
const __VLS_40 = {}.Save;
/** @type {[typeof __VLS_components.Save, ]} */ ;
// @ts-ignore
const __VLS_41 = __VLS_asFunctionalComponent(__VLS_40, new __VLS_40({
    size: (18),
}));
const __VLS_42 = __VLS_41({
    size: (18),
}, ...__VLS_functionalComponentArgsRest(__VLS_41));
__VLS_asFunctionalElement(__VLS_intrinsicElements.span, __VLS_intrinsicElements.span)({});
/** @type {__VLS_StyleScopedClasses['max-w-5xl']} */ ;
/** @type {__VLS_StyleScopedClasses['mx-auto']} */ ;
/** @type {__VLS_StyleScopedClasses['space-y-10']} */ ;
/** @type {__VLS_StyleScopedClasses['animate-in']} */ ;
/** @type {__VLS_StyleScopedClasses['fade-in']} */ ;
/** @type {__VLS_StyleScopedClasses['duration-700']} */ ;
/** @type {__VLS_StyleScopedClasses['pb-16']} */ ;
/** @type {__VLS_StyleScopedClasses['flex']} */ ;
/** @type {__VLS_StyleScopedClasses['items-center']} */ ;
/** @type {__VLS_StyleScopedClasses['gap-6']} */ ;
/** @type {__VLS_StyleScopedClasses['mb-12']} */ ;
/** @type {__VLS_StyleScopedClasses['px-2']} */ ;
/** @type {__VLS_StyleScopedClasses['w-14']} */ ;
/** @type {__VLS_StyleScopedClasses['h-14']} */ ;
/** @type {__VLS_StyleScopedClasses['rounded-2xl']} */ ;
/** @type {__VLS_StyleScopedClasses['bg-[#141210]']} */ ;
/** @type {__VLS_StyleScopedClasses['border']} */ ;
/** @type {__VLS_StyleScopedClasses['border-amber-500/10']} */ ;
/** @type {__VLS_StyleScopedClasses['flex']} */ ;
/** @type {__VLS_StyleScopedClasses['items-center']} */ ;
/** @type {__VLS_StyleScopedClasses['justify-center']} */ ;
/** @type {__VLS_StyleScopedClasses['shadow-2xl']} */ ;
/** @type {__VLS_StyleScopedClasses['shadow-black']} */ ;
/** @type {__VLS_StyleScopedClasses['text-amber-400/80']} */ ;
/** @type {__VLS_StyleScopedClasses['w-6']} */ ;
/** @type {__VLS_StyleScopedClasses['h-6']} */ ;
/** @type {__VLS_StyleScopedClasses['text-3xl']} */ ;
/** @type {__VLS_StyleScopedClasses['font-bold']} */ ;
/** @type {__VLS_StyleScopedClasses['text-amber-50/90']} */ ;
/** @type {__VLS_StyleScopedClasses['tracking-tight']} */ ;
/** @type {__VLS_StyleScopedClasses['text-[10px]']} */ ;
/** @type {__VLS_StyleScopedClasses['text-zinc-600']} */ ;
/** @type {__VLS_StyleScopedClasses['font-black']} */ ;
/** @type {__VLS_StyleScopedClasses['uppercase']} */ ;
/** @type {__VLS_StyleScopedClasses['tracking-[0.4em]']} */ ;
/** @type {__VLS_StyleScopedClasses['mt-1.5']} */ ;
/** @type {__VLS_StyleScopedClasses['opacity-60']} */ ;
/** @type {__VLS_StyleScopedClasses['grid']} */ ;
/** @type {__VLS_StyleScopedClasses['grid-cols-1']} */ ;
/** @type {__VLS_StyleScopedClasses['lg:grid-cols-2']} */ ;
/** @type {__VLS_StyleScopedClasses['gap-8']} */ ;
/** @type {__VLS_StyleScopedClasses['config-card']} */ ;
/** @type {__VLS_StyleScopedClasses['card-header']} */ ;
/** @type {__VLS_StyleScopedClasses['text-blue-500']} */ ;
/** @type {__VLS_StyleScopedClasses['space-y-8']} */ ;
/** @type {__VLS_StyleScopedClasses['mt-8']} */ ;
/** @type {__VLS_StyleScopedClasses['setting-row']} */ ;
/** @type {__VLS_StyleScopedClasses['info']} */ ;
/** @type {__VLS_StyleScopedClasses['lbl']} */ ;
/** @type {__VLS_StyleScopedClasses['dsc']} */ ;
/** @type {__VLS_StyleScopedClasses['sw-wrapper']} */ ;
/** @type {__VLS_StyleScopedClasses['sw-slider']} */ ;
/** @type {__VLS_StyleScopedClasses['relative']} */ ;
/** @type {__VLS_StyleScopedClasses['p-7']} */ ;
/** @type {__VLS_StyleScopedClasses['bg-black/40']} */ ;
/** @type {__VLS_StyleScopedClasses['rounded-[2rem]']} */ ;
/** @type {__VLS_StyleScopedClasses['border']} */ ;
/** @type {__VLS_StyleScopedClasses['border-white/[0.03]']} */ ;
/** @type {__VLS_StyleScopedClasses['shadow-inner']} */ ;
/** @type {__VLS_StyleScopedClasses['group']} */ ;
/** @type {__VLS_StyleScopedClasses['flex']} */ ;
/** @type {__VLS_StyleScopedClasses['justify-between']} */ ;
/** @type {__VLS_StyleScopedClasses['items-center']} */ ;
/** @type {__VLS_StyleScopedClasses['mb-5']} */ ;
/** @type {__VLS_StyleScopedClasses['px-1']} */ ;
/** @type {__VLS_StyleScopedClasses['text-[10px]']} */ ;
/** @type {__VLS_StyleScopedClasses['font-black']} */ ;
/** @type {__VLS_StyleScopedClasses['text-zinc-600']} */ ;
/** @type {__VLS_StyleScopedClasses['uppercase']} */ ;
/** @type {__VLS_StyleScopedClasses['tracking-widest']} */ ;
/** @type {__VLS_StyleScopedClasses['text-zinc-800']} */ ;
/** @type {__VLS_StyleScopedClasses['relative']} */ ;
/** @type {__VLS_StyleScopedClasses['shortcut-input']} */ ;
/** @type {__VLS_StyleScopedClasses['absolute']} */ ;
/** @type {__VLS_StyleScopedClasses['-bottom-7']} */ ;
/** @type {__VLS_StyleScopedClasses['left-0']} */ ;
/** @type {__VLS_StyleScopedClasses['right-0']} */ ;
/** @type {__VLS_StyleScopedClasses['flex']} */ ;
/** @type {__VLS_StyleScopedClasses['justify-center']} */ ;
/** @type {__VLS_StyleScopedClasses['text-[9px]']} */ ;
/** @type {__VLS_StyleScopedClasses['text-red-500']} */ ;
/** @type {__VLS_StyleScopedClasses['font-bold']} */ ;
/** @type {__VLS_StyleScopedClasses['uppercase']} */ ;
/** @type {__VLS_StyleScopedClasses['tracking-widest']} */ ;
/** @type {__VLS_StyleScopedClasses['flex']} */ ;
/** @type {__VLS_StyleScopedClasses['items-center']} */ ;
/** @type {__VLS_StyleScopedClasses['gap-1.5']} */ ;
/** @type {__VLS_StyleScopedClasses['bg-[#0c0b0a]']} */ ;
/** @type {__VLS_StyleScopedClasses['px-3']} */ ;
/** @type {__VLS_StyleScopedClasses['py-1']} */ ;
/** @type {__VLS_StyleScopedClasses['rounded-full']} */ ;
/** @type {__VLS_StyleScopedClasses['border']} */ ;
/** @type {__VLS_StyleScopedClasses['border-red-500/20']} */ ;
/** @type {__VLS_StyleScopedClasses['config-card']} */ ;
/** @type {__VLS_StyleScopedClasses['card-header']} */ ;
/** @type {__VLS_StyleScopedClasses['text-amber-500']} */ ;
/** @type {__VLS_StyleScopedClasses['space-y-7']} */ ;
/** @type {__VLS_StyleScopedClasses['mt-8']} */ ;
/** @type {__VLS_StyleScopedClasses['setting-row-sm']} */ ;
/** @type {__VLS_StyleScopedClasses['flex']} */ ;
/** @type {__VLS_StyleScopedClasses['items-center']} */ ;
/** @type {__VLS_StyleScopedClasses['gap-3']} */ ;
/** @type {__VLS_StyleScopedClasses['text-zinc-700']} */ ;
/** @type {__VLS_StyleScopedClasses['lbl-sm']} */ ;
/** @type {__VLS_StyleScopedClasses['sw-wrapper']} */ ;
/** @type {__VLS_StyleScopedClasses['sm']} */ ;
/** @type {__VLS_StyleScopedClasses['sw-slider']} */ ;
/** @type {__VLS_StyleScopedClasses['sm']} */ ;
/** @type {__VLS_StyleScopedClasses['setting-row-sm']} */ ;
/** @type {__VLS_StyleScopedClasses['flex']} */ ;
/** @type {__VLS_StyleScopedClasses['items-center']} */ ;
/** @type {__VLS_StyleScopedClasses['gap-3']} */ ;
/** @type {__VLS_StyleScopedClasses['text-zinc-700']} */ ;
/** @type {__VLS_StyleScopedClasses['lbl-sm']} */ ;
/** @type {__VLS_StyleScopedClasses['sw-wrapper']} */ ;
/** @type {__VLS_StyleScopedClasses['sm']} */ ;
/** @type {__VLS_StyleScopedClasses['sw-slider']} */ ;
/** @type {__VLS_StyleScopedClasses['sm']} */ ;
/** @type {__VLS_StyleScopedClasses['pt-8']} */ ;
/** @type {__VLS_StyleScopedClasses['border-t']} */ ;
/** @type {__VLS_StyleScopedClasses['border-white/[0.03]']} */ ;
/** @type {__VLS_StyleScopedClasses['space-y-6']} */ ;
/** @type {__VLS_StyleScopedClasses['flex']} */ ;
/** @type {__VLS_StyleScopedClasses['justify-between']} */ ;
/** @type {__VLS_StyleScopedClasses['items-end']} */ ;
/** @type {__VLS_StyleScopedClasses['flex']} */ ;
/** @type {__VLS_StyleScopedClasses['flex-col']} */ ;
/** @type {__VLS_StyleScopedClasses['gap-1']} */ ;
/** @type {__VLS_StyleScopedClasses['lbl-sm']} */ ;
/** @type {__VLS_StyleScopedClasses['flex']} */ ;
/** @type {__VLS_StyleScopedClasses['items-center']} */ ;
/** @type {__VLS_StyleScopedClasses['gap-2']} */ ;
/** @type {__VLS_StyleScopedClasses['font-bold']} */ ;
/** @type {__VLS_StyleScopedClasses['text-zinc-300']} */ ;
/** @type {__VLS_StyleScopedClasses['text-amber-500/60']} */ ;
/** @type {__VLS_StyleScopedClasses['text-[8px]']} */ ;
/** @type {__VLS_StyleScopedClasses['text-zinc-700']} */ ;
/** @type {__VLS_StyleScopedClasses['uppercase']} */ ;
/** @type {__VLS_StyleScopedClasses['font-black']} */ ;
/** @type {__VLS_StyleScopedClasses['tracking-widest']} */ ;
/** @type {__VLS_StyleScopedClasses['font-mono']} */ ;
/** @type {__VLS_StyleScopedClasses['text-amber-200']} */ ;
/** @type {__VLS_StyleScopedClasses['text-sm']} */ ;
/** @type {__VLS_StyleScopedClasses['font-bold']} */ ;
/** @type {__VLS_StyleScopedClasses['bg-amber-500/10']} */ ;
/** @type {__VLS_StyleScopedClasses['px-3']} */ ;
/** @type {__VLS_StyleScopedClasses['py-1']} */ ;
/** @type {__VLS_StyleScopedClasses['rounded-lg']} */ ;
/** @type {__VLS_StyleScopedClasses['border']} */ ;
/** @type {__VLS_StyleScopedClasses['border-amber-500/20']} */ ;
/** @type {__VLS_StyleScopedClasses['shadow-[0_0_15px_rgba(245,158,11,0.1)]']} */ ;
/** @type {__VLS_StyleScopedClasses['range-container']} */ ;
/** @type {__VLS_StyleScopedClasses['precision-range']} */ ;
/** @type {__VLS_StyleScopedClasses['range-ticks']} */ ;
/** @type {__VLS_StyleScopedClasses['flex']} */ ;
/** @type {__VLS_StyleScopedClasses['justify-between']} */ ;
/** @type {__VLS_StyleScopedClasses['items-center']} */ ;
/** @type {__VLS_StyleScopedClasses['pt-10']} */ ;
/** @type {__VLS_StyleScopedClasses['border-t']} */ ;
/** @type {__VLS_StyleScopedClasses['border-white/[0.03]']} */ ;
/** @type {__VLS_StyleScopedClasses['purge-btn']} */ ;
/** @type {__VLS_StyleScopedClasses['group']} */ ;
/** @type {__VLS_StyleScopedClasses['group-hover:text-red-500']} */ ;
/** @type {__VLS_StyleScopedClasses['transition-colors']} */ ;
/** @type {__VLS_StyleScopedClasses['save-btn-jewelry']} */ ;
var __VLS_dollars;
const __VLS_self = (await import('vue')).defineComponent({
    setup() {
        return {
            Keyboard: Keyboard,
            Timer: Timer,
            Save: Save,
            Trash2: Trash2,
            Monitor: Monitor,
            Cpu: Cpu,
            Volume2: Volume2,
            Eye: Eye,
            AlertTriangle: AlertTriangle,
            store: store,
            isRecording: isRecording,
            showKeyWarning: showKeyWarning,
            startRecording: startRecording,
            stopRecording: stopRecording,
            handleKeyDown: handleKeyDown,
            handleSave: handleSave,
            sliderProgress: sliderProgress,
        };
    },
});
export default (await import('vue')).defineComponent({
    setup() {
        return {};
    },
});
; /* PartiallyEnd: #4569/main.vue */
//# sourceMappingURL=Settings.vue.js.map