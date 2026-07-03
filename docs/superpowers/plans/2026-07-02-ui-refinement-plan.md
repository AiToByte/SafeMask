# SafeMask UI Refinement Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development or superpowers:executing-plans.

**Goal:** Refine SafeMask UI — increase font sizes (+1~2px), lightly expand spacing, enrich color accents while keeping the dark amber theme.

**Architecture:** All changes are frontend-only (React 19 + Tailwind CSS). No Rust/Tauri changes.

**Tech Stack:** React 19, TypeScript, Zustand, Tailwind CSS v3, Framer Motion, Vite 6

---

### Task 1: tailwind.config.js — 添加设计 Token
**Files:** `tailwind.config.js`
Add borderRadius/colors/shadow/spacing design tokens.

### Task 2: App.tsx — 布局外壳间距增大
**Files:** `src/App.tsx`
`px-10 py-4` → `px-12 py-6`

### Task 3: Sidebar — 加宽、图标放大
**Files:** `src/components/layout/Sidebar.tsx`
Width `w-20`→`w-24`, icon containers larger, tooltips bigger

### Task 4: Header — 标题放大、胶囊增大
**Files:** `src/components/layout/Header.tsx`
Title 18px→20px, capsule larger, mode label font upgrade

### Task 5: StatCard — 统计卡字体放大
**Files:** `src/components/dashboard/StatCard.tsx`
Title 10px→12px, unit 9px→11px, card larger

### Task 6: FileProcessor — 文件拖拽区
**Files:** `src/components/dashboard/FileProcessor.tsx`
Main text 20px→24px, subtitle 14px→16px

### Task 7: Button 系统
**Files:** `src/components/ui/Button.tsx`
Font 11px→12px, padding larger, rounded tokenized

### Task 8: Input 表单组件
**Files:** `src/components/ui/Input.tsx`
Padding/font larger, placeholder contrast improved

### Task 9: Toggle 开关
**Files:** `src/components/ui/Toggle.tsx`
Switch `w-12`→`w-14`, thumb proportional

### Task 10: Card/GlassPanel/Badge/EmptyState
**Files:** `src/components/ui/Card.tsx`, `GlassPanel.tsx`, `Badge.tsx`, `EmptyState.tsx`
Uniform padding/font size increases

### Task 11: RuleManager
**Files:** `src/components/rules/RuleManager.tsx`
List/form/sandbox font size increase + indigo accent

### Task 12: HistoryList
**Files:** `src/components/history/HistoryList.tsx`
Title 20px→24px, code/timestamp larger + rose/cyan accents

### Task 13: SettingsPage
**Files:** `src/components/settings/SettingsPage.tsx`
Section spacing, toggle/slider/label unified enlargement

### Task 14: ExitConfirm + MagicFeedback
**Files:** `src/components/overlay/ExitConfirm.tsx`, `src/components/feedback/MagicFeedback.tsx`
Title/button/text larger

### Task 15: style.css
**Files:** `src/style.css`
Scrollbar 2px→3px, glassmorphism enhanced, glow utility classes
