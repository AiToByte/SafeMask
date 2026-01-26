/// <reference types="../node_modules/.vue-global-types/vue_3.5_0_0_0.d.ts" />
import { onMounted, onUnmounted } from 'vue';
import { listen } from "@tauri-apps/api/event";
import { useAppStore } from './stores/useAppStore';
// 导入重构后的高质量组件
import Sidebar from './components/Sidebar.vue';
import StatCard from './components/StatCard.vue';
import FileProcessor from './components/FileProcessor.vue';
import ExitConfirm from './components/ExitConfirm.vue';
import HistoryList from './components/HistoryList.vue';
import RuleManager from './components/RuleManager.vue';
const store = useAppStore();
// 存储监听器卸载函数，防止内存泄漏
let unlistenProgress;
let unlistenMasked;
onMounted(async () => {
    // 1. 初始化从 Rust 后端拉取规则统计信息
    await store.fetchStats();
    await store.fetchHistory();
    // 2. 🚀 开启全局实时监听（核心修复）
    await store.initEventListeners();
    // 2. 监听文件处理进度事件 (来自 processor.rs 的保序流水线)
    unlistenProgress = await listen("file-progress", (event) => {
        // 自动更新 Pinia Store 中的进度状态，FileProcessor 组件会响应式更新 UI
        store.progress = event.payload.percentage;
    });
    // 3. 监听剪贴板脱敏事件 (方案一：原生钩子触发)
    unlistenMasked = await listen("masked-event", (event) => {
        // 可以在此处集成 Toast 通知库，目前先打印日志
        console.info("🛡️ SafeMask Notification:", event.payload);
    });
});
// 组件销毁时取消系统事件监听
onUnmounted(() => {
    if (unlistenProgress)
        unlistenProgress();
    if (unlistenMasked)
        unlistenMasked();
});
debugger; /* PartiallyEnd: #3632/scriptSetup.vue */
const __VLS_ctx = {};
let __VLS_components;
let __VLS_directives;
// CSS variable injection 
// CSS variable injection end 
__VLS_asFunctionalElement(__VLS_intrinsicElements.div, __VLS_intrinsicElements.div)({
    ...{ class: "flex h-screen bg-[#09090b] text-zinc-100 select-none overflow-hidden font-sans" },
});
/** @type {[typeof Sidebar, ]} */ ;
// @ts-ignore
const __VLS_0 = __VLS_asFunctionalComponent(Sidebar, new Sidebar({}));
const __VLS_1 = __VLS_0({}, ...__VLS_functionalComponentArgsRest(__VLS_0));
__VLS_asFunctionalElement(__VLS_intrinsicElements.main, __VLS_intrinsicElements.main)({
    ...{ class: "flex-1 flex flex-col min-w-0" },
});
__VLS_asFunctionalElement(__VLS_intrinsicElements.header, __VLS_intrinsicElements.header)({
    ...{ class: "flex justify-between items-end px-12 pt-12 pb-8 border-b border-zinc-800/30" },
});
__VLS_asFunctionalElement(__VLS_intrinsicElements.div, __VLS_intrinsicElements.div)({
    ...{ class: "space-y-1" },
});
__VLS_asFunctionalElement(__VLS_intrinsicElements.h1, __VLS_intrinsicElements.h1)({
    ...{ class: "text-3xl font-extrabold tracking-tight bg-clip-text text-transparent bg-gradient-to-br from-white to-zinc-500" },
});
__VLS_asFunctionalElement(__VLS_intrinsicElements.p, __VLS_intrinsicElements.p)({
    ...{ class: "text-zinc-500 text-sm font-medium" },
});
__VLS_asFunctionalElement(__VLS_intrinsicElements.div, __VLS_intrinsicElements.div)({
    ...{ class: "flex items-center gap-4 bg-zinc-900/50 border border-zinc-800 px-5 py-3 rounded-2xl transition-all hover:border-zinc-700" },
});
__VLS_asFunctionalElement(__VLS_intrinsicElements.div, __VLS_intrinsicElements.div)({
    ...{ class: "flex flex-col items-end" },
});
__VLS_asFunctionalElement(__VLS_intrinsicElements.span, __VLS_intrinsicElements.span)({
    ...{ class: "text-xs font-bold uppercase tracking-wider text-zinc-400" },
});
__VLS_asFunctionalElement(__VLS_intrinsicElements.span, __VLS_intrinsicElements.span)({
    ...{ class: "text-[10px] text-zinc-600 font-mono" },
});
(__VLS_ctx.store.isMonitorOn ? 'ACTIVE' : 'DISABLED');
__VLS_asFunctionalElement(__VLS_intrinsicElements.button, __VLS_intrinsicElements.button)({
    ...{ onClick: (__VLS_ctx.store.toggleMonitor) },
    ...{ class: "w-12 h-6 rounded-full relative transition-all duration-300 focus:outline-none shadow-inner" },
    ...{ class: (__VLS_ctx.store.isMonitorOn ? 'bg-blue-600 shadow-blue-500/20' : 'bg-zinc-800') },
});
__VLS_asFunctionalElement(__VLS_intrinsicElements.div, __VLS_intrinsicElements.div)({
    ...{ class: "absolute top-1 left-1 bg-white w-4 h-4 rounded-full transition-transform duration-300 shadow-sm" },
    ...{ class: ({ 'translate-x-6': __VLS_ctx.store.isMonitorOn }) },
});
__VLS_asFunctionalElement(__VLS_intrinsicElements.div, __VLS_intrinsicElements.div)({
    ...{ class: "flex-1 p-12 overflow-y-auto custom-scroll" },
});
if (__VLS_ctx.store.activeTab === 'dashboard') {
    __VLS_asFunctionalElement(__VLS_intrinsicElements.div, __VLS_intrinsicElements.div)({
        ...{ class: "space-y-10 animate-in fade-in slide-in-from-bottom-2" },
    });
    __VLS_asFunctionalElement(__VLS_intrinsicElements.div, __VLS_intrinsicElements.div)({
        ...{ class: "grid grid-cols-3 gap-6" },
    });
    /** @type {[typeof StatCard, ]} */ ;
    // @ts-ignore
    const __VLS_3 = __VLS_asFunctionalComponent(StatCard, new StatCard({
        ...{ 'onClick': {} },
        title: "已加载规则",
        value: (__VLS_ctx.store.ruleCount),
        unit: "REG_RULES",
        clickable: true,
    }));
    const __VLS_4 = __VLS_3({
        ...{ 'onClick': {} },
        title: "已加载规则",
        value: (__VLS_ctx.store.ruleCount),
        unit: "REG_RULES",
        clickable: true,
    }, ...__VLS_functionalComponentArgsRest(__VLS_3));
    let __VLS_6;
    let __VLS_7;
    let __VLS_8;
    const __VLS_9 = {
        onClick: (...[$event]) => {
            if (!(__VLS_ctx.store.activeTab === 'dashboard'))
                return;
            __VLS_ctx.store.activeTab = 'rules';
        }
    };
    var __VLS_5;
    /** @type {[typeof StatCard, ]} */ ;
    // @ts-ignore
    const __VLS_10 = __VLS_asFunctionalComponent(StatCard, new StatCard({
        ...{ 'onClick': {} },
        title: "历史拦截",
        value: (__VLS_ctx.store.historyList.length),
        color: "text-amber-400",
        clickable: true,
    }));
    const __VLS_11 = __VLS_10({
        ...{ 'onClick': {} },
        title: "历史拦截",
        value: (__VLS_ctx.store.historyList.length),
        color: "text-amber-400",
        clickable: true,
    }, ...__VLS_functionalComponentArgsRest(__VLS_10));
    let __VLS_13;
    let __VLS_14;
    let __VLS_15;
    const __VLS_16 = {
        onClick: (...[$event]) => {
            if (!(__VLS_ctx.store.activeTab === 'dashboard'))
                return;
            __VLS_ctx.store.activeTab = 'history';
        }
    };
    var __VLS_12;
    /** @type {[typeof StatCard, ]} */ ;
    // @ts-ignore
    const __VLS_17 = __VLS_asFunctionalComponent(StatCard, new StatCard({
        title: "引擎架构",
        value: "HYBRID",
        color: "text-blue-400",
    }));
    const __VLS_18 = __VLS_17({
        title: "引擎架构",
        value: "HYBRID",
        color: "text-blue-400",
    }, ...__VLS_functionalComponentArgsRest(__VLS_17));
    /** @type {[typeof FileProcessor, ]} */ ;
    // @ts-ignore
    const __VLS_20 = __VLS_asFunctionalComponent(FileProcessor, new FileProcessor({
        ...{ class: "min-h-[320px]" },
    }));
    const __VLS_21 = __VLS_20({
        ...{ class: "min-h-[320px]" },
    }, ...__VLS_functionalComponentArgsRest(__VLS_20));
}
else if (__VLS_ctx.store.activeTab === 'history') {
    /** @type {[typeof HistoryList, ]} */ ;
    // @ts-ignore
    const __VLS_23 = __VLS_asFunctionalComponent(HistoryList, new HistoryList({}));
    const __VLS_24 = __VLS_23({}, ...__VLS_functionalComponentArgsRest(__VLS_23));
}
else if (__VLS_ctx.store.activeTab === 'rules') {
    /** @type {[typeof RuleManager, ]} */ ;
    // @ts-ignore
    const __VLS_26 = __VLS_asFunctionalComponent(RuleManager, new RuleManager({}));
    const __VLS_27 = __VLS_26({}, ...__VLS_functionalComponentArgsRest(__VLS_26));
}
if (__VLS_ctx.store.activeTab === 'dashboard') {
    __VLS_asFunctionalElement(__VLS_intrinsicElements.footer, __VLS_intrinsicElements.footer)({
        ...{ class: "text-center pt-10 opacity-30" },
    });
    __VLS_asFunctionalElement(__VLS_intrinsicElements.p, __VLS_intrinsicElements.p)({
        ...{ class: "text-[10px] font-mono uppercase tracking-widest" },
    });
}
/** @type {[typeof ExitConfirm, ]} */ ;
// @ts-ignore
const __VLS_29 = __VLS_asFunctionalComponent(ExitConfirm, new ExitConfirm({}));
const __VLS_30 = __VLS_29({}, ...__VLS_functionalComponentArgsRest(__VLS_29));
/** @type {__VLS_StyleScopedClasses['flex']} */ ;
/** @type {__VLS_StyleScopedClasses['h-screen']} */ ;
/** @type {__VLS_StyleScopedClasses['bg-[#09090b]']} */ ;
/** @type {__VLS_StyleScopedClasses['text-zinc-100']} */ ;
/** @type {__VLS_StyleScopedClasses['select-none']} */ ;
/** @type {__VLS_StyleScopedClasses['overflow-hidden']} */ ;
/** @type {__VLS_StyleScopedClasses['font-sans']} */ ;
/** @type {__VLS_StyleScopedClasses['flex-1']} */ ;
/** @type {__VLS_StyleScopedClasses['flex']} */ ;
/** @type {__VLS_StyleScopedClasses['flex-col']} */ ;
/** @type {__VLS_StyleScopedClasses['min-w-0']} */ ;
/** @type {__VLS_StyleScopedClasses['flex']} */ ;
/** @type {__VLS_StyleScopedClasses['justify-between']} */ ;
/** @type {__VLS_StyleScopedClasses['items-end']} */ ;
/** @type {__VLS_StyleScopedClasses['px-12']} */ ;
/** @type {__VLS_StyleScopedClasses['pt-12']} */ ;
/** @type {__VLS_StyleScopedClasses['pb-8']} */ ;
/** @type {__VLS_StyleScopedClasses['border-b']} */ ;
/** @type {__VLS_StyleScopedClasses['border-zinc-800/30']} */ ;
/** @type {__VLS_StyleScopedClasses['space-y-1']} */ ;
/** @type {__VLS_StyleScopedClasses['text-3xl']} */ ;
/** @type {__VLS_StyleScopedClasses['font-extrabold']} */ ;
/** @type {__VLS_StyleScopedClasses['tracking-tight']} */ ;
/** @type {__VLS_StyleScopedClasses['bg-clip-text']} */ ;
/** @type {__VLS_StyleScopedClasses['text-transparent']} */ ;
/** @type {__VLS_StyleScopedClasses['bg-gradient-to-br']} */ ;
/** @type {__VLS_StyleScopedClasses['from-white']} */ ;
/** @type {__VLS_StyleScopedClasses['to-zinc-500']} */ ;
/** @type {__VLS_StyleScopedClasses['text-zinc-500']} */ ;
/** @type {__VLS_StyleScopedClasses['text-sm']} */ ;
/** @type {__VLS_StyleScopedClasses['font-medium']} */ ;
/** @type {__VLS_StyleScopedClasses['flex']} */ ;
/** @type {__VLS_StyleScopedClasses['items-center']} */ ;
/** @type {__VLS_StyleScopedClasses['gap-4']} */ ;
/** @type {__VLS_StyleScopedClasses['bg-zinc-900/50']} */ ;
/** @type {__VLS_StyleScopedClasses['border']} */ ;
/** @type {__VLS_StyleScopedClasses['border-zinc-800']} */ ;
/** @type {__VLS_StyleScopedClasses['px-5']} */ ;
/** @type {__VLS_StyleScopedClasses['py-3']} */ ;
/** @type {__VLS_StyleScopedClasses['rounded-2xl']} */ ;
/** @type {__VLS_StyleScopedClasses['transition-all']} */ ;
/** @type {__VLS_StyleScopedClasses['hover:border-zinc-700']} */ ;
/** @type {__VLS_StyleScopedClasses['flex']} */ ;
/** @type {__VLS_StyleScopedClasses['flex-col']} */ ;
/** @type {__VLS_StyleScopedClasses['items-end']} */ ;
/** @type {__VLS_StyleScopedClasses['text-xs']} */ ;
/** @type {__VLS_StyleScopedClasses['font-bold']} */ ;
/** @type {__VLS_StyleScopedClasses['uppercase']} */ ;
/** @type {__VLS_StyleScopedClasses['tracking-wider']} */ ;
/** @type {__VLS_StyleScopedClasses['text-zinc-400']} */ ;
/** @type {__VLS_StyleScopedClasses['text-[10px]']} */ ;
/** @type {__VLS_StyleScopedClasses['text-zinc-600']} */ ;
/** @type {__VLS_StyleScopedClasses['font-mono']} */ ;
/** @type {__VLS_StyleScopedClasses['w-12']} */ ;
/** @type {__VLS_StyleScopedClasses['h-6']} */ ;
/** @type {__VLS_StyleScopedClasses['rounded-full']} */ ;
/** @type {__VLS_StyleScopedClasses['relative']} */ ;
/** @type {__VLS_StyleScopedClasses['transition-all']} */ ;
/** @type {__VLS_StyleScopedClasses['duration-300']} */ ;
/** @type {__VLS_StyleScopedClasses['focus:outline-none']} */ ;
/** @type {__VLS_StyleScopedClasses['shadow-inner']} */ ;
/** @type {__VLS_StyleScopedClasses['absolute']} */ ;
/** @type {__VLS_StyleScopedClasses['top-1']} */ ;
/** @type {__VLS_StyleScopedClasses['left-1']} */ ;
/** @type {__VLS_StyleScopedClasses['bg-white']} */ ;
/** @type {__VLS_StyleScopedClasses['w-4']} */ ;
/** @type {__VLS_StyleScopedClasses['h-4']} */ ;
/** @type {__VLS_StyleScopedClasses['rounded-full']} */ ;
/** @type {__VLS_StyleScopedClasses['transition-transform']} */ ;
/** @type {__VLS_StyleScopedClasses['duration-300']} */ ;
/** @type {__VLS_StyleScopedClasses['shadow-sm']} */ ;
/** @type {__VLS_StyleScopedClasses['flex-1']} */ ;
/** @type {__VLS_StyleScopedClasses['p-12']} */ ;
/** @type {__VLS_StyleScopedClasses['overflow-y-auto']} */ ;
/** @type {__VLS_StyleScopedClasses['custom-scroll']} */ ;
/** @type {__VLS_StyleScopedClasses['space-y-10']} */ ;
/** @type {__VLS_StyleScopedClasses['animate-in']} */ ;
/** @type {__VLS_StyleScopedClasses['fade-in']} */ ;
/** @type {__VLS_StyleScopedClasses['slide-in-from-bottom-2']} */ ;
/** @type {__VLS_StyleScopedClasses['grid']} */ ;
/** @type {__VLS_StyleScopedClasses['grid-cols-3']} */ ;
/** @type {__VLS_StyleScopedClasses['gap-6']} */ ;
/** @type {__VLS_StyleScopedClasses['min-h-[320px]']} */ ;
/** @type {__VLS_StyleScopedClasses['text-center']} */ ;
/** @type {__VLS_StyleScopedClasses['pt-10']} */ ;
/** @type {__VLS_StyleScopedClasses['opacity-30']} */ ;
/** @type {__VLS_StyleScopedClasses['text-[10px]']} */ ;
/** @type {__VLS_StyleScopedClasses['font-mono']} */ ;
/** @type {__VLS_StyleScopedClasses['uppercase']} */ ;
/** @type {__VLS_StyleScopedClasses['tracking-widest']} */ ;
var __VLS_dollars;
const __VLS_self = (await import('vue')).defineComponent({
    setup() {
        return {
            Sidebar: Sidebar,
            StatCard: StatCard,
            FileProcessor: FileProcessor,
            ExitConfirm: ExitConfirm,
            HistoryList: HistoryList,
            RuleManager: RuleManager,
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