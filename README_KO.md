<div align="center">

<img src="src-tauri/icons/icon.png" width="128" alt="SafeMask logo"/>

# SafeMask

**물리 우주에서는 진실을 유지하고, 디지털 우주에서는 안전을 교환합니다.**

AI 시대를 위한 산업급 **로컬 우선** 개인정보 마스킹 — 클립보드, 파일, 규칙을 완전 오프라인으로.

<br/>

[![Rust](https://img.shields.io/badge/Rust-2024-orange?logo=rust)](https://www.rust-lang.org/) [![Tauri](https://img.shields.io/badge/Tauri-v2-blue)](https://v2.tauri.app/) [![React](https://img.shields.io/badge/React-19-61dafb?logo=react)](https://react.dev/) [![License](https://img.shields.io/badge/License-MIT-gray)](LICENSE) [![Offline](https://img.shields.io/badge/Privacy-100%25%20Offline-teal)](#-개인정보--보안) [![Release](https://img.shields.io/badge/Release-v2.1.x-emerald)](https://github.com/AiToByte/SafeMask/releases)
<br/>

**언어:** [English](README.md) · [简体中文](README_CN.md) · [日本語](README_JA.md) · **한국어** · [Русский](README_RU.md)

</div>

---

## 목차

- [왜 SafeMask인가?](#왜-safemask인가)
- [동작 예시](#동작-예시)
- [주요 기능](#주요-기능)
- [로컬 AI 엔진(온디바이스 NER)](#로컬-ai-엔진온디바이스-ner)
- [설치](#설치)
- [사용 방법](#사용-방법)
- [개발](#개발)
- [아키텍처(요약)](#아키텍처요약)
- [문서 색인](#문서-색인)
- [개인정보 · 보안](#개인정보--보안)
- [자주 묻는 질문](#자주-묻는-질문)
- [릴리스](#릴리스)
- [기여](#기여)
- [라이선스](#라이선스)

---

## 왜 SafeMask인가?

로그·코드·회의록을 ChatGPT, Claude 등에 붙여넣을 때 API 키, 전화번호, 이메일, 사내 IP, 실명 등이 함께 유출됩니다.

**SafeMask**는 기기 안에서만 동작합니다. 텔레메트리 없음, 콘텐츠 클라우드 업로드 없음. 규칙·정규식·선택적 로컬 AI 모델·기록·마스킹이 모두 오프라인입니다.

| 문제 | SafeMask 해결 |
|------|----------------|
| AI에 비밀 오붙여넣기 | **Magic Paste**로 마스킹 후 원본 클립보드 복원 |
| 로컬 작업에 원문 필요 | **Shadow 모드**가 기본적으로 평문 유지 |
| 화면 공유 등 고위험 | **Sentry 모드**가 복사 후 자동 정화 |
| 정규식으로 잡기 어려운 이름·주소 | **로컬 AI NER** — 양자화 ONNX 모델, 100% 온디바이스 추론 |
| 대용량 로그 | **mmap + Rayon** 순서 보장 파이프라인 |
| 조직 맞춤 정책 | **규칙 관리** + YAML 가져오기/내보내기 |

---

## 동작 예시

안전한 붙여넣기까지 단 한 번의 키 입력. **마스킹 전** — 복사한 내용:

```text
2026-07-24 10:32:01 INFO user=张伟 email=zhang.wei@example.com phone=13812345678 ip=192.168.31.10 key=sk-a1b2c3d4e5f6g7h8i9j0k1l2m3n4o5p6
```

**마스킹 후** — 실제로 LLM에 전달되는 내용:

```text
2026-07-24 10:32:01 INFO user=<PERSON> email=<EMAIL> phone=<CHINA_MOBILE> ip=<IPv4> key=<OPENAI_KEY>
```

이름은 **AI 모델**이, 나머지는 **내장 규칙**이 탐지 — 붙여넣기 직후 원본 클립보드는 자동 복원됩니다.

---

## 주요 기능

### 듀얼 유니버스 클립보드

- **Shadow(기본)** — `Ctrl+C`는 원문 유지. **`Alt+V`**(설정 가능)로 Magic Paste: 백업 → 마스킹 → 붙여넣기 → 복원.
- **Sentry** — 복사 직후 자동 살균.
- **`Alt+M`** 모드 전환, 항상 위 창 지원.

### 하이브리드 탐지 엔진

- **Aho-Corasick** 사전 / 키워드(우선순위 100)
- **바이트 단위 정규식**(우선순위 90)
- **선택적 로컬 AI NER**(우선순위 50) — [아래](#로컬-ai-엔진온디바이스-ner) 참조
- **서브 스팬 카빙 충돌 해소** — 좁은 고우선순위 스팬이 넓은 저우선순위 스팬을 절단. 규칙 일치는 겹치는 AI 스팬을 억제

### 규칙과 설정

- 내장 YAML 규칙 팩(AI 키, 네트워크, 개인정보, 코드, DB 접속 문자열 등)
- 커스텀 규칙 + 라이브 정규식 샌드박스
- **다중 파일 YAML 가져오기**(동명 커스텀 덮어쓰기, 내장 동명 건드넘김)
- **커스텀 규칙 내보내기** + 템플릿 다운로드
- 전역 래퍼 스타일: `<TAG>` / `[TAG]` — 모든 규칙과 AI 라벨에 즉시 적용

### 파일 파이프라인

- mmap, 약 8MB 청크, 줄 경계 안전 분할
- 멀티코어 순서 기록 및 배압 제어
- 대용량 파일에서도 안정적인 메모리 사용량

### UI / 테마

- React 19 + Zustand + Tailwind, 네이티브 윈도우 크롬
- 확장 가능한 테마: **Default(인더스트리얼 앰버)** / **Claude(웜 페이퍼)**

### 선택적 감사 기록

- 옵트인 Markdown 감사 라이터(평문 PII가 디스크에 저장됨 — **기본 꺼짐**)

---

## 로컬 AI 엔진(온디바이스 NER)

SafeMask에는 규칙으로 표현하기 어려운 **사람 이름, 주소, 조직명** 등 문맥 의존 엔티티를 위한 선택적 AI 레이어가 있습니다.

### 개요

- `openai/privacy-filter` 기반 양자화(**q4**) ONNX NER 모델 — 8레이어 MoE 토큰 분류기, 33개 BIOES 라벨.
- 추론은 ONNX Runtime + HuggingFace tokenizers로 **100% 온디바이스** 실행. 텍스트가 기기 밖으로 나가지 않습니다.
- 모델은 설치 프로그램에 **포함되지 않습니다** — 명시적인 1회 다운로드가 필요합니다.

### 탐지 항목

| 모델 엔티티 | 마스크 라벨 | 예시 |
|---|---|---|
| `private_person` | `<PERSON>` | 张伟, John Smith |
| `private_email` | `<EMAIL>` | zhang.wei@example.com |
| `private_phone` | `<PHONE>` | +86 138 1234 5678 |
| `private_address` | `<ADDRESS>` | 北京市朝阳区… |
| `account_number` | `<BANK_CARD>` | 6222 0212 3456 7890 |
| `private_date` | `<DATE>` | 개인 날짜 |
| `private_url` | `<URL>` | 개인 URL |
| `secret` | `<API_KEY>` | 토큰·시크릿 |

### 활성화 방법

**방법 A — 원클릭 다운로드(권장)**

1. **설정 → AI 엔진**을 엽니다.
2. **다운로드** 클릭(약 550MB zip, 설치 중 약 2GB 여유 공간 필요).
3. 여러 미러에서 자동 폴백으로 다운로드되고, SHA-256 검증·압축 해제 후 **핫 로드** — 재시작 불필요.

**방법 B — 셀프 서비스 미러**

서버 대역폭이 제한되어 있으므로 인앱 다운로드가 느린 경우 아래 미러 중 하나에서 `privacy-filter.zip`을 받으세요:

| 미러 | 링크 | 코드 |
|---|---|---|
| HuggingFace | [privacy-filter.zip](https://huggingface.co/buckets/XiaoShengCYZ/AI_Models/resolve/privacy-filter.zip?download=true) | — |
| Quark 넷디스크 (夸克网盘) | [pan.quark.cn](https://pan.quark.cn/s/51647902f801?pwd=HQ1Y) | `HQ1Y` |
| Baidu 넷디스크 (百度网盘) | [pan.baidu.com](https://pan.baidu.com/s/1mDBr0mdo2r-guC4LshF87w?pwd=ba7b) | `ba7b` |

zip을 `SafeMask.exe` 옆의 `models/privacy-filter/`에 압축 해제한 후 SafeMask를 재시작하세요.

**방법 C — 수동 배치**

다음 파일을 `SafeMask.exe` 옆의 `models/privacy-filter/`에 배치:

```text
models/privacy-filter/
├── model_q4.onnx         # 양자화 모델(엔트리)
├── model_q4.onnx_data    # 양자화 가중치(약 875MB)
├── tokenizer.json        # HuggingFace 토크나이저
└── config.json           # id2label 메타데이터
```

> 시작 시 작업 디렉터리의 `./models`, 앱 로컬 데이터 디렉터리, 리소스 디렉터리도 순서대로 검색합니다.

### 동작 특성

- **지연 로드** — 모델은 첫 마스킹 실행 시 백그라운드에서 로드(보통 1~3분, 타임아웃 5분). 상태와 경과 시간은 설정에 표시되며 상세는 `ai_model_load.log`에 기록됩니다.
- **규칙 우선** — AI 우선순위 50, 규칙 90~100. 같은 텍스트에서는 결정론적 규칙 일치가 항상 우선합니다.
- **런타임 토글** — 설정에서 언제든 AI 온/오프. 모델 파일이 있으면 시작 시 자동 활성화.
- **우아한 저하** — 모델이 없어도 나머지 기능은 동일. 규칙 전용 모드로 자동 복귀합니다.

### 튜닝

| 항목 | 기본값 | 설명 |
|---|---|---|
| `ORT_NUM_THREADS` 환경 변수 | `2` | ONNX Runtime 추론 스레드 수 |
| 신뢰도 임계값 | `0.5` | 미만의 AI 스팬은 폐기 |
| 컨텍스트 윈도우 | 512 토큰 | 추론 1회당 길이 |

---

## 설치

### 빌드된 설치 프로그램(권장)

[**GitHub Releases**](https://github.com/AiToByte/SafeMask/releases)에서 다운로드:

| 패키지 | 설명 |
|---|---|
| `SafeMask_x.y.z_x64-setup.exe` | Windows NSIS 설치 프로그램 — 권장 |
| `SafeMask_x.y.z_x64_zh-CN.msi` | Windows MSI |
| macOS / Linux 패키지 | `v*` 태그마다 CI가 생성 |

AI 모델은 선택 사항이며 앱 내에서 별도로 다운로드합니다([로컬 AI 엔진](#로컬-ai-엔진온디바이스-ner) 참조).

---

## 사용 방법

### 클립보드 워크플로

1. 평소처럼 복사(`Ctrl+C`) — **Shadow 모드**에서는 원문이 클립보드에 유지됩니다.
2. 대상(ChatGPT, Claude, 웹 폼…)에 포커스.
3. **`Alt+V`** 입력 — SafeMask가 백업 → 마스킹 → 안전한 텍스트 붙여넣기 → 원본 클립보드 복원.
4. 엄격한 관리가 필요하면 **`Alt+M`**으로 **Sentry 모드** — 복사할 때마다 즉시 정화.

| 단축키 | 동작 | 변경 가능 |
|---|---|---|
| `Alt+V` | Magic Paste(마스킹 후 붙여넣기) | ✅ 키와 붙여넣기 지연 |
| `Alt+M` | Shadow / Sentry 전환 | ❌(고정) |

### 커스텀 규칙

**규칙** 화면에서 추가하거나 YAML을 가져옵니다:

```yaml
group: "MY_COMPANY"
rules:
  - name: "Internal_Project_Code"
    pattern: '\bPRJ-\d{6}\b'
    mask: "<PROJECT_CODE>"
    priority: 10
```

저장 전 내장 샌드박스에서 라이브 테스트 가능. 전체 형식은 [docs/RULES_IMPORT.md](docs/RULES_IMPORT.md) 참조.

### 마스크 래퍼 스타일

대괄호를 선호한다면 **설정 → 래퍼 스타일**에서 `<TAG>`와 `[TAG]`를 즉시 전환 — 내장·커스텀·AI 라벨 모두에 적용됩니다.

### 파일 처리

로그 파일을 대시보드에 드래그(또는 파일 선택) — 메모리 매핑 + 멀티코어 파이프라인으로 스트리밍 처리하고 원본 옆에 마스킹된 복사본을 출력합니다. 대용량 파일에서도 메모리는 안정적입니다.

---

## 개발

### 요구 사항

- Node.js 18+
- Rust stable(edition 2024 툴체인)
- [Tauri v2 사전 요구](https://v2.tauri.app/start/prerequisites/)

### 실행

```bash
# JS 의존성 설치
npm install

# 데스크톱 앱(Vite + Tauri)
npm run tauri dev

# 프런트엔드만(http://127.0.0.1:18924)
npm run dev
```

### 빌드

```bash
npm run build
npm run tauri build
```

### Rust 검사(워크스페이스 루트)

```bash
cargo check  -p SafeMask
cargo test   -p SafeMask
cargo clippy -p SafeMask -- -D warnings
cargo fmt    -p SafeMask
```

### 환경 변수

| 변수 | 기본값 | 용도 |
|---|---|---|
| `SAFEMASK_THREADS` | `2` | Rayon 워커 스레드 수(파일 파이프라인) |
| `ORT_NUM_THREADS` | `2` | ONNX Runtime 추론 스레드 수 |

---

## 아키텍처(요약)

```
React 19 UI
    │  invoke / events
    ▼
api/*  (Tauri 커맨드)
    ▼
orchestrator  (Shadow / Sentry)
    ▼
hybrid_engine  → recognizers → resolver → masking
    ▼
infra  (clipboard, mmap files, ONNX, config, records)
```

- `core/`는 Tauri 임포트가 **0** — 독립 단위 테스트 가능
- 오프셋은 UTF-8 **바이트 오프셋**
- 커스텀 규칙은 앱 `custom/` 스토리지의 `user_rules.yaml`

더 보기: [CLAUDE.md](CLAUDE.md) · [docs/](docs/) · [docs/RULES_IMPORT.md](docs/RULES_IMPORT.md) · [docs/THEMES.md](docs/THEMES.md)

---

## 문서 색인

| 문서 | 설명 |
|------|------|
| [README.md](README.md) | English |
| [README_CN.md](README_CN.md) | 简体中文 |
| [README_JA.md](README_JA.md) | 日本語 |
| [README_KO.md](README_KO.md) | 한국어(본 문서) |
| [README_RU.md](README_RU.md) | Русский |
| [CLAUDE.md](CLAUDE.md) | 아키텍처 노트 |
| [DEVELOPMENT.md](DEVELOPMENT.md) | 개발 가이드 |
| [docs/使用手册.md](docs/使用手册.md) | 사용자 핸드북(중국어) |
| [docs/RULES_IMPORT.md](docs/RULES_IMPORT.md) | 규칙 가져오기/내보내기 |
| [docs/THEMES.md](docs/THEMES.md) | 테마 시스템 |
| [docs/record-writer.md](docs/record-writer.md) | 감사 기록 라이터 |
| [GitHub Releases](https://github.com/AiToByte/SafeMask/releases) | 빌드된 설치 프로그램 |
| [Issues](https://github.com/AiToByte/SafeMask/issues) | 버그 신고·기능 요청 |
| [LICENSE](LICENSE) | MIT |

---

## 개인정보 · 보안

- 마스킹 콘텐츠의 클라우드 전송은 **절대 없음**
- **AI 추론은 100% 온디바이스**. 유일한 네트워크 트래픽은 선택적·명시적 1회 모델 다운로드(허용 목록 미러만)
- 모델 다운로드는 사용자가 시작하는 선택 작업
- 감사 기록은 기본 꺼짐(켜면 평문 원본이 로컬에 저장됨)
- 신뢰할 수 없는 규칙 파일 가져오기 주의

---

## 자주 묻는 질문

**클립보드나 파일이 외부로 전송되나요?**
아니요. 규칙·AI 모두 모든 마스킹이 로컬에서 실행됩니다. 텔레메트리도 콘텐츠 업로드도 없습니다.

**AI 모델 없이도 사용할 수 있나요?**
네. 규칙 엔진(사전 + 정규식)만으로 완전히 동작합니다. AI는 이름·주소 같은 문맥 의존 엔티티를 보강하는 레이어입니다.

**첫 AI 마스킹이 느린 이유는?**
약 900MB 모델이 첫 사용 시 지연 로드되기 때문입니다(보통 1~3분). 이후에는 즉시입니다. 진행 상황은 설정 → AI 엔진에 표시됩니다.

**AI 모델은 어디에 저장되나요?**
실행 파일 옆의 `models/privacy-filter/`(또는 앱 로컬 데이터/리소스 디렉터리). 폴더를 삭제하면 완전히 제거됩니다.

**자체 탐지 규칙을 사용할 수 있나요?**
네 — 라이브 샌드박스가 있는 커스텀 YAML 규칙과 다중 파일 가져오기/내보내기를 지원합니다. [docs/RULES_IMPORT.md](docs/RULES_IMPORT.md) 참조.

---

## 릴리스

빌드된 설치 프로그램: [GitHub Releases](https://github.com/AiToByte/SafeMask/releases)

`v*` 태그에서 CI가 멀티 플랫폼 패키지(Windows / macOS / Linux)를 생성합니다.

---

## 기여

1. 작고 집중된 PR 권장
2. 푸시 전 `cargo test -p SafeMask`와 `npm run build` 실행
3. 기존 레이아웃 준수: `core/` 순수 로직, `infra/` OS, `api/` IPC
4. 새 테마: [docs/THEMES.md](docs/THEMES.md) 참조
5. 새 규칙 팩: `RuleGroup` 호환 또는 규칙 배열 YAML

이슈·PR 환영: [AiToByte/SafeMask](https://github.com/AiToByte/SafeMask)

---

## 라이선스

[MIT](LICENSE) © SafeMask / AiToByte

---

<div align="center">

[English](README.md) · [简体中文](README_CN.md) · [日本語](README_JA.md) · **한국어** · [Русский](README_RU.md)

</div>
