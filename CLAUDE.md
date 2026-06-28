# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

SafeMask is a Tauri v2 desktop application for privacy data masking. It runs 100% offline, processing sensitive data locally through a high-performance Rust backend with a Vue 3 frontend.

## Development Commands

```bash
# Install dependencies
npm install

# Start development (launches Vite dev server + Tauri window)
npm run tauri dev

# Build for production
npm run tauri build

# Frontend only (Vite dev server, no Tauri)
npm run dev

# Type check frontend
npm run build  # runs vue-tsc && vite build

# Rust commands (run from src-tauri/)
cargo check          # Fast compilation check
cargo fmt            # Format Rust code
cargo clippy -- -D warnings  # Lint
cargo test           # Run all tests
cargo test -- --nocapture   # Show println output
cargo test test_name         # Run specific test
```

## Architecture

### Tauri Workspace Structure

```
SafeMask/
├── src/                  # Vue 3 frontend
│   ├── components/       # UI components (Vue SFC)
│   ├── services/api.ts   # Tauri IPC command wrappers
│   ├── stores/           # Pinia state management
│   └── style.css         # Global styles (Tailwind)
├── src-tauri/            # Rust backend (workspace member)
│   └── src/
│       ├── api/          # Tauri Commands (IPC interface layer)
│       ├── common/       # Global types: AppState, AppError, events
│       ├── core/         # Pure business logic (no Tauri dependency)
│       └── infra/        # OS interactions: clipboard, filesystem, config
└── package.json
```

### Backend Layers (src-tauri/src/)

- **api/**: `#[tauri::command]` functions that handle frontend requests. Files: `system.rs`, `text.rs`, `files.rs`
- **core/**: Masking engine (`engine.rs`), rule definitions (`rules.rs`), config models (`config.rs`). No Tauri imports — independently testable.
- **common/**: Shared types — `AppState` (global state with Arc/RwLock), `AppError` (thiserror), event name constants
- **infra/**: Platform-specific implementations — clipboard monitoring (`clipboard/`), parallel file processing with mmap (`fs/`), config persistence (`config/`)

### Frontend (src/)

- **App.vue**: Main layout with sidebar navigation, universe mode indicator, settings panel
- **components/**: RuleManager, FileProcessor, HistoryList, Settings, Sidebar, StatCard, ExitConfirm, MagicFeedback
- **services/api.ts**: TypeScript wrappers for all Tauri IPC commands
- **stores/useAppStore.ts**: Pinia store for global app state

### Key Rust Dependencies

- `aho-corasick` + `regex`: Hybrid matching engine (literal + regex patterns)
- `rayon`: CPU parallelism for masking operations
- `memmap2` + `memchr`: Zero-copy file processing for large files
- `arboard` + `clipboard-master`: Cross-platform clipboard access and monitoring
- `enigo`: Keyboard simulation for Magic Paste (Alt+V)
- `parking_lot`: Faster mutex implementations
- `mimalloc`: Memory allocator

## Core Features

- **Shadow Mode (default)**: Clipboard passes through untouched; masking happens only on Alt+V paste into AI tools
- **Sentry Mode**: Active clipboard interception — all copied data is masked immediately
- **File Processing**: Parallel masking for large files (txt, log, docx, xlsx, pdf) using mmap + ordered pipeline
- **Rule Sandbox**: Real-time regex testing with error backtracking

## IPC Flow

Frontend calls `invoke()` from `@tauri-apps/api` → Tauri routes to `#[tauri::command]` in `api/` → `api/` delegates to `core/` for business logic → `infra/` handles OS interactions → Result returned to frontend

## Configuration

- Rules are YAML files stored in `src-tauri/rules/` (built-in) and `src-tauri/custom/` (user-created)
- App settings persisted via `infra/config/loader.rs`
- Global shortcuts managed by `tauri-plugin-global-shortcut`
