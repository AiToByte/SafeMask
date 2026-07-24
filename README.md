<div align="center">

<img src="src-tauri/icons/icon.png" width="128" alt="SafeMask logo"/>

# SafeMask

**Keep the truth in the physical universe. Exchange safety in the digital universe.**

Industrial-grade **local-first** privacy masking for the AI era — clipboard, files, and rules, fully offline.

<br/>

[![Rust](https://img.shields.io/badge/Rust-2024-orange?logo=rust)](https://www.rust-lang.org/) [![Tauri](https://img.shields.io/badge/Tauri-v2-blue)](https://v2.tauri.app/) [![React](https://img.shields.io/badge/React-19-61dafb?logo=react)](https://react.dev/) [![License](https://img.shields.io/badge/License-MIT-gray)](LICENSE) [![Offline](https://img.shields.io/badge/Privacy-100%25%20Offline-teal)](#-privacy--security) [![Release](https://img.shields.io/badge/Release-v2.1.x-emerald)](https://github.com/AiToByte/SafeMask/releases)
<br/>

**Languages:** **English** · [简体中文](README_CN.md) · [日本語](README_JA.md) · [한국어](README_KO.md) · [Русский](README_RU.md)

</div>

---

## Table of contents

- [Why SafeMask?](#why-safemask)
- [See it in action](#see-it-in-action)
- [Features](#features)
- [Local AI engine (on-device NER)](#local-ai-engine-on-device-ner)
- [Installation](#installation)
- [Usage](#usage)
- [Development](#development)
- [Architecture (overview)](#architecture-overview)
- [Documentation index](#documentation-index)
- [Privacy & security](#privacy--security)
- [FAQ](#faq)
- [Releases](#releases)
- [Contributing](#contributing)
- [License](#license)

---

## Why SafeMask?

When you paste logs, code, or notes into ChatGPT, Claude, or any LLM, secrets travel with them: API keys, phone numbers, emails, internal IPs, real names, and more.

**SafeMask** runs entirely on your machine. No telemetry. No cloud upload of content. Rules, regex, an optional on-device AI model, history, and masking all stay offline.

| Problem | SafeMask approach |
|--------|-------------------|
| Accidental paste of secrets into AI | **Magic Paste** injects masked text, then restores the original clipboard |
| Need original text for local tools | **Shadow mode** keeps plaintext on the system clipboard by default |
| Screen sharing / strict environments | **Sentry mode** actively sanitizes the clipboard after every copy |
| Names & addresses that regex can't catch | **Local AI NER** — a quantized ONNX model running 100% on-device |
| Huge log files | **mmap + Rayon** pipeline with ordered write-back |
| Custom policies | **Rule manager** with import / export YAML |

---

## See it in action

One keystroke between you and a safe paste. **Before** — what you copied:

```text
2026-07-24 10:32:01 INFO user=张伟 email=zhang.wei@example.com phone=13812345678 ip=192.168.31.10 key=sk-a1b2c3d4e5f6g7h8i9j0k1l2m3n4o5p6
```

**After** — what actually reaches the LLM:

```text
2026-07-24 10:32:01 INFO user=<PERSON> email=<EMAIL> phone=<CHINA_MOBILE> ip=<IPv4> key=<OPENAI_KEY>
```

The name was caught by the **AI model**, the rest by **built-in rules** — your original clipboard is restored immediately after the paste.

---

## Features

### Dual-universe clipboard

- **Shadow mode (default)** — `Ctrl+C` keeps the real text. Press **`Alt+V`** (configurable) for Magic Paste: backup → mask → paste → restore.
- **Sentry mode** — sensitive clipboard content is bleached automatically after copy.
- Toggle modes with **`Alt+M`**. Optional always-on-top window pin.

### Hybrid detection engine

- **Aho-Corasick** for dictionaries / keywords (priority 100)
- **Byte-level regex** for high-throughput pattern matching (priority 90)
- **Optional local AI NER** (priority 50) — see [below](#local-ai-engine-on-device-ner)
- **Sub-span carving** conflict resolution — narrow high-priority spans carve wide low-priority ones, and rule hits suppress overlapping AI spans

### Rules & configuration

- Built-in YAML rule packs (auth keys, network, personal info, code, database URIs, …)
- Custom rules with a live regex sandbox
- **Multi-file YAML import** (overwrites same-name custom rules, skips built-in name conflicts)
- **Export custom rules** + downloadable import template
- Global mask wrapper style: `<TAG>` or `[TAG]` — applied to every rule and AI label at runtime

### File pipeline

- Memory-mapped I/O, ~8 MB chunks, line-safe splits
- Ordered multi-core processing with backpressure
- Stable RAM footprint on very large inputs

### UI & themes

- React 19 + Zustand + Tailwind, native window chrome
- Extensible theme system: **Default (industrial amber)** and **Claude (warm paper)**

### Optional audit records

- Opt-in Markdown audit writer for reviewing what was masked (plaintext PII on disk — **off by default**)

---

## Local AI engine (on-device NER)

SafeMask ships with an optional AI layer for what rules can't express: **person names, addresses, organizations** and other context-dependent entities.

### What it is

- A quantized (**q4**) ONNX NER model based on `openai/privacy-filter` — an 8-layer MoE token classifier with 33 BIOES labels.
- Inference runs **100% on-device** via ONNX Runtime + HuggingFace tokenizers. No text ever leaves your machine.
- The model is **not bundled** with the installer — it's a separate, explicit, one-time download.

### What it recognizes

| Model entity | Mask label | Example |
|---|---|---|
| `private_person` | `<PERSON>` | 张伟, John Smith |
| `private_email` | `<EMAIL>` | zhang.wei@example.com |
| `private_phone` | `<PHONE>` | +86 138 1234 5678 |
| `private_address` | `<ADDRESS>` | 北京市朝阳区… |
| `account_number` | `<BANK_CARD>` | 6222 0212 3456 7890 |
| `private_date` | `<DATE>` | personal dates |
| `private_url` | `<URL>` | personal URLs |
| `secret` | `<API_KEY>` | tokens & secrets |

### How to enable

**Option A — one-click download (recommended)**

1. Open **Settings → AI Engine**.
2. Click **Download** (~550 MB zip; ~2 GB free disk required during install).
3. The model downloads from a list of mirrors with automatic fallback, is verified (SHA-256), extracted, and **hot-loaded** — no restart needed.

**Option B — self-service mirrors**

Server bandwidth is limited — if the in-app download is slow, grab `privacy-filter.zip` from any mirror:

| Mirror | Link | Code |
|---|---|---|
| HuggingFace | [privacy-filter.zip](https://huggingface.co/buckets/XiaoShengCYZ/AI_Models/resolve/privacy-filter.zip?download=true) | — |
| Quark Netdisk (夸克网盘) | [pan.quark.cn](https://pan.quark.cn/s/51647902f801?pwd=HQ1Y) | `HQ1Y` |
| Baidu Netdisk (百度网盘) | [pan.baidu.com](https://pan.baidu.com/s/1mDBr0mdo2r-guC4LshF87w?pwd=ba7b) | `ba7b` |

Extract the zip into `models/privacy-filter/` next to `SafeMask.exe`, then restart SafeMask.

**Option C — manual placement**

Place these files into `models/privacy-filter/` next to `SafeMask.exe`:

```text
models/privacy-filter/
├── model_q4.onnx         # quantized model (stub)
├── model_q4.onnx_data    # quantized weights (~875 MB)
├── tokenizer.json        # HuggingFace tokenizer
└── config.json           # id2label metadata
```

> Also searched at startup: `./models` (working directory), the app-local-data directory, and the app resources directory.

### How it behaves

- **Lazy loading** — the model stays on disk until the first masked copy/paste, then loads in the background (typically 1–3 min, 5 min timeout). Status and elapsed time are shown in Settings; details are logged to `ai_model_load.log`.
- **Rules win overlaps** — AI runs at priority 50, rules at 90–100, so deterministic rule hits always take precedence over AI guesses on the same text.
- **Runtime toggle** — switch AI on/off anytime in Settings. When the model files exist, AI is auto-enabled at startup.
- **Graceful degradation** — without the model, everything else works identically; SafeMask simply falls back to rules-only mode.

### Tuning

| Knob | Default | Notes |
|---|---|---|
| `ORT_NUM_THREADS` env var | `2` | ONNX Runtime inference threads |
| Confidence threshold | `0.5` | Below this, AI spans are discarded |
| Context window | 512 tokens | Per inference pass |

---

## Installation

### Prebuilt installers (recommended)

Download from [**GitHub Releases**](https://github.com/AiToByte/SafeMask/releases):

| Package | Notes |
|---|---|
| `SafeMask_x.y.z_x64-setup.exe` | Windows NSIS installer — recommended |
| `SafeMask_x.y.z_x64_zh-CN.msi` | Windows MSI |
| macOS / Linux packages | Produced by CI on every `v*` tag |

The AI model is optional and downloaded separately from inside the app (see [Local AI engine](#local-ai-engine-on-device-ner)).

---

## Usage

### Clipboard workflow

1. Copy anything as usual (`Ctrl+C`) — in **Shadow mode** the real text stays on your clipboard.
2. Focus the target (ChatGPT, Claude, a web form…).
3. Press **`Alt+V`** — SafeMask backs up your clipboard, masks the text, pastes the safe version, then restores your original clipboard.
4. Need strict hygiene? Toggle **Sentry mode** with **`Alt+M`** — every copy is sanitized in place.

| Shortcut | Action | Configurable |
|---|---|---|
| `Alt+V` | Magic Paste (mask & paste) | ✅ shortcut & paste delay |
| `Alt+M` | Toggle Shadow / Sentry mode | ❌ (hard-bound) |

### Custom rules

Add your own patterns in **Rules**, or import a YAML pack:

```yaml
group: "MY_COMPANY"
rules:
  - name: "Internal_Project_Code"
    pattern: '\bPRJ-\d{6}\b'
    mask: "<PROJECT_CODE>"
    priority: 10
```

Test patterns live in the built-in regex sandbox before saving. See [docs/RULES_IMPORT.md](docs/RULES_IMPORT.md) for the full format.

### Mask wrapper style

Prefer square brackets? **Settings → Mask wrapper style** switches every mask between `<TAG>` and `[TAG]` instantly — built-in rules, custom rules, and AI labels all follow.

### File processing

Drag a log file onto the dashboard (or use the file picker) — SafeMask streams it through a memory-mapped, multi-core pipeline and writes a masked copy next to the original, with a stable RAM footprint even on very large files.

---

## Development

### Requirements

- Node.js 18+
- Rust stable (edition 2024 toolchain)
- Platform build deps for [Tauri v2](https://v2.tauri.app/start/prerequisites/)

### Run

```bash
# Install JS deps
npm install

# Full desktop app (Vite + Tauri)
npm run tauri dev

# Frontend only (http://127.0.0.1:18924)
npm run dev
```

### Build

```bash
npm run build
npm run tauri build
```

### Rust checks (workspace root)

```bash
cargo check  -p SafeMask
cargo test   -p SafeMask
cargo clippy -p SafeMask -- -D warnings
cargo fmt    -p SafeMask
```

### Environment variables

| Variable | Default | Purpose |
|---|---|---|
| `SAFEMASK_THREADS` | `2` | Rayon worker threads (file pipeline) |
| `ORT_NUM_THREADS` | `2` | ONNX Runtime inference threads |

---

## Architecture (overview)

```
React 19 UI
    │  invoke / events
    ▼
api/*  (Tauri commands)
    ▼
orchestrator  (Shadow / Sentry)
    ▼
hybrid_engine  → recognizers → resolver → masking
    ▼
infra  (clipboard, mmap files, ONNX, config, records)
```

- `core/` has **zero** Tauri imports — unit-testable in isolation
- Offsets are **byte offsets** on UTF-8 buffers
- Custom rules live under the app `custom/` storage path (`user_rules.yaml`)

More detail: [CLAUDE.md](CLAUDE.md) · [docs/](docs/) · [docs/RULES_IMPORT.md](docs/RULES_IMPORT.md) · [docs/THEMES.md](docs/THEMES.md)

---

## Documentation index

| Doc | Description |
|-----|-------------|
| [README.md](README.md) | English (this file) |
| [README_CN.md](README_CN.md) | 简体中文 |
| [README_JA.md](README_JA.md) | 日本語 |
| [README_KO.md](README_KO.md) | 한국어 |
| [README_RU.md](README_RU.md) | Русский |
| [CLAUDE.md](CLAUDE.md) | Contributor / agent architecture notes |
| [DEVELOPMENT.md](DEVELOPMENT.md) | Development guide |
| [docs/使用手册.md](docs/使用手册.md) | User handbook (Chinese) |
| [docs/RULES_IMPORT.md](docs/RULES_IMPORT.md) | Rule import / export format |
| [docs/THEMES.md](docs/THEMES.md) | Theme system |
| [docs/record-writer.md](docs/record-writer.md) | Audit record writer |
| [GitHub Releases](https://github.com/AiToByte/SafeMask/releases) | Prebuilt installers |
| [Issues](https://github.com/AiToByte/SafeMask/issues) | Bug reports & feature requests |
| [LICENSE](LICENSE) | MIT |

---

## Privacy & security

- **No cloud API** for masking content — ever
- **AI inference is 100% on-device**; the only network traffic is the optional, explicit one-time model download from allow-listed mirrors
- Model download is optional and user-initiated
- Audit writing is opt-in (stores sensitive originals locally)
- Prefer reviewing custom rules before importing from untrusted sources

---

## FAQ

**Does SafeMask send my clipboard or files anywhere?**
No. All masking — rules and AI alike — runs locally. There is no telemetry and no content upload.

**Does SafeMask work without the AI model?**
Yes. The rule engine (dictionaries + regex) is fully functional on its own. AI is an additive layer for names, addresses, and similar context-dependent entities.

**Why is the first AI-masked paste slow?**
The ~900 MB model is lazy-loaded on first use (typically 1–3 minutes). Subsequent masking is immediate. Progress is shown in Settings → AI Engine.

**Where is the AI model stored?**
In `models/privacy-filter/` next to the executable (or in the app-local-data / resources directory). Delete that folder to remove it completely.

**Can I use my own detection rules?**
Yes — custom YAML rules with a live regex sandbox, plus multi-file import/export. See [docs/RULES_IMPORT.md](docs/RULES_IMPORT.md).

---

## Releases

Prebuilt installers: [GitHub Releases](https://github.com/AiToByte/SafeMask/releases)

Tag-driven CI produces multi-platform packages (Windows / macOS / Linux) on `v*` tags.

---

## Contributing

1. Prefer small, focused PRs
2. Run `cargo test -p SafeMask` and `npm run build` before push
3. Follow existing layout: `core/` pure logic, `infra/` OS, `api/` IPC
4. New themes: see [docs/THEMES.md](docs/THEMES.md)
5. New rule packs: YAML compatible with `RuleGroup` or bare rule arrays

Issues and PRs welcome: [AiToByte/SafeMask](https://github.com/AiToByte/SafeMask)

---

## License

[MIT](LICENSE) © SafeMask contributors / AiToByte

---

<div align="center">

**English** · [简体中文](README_CN.md) · [日本語](README_JA.md) · [한국어](README_KO.md) · [Русский](README_RU.md)

</div>
