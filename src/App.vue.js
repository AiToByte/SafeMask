/// <reference types="../node_modules/.vue-global-types/vue_3.5_0_0_0.d.ts" />
import { onMounted } from 'vue';
import { useAppStore } from './stores/useAppStore';
import { Pin, PinOff, Shield, Ghost, Activity } from 'lucide-vue-next';
import Sidebar from './components/Sidebar.vue';
import MagicFeedback from './components/MagicFeedback.vue';
import FileProcessor from './components/FileProcessor.vue';
import HistoryList from './components/HistoryList.vue';
import RuleManager from './components/RuleManager.vue';
import SettingsPage from './components/Settings.vue';
import ExitConfirm from './components/ExitConfirm.vue';
import StatCard from './components/StatCard.vue';
const store = useAppStore();
onMounted(() => store.bootstrap());
debugger; /* PartiallyEnd: #3632/scriptSetup.vue */
const __VLS_ctx = {};
let __VLS_components;
let __VLS_directives;
// CSS variable injection 
// CSS variable injection end 
__VLS_asFunctionalElement(__VLS_intrinsicElements.div, __VLS_intrinsicElements.div)({
    ...{ class: "flex h-screen bg-[#0c0b0a] text-amber-50/90 select-none overflow-hidden font-sans" },
});
/** @type {[typeof MagicFeedback, ]} */ ;
// @ts-ignore
const __VLS_0 = __VLS_asFunctionalComponent(MagicFeedback, new MagicFeedback({}));
const __VLS_1 = __VLS_0({}, ...__VLS_functionalComponentArgsRest(__VLS_0));
/** @type {[typeof Sidebar, ]} */ ;
// @ts-ignore
const __VLS_3 = __VLS_asFunctionalComponent(Sidebar, new Sidebar({}));
const __VLS_4 = __VLS_3({}, ...__VLS_functionalComponentArgsRest(__VLS_3));
__VLS_asFunctionalElement(__VLS_intrinsicElements.main, __VLS_intrinsicElements.main)({
    ...{ class: "flex-1 flex flex-col min-w-0 relative" },
});
__VLS_asFunctionalElement(__VLS_intrinsicElements.div, __VLS_intrinsicElements.div)({
    ...{ class: "absolute top-0 left-1/4 w-[60%] h-[30%] bg-amber-600/[0.02] blur-[120px] pointer-events-none" },
});
__VLS_asFunctionalElement(__VLS_intrinsicElements.header, __VLS_intrinsicElements.header)({
    ...{ class: "h-20 flex items-center justify-between px-10 z-40 border-b border-white/[0.03] bg-[#0c0b0a]/60 backdrop-blur-xl" },
});
__VLS_asFunctionalElement(__VLS_intrinsicElements.div, __VLS_intrinsicElements.div)({
    ...{ class: "flex items-center gap-5" },
});
__VLS_asFunctionalElement(__VLS_intrinsicElements.div, __VLS_intrinsicElements.div)({
    ...{ class: "w-10 h-10 rounded-lg bg-[#141210] border border-amber-500/10 flex items-center justify-center shadow-2xl relative overflow-hidden group" },
});
const __VLS_6 = {}.Activity;
/** @type {[typeof __VLS_components.Activity, ]} */ ;
// @ts-ignore
const __VLS_7 = __VLS_asFunctionalComponent(__VLS_6, new __VLS_6({
    ...{ class: "text-amber-500 w-4 h-4 relative z-10" },
}));
const __VLS_8 = __VLS_7({
    ...{ class: "text-amber-500 w-4 h-4 relative z-10" },
}, ...__VLS_functionalComponentArgsRest(__VLS_7));
__VLS_asFunctionalElement(__VLS_intrinsicElements.div, __VLS_intrinsicElements.div)({});
__VLS_asFunctionalElement(__VLS_intrinsicElements.h1, __VLS_intrinsicElements.h1)({
    ...{ class: "text-lg font-bold tracking-tight text-amber-50/90 flex items-center gap-3" },
});
__VLS_asFunctionalElement(__VLS_intrinsicElements.div, __VLS_intrinsicElements.div)({
    ...{ class: "h-3 w-[1px] bg-white/10" },
});
__VLS_asFunctionalElement(__VLS_intrinsicElements.span, __VLS_intrinsicElements.span)({
    ...{ class: "text-zinc-500 font-medium text-sm tracking-widest" },
});
__VLS_asFunctionalElement(__VLS_intrinsicElements.p, __VLS_intrinsicElements.p)({
    ...{ class: "text-[8px] text-zinc-600 font-bold tracking-[0.1em] uppercase" },
});
__VLS_asFunctionalElement(__VLS_intrinsicElements.div, __VLS_intrinsicElements.div)({
    ...{ class: "flex items-center gap-3" },
});
__VLS_asFunctionalElement(__VLS_intrinsicElements.button, __VLS_intrinsicElements.button)({
    ...{ onClick: (__VLS_ctx.store.toggleAlwaysOnTop) },
    ...{ class: "w-9 h-9 rounded-lg border transition-all duration-300 flex items-center justify-center active:scale-90" },
    ...{ class: (__VLS_ctx.store.isAlwaysOnTop
            ? 'bg-amber-500/20 border-amber-500/40 text-amber-300 shadow-[0_0_15px_rgba(245,158,11,0.2)]'
            : 'bg-white/[0.02] border-white/5 text-zinc-500 hover:border-amber-500/20') },
});
const __VLS_10 = ((__VLS_ctx.store.isAlwaysOnTop ? __VLS_ctx.PinOff : __VLS_ctx.Pin));
// @ts-ignore
const __VLS_11 = __VLS_asFunctionalComponent(__VLS_10, new __VLS_10({
    size: (14),
}));
const __VLS_12 = __VLS_11({
    size: (14),
}, ...__VLS_functionalComponentArgsRest(__VLS_11));
__VLS_asFunctionalElement(__VLS_intrinsicElements.div, __VLS_intrinsicElements.div)({
    ...{ onClick: (__VLS_ctx.store.toggleVaultMode) },
    ...{ class: "group flex items-center gap-4 bg-[#141210] border border-white/[0.05] h-9 px-4 rounded-xl cursor-pointer hover:border-blue-500/30 transition-all duration-500 active:scale-95" },
});
__VLS_asFunctionalElement(__VLS_intrinsicElements.span, __VLS_intrinsicElements.span)({
    ...{ class: "text-[10px] font-bold tracking-widest transition-colors duration-300" },
    ...{ class: (__VLS_ctx.store.settings.shadow_mode_enabled ? 'text-amber-200/80' : 'text-blue-300/80') },
});
(__VLS_ctx.store.settings.shadow_mode_enabled ? '影子宇宙' : '哨兵宇宙');
__VLS_asFunctionalElement(__VLS_intrinsicElements.div, __VLS_intrinsicElements.div)({
    ...{ class: "w-5 h-5 flex items-center justify-center rounded-md bg-white/[0.02] border border-white/5" },
});
const __VLS_14 = ((__VLS_ctx.store.settings.shadow_mode_enabled ? __VLS_ctx.Ghost : __VLS_ctx.Shield));
// @ts-ignore
const __VLS_15 = __VLS_asFunctionalComponent(__VLS_14, new __VLS_14({
    size: (10),
    ...{ class: (__VLS_ctx.store.settings.shadow_mode_enabled ? 'text-amber-200' : 'text-blue-300') },
}));
const __VLS_16 = __VLS_15({
    size: (10),
    ...{ class: (__VLS_ctx.store.settings.shadow_mode_enabled ? 'text-amber-200' : 'text-blue-300') },
}, ...__VLS_functionalComponentArgsRest(__VLS_15));
__VLS_asFunctionalElement(__VLS_intrinsicElements.div, __VLS_intrinsicElements.div)({
    ...{ class: "flex-1 overflow-hidden px-10 py-4 flex flex-col" },
});
const __VLS_18 = {}.Transition;
/** @type {[typeof __VLS_components.Transition, typeof __VLS_components.Transition, ]} */ ;
// @ts-ignore
const __VLS_19 = __VLS_asFunctionalComponent(__VLS_18, new __VLS_18({
    name: "page",
    mode: "out-in",
}));
const __VLS_20 = __VLS_19({
    name: "page",
    mode: "out-in",
}, ...__VLS_functionalComponentArgsRest(__VLS_19));
__VLS_21.slots.default;
__VLS_asFunctionalElement(__VLS_intrinsicElements.div, __VLS_intrinsicElements.div)({
    key: (__VLS_ctx.store.activeTab),
    ...{ class: "max-w-6xl mx-auto w-full h-full flex flex-col" },
});
if (__VLS_ctx.store.activeTab === 'dashboard') {
    __VLS_asFunctionalElement(__VLS_intrinsicElements.div, __VLS_intrinsicElements.div)({
        ...{ class: "flex-1 flex flex-col gap-4" },
    });
    __VLS_asFunctionalElement(__VLS_intrinsicElements.div, __VLS_intrinsicElements.div)({
        ...{ class: "grid grid-cols-3 gap-4 shrink-0" },
    });
    /** @type {[typeof StatCard, ]} */ ;
    // @ts-ignore
    const __VLS_22 = __VLS_asFunctionalComponent(StatCard, new StatCard({
        ...{ 'onClick': {} },
        title: "已装载脱敏规则",
        value: (__VLS_ctx.store.ruleCount),
        unit: "Patterns",
        color: "text-amber-200",
        type: "amber",
        clickable: true,
    }));
    const __VLS_23 = __VLS_22({
        ...{ 'onClick': {} },
        title: "已装载脱敏规则",
        value: (__VLS_ctx.store.ruleCount),
        unit: "Patterns",
        color: "text-amber-200",
        type: "amber",
        clickable: true,
    }, ...__VLS_functionalComponentArgsRest(__VLS_22));
    let __VLS_25;
    let __VLS_26;
    let __VLS_27;
    const __VLS_28 = {
        onClick: (...[$event]) => {
            if (!(__VLS_ctx.store.activeTab === 'dashboard'))
                return;
            __VLS_ctx.store.activeTab = 'rules';
        }
    };
    var __VLS_24;
    /** @type {[typeof StatCard, ]} */ ;
    // @ts-ignore
    const __VLS_29 = __VLS_asFunctionalComponent(StatCard, new StatCard({
        ...{ 'onClick': {} },
        title: "累计隐私审计记录",
        value: (__VLS_ctx.store.historyList.length),
        unit: "Records",
        color: "text-blue-300",
        type: "blue",
        clickable: true,
    }));
    const __VLS_30 = __VLS_29({
        ...{ 'onClick': {} },
        title: "累计隐私审计记录",
        value: (__VLS_ctx.store.historyList.length),
        unit: "Records",
        color: "text-blue-300",
        type: "blue",
        clickable: true,
    }, ...__VLS_functionalComponentArgsRest(__VLS_29));
    let __VLS_32;
    let __VLS_33;
    let __VLS_34;
    const __VLS_35 = {
        onClick: (...[$event]) => {
            if (!(__VLS_ctx.store.activeTab === 'dashboard'))
                return;
            __VLS_ctx.store.activeTab = 'history';
        }
    };
    var __VLS_31;
    /** @type {[typeof StatCard, ]} */ ;
    // @ts-ignore
    const __VLS_36 = __VLS_asFunctionalComponent(StatCard, new StatCard({
        title: "脱敏引擎状态",
        value: "无损运行",
        unit: "Normal",
        color: "text-emerald-300",
        type: "emerald",
    }));
    const __VLS_37 = __VLS_36({
        title: "脱敏引擎状态",
        value: "无损运行",
        unit: "Normal",
        color: "text-emerald-300",
        type: "emerald",
    }, ...__VLS_functionalComponentArgsRest(__VLS_36));
    __VLS_asFunctionalElement(__VLS_intrinsicElements.div, __VLS_intrinsicElements.div)({
        ...{ class: "flex-1 min-h-0 relative" },
    });
    /** @type {[typeof FileProcessor, ]} */ ;
    // @ts-ignore
    const __VLS_39 = __VLS_asFunctionalComponent(FileProcessor, new FileProcessor({
        ...{ class: "h-full bg-[#110f0e]/50 border border-white/[0.02] shadow-2xl" },
    }));
    const __VLS_40 = __VLS_39({
        ...{ class: "h-full bg-[#110f0e]/50 border border-white/[0.02] shadow-2xl" },
    }, ...__VLS_functionalComponentArgsRest(__VLS_39));
    __VLS_asFunctionalElement(__VLS_intrinsicElements.footer, __VLS_intrinsicElements.footer)({
        ...{ class: "flex justify-center py-1 opacity-10" },
    });
    __VLS_asFunctionalElement(__VLS_intrinsicElements.p, __VLS_intrinsicElements.p)({
        ...{ class: "text-[7px] font-mono uppercase tracking-[0.5em] text-white" },
    });
}
else {
    __VLS_asFunctionalElement(__VLS_intrinsicElements.div, __VLS_intrinsicElements.div)({
        ...{ class: "flex-1 overflow-y-auto custom-scroll" },
    });
    if (__VLS_ctx.store.activeTab === 'history') {
        /** @type {[typeof HistoryList, ]} */ ;
        // @ts-ignore
        const __VLS_42 = __VLS_asFunctionalComponent(HistoryList, new HistoryList({}));
        const __VLS_43 = __VLS_42({}, ...__VLS_functionalComponentArgsRest(__VLS_42));
    }
    else if (__VLS_ctx.store.activeTab === 'rules') {
        /** @type {[typeof RuleManager, ]} */ ;
        // @ts-ignore
        const __VLS_45 = __VLS_asFunctionalComponent(RuleManager, new RuleManager({}));
        const __VLS_46 = __VLS_45({}, ...__VLS_functionalComponentArgsRest(__VLS_45));
    }
    else if (__VLS_ctx.store.activeTab === 'settings') {
        /** @type {[typeof SettingsPage, ]} */ ;
        // @ts-ignore
        const __VLS_48 = __VLS_asFunctionalComponent(SettingsPage, new SettingsPage({}));
        const __VLS_49 = __VLS_48({}, ...__VLS_functionalComponentArgsRest(__VLS_48));
    }
}
var __VLS_21;
/** @type {[typeof ExitConfirm, ]} */ ;
// @ts-ignore
const __VLS_51 = __VLS_asFunctionalComponent(ExitConfirm, new ExitConfirm({}));
const __VLS_52 = __VLS_51({}, ...__VLS_functionalComponentArgsRest(__VLS_51));
/** @type {__VLS_StyleScopedClasses['flex']} */ ;
/** @type {__VLS_StyleScopedClasses['h-screen']} */ ;
/** @type {__VLS_StyleScopedClasses['bg-[#0c0b0a]']} */ ;
/** @type {__VLS_StyleScopedClasses['text-amber-50/90']} */ ;
/** @type {__VLS_StyleScopedClasses['select-none']} */ ;
/** @type {__VLS_StyleScopedClasses['overflow-hidden']} */ ;
/** @type {__VLS_StyleScopedClasses['font-sans']} */ ;
/** @type {__VLS_StyleScopedClasses['flex-1']} */ ;
/** @type {__VLS_StyleScopedClasses['flex']} */ ;
/** @type {__VLS_StyleScopedClasses['flex-col']} */ ;
/** @type {__VLS_StyleScopedClasses['min-w-0']} */ ;
/** @type {__VLS_StyleScopedClasses['relative']} */ ;
/** @type {__VLS_StyleScopedClasses['absolute']} */ ;
/** @type {__VLS_StyleScopedClasses['top-0']} */ ;
/** @type {__VLS_StyleScopedClasses['left-1/4']} */ ;
/** @type {__VLS_StyleScopedClasses['w-[60%]']} */ ;
/** @type {__VLS_StyleScopedClasses['h-[30%]']} */ ;
/** @type {__VLS_StyleScopedClasses['bg-amber-600/[0.02]']} */ ;
/** @type {__VLS_StyleScopedClasses['blur-[120px]']} */ ;
/** @type {__VLS_StyleScopedClasses['pointer-events-none']} */ ;
/** @type {__VLS_StyleScopedClasses['h-20']} */ ;
/** @type {__VLS_StyleScopedClasses['flex']} */ ;
/** @type {__VLS_StyleScopedClasses['items-center']} */ ;
/** @type {__VLS_StyleScopedClasses['justify-between']} */ ;
/** @type {__VLS_StyleScopedClasses['px-10']} */ ;
/** @type {__VLS_StyleScopedClasses['z-40']} */ ;
/** @type {__VLS_StyleScopedClasses['border-b']} */ ;
/** @type {__VLS_StyleScopedClasses['border-white/[0.03]']} */ ;
/** @type {__VLS_StyleScopedClasses['bg-[#0c0b0a]/60']} */ ;
/** @type {__VLS_StyleScopedClasses['backdrop-blur-xl']} */ ;
/** @type {__VLS_StyleScopedClasses['flex']} */ ;
/** @type {__VLS_StyleScopedClasses['items-center']} */ ;
/** @type {__VLS_StyleScopedClasses['gap-5']} */ ;
/** @type {__VLS_StyleScopedClasses['w-10']} */ ;
/** @type {__VLS_StyleScopedClasses['h-10']} */ ;
/** @type {__VLS_StyleScopedClasses['rounded-lg']} */ ;
/** @type {__VLS_StyleScopedClasses['bg-[#141210]']} */ ;
/** @type {__VLS_StyleScopedClasses['border']} */ ;
/** @type {__VLS_StyleScopedClasses['border-amber-500/10']} */ ;
/** @type {__VLS_StyleScopedClasses['flex']} */ ;
/** @type {__VLS_StyleScopedClasses['items-center']} */ ;
/** @type {__VLS_StyleScopedClasses['justify-center']} */ ;
/** @type {__VLS_StyleScopedClasses['shadow-2xl']} */ ;
/** @type {__VLS_StyleScopedClasses['relative']} */ ;
/** @type {__VLS_StyleScopedClasses['overflow-hidden']} */ ;
/** @type {__VLS_StyleScopedClasses['group']} */ ;
/** @type {__VLS_StyleScopedClasses['text-amber-500']} */ ;
/** @type {__VLS_StyleScopedClasses['w-4']} */ ;
/** @type {__VLS_StyleScopedClasses['h-4']} */ ;
/** @type {__VLS_StyleScopedClasses['relative']} */ ;
/** @type {__VLS_StyleScopedClasses['z-10']} */ ;
/** @type {__VLS_StyleScopedClasses['text-lg']} */ ;
/** @type {__VLS_StyleScopedClasses['font-bold']} */ ;
/** @type {__VLS_StyleScopedClasses['tracking-tight']} */ ;
/** @type {__VLS_StyleScopedClasses['text-amber-50/90']} */ ;
/** @type {__VLS_StyleScopedClasses['flex']} */ ;
/** @type {__VLS_StyleScopedClasses['items-center']} */ ;
/** @type {__VLS_StyleScopedClasses['gap-3']} */ ;
/** @type {__VLS_StyleScopedClasses['h-3']} */ ;
/** @type {__VLS_StyleScopedClasses['w-[1px]']} */ ;
/** @type {__VLS_StyleScopedClasses['bg-white/10']} */ ;
/** @type {__VLS_StyleScopedClasses['text-zinc-500']} */ ;
/** @type {__VLS_StyleScopedClasses['font-medium']} */ ;
/** @type {__VLS_StyleScopedClasses['text-sm']} */ ;
/** @type {__VLS_StyleScopedClasses['tracking-widest']} */ ;
/** @type {__VLS_StyleScopedClasses['text-[8px]']} */ ;
/** @type {__VLS_StyleScopedClasses['text-zinc-600']} */ ;
/** @type {__VLS_StyleScopedClasses['font-bold']} */ ;
/** @type {__VLS_StyleScopedClasses['tracking-[0.1em]']} */ ;
/** @type {__VLS_StyleScopedClasses['uppercase']} */ ;
/** @type {__VLS_StyleScopedClasses['flex']} */ ;
/** @type {__VLS_StyleScopedClasses['items-center']} */ ;
/** @type {__VLS_StyleScopedClasses['gap-3']} */ ;
/** @type {__VLS_StyleScopedClasses['w-9']} */ ;
/** @type {__VLS_StyleScopedClasses['h-9']} */ ;
/** @type {__VLS_StyleScopedClasses['rounded-lg']} */ ;
/** @type {__VLS_StyleScopedClasses['border']} */ ;
/** @type {__VLS_StyleScopedClasses['transition-all']} */ ;
/** @type {__VLS_StyleScopedClasses['duration-300']} */ ;
/** @type {__VLS_StyleScopedClasses['flex']} */ ;
/** @type {__VLS_StyleScopedClasses['items-center']} */ ;
/** @type {__VLS_StyleScopedClasses['justify-center']} */ ;
/** @type {__VLS_StyleScopedClasses['active:scale-90']} */ ;
/** @type {__VLS_StyleScopedClasses['group']} */ ;
/** @type {__VLS_StyleScopedClasses['flex']} */ ;
/** @type {__VLS_StyleScopedClasses['items-center']} */ ;
/** @type {__VLS_StyleScopedClasses['gap-4']} */ ;
/** @type {__VLS_StyleScopedClasses['bg-[#141210]']} */ ;
/** @type {__VLS_StyleScopedClasses['border']} */ ;
/** @type {__VLS_StyleScopedClasses['border-white/[0.05]']} */ ;
/** @type {__VLS_StyleScopedClasses['h-9']} */ ;
/** @type {__VLS_StyleScopedClasses['px-4']} */ ;
/** @type {__VLS_StyleScopedClasses['rounded-xl']} */ ;
/** @type {__VLS_StyleScopedClasses['cursor-pointer']} */ ;
/** @type {__VLS_StyleScopedClasses['hover:border-blue-500/30']} */ ;
/** @type {__VLS_StyleScopedClasses['transition-all']} */ ;
/** @type {__VLS_StyleScopedClasses['duration-500']} */ ;
/** @type {__VLS_StyleScopedClasses['active:scale-95']} */ ;
/** @type {__VLS_StyleScopedClasses['text-[10px]']} */ ;
/** @type {__VLS_StyleScopedClasses['font-bold']} */ ;
/** @type {__VLS_StyleScopedClasses['tracking-widest']} */ ;
/** @type {__VLS_StyleScopedClasses['transition-colors']} */ ;
/** @type {__VLS_StyleScopedClasses['duration-300']} */ ;
/** @type {__VLS_StyleScopedClasses['w-5']} */ ;
/** @type {__VLS_StyleScopedClasses['h-5']} */ ;
/** @type {__VLS_StyleScopedClasses['flex']} */ ;
/** @type {__VLS_StyleScopedClasses['items-center']} */ ;
/** @type {__VLS_StyleScopedClasses['justify-center']} */ ;
/** @type {__VLS_StyleScopedClasses['rounded-md']} */ ;
/** @type {__VLS_StyleScopedClasses['bg-white/[0.02]']} */ ;
/** @type {__VLS_StyleScopedClasses['border']} */ ;
/** @type {__VLS_StyleScopedClasses['border-white/5']} */ ;
/** @type {__VLS_StyleScopedClasses['flex-1']} */ ;
/** @type {__VLS_StyleScopedClasses['overflow-hidden']} */ ;
/** @type {__VLS_StyleScopedClasses['px-10']} */ ;
/** @type {__VLS_StyleScopedClasses['py-4']} */ ;
/** @type {__VLS_StyleScopedClasses['flex']} */ ;
/** @type {__VLS_StyleScopedClasses['flex-col']} */ ;
/** @type {__VLS_StyleScopedClasses['max-w-6xl']} */ ;
/** @type {__VLS_StyleScopedClasses['mx-auto']} */ ;
/** @type {__VLS_StyleScopedClasses['w-full']} */ ;
/** @type {__VLS_StyleScopedClasses['h-full']} */ ;
/** @type {__VLS_StyleScopedClasses['flex']} */ ;
/** @type {__VLS_StyleScopedClasses['flex-col']} */ ;
/** @type {__VLS_StyleScopedClasses['flex-1']} */ ;
/** @type {__VLS_StyleScopedClasses['flex']} */ ;
/** @type {__VLS_StyleScopedClasses['flex-col']} */ ;
/** @type {__VLS_StyleScopedClasses['gap-4']} */ ;
/** @type {__VLS_StyleScopedClasses['grid']} */ ;
/** @type {__VLS_StyleScopedClasses['grid-cols-3']} */ ;
/** @type {__VLS_StyleScopedClasses['gap-4']} */ ;
/** @type {__VLS_StyleScopedClasses['shrink-0']} */ ;
/** @type {__VLS_StyleScopedClasses['flex-1']} */ ;
/** @type {__VLS_StyleScopedClasses['min-h-0']} */ ;
/** @type {__VLS_StyleScopedClasses['relative']} */ ;
/** @type {__VLS_StyleScopedClasses['h-full']} */ ;
/** @type {__VLS_StyleScopedClasses['bg-[#110f0e]/50']} */ ;
/** @type {__VLS_StyleScopedClasses['border']} */ ;
/** @type {__VLS_StyleScopedClasses['border-white/[0.02]']} */ ;
/** @type {__VLS_StyleScopedClasses['shadow-2xl']} */ ;
/** @type {__VLS_StyleScopedClasses['flex']} */ ;
/** @type {__VLS_StyleScopedClasses['justify-center']} */ ;
/** @type {__VLS_StyleScopedClasses['py-1']} */ ;
/** @type {__VLS_StyleScopedClasses['opacity-10']} */ ;
/** @type {__VLS_StyleScopedClasses['text-[7px]']} */ ;
/** @type {__VLS_StyleScopedClasses['font-mono']} */ ;
/** @type {__VLS_StyleScopedClasses['uppercase']} */ ;
/** @type {__VLS_StyleScopedClasses['tracking-[0.5em]']} */ ;
/** @type {__VLS_StyleScopedClasses['text-white']} */ ;
/** @type {__VLS_StyleScopedClasses['flex-1']} */ ;
/** @type {__VLS_StyleScopedClasses['overflow-y-auto']} */ ;
/** @type {__VLS_StyleScopedClasses['custom-scroll']} */ ;
var __VLS_dollars;
const __VLS_self = (await import('vue')).defineComponent({
    setup() {
        return {
            Pin: Pin,
            PinOff: PinOff,
            Shield: Shield,
            Ghost: Ghost,
            Activity: Activity,
            Sidebar: Sidebar,
            MagicFeedback: MagicFeedback,
            FileProcessor: FileProcessor,
            HistoryList: HistoryList,
            RuleManager: RuleManager,
            SettingsPage: SettingsPage,
            ExitConfirm: ExitConfirm,
            StatCard: StatCard,
            store: store,
        };
    },
});
export default (await import('vue')).defineComponent({
    setup() {
        return {};
    },
});
; /* PartiallyEnd: #4569/main.vue */
//# sourceMappingURL=App.vue.js.map