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

React 19 + TypeScript + Zustand + Tailwind CSS v3 + Framer Motion + Vite 6. No routing library — tabs managed via Zustand `activeTab` state and `AnimatePresence`. UI components in `src/components/{dashboard,feedback,history,layout,overlay,rules,settings,ui}/`.

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
