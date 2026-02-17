/// <reference types="../../node_modules/.vue-global-types/vue_3.5_0_0_0.d.ts" />
import { ref } from 'vue';
import { Home, Library, Settings, ShieldCheck, ClipboardCopy } from 'lucide-vue-next';
import { useAppStore } from '../stores/useAppStore';
// 🚀 获取全局状态 Store
const store = useAppStore();
const activeTab = ref('dashboard');
/**
 * 菜单配置项
 * - id: 必须与 App.vue 中 v-if 的判断条件字符串严格对应
 * - icon: Lucide 图标组件
 * - label: 工具提示文本
 */
const menuItems = [
    { id: 'dashboard', icon: Home, label: '仪表盘' },
    { id: 'history', icon: ClipboardCopy, label: '记录对比' },
    { id: 'rules', icon: Library, label: '规则管理' },
];
debugger; /* PartiallyEnd: #3632/scriptSetup.vue */
const __VLS_ctx = {};
let __VLS_components;
let __VLS_directives;
// CSS variable injection 
// CSS variable injection end 
__VLS_asFunctionalElement(__VLS_intrinsicElements.nav, __VLS_intrinsicElements.nav)({
    ...{ class: "w-20 flex flex-col items-center py-8 bg-[#0c0c0e] border-r border-zinc-800/50 z-50" },
});
__VLS_asFunctionalElement(__VLS_intrinsicElements.div, __VLS_intrinsicElements.div)({
    ...{ class: "w-12 h-12 bg-gradient-to-br from-blue-500 to-indigo-600 rounded-2xl flex items-center justify-center shadow-lg shadow-blue-500/20 mb-12 group cursor-pointer transition-all duration-300 hover:shadow-blue-500/40 hover:scale-105" },
});
const __VLS_0 = {}.ShieldCheck;
/** @type {[typeof __VLS_components.ShieldCheck, ]} */ ;
// @ts-ignore
const __VLS_1 = __VLS_asFunctionalComponent(__VLS_0, new __VLS_0({
    ...{ class: "text-white w-7 h-7 group-hover:scale-110 transition-transform duration-300" },
}));
const __VLS_2 = __VLS_1({
    ...{ class: "text-white w-7 h-7 group-hover:scale-110 transition-transform duration-300" },
}, ...__VLS_functionalComponentArgsRest(__VLS_1));
__VLS_asFunctionalElement(__VLS_intrinsicElements.div, __VLS_intrinsicElements.div)({
    ...{ class: "flex flex-col gap-6 flex-1" },
});
for (const [item] of __VLS_getVForSourceType((__VLS_ctx.menuItems))) {
    __VLS_asFunctionalElement(__VLS_intrinsicElements.button, __VLS_intrinsicElements.button)({
        ...{ onClick: (...[$event]) => {
                __VLS_ctx.store.activeTab = item.id;
            } },
        key: (item.id),
        title: (item.label),
        ...{ class: "sidebar-item relative group overflow-hidden" },
        ...{ class: ([
                __VLS_ctx.store.activeTab === item.id
                    ? 'text-blue-400 shadow-[inset_0_0_12px_rgba(59,130,246,0.1)]'
                    : 'hover:text-zinc-200'
            ]) },
    });
    if (__VLS_ctx.store.activeTab === item.id) {
        __VLS_asFunctionalElement(__VLS_intrinsicElements.div, __VLS_intrinsicElements.div)({
            ...{ class: "absolute inset-0 bg-gradient-to-r from-blue-600/20 via-indigo-500/20 to-blue-600/20 animate-flow" },
        });
    }
    const __VLS_4 = ((item.icon));
    // @ts-ignore
    const __VLS_5 = __VLS_asFunctionalComponent(__VLS_4, new __VLS_4({
        ...{ class: "w-6 h-6 relative z-10 transition-all duration-300" },
        strokeWidth: (__VLS_ctx.store.activeTab === item.id ? 2.5 : 2),
        ...{ class: ({ 'animate-glow': __VLS_ctx.store.activeTab === item.id }) },
    }));
    const __VLS_6 = __VLS_5({
        ...{ class: "w-6 h-6 relative z-10 transition-all duration-300" },
        strokeWidth: (__VLS_ctx.store.activeTab === item.id ? 2.5 : 2),
        ...{ class: ({ 'animate-glow': __VLS_ctx.store.activeTab === item.id }) },
    }, ...__VLS_functionalComponentArgsRest(__VLS_5));
    if (__VLS_ctx.store.activeTab === item.id) {
        __VLS_asFunctionalElement(__VLS_intrinsicElements.div, __VLS_intrinsicElements.div)({
            ...{ class: "absolute -left-4 w-1 h-6 bg-gradient-to-b from-blue-500 to-indigo-500 rounded-r-full shadow-[0_0_15px_rgba(59,130,246,0.8)] animate-slide-in" },
        });
    }
    __VLS_asFunctionalElement(__VLS_intrinsicElements.span, __VLS_intrinsicElements.span)({
        ...{ class: "absolute left-full ml-4 px-3 py-1.5 bg-zinc-800/90 text-white text-xs font-medium rounded-lg border border-blue-500/20 shadow-lg opacity-0 group-hover:opacity-100 transition-all duration-300 translate-x-2 group-hover:translate-x-0 pointer-events-none z-50 whitespace-nowrap" },
    });
    (item.label);
}
__VLS_asFunctionalElement(__VLS_intrinsicElements.button, __VLS_intrinsicElements.button)({
    ...{ onClick: (...[$event]) => {
            __VLS_ctx.store.activeTab = 'settings';
        } },
    ...{ class: "sidebar-item relative group overflow-hidden mt-auto" },
    ...{ class: ({ 'text-blue-400': __VLS_ctx.store.activeTab === 'settings' }) },
});
if (__VLS_ctx.store.activeTab === 'settings') {
    __VLS_asFunctionalElement(__VLS_intrinsicElements.div, __VLS_intrinsicElements.div)({
        ...{ class: "absolute inset-0 bg-gradient-to-r from-blue-600/20 via-indigo-500/20 to-blue-600/20 animate-flow" },
    });
}
const __VLS_8 = {}.Settings;
/** @type {[typeof __VLS_components.Settings, ]} */ ;
// @ts-ignore
const __VLS_9 = __VLS_asFunctionalComponent(__VLS_8, new __VLS_8({
    ...{ class: "w-6 h-6 relative z-10" },
    ...{ class: ({ 'animate-glow': __VLS_ctx.store.activeTab === 'settings' }) },
}));
const __VLS_10 = __VLS_9({
    ...{ class: "w-6 h-6 relative z-10" },
    ...{ class: ({ 'animate-glow': __VLS_ctx.store.activeTab === 'settings' }) },
}, ...__VLS_functionalComponentArgsRest(__VLS_9));
/** @type {__VLS_StyleScopedClasses['w-20']} */ ;
/** @type {__VLS_StyleScopedClasses['flex']} */ ;
/** @type {__VLS_StyleScopedClasses['flex-col']} */ ;
/** @type {__VLS_StyleScopedClasses['items-center']} */ ;
/** @type {__VLS_StyleScopedClasses['py-8']} */ ;
/** @type {__VLS_StyleScopedClasses['bg-[#0c0c0e]']} */ ;
/** @type {__VLS_StyleScopedClasses['border-r']} */ ;
/** @type {__VLS_StyleScopedClasses['border-zinc-800/50']} */ ;
/** @type {__VLS_StyleScopedClasses['z-50']} */ ;
/** @type {__VLS_StyleScopedClasses['w-12']} */ ;
/** @type {__VLS_StyleScopedClasses['h-12']} */ ;
/** @type {__VLS_StyleScopedClasses['bg-gradient-to-br']} */ ;
/** @type {__VLS_StyleScopedClasses['from-blue-500']} */ ;
/** @type {__VLS_StyleScopedClasses['to-indigo-600']} */ ;
/** @type {__VLS_StyleScopedClasses['rounded-2xl']} */ ;
/** @type {__VLS_StyleScopedClasses['flex']} */ ;
/** @type {__VLS_StyleScopedClasses['items-center']} */ ;
/** @type {__VLS_StyleScopedClasses['justify-center']} */ ;
/** @type {__VLS_StyleScopedClasses['shadow-lg']} */ ;
/** @type {__VLS_StyleScopedClasses['shadow-blue-500/20']} */ ;
/** @type {__VLS_StyleScopedClasses['mb-12']} */ ;
/** @type {__VLS_StyleScopedClasses['group']} */ ;
/** @type {__VLS_StyleScopedClasses['cursor-pointer']} */ ;
/** @type {__VLS_StyleScopedClasses['transition-all']} */ ;
/** @type {__VLS_StyleScopedClasses['duration-300']} */ ;
/** @type {__VLS_StyleScopedClasses['hover:shadow-blue-500/40']} */ ;
/** @type {__VLS_StyleScopedClasses['hover:scale-105']} */ ;
/** @type {__VLS_StyleScopedClasses['text-white']} */ ;
/** @type {__VLS_StyleScopedClasses['w-7']} */ ;
/** @type {__VLS_StyleScopedClasses['h-7']} */ ;
/** @type {__VLS_StyleScopedClasses['group-hover:scale-110']} */ ;
/** @type {__VLS_StyleScopedClasses['transition-transform']} */ ;
/** @type {__VLS_StyleScopedClasses['duration-300']} */ ;
/** @type {__VLS_StyleScopedClasses['flex']} */ ;
/** @type {__VLS_StyleScopedClasses['flex-col']} */ ;
/** @type {__VLS_StyleScopedClasses['gap-6']} */ ;
/** @type {__VLS_StyleScopedClasses['flex-1']} */ ;
/** @type {__VLS_StyleScopedClasses['sidebar-item']} */ ;
/** @type {__VLS_StyleScopedClasses['relative']} */ ;
/** @type {__VLS_StyleScopedClasses['group']} */ ;
/** @type {__VLS_StyleScopedClasses['overflow-hidden']} */ ;
/** @type {__VLS_StyleScopedClasses['absolute']} */ ;
/** @type {__VLS_StyleScopedClasses['inset-0']} */ ;
/** @type {__VLS_StyleScopedClasses['bg-gradient-to-r']} */ ;
/** @type {__VLS_StyleScopedClasses['from-blue-600/20']} */ ;
/** @type {__VLS_StyleScopedClasses['via-indigo-500/20']} */ ;
/** @type {__VLS_StyleScopedClasses['to-blue-600/20']} */ ;
/** @type {__VLS_StyleScopedClasses['animate-flow']} */ ;
/** @type {__VLS_StyleScopedClasses['w-6']} */ ;
/** @type {__VLS_StyleScopedClasses['h-6']} */ ;
/** @type {__VLS_StyleScopedClasses['relative']} */ ;
/** @type {__VLS_StyleScopedClasses['z-10']} */ ;
/** @type {__VLS_StyleScopedClasses['transition-all']} */ ;
/** @type {__VLS_StyleScopedClasses['duration-300']} */ ;
/** @type {__VLS_StyleScopedClasses['absolute']} */ ;
/** @type {__VLS_StyleScopedClasses['-left-4']} */ ;
/** @type {__VLS_StyleScopedClasses['w-1']} */ ;
/** @type {__VLS_StyleScopedClasses['h-6']} */ ;
/** @type {__VLS_StyleScopedClasses['bg-gradient-to-b']} */ ;
/** @type {__VLS_StyleScopedClasses['from-blue-500']} */ ;
/** @type {__VLS_StyleScopedClasses['to-indigo-500']} */ ;
/** @type {__VLS_StyleScopedClasses['rounded-r-full']} */ ;
/** @type {__VLS_StyleScopedClasses['shadow-[0_0_15px_rgba(59,130,246,0.8)]']} */ ;
/** @type {__VLS_StyleScopedClasses['animate-slide-in']} */ ;
/** @type {__VLS_StyleScopedClasses['absolute']} */ ;
/** @type {__VLS_StyleScopedClasses['left-full']} */ ;
/** @type {__VLS_StyleScopedClasses['ml-4']} */ ;
/** @type {__VLS_StyleScopedClasses['px-3']} */ ;
/** @type {__VLS_StyleScopedClasses['py-1.5']} */ ;
/** @type {__VLS_StyleScopedClasses['bg-zinc-800/90']} */ ;
/** @type {__VLS_StyleScopedClasses['text-white']} */ ;
/** @type {__VLS_StyleScopedClasses['text-xs']} */ ;
/** @type {__VLS_StyleScopedClasses['font-medium']} */ ;
/** @type {__VLS_StyleScopedClasses['rounded-lg']} */ ;
/** @type {__VLS_StyleScopedClasses['border']} */ ;
/** @type {__VLS_StyleScopedClasses['border-blue-500/20']} */ ;
/** @type {__VLS_StyleScopedClasses['shadow-lg']} */ ;
/** @type {__VLS_StyleScopedClasses['opacity-0']} */ ;
/** @type {__VLS_StyleScopedClasses['group-hover:opacity-100']} */ ;
/** @type {__VLS_StyleScopedClasses['transition-all']} */ ;
/** @type {__VLS_StyleScopedClasses['duration-300']} */ ;
/** @type {__VLS_StyleScopedClasses['translate-x-2']} */ ;
/** @type {__VLS_StyleScopedClasses['group-hover:translate-x-0']} */ ;
/** @type {__VLS_StyleScopedClasses['pointer-events-none']} */ ;
/** @type {__VLS_StyleScopedClasses['z-50']} */ ;
/** @type {__VLS_StyleScopedClasses['whitespace-nowrap']} */ ;
/** @type {__VLS_StyleScopedClasses['sidebar-item']} */ ;
/** @type {__VLS_StyleScopedClasses['relative']} */ ;
/** @type {__VLS_StyleScopedClasses['group']} */ ;
/** @type {__VLS_StyleScopedClasses['overflow-hidden']} */ ;
/** @type {__VLS_StyleScopedClasses['mt-auto']} */ ;
/** @type {__VLS_StyleScopedClasses['absolute']} */ ;
/** @type {__VLS_StyleScopedClasses['inset-0']} */ ;
/** @type {__VLS_StyleScopedClasses['bg-gradient-to-r']} */ ;
/** @type {__VLS_StyleScopedClasses['from-blue-600/20']} */ ;
/** @type {__VLS_StyleScopedClasses['via-indigo-500/20']} */ ;
/** @type {__VLS_StyleScopedClasses['to-blue-600/20']} */ ;
/** @type {__VLS_StyleScopedClasses['animate-flow']} */ ;
/** @type {__VLS_StyleScopedClasses['w-6']} */ ;
/** @type {__VLS_StyleScopedClasses['h-6']} */ ;
/** @type {__VLS_StyleScopedClasses['relative']} */ ;
/** @type {__VLS_StyleScopedClasses['z-10']} */ ;
var __VLS_dollars;
const __VLS_self = (await import('vue')).defineComponent({
    setup() {
        return {
            Settings: Settings,
            ShieldCheck: ShieldCheck,
            store: store,
            menuItems: menuItems,
        };
    },
});
export default (await import('vue')).defineComponent({
    setup() {
        return {};
    },
});
; /* PartiallyEnd: #4569/main.vue */
//# sourceMappingURL=Sidebar.vue.js.map