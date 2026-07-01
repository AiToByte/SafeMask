# Plan: Vue 3 → React Migration & UI Enhancement

**Source**: User request — migrate frontend from Vue to React with enhanced aesthetics, fluidity, and component detail  
**Complexity**: Large  
**Original Framework**: Vue 3 + Pinia + `<script setup>`  
**Target Framework**: React 19 + Zustand + Framer Motion  
**Created**: 2026-06-30

## Summary

Migrate SafeMask's existing Vue 3 Composition API frontend to React while preserving 100% of backend Tauri IPC compatibility. Beyond a 1:1 feature port, the React version will introduce Framer Motion for fluid transitions, richer micro-interactions, a design-system token layer, and polished component-level details absent in the current Vue implementation.

---

## Patterns to Mirror

| Category | Source | Pattern |
|---|---|---|
| Naming | `src/main.ts:1-2` | PascalCase components, camelCase functions, kebab-case CSS |
| IPC Layer | `src/services/api.ts` | Keep unchanged — pure TS, framework-agnostic |
| State Shape | `src/stores/useAppStore.ts` | Same reactive fields (`settings`, `activeTab`, `historyList`, etc.) |
| Event Listeners | `src/stores/useAppStore.ts:41-65` | Tauri `listen()` for `new-history`, `magic-feedback`, `mode-switch-event`, `file-progress` |
| Tailwind Classes | `src/App.vue`, all components | Preserve existing utility classes; enhance with Framer Motion variants |
| Audio Feedback | `src/stores/useAppStore.ts:67-101` | Web Audio API oscillator pattern — extract to shared hook |
| Rust Backend | `src-tauri/src/api/` | **ZERO changes** — IPC commands unchanged |

---

## Files to Change

| File | Action | Why |
|---|---|---|
| `src/main.tsx` | CREATE | React entry point (replaces `main.ts`) |
| `src/App.tsx` | CREATE | Root layout with Framer Motion page transitions |
| `src/main.ts` | DELETE | Vue entry point |
| `src/App.vue` | DELETE | Vue root component |
| `src/components/*.vue` | DELETE | All 9 Vue SFCs |
| `src/components/ui/*.tsx` | CREATE | Design-system primitives (Button, Card, Toggle, Input, Badge) |
| `src/components/layout/Sidebar.tsx` | CREATE | Navigation sidebar with tooltips |
| `src/components/layout/Header.tsx` | CREATE | Top bar with mode switch capsule |
| `src/components/dashboard/StatCard.tsx` | CREATE | Stats card with glow effects |
| `src/components/dashboard/FileProcessor.tsx` | CREATE | Drag-drop file processing |
| `src/components/history/HistoryList.tsx` | CREATE | Audit trail with search |
| `src/components/rules/RuleManager.tsx` | CREATE | Rule CRUD + sandbox |
| `src/components/settings/SettingsPage.tsx` | CREATE | System configuration |
| `src/components/feedback/MagicFeedback.tsx` | CREATE | Toast notifications |
| `src/components/overlay/ExitConfirm.tsx` | CREATE | Exit dialog |
| `src/hooks/useAppStore.ts` | CREATE | Zustand store (replaces Pinia) |
| `src/hooks/useAudioFeedback.ts` | CREATE | Web Audio API oscillator hook |
| `src/hooks/useTauriEvents.ts` | CREATE | Tauri event listener lifecycle hook |
| `src/lib/utils.ts` | CREATE | Shared utilities (`cn()`, formatters) |
| `src/services/api.ts` | KEEP | **No changes** — framework-agnostic |
| `src/style.css` | UPDATE | Add design tokens, animation keyframes |
| `package.json` | UPDATE | Replace Vue deps with React deps |
| `vite.config.ts` | UPDATE | Swap `@vitejs/plugin-vue` → `@vitejs/plugin-react` |
| `tsconfig.json` | UPDATE | Set `jsx: "react-jsx"` |
| `tailwind.config.js` | UPDATE | Add React file extensions to content paths |
| `index.html` | UPDATE | Point to `/src/main.tsx` |

---

## Tasks

### Phase 1: Foundation & Tooling (Day 1)

#### Task 1.1: Replace Vue dependencies with React
- **Action**: Update `package.json` — remove `vue`, `pinia`, `lucide-vue-next`, `@vitejs/plugin-vue`, `vue-tsc`; add `react`, `react-dom`, `zustand`, `lucide-react`, `framer-motion`, `@vitejs/plugin-react`, `clsx`, `tailwind-merge`
- **Mirror**: Current `package.json:1-31`
- **Validate**: `npm install` succeeds, no peer dependency conflicts

#### Task 1.2: Update build configuration
- **Action**: 
  - `vite.config.ts`: swap Vue plugin → React plugin
  - `tsconfig.json`: set `"jsx": "react-jsx"`, update includes to `.tsx`
  - `tailwind.config.js`: add `"./src/**/*.{tsx,jsx}"` to content
  - `index.html`: change script src to `/src/main.tsx`
- **Mirror**: Current `vite.config.ts`, `tsconfig.json:7` already has `"jsx": "preserve"`, change to `"react-jsx"`
- **Validate**: `npx tsc --noEmit` passes (after Phase 1 tasks written)

#### Task 1.3: Create design system primitives (`src/components/ui/`)
- **Action**: Create reusable primitives matching current visual language:
  - `GlassPanel.tsx` — base container with backdrop-blur, border, rounded-[2.5rem]
  - `Card.tsx` — config-card base
  - `Button.tsx` — variants (primary/secondary/danger/ghost)
  - `Toggle.tsx` — checkbox switch (mirrors current `.sw-wrapper` CSS)
  - `Input.tsx` — styled inputs with focus glow
  - `Badge.tsx` — custom/system tags
  - `EmptyState.tsx` — centered empty message with icon
- **Mirror**: Current `style.css` glass-morphism class, component-scoped CSS patterns
- **Validate**: Components render in isolation

#### Task 1.4: Create `cn()` utility
- **Action**: Implement `clsx` + `tailwind-merge` helper matching the project's Tailwind usage pattern
- **Mirror**: Standard `cn()` pattern from shadcn/ui
- **Validate**: Unit test with conflicting Tailwind classes

---

### Phase 2: State & Services (Day 1–2)

#### Task 2.1: Implement Zustand store (`src/hooks/useAppStore.ts`)
- **Action**: Port all Pinia state (`useAppStore.ts`) to Zustand with identical shape:
  - State: `settings`, `isMonitorOn`, `ruleCount`, `activeTab`, `historyList`, `allRulesList`, `activeFeedback`, `progress`, `isProcessing`, `currentFileName`, `appInfo`, `aiEngineStatus`, `engineInfo`, `isAlwaysOnTop`
  - Actions: `bootstrap`, `toggleVaultMode`, `fetchStats`, `fetchHistory`, `fetchAllRules`, `clearHistory`, `fetchAiStatus`, `fetchEngineInfo`, `toggleAiEngine`, `toggleAlwaysOnTop`, `playFeedbackSound`
- **Mirror**: Current `src/stores/useAppStore.ts` — exact same reactive shape
- **Validate**: TypeScript compiles, unit test for bootstrap flow

#### Task 2.2: Extract `useAudioFeedback` hook
- **Action**: Move Web Audio API oscillator logic from Pinia store to standalone hook
  - Accept `enabled: boolean` param
  - Return `play(type)` function
- **Mirror**: Current `src/stores/useAppStore.ts:67-101`
- **Validate**: Plays test tones in browser

#### Task 2.3: Create `useTauriEvents` hook
- **Action**: Encapsulate Tauri `listen()` calls with React lifecycle:
  - Auto-subscribe on mount, unsubscribe on unmount
  - Accept event name + callback, return cleanup
  - Handle `new-history`, `magic-feedback`, `mode-switch-event`, `file-progress`
- **Mirror**: Current `src/stores/useAppStore.ts:41-65`
- **Validate**: Events fire in dev mode

---

### Phase 3: Layout Components (Day 2)

#### Task 3.1: Implement Sidebar
- **Action**: Port `Sidebar.vue` → `Sidebar.tsx`
  - Nav items: Dashboard, History, Rules, Settings
  - Active state with left indicator bar + glow animation
  - Hover tooltips
  - **Enhancement**: Framer Motion `layoutId` for smooth active indicator movement; staggered icon entrance animation
- **Mirror**: Current `src/components/Sidebar.vue`
- **Validate**: Click navigation, active state transitions, tooltip visibility

#### Task 3.2: Implement Header
- **Action**: Extract header from `App.vue` → standalone `Header.tsx`
  - Logo + title area
  - Pin/Unpin button with glow state
  - Universe Mode toggle capsule with hover tooltip
  - **Enhancement**: Framer Motion `AnimatePresence` for mode icon swap; subtle pulse on mode indicator
- **Mirror**: Current `src/App.vue:31-101`
- **Validate**: Mode toggle calls backend, visual state updates

#### Task 3.3: Implement App shell
- **Action**: Create `App.tsx` root layout
  - Flex layout: Sidebar + Main area (Header + Content)
  - Tab-based routing via `store.activeTab` (no React Router needed — matches current pattern)
  - **Enhancement**: `AnimatePresence` for page transitions with custom spring physics; ambient background glow
- **Mirror**: Current `src/App.vue:21-141`
- **Validate**: All tabs render, transitions smooth

---

### Phase 4: Dashboard Components (Day 2–3)

#### Task 4.1: Implement StatCard with micro-interactions
- **Action**: Port `StatCard.vue` → `StatCard.tsx`
  - Type-specific colors (amber/blue/emerald)
  - Ambient glow, left indicator bar, hover lift
  - **Enhancement**: Animated counter on value change (Framer Motion `animate` with spring); hover glow expansion
- **Mirror**: Current `src/components/StatCard.vue`
- **Validate**: Cards render with values, click navigation works

#### Task 4.2: Implement FileProcessor
- **Action**: Port `FileProcessor.vue` → `FileProcessor.tsx`
  - Drag-and-drop zone + click-to-browse
  - Progress bar with animated gradient
  - Processing state with filename + percentage
  - Tauri drag-drop event listener
  - **Enhancement**: Framer Motion layout animations for state transitions; animated border gradient on drag-hover; particle-like progress bar shimmer
- **Mirror**: Current `src/components/FileProcessor.vue`
- **Validate**: File selection opens dialog, progress bar animates, completion dialog shows

---

### Phase 5: Feature Pages (Day 3–4)

#### Task 5.1: Implement HistoryList
- **Action**: Port `HistoryList.vue` → `HistoryList.tsx`
  - Search bar with focus glow effect
  - History cards with raw/masked comparison
  - Copy buttons with success state
  - Mode badges (Shadow/Sentry)
  - Clear button
  - **Enhancement**: Staggered list entrance animation; smooth search filter transitions with `AnimatePresence`; copy success micro-interaction (scale + checkmark)
- **Mirror**: Current `src/components/HistoryList.vue`
- **Validate**: Search filters work, copy buttons function, cards render

#### Task 5.2: Implement RuleManager
- **Action**: Port `RuleManager.vue` → `RuleManager.tsx`
  - Left panel: searchable rule list with active selection
  - Right panel: rule form with validation
  - Sandbox with real-time regex testing
  - Save / Save-as-new / Delete operations
  - **Enhancement**: Active rule highlight with animated border; form field focus transitions; sandbox output fade-in; validation error shake animation
- **Mirror**: Current `src/components/RuleManager.vue` (largest component at ~450 lines)
- **Validate**: Rule CRUD works, sandbox tests real-time, validation errors display

#### Task 5.3: Implement SettingsPage
- **Action**: Port `Settings.vue` → `SettingsPage.tsx`
  - Kernel behavior section (shadow mode toggle, shortcut recording)
  - AI Engine section (status, model list, recognizers)
  - Feedback section (visual/audio toggles, delay slider)
  - About section (developer info, links, email copy)
  - **Enhancement**: AI loading progress with animated brain icon; smoother range slider; email copy with ripple effect
- **Mirror**: Current `src/components/Settings.vue`
- **Validate**: Settings save to backend, AI status refreshes, copy works

---

### Phase 6: Overlay Components (Day 4)

#### Task 6.1: Implement MagicFeedback (Toast)
- **Action**: Port `MagicFeedback.vue` → `MagicFeedback.tsx`
  - Mode change toast (Shadow/Sentry)
  - Paste masked/original feedback
  - Glass morphism style
  - **Enhancement**: `AnimatePresence` entrance from top with spring; auto-dismiss with exit animation; stacking multiple toasts
- **Mirror**: Current `src/components/MagicFeedback.vue`
- **Validate**: Toast appears on mode switch, auto-dismisses

#### Task 6.2: Implement ExitConfirm dialog
- **Action**: Port `ExitConfirm.vue` → `ExitConfirm.tsx`
  - Modal with backdrop blur
  - Minimize to tray / Quit buttons
  - "Remember choice" checkbox
  - **Enhancement**: Framer Motion scale-in backdrop + modal entrance; button hover glow
- **Mirror**: Current `src/components/ExitConfirm.vue`
- **Validate**: Dialog appears on close event, buttons trigger correct actions

---

### Phase 7: Polish & Quality (Day 4–5)

#### Task 7.1: Global animation system
- **Action**: Define shared Framer Motion variants in `src/lib/animations.ts`:
  - `fadeInUp`, `fadeIn`, `staggerContainer`, `staggerItem`
  - `scaleIn`, `slideInLeft`, `slideInRight`
  - `pageTransition` with spring physics
  - Apply consistently across all components
- **Validate**: All page transitions use shared variants, no hardcoded values

#### Task 7.2: Enhanced component details
- **Action**: Add polish to every component:
  - Hover states on all interactive elements
  - Focus ring animations on inputs
  - Skeleton loading states for async data
  - Error boundary wrapper
  - Subtle grain texture overlay on glass panels
  - Reduced motion support (`prefers-reduced-motion`)
- **Validate**: Every interactive element has visible hover/focus/active states

#### Task 7.3: Accessibility & keyboard navigation
- **Action**: 
  - Tab index ordering for sidebar → header → content
  - Keyboard shortcut hints in tooltips
  - Focus trap in ExitConfirm modal
  - ARIA labels on icon-only buttons
- **Validate**: Full keyboard navigation through all features

---

### Phase 8: Cleanup & Verification (Day 5)

#### Task 8.1: Remove Vue artifacts
- **Action**: Delete all `.vue` files, remove `src/main.ts`, verify no Vue imports remain
- **Validate**: `grep -r "\.vue\|vue\|pinia" src/` returns empty

#### Task 8.2: Full build verification
- **Action**: Run `npm run build` (tsc + vite build), fix any issues
- **Validate**: Build succeeds, `dist/` contains React bundle

#### Task 8.3: Tauri integration test
- **Action**: Run `npm run tauri build` (or `cargo check` for frontend-to-backend IPC)
- **Validate**: All IPC commands resolve correctly, no Tauri errors

#### Task 8.4: Update CLAUDE.md
- **Action**: Update project documentation to reflect React stack
- **Validate**: CLAUDE.md accurately describes new architecture

---

## Validation

```bash
# TypeScript check (should pass after each phase)
npx tsc --noEmit

# Vite build (final verification)
npm run build

# Tauri check (IPC compatibility)
cd src-tauri && cargo check

# Verify no Vue remnants
grep -r "\.vue" src/ --include="*.tsx" --include="*.ts"
grep -r "from 'vue'" src/
grep -r "from 'pinia'" src/
```

---

## Risks

| Risk | Likelihood | Mitigation |
|---|---|---|
| Complex RuleManager (450 lines) has subtle state interactions | MEDIUM | Extract sandbox logic into separate `useSandbox` hook; test real-time regex flow early |
| Tauri IPC event listeners behave differently in React lifecycle | MEDIUM | Use `useTauriEvents` hook with explicit cleanup; test with StrictMode double-mount |
| Audio feedback context creation conflicts with React StrictMode | LOW | `useRef` for AudioContext singleton; guard double-create |
| `lucide-react` API differs from `lucide-vue-next` | LOW | Same icon names, same props — only import path changes |
| Large bundle increase from Framer Motion | LOW | Tree-shake only used components (`motion`, `AnimatePresence`); monitor bundle size |
| Tailwind classes use Vue-specific syntax (`v-if`, `:class`) | MEDIUM | Manual review of all template conditionals during port; systematic find-replace |

---

## Architecture Comparison

### Before (Vue)
```
src/
├── main.ts                    # createApp + Pinia
├── App.vue                    # Monolithic layout (141 lines)
├── components/
│   ├── Sidebar.vue            # Nav with scoped styles
│   ├── FileProcessor.vue      # Drag-drop
│   ├── HistoryList.vue        # Audit trail
│   ├── RuleManager.vue        # 450 lines — largest
│   ├── Settings.vue           # 625 lines — even larger!
│   ├── StatCard.vue           # Dashboard card
│   ├── MagicFeedback.vue      # Toast
│   ├── ExitConfirm.vue        # Modal
│   └── AiStatus.vue           # Unused in current App
├── services/api.ts            # IPC layer
├── stores/useAppStore.ts      # Pinia (monolithic, 134 lines)
└── style.css                  # Global styles
```

### After (React)
```
src/
├── main.tsx                   # createRoot + Zustand
├── App.tsx                    # Layout shell (~60 lines) with AnimatePresence
├── components/
│   ├── ui/                    # Design system primitives
│   │   ├── GlassPanel.tsx
│   │   ├── Card.tsx
│   │   ├── Button.tsx
│   │   ├── Toggle.tsx
│   │   ├── Input.tsx
│   │   ├── Badge.tsx
│   │   └── EmptyState.tsx
│   ├── layout/
│   │   ├── Sidebar.tsx
│   │   └── Header.tsx
│   ├── dashboard/
│   │   ├── StatCard.tsx
│   │   └── FileProcessor.tsx
│   ├── history/
│   │   └── HistoryList.tsx
│   ├── rules/
│   │   └── RuleManager.tsx
│   ├── settings/
│   │   └── SettingsPage.tsx
│   ├── feedback/
│   │   └── MagicFeedback.tsx
│   └── overlay/
│       └── ExitConfirm.tsx
├── hooks/
│   ├── useAppStore.ts         # Zustand store
│   ├── useAudioFeedback.ts    # Audio oscillator
│   └── useTauriEvents.ts      # Event lifecycle
├── lib/
│   ├── utils.ts               # cn() helper
│   └── animations.ts          # Framer Motion variants
├── services/
│   └── api.ts                 # UNCHANGED
└── style.css                  # Enhanced with design tokens
```

### Key Improvements Over Vue Version

| Aspect | Vue Current | React Target |
|---|---|---|
| **Component organization** | Flat `components/` folder | Domain-grouped folders (ui/, layout/, dashboard/, etc.) |
| **State management** | Pinia monolith (134 lines) | Zustand + extracted hooks (audio, events, sandbox) |
| **Animations** | CSS transitions + `animate-in` utility classes | Framer Motion with physics-based springs, shared variants |
| **UI primitives** | Inline utility classes, no design system | Dedicated `ui/` primitives with consistent API |
| **Page transitions** | Vue `<Transition>` with fade + translateY | `AnimatePresence` with staggered children, custom spring |
| **Interactions** | Basic hover + active states | Hover glow, focus rings, skeleton loading, reduced-motion |
| **Code reuse** | Copy-paste between components | Shared hooks (`useAudioFeedback`, `useTauriEvents`) |
| **Accessibility** | Minimal aria attributes | Keyboard nav, focus traps, aria labels, reduced motion |

---

## Design Direction

**Style**: Dark luxury with disciplined amber accent — maintain current visual identity while adding depth

**Key visual upgrades:**
1. **Glass morphism depth**: Multi-layer shadows on panels, subtle grain texture overlay
2. **Ambient glow system**: Each section type (amber/blue/emerald/purple) gets matching ambient light
3. **Micro-interactions**: Copy button ripple, toggle switch haptic-like spring, list item hover lift
4. **Typography**: Maintain Inter + JetBrains Mono; add tracking-based hierarchy
5. **States**: Every interactive element gets `hover`, `focus-visible`, `active`, and `disabled` states

---

## Acceptance

- [ ] All 9 Vue components successfully ported to React TSX
- [ ] Zustand store matches Pinia state shape exactly
- [ ] All Tauri IPC calls function identically
- [ ] Zero Vue/Pinia imports remain in `src/`
- [ ] `npm run build` succeeds (tsc + vite)
- [ ] `cargo check` passes (IPC types unchanged)
- [ ] Framer Motion transitions present on all page changes
- [ ] Every interactive element has 3+ states (hover/focus/active)
- [ ] Keyboard navigation works through all features
- [ ] `prefers-reduced-motion` disables animations
- [ ] CLAUDE.md updated with new React architecture
