# AGENTS.md — SafeMask

## First: existing CLAUDE.md is stale

The `CLAUDE.md` in this repo describes Vue 3 + Pinia — that is **wrong**. The frontend is **React 19 + Zustand**. Trust `AGENTS.md` over `CLAUDE.md`.

## Commands

```bash
# Frontend typecheck (tsc, not vue-tsc — noEmit mode)
npm run build

# Rust (run from repo root, not src-tauri/)
cargo check -p SafeMask
cargo fmt -p SafeMask
cargo clippy -p SafeMask -- -D warnings
cargo test -p SafeMask                        # inline #[cfg(test)] modules
cargo test -p SafeMask test_name -- --nocapture

# Full dev
npm run tauri dev

# Frontend-only dev server (no Tauri window)
npm run dev
```

- `npm run build` = `tsc && vite build` (TypeScript compiler, not vue-tsc)
- Rust tests are **inline `#[cfg(test)]` modules** only — no `tests/` directory
- Vite dev server runs on `127.0.0.1:18924`, ports are strict

## Architecture

### Workspace (Cargo workspace, resolver 2)

```
SafeMask/                    # repo root — Cargo workspace
├── src-tauri/               # workspace member
│   └── src/
│       ├── main.rs          # binary entrypoint (Tauri setup)
│       ├── lib.rs           # library entrypoint (re-exports modules)
│       ├── api/             # #[tauri::command] IPC handlers
│       ├── core/            # pure business logic (no Tauri deps)
│       │   ├── hybrid_engine.rs  # main engine
│       │   ├── recognizer/       # pluggable recognizers (AC, regex, NER, checksum)
│       │   ├── resolver/         # conflict resolution
│       │   ├── masking/          # masking strategies
│       │   └── orchestrator/     # business orchestration (SceneMode)
│       ├── common/           # AppState, AppError, event constants
│       └── infra/            # clipboard, fs (mmap pipeline), config, AI (ONNX)
├── src/                     # React 19 frontend
│   ├── App.tsx              # root component
│   ├── main.tsx             # ReactDOM.createRoot
│   ├── hooks/useAppStore.ts # Zustand store
│   └── services/api.ts      # Tauri IPC wrappers (invoke)
├── tsconfig.json           # paths: "@/*" -> "./src/*"
└── vite.config.ts          # @vitejs/plugin-react
```

### Key architectural facts

- **Two Rust entrypoints**: `lib.rs` (library crate `safemask_lib` → staticlib/cdylib/rlib) and `main.rs` (binary). `main.rs` re-declares `mod api; mod common; mod core; mod infra;` separately from `lib.rs`.
- **`core/` has zero Tauri imports** — independently testable. The `orchestrator/` layer wraps `core` capabilities for clipboard workflows.
- **AI engine**: ONNX Runtime via `ort` crate with HuggingFace `tokenizers`. Models expected at `src-tauri/models/privacy-filter/{model_q4.onnx, tokenizer.json}`. Auto-discovered at startup; graceful degradation on miss.
- **Thread pool**: Rayon default-limited to 2 threads via `SAFEMASK_THREADS` env var or fallback. ONNX Runtime also limited to 2 threads via `ORT_NUM_THREADS`.
- **Memory allocator**: `mimalloc` via `#[global_allocator]`.

### Frontend stack

React 19 + TypeScript + Zustand + Tailwind CSS v3 + Vite 6. No routing library — tabs managed via Zustand `activeTab` state. UI components in `src/components/{dashboard,feedback,history,layout,overlay,rules,settings,ui}/`.

### Conventions

- **Rust edition 2024** — unusual. Uses `let _ = expr;` pattern, unsafe env var sets, `parking_lot` over std locks.
- **Tauri v2** with plugins: `global-shortcut`, `dialog`, `opener`, `notification`.
- **Capabilities** in `src-tauri/capabilities/default.json` — declare IPC permissions there.
- **Tauri plugin registrations** happen in `main.rs` `Builder::default().plugin(...)`. Command handlers listed in `invoke_handler(generate_handler![...])`.
- **Cargo proxy** in `.cargo/config.toml` (`127.0.0.1:7890`). CI explicitly strips it (`rm -rf .cargo/config*`).
- **IP v4 hardcoded to 127.0.0.1:18924** in vite.config.ts.

### Release

- CI: `.github/workflows/release.yml` — triggers on `v*` tag push, builds for macOS/Linux/Windows via `tauri-action`.
- Updater: configured via `tauri.conf.json` plugins.updater pointing to GitHub releases.

## Session Summary

### Completed
- **Window decor revert**: tauri.conf.json: `decorations: false` + `transparent: true` → `decorations: true`; deleted TitleBar.tsx; removed transparent CSS
- **Rounded Win32 corners**: Added `DwmSetWindowAttribute(DWMWCP_ROUND)` via raw FFI in `main.rs:setup_window_handlers()`
- **UI audit**: 42 issues across 8 categories documented
- **UI fix batch 1** (this session):
  - HistoryList.tsx:132 — `group/card` added (fixes broken group-hover/card on Audit-ID)
  - HistoryList.tsx:158 — `text-zinc-800` → `text-zinc-600` (better Audit-ID contrast)
  - HistoryList.tsx:60,79 — `type="button"` added (prevents unwanted form submissions)
  - FileProcessor.tsx:155 — `relative` added to progress bar (fixes shimmer positioned against viewport)
  - App.tsx:182 — `opacity-10` → `opacity-30` (footer now faintly visible)
  - style.css: deleted 38 lines of dead CSS (legacy animations: fade-in, zoom-in, slide-in-from-top, shake; glow utilities: glow-{amber,blue,indigo})
  - style.css: added global `*:focus-visible` ring via `@apply`
- **All builds pass**: `npm run build` (tsc + vite) clean

### Startup Performance Optimization (session 1)
- **bootstrap IPC 并行化**: `useAppStore.ts:76-96` — 6 个串行 `await` 改为 `Promise.all`，启动时间预计减少 500-1500ms
- **首屏免动画**: `App.tsx` `AnimatePresence` + `Sidebar.tsx` stagger container 添加 `initial={hasMounted.current}`，跳过首帧入场动画，内容直接呈现
- **字体加载优化**: `@import` 从 `style.css` 移出 → `index.html` 的 `<link rel="stylesheet">`；添加 `<link rel="preconnect">` 到 fonts.googleapis.com / fonts.gstatic.com，消除 CSS @import 阻塞
- **Vite 代码分割**: `vite.config.ts` 添加 `manualChunks`，单包 437 KB → 6 个并行 chunk：
  - `vendor-react`: 3.9 KB
  - `vendor-framer`: 135 KB
  - `vendor-lucide`: 19.7 KB
  - `vendor-tauri`: 16 KB
  - `vendor-state`: 0.65 KB
  - `index` (app code): 270 KB
- **Build time**: 22s → 5s (得益于并行 chunk 和 esbuild 默认压缩)
- `npm run build` + `cargo check -p SafeMask` clean

### Deep Optimization: framer-motion removal, font swap, bootstrap split (session 2)
- **Crash fix (startup)**: Removed `visible: false` from tauri.conf.json; removed redundant `getCurrentWindow().show()` calls from `bootstrap()` — window now appears after explicit `show()` only
- **framer-motion → CSS transitions**: Removed all `motion.*` / `AnimatePresence` imports from 9 files (Sidebar, Header, StatCard, FileProcessor, MagicFeedback, ExitConfirm, SkeletonPage, App.tsx, style.css). Replaced with CSS `transition-*` + Tailwind `opacity`/`translate` classes. Eliminated ~400 modules from bundle (2011 → 1610). Vendor chunk for framer-motion removed from vite.config.ts.
- **Bootstrap split**: Changed from 6× parallel IPC calls to 2 critical (settings, stats) + 4 deferred (history, appInfo, aiStatus, engineInfo) via `setTimeout(cb, 100)` — first screen renders without waiting for non-critical data
- **Font optimization**: Removed self-hosted `inter.woff2` and `jetbrains-mono.woff2`, dropped Google Fonts `<link>` from index.html, switched to system font stack (`system-ui`). Removed all `@font-face` declarations from style.css.
- **Lazy loading**: Added `React.lazy(() => import(...))` for MagicFeedback, ExitConfirm, FileProcessor — these components no longer block initial render
- **Skeleton cleanup**: Removed `animate-pulse` CSS class from skeleton page, deleted `src/lib/animations.ts` (was only used by framer-motion wrappers)
- **Build verified**: `npm run build` (tsc + vite) clean at 4.12s; `cargo check -p SafeMask` clean

### Record Writer — AI Training Record Persistence (session 3)
- **Design**: Approved `.md` + YAML front matter format; append mode; 150-record per-file cap with seq numbering; auto-detect output dir from `{app_data}/records/YYYY/YYYY-MM-DD-{seq:03}.md`
- **Trait**: `trait RecordWriter: Send + Sync` via `#[async_trait]` in `infra/record_writer/mod.rs`
- **Implementation**: `MarkdownRecordWriter` uses `tokio::sync::mpsc` channel + background `tokio::spawn` task. Flushes every 5s OR 10 items. Each record includes fenced original/masked code blocks + entity table + stats summary.
- **Hooks**: Clipboard (`handler.rs:record_privacy_history()`) and file processing (`api/files.rs` after `process_file`) both write records via `Arc<dyn RecordWriter>` cloned from parking_lot guard before `.await`
- **Entity plumbing**: `ProcessStats.entities: Vec<EntitySpanBrief>` — all format handlers use `mask_line_with_entities`. mmap parallel path collects via `Arc<Mutex<Vec>>`
- **Lifecycle**: `rebuild_record_writer()` in `system.rs` called on settings update; `init_record_writer()` at startup
- **Settings toggle**: `AppSettings.record_writer_enabled: bool` (default `false`) — frontend `SettingToggle` with `FileText` icon in Kernel section
- **Verified**: `cargo check` clean; `cargo clippy` — zero new warnings (only 19 pre-existing); `cargo test` — 91/91 pass (88 existing + 3 new); `npm run build` clean

### Next Up
- Refactor SettingsPage inline toggles → shared Toggle component
- React.lazy 懒加载非首屏页面 (History/Rules/Settings)
- Add aria-labels across interactive elements
- Ongoing P1-P2 items from audit
