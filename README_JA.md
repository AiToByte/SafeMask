<div align="center">

<img src="src-tauri/icons/icon.png" width="128" alt="SafeMask logo"/>

# SafeMask

**物理宇宙では真実を保ち、デジタル宇宙では安全を交換する。**

AI 時代のための産業グレード **ローカルファースト** プライバシーマスキング — クリップボード・ファイル・ルールを完全オフラインで。

<br/>

[![Rust](https://img.shields.io/badge/Rust-2024-orange?logo=rust)](https://www.rust-lang.org/) [![Tauri](https://img.shields.io/badge/Tauri-v2-blue)](https://v2.tauri.app/) [![React](https://img.shields.io/badge/React-19-61dafb?logo=react)](https://react.dev/) [![License](https://img.shields.io/badge/License-MIT-gray)](LICENSE) [![Offline](https://img.shields.io/badge/Privacy-100%25%20Offline-teal)](#-プライバシーとセキュリティ) [![Release](https://img.shields.io/badge/Release-v2.1.x-emerald)](https://github.com/AiToByte/SafeMask/releases)
<br/>

**言語：** [English](README.md) · [简体中文](README_CN.md) · **日本語** · [한국어](README_KO.md) · [Русский](README_RU.md)

</div>

---

## 目次

- [なぜ SafeMask か？](#なぜ-safemask-か)
- [動作イメージ](#動作イメージ)
- [主な機能](#主な機能)
- [ローカル AI エンジン（オンデバイス NER）](#ローカル-ai-エンジンオンデバイス-ner)
- [インストール](#インストール)
- [使い方](#使い方)
- [開発](#開発)
- [アーキテクチャ（概要）](#アーキテクチャ概要)
- [ドキュメント索引](#ドキュメント索引)
- [プライバシーとセキュリティ](#プライバシーとセキュリティ)
- [よくある質問](#よくある質問)
- [リリース](#リリース)
- [コントリビュート](#コントリビュート)
- [ライセンス](#ライセンス)

---

## なぜ SafeMask か？

ChatGPT や Claude などにログ・コード・議事録を貼ると、API キー、電話番号、メール、社内 IP、実名などが一緒に流出します。

**SafeMask** はすべて端末上で動作します。テレメトリなし、コンテンツのクラウド送信なし。ルール・正規表現・任意のローカル AI モデル・履歴・マスキングはオフライン完結です。

| 課題 | SafeMask の解決 |
|------|-----------------|
| AI への誤ペースト | **Magic Paste** でマスク文を注入し、元のクリップボードを復元 |
| ローカル作業に原文が必要 | **Shadow モード** は既定で平文を保持 |
| 画面共有など高リスク | **Sentry モード** がコピー後に自動サニタイズ |
| 正規表現では拾えない人名・住所 | **ローカル AI NER** — 量子化 ONNX モデルで 100% オンデバイス推論 |
| 巨大ログ | **mmap + Rayon** 順序保証パイプライン |
| 独自ポリシー | **ルール管理** と YAML インポート/エクスポート |

---

## 動作イメージ

安全なペーストまで、ワンキー。**マスク前** — コピーした内容：

```text
2026-07-24 10:32:01 INFO user=张伟 email=zhang.wei@example.com phone=13812345678 ip=192.168.31.10 key=sk-a1b2c3d4e5f6g7h8i9j0k1l2m3n4o5p6
```

**マスク後** — 実際に LLM へ送られる内容：

```text
2026-07-24 10:32:01 INFO user=<PERSON> email=<EMAIL> phone=<CHINA_MOBILE> ip=<IPv4> key=<OPENAI_KEY>
```

人名は **AI モデル**、その他は**組み込みルール**が検出 — ペースト直後に元のクリップボードは自動復元されます。

---

## 主な機能

### デュアル・ユニバース クリップボード

- **Shadow（既定）** — `Ctrl+C` は原文を保持。**`Alt+V`**（変更可）で Magic Paste：バックアップ → マスク → ペースト → 復元。
- **Sentry** — コピー後に自動漂白。
- **`Alt+M`** でモード切替。ウィンドウ常時前面対応。

### ハイブリッド検出エンジン

- **Aho-Corasick** 辞書 / キーワード（優先度 100）
- **バイト級正規表現**（優先度 90）
- **任意のローカル AI NER**（優先度 50）— [下記](#ローカル-ai-エンジンオンデバイス-ner)参照
- **サブスパン切削による競合解決** — 狭い高優先度スパンが広い低優先度スパンを切削。ルール一致は重複する AI スパンを抑制

### ルールと設定

- 組み込み YAML ルールパック（AI キー、ネットワーク、個人情報、コード、DB 接続文字列など）
- カスタムルール + ライブ正規表現サンドボックス
- **複数 YAML インポート**（同名カスタムは上書き、組み込み同名はスキップ）
- **カスタムルールのエクスポート** + テンプレート
- グローバルラッパー：`<TAG>` / `[TAG]` — 全ルールと AI ラベルに即時反映

### ファイルパイプライン

- mmap、約 8MB チャンク、行境界安全分割
- マルチコア順序書き出しと背圧制御
- 巨大ファイルでも安定したメモリ使用量

### UI / テーマ

- React 19 + Zustand + Tailwind、ネイティブウィンドウ装飾
- 拡張可能なテーマ：**Default（インダストリアルアンバー）** / **Claude（ウォームペーパー）**

### 任意の監査レコード

- オプトインの Markdown 監査ライター（平文 PII をディスクに保存 — **既定オフ**）

---

## ローカル AI エンジン（オンデバイス NER）

SafeMask には、ルールでは表現しにくい**人名・住所・組織名**など文脈依存のエンティティを拾うための任意の AI レイヤーがあります。

### 概要

- `openai/privacy-filter` ベースの量子化（**q4**）ONNX NER モデル — 8 層 MoE トークン分類器、33 種類の BIOES ラベル。
- 推論は ONNX Runtime + HuggingFace tokenizers で **100% オンデバイス**。テキストが端末の外に出ることはありません。
- モデルはインストーラーに**同梱されません** — 明示的な一度きりのダウンロードが必要です。

### 検出できるもの

| モデルエンティティ | マスクラベル | 例 |
|---|---|---|
| `private_person` | `<PERSON>` | 张伟、John Smith |
| `private_email` | `<EMAIL>` | zhang.wei@example.com |
| `private_phone` | `<PHONE>` | +86 138 1234 5678 |
| `private_address` | `<ADDRESS>` | 北京市朝陽区… |
| `account_number` | `<BANK_CARD>` | 6222 0212 3456 7890 |
| `private_date` | `<DATE>` | 個人的な日付 |
| `private_url` | `<URL>` | 個人的な URL |
| `secret` | `<API_KEY>` | トークン・シークレット |

### 有効化方法

**方法 A — ワンクリックダウンロード（推奨）**

1. **設定 → AI エンジン** を開く。
2. **ダウンロード** をクリック（約 550MB の zip。インストール中は約 2GB の空き容量が必要）。
3. 複数ミラーから自動フォールバックでダウンロードされ、SHA-256 検証・展開後に**ホットロード** — 再起動不要。

**方法 B — セルフサービスミラー**

サーバー帯域に限りがあるため、アプリ内ダウンロードが遅い場合は以下のいずれかのミラーから `privacy-filter.zip` を入手してください：

| ミラー | リンク | コード |
|---|---|---|
| HuggingFace | [privacy-filter.zip](https://huggingface.co/buckets/XiaoShengCYZ/AI_Models/resolve/privacy-filter.zip?download=true) | — |
| Quark 网盘 (夸克网盘) | [pan.quark.cn](https://pan.quark.cn/s/51647902f801?pwd=HQ1Y) | `HQ1Y` |
| Baidu 网盘 (百度网盘) | [pan.baidu.com](https://pan.baidu.com/s/1mDBr0mdo2r-guC4LshF87w?pwd=ba7b) | `ba7b` |

zip を `SafeMask.exe` と同じ階層の `models/privacy-filter/` に展開し、SafeMask を再起動してください。

**方法 C — 手動配置**

以下のファイルを `SafeMask.exe` と同じ階層の `models/privacy-filter/` に配置：

```text
models/privacy-filter/
├── model_q4.onnx         # 量子化モデル（エントリ）
├── model_q4.onnx_data    # 量子化ウェイト（約 875MB）
├── tokenizer.json        # HuggingFace トークナイザー
└── config.json           # id2label メタデータ
```

> 起動時には作業ディレクトリの `./models`、アプリのローカルデータディレクトリ、リソースディレクトリも順に検索されます。

### 動作の特徴

- **遅延ロード** — モデルは初回のマスク実行時にバックグラウンドでロード（通常 1〜3 分、タイムアウト 5 分）。状態と経過時間は設定画面に表示され、詳細は `ai_model_load.log` に記録されます。
- **ルール優先** — AI の優先度は 50、ルールは 90〜100。同一テキストでは決定論的なルール一致が常に優先されます。
- **ランタイム切替** — 設定からいつでも AI をオン/オフ可能。モデルファイルが存在すれば起動時に自動有効化。
- **グレースフルデグラデーション** — モデルがなくても他の機能は同一。ルールのみのモードに自動で戻ります。

### チューニング

| 項目 | 既定値 | 説明 |
|---|---|---|
| `ORT_NUM_THREADS` 環境変数 | `2` | ONNX Runtime の推論スレッド数 |
| 信頼度しきい値 | `0.5` | これ未満の AI スパンは破棄 |
| コンテキストウィンドウ | 512 トークン | 推論 1 回あたりの長さ |

---

## インストール

### ビルド済みインストーラー（推奨）

[**GitHub Releases**](https://github.com/AiToByte/SafeMask/releases) からダウンロード：

| パッケージ | 説明 |
|---|---|
| `SafeMask_x.y.z_x64-setup.exe` | Windows NSIS インストーラー — 推奨 |
| `SafeMask_x.y.z_x64_zh-CN.msi` | Windows MSI |
| macOS / Linux パッケージ | `v*` タグごとに CI が生成 |

AI モデルは任意で、アプリ内から別途ダウンロードします（[ローカル AI エンジン](#ローカル-ai-エンジンオンデバイス-ner)参照）。

---

## 使い方

### クリップボードのワークフロー

1. いつも通りコピー（`Ctrl+C`）— **Shadow モード**では原文がクリップボードに残ります。
2. 貼り付け先（ChatGPT、Claude、Web フォーム…）をフォーカス。
3. **`Alt+V`** を押す — SafeMask がバックアップ → マスク → 安全なテキストをペースト → 元のクリップボードを復元。
4. 厳格な運用が必要なら **`Alt+M`** で **Sentry モード** — コピーのたびにその場でサニタイズ。

| ショートカット | 動作 | 変更可否 |
|---|---|---|
| `Alt+V` | Magic Paste（マスクしてペースト） | ✅ キーとペースト遅延 |
| `Alt+M` | Shadow / Sentry 切替 | ❌（固定） |

### カスタムルール

**ルール**画面で追加するか、YAML をインポート：

```yaml
group: "MY_COMPANY"
rules:
  - name: "Internal_Project_Code"
    pattern: '\bPRJ-\d{6}\b'
    mask: "<PROJECT_CODE>"
    priority: 10
```

保存前に組み込みサンドボックスでライブテスト可能。完全なフォーマットは [docs/RULES_IMPORT.md](docs/RULES_IMPORT.md) を参照。

### マスクラッパースタイル

角括弧派の方は **設定 → ラッパースタイル** で `<TAG>` と `[TAG]` を即時切替 — 組み込み・カスタム・AI ラベルのすべてに反映されます。

### ファイル処理

ログファイルをダッシュボードにドラッグ（またはファイル選択）— メモリマップ + マルチコアのパイプラインでストリーム処理し、元ファイルの横にマスク済みコピーを出力。巨大ファイルでもメモリは安定。

---

## 開発

### 前提

- Node.js 18+
- Rust stable（edition 2024 ツールチェーン）
- [Tauri v2 のプラットフォーム依存](https://v2.tauri.app/start/prerequisites/)

### 実行

```bash
# JS 依存のインストール
npm install

# デスクトップアプリ（Vite + Tauri）
npm run tauri dev

# フロントエンドのみ（http://127.0.0.1:18924）
npm run dev
```

### ビルド

```bash
npm run build
npm run tauri build
```

### Rust チェック（ワークスペースルート）

```bash
cargo check  -p SafeMask
cargo test   -p SafeMask
cargo clippy -p SafeMask -- -D warnings
cargo fmt    -p SafeMask
```

### 環境変数

| 変数 | 既定値 | 用途 |
|---|---|---|
| `SAFEMASK_THREADS` | `2` | Rayon ワーカースレッド数（ファイル処理） |
| `ORT_NUM_THREADS` | `2` | ONNX Runtime 推論スレッド数 |

---

## アーキテクチャ（概要）

```
React 19 UI
    │  invoke / events
    ▼
api/*  (Tauri コマンド)
    ▼
orchestrator  (Shadow / Sentry)
    ▼
hybrid_engine  → recognizers → resolver → masking
    ▼
infra  (clipboard, mmap files, ONNX, config, records)
```

- `core/` は Tauri への依存が**ゼロ** — 単体テスト可能
- オフセットは UTF-8 の**バイトオフセット**
- カスタムルールはアプリの `custom/` ストレージ内 `user_rules.yaml`

詳細：[CLAUDE.md](CLAUDE.md) · [docs/](docs/) · [docs/RULES_IMPORT.md](docs/RULES_IMPORT.md) · [docs/THEMES.md](docs/THEMES.md)

---

## ドキュメント索引

| ドキュメント | 内容 |
|--------------|------|
| [README.md](README.md) | English |
| [README_CN.md](README_CN.md) | 简体中文 |
| [README_JA.md](README_JA.md) | 日本語（本ファイル） |
| [README_KO.md](README_KO.md) | 한국어 |
| [README_RU.md](README_RU.md) | Русский |
| [CLAUDE.md](CLAUDE.md) | アーキテクチャノート |
| [DEVELOPMENT.md](DEVELOPMENT.md) | 開発ガイド |
| [docs/使用手册.md](docs/使用手册.md) | ユーザーハンドブック（中国語） |
| [docs/RULES_IMPORT.md](docs/RULES_IMPORT.md) | ルールのインポート/エクスポート |
| [docs/THEMES.md](docs/THEMES.md) | テーマシステム |
| [docs/record-writer.md](docs/record-writer.md) | 監査レコードライター |
| [GitHub Releases](https://github.com/AiToByte/SafeMask/releases) | ビルド済みインストーラー |
| [Issues](https://github.com/AiToByte/SafeMask/issues) | バグ報告・機能要望 |
| [LICENSE](LICENSE) | MIT |

---

## プライバシーとセキュリティ

- マスキング内容のクラウド送信は**一切なし**
- **AI 推論は 100% オンデバイス**。唯一のネットワーク通信は、任意かつ明示的な一度きりのモデルダウンロード（許可リスト済みミラーのみ）
- モデルダウンロードはユーザー起点の任意操作
- 監査ログは既定オフ（有効化すると平文の原文がローカルに保存されます）
- 信頼できないルールファイルのインポートに注意

---

## よくある質問

**クリップボードやファイルが外部に送られることはありますか？**
ありません。ルール・AI を問わず、すべてのマスキングはローカルで実行されます。テレメトリもコンテンツのアップロードもありません。

**AI モデルなしでも使えますか？**
使えます。ルールエンジン（辞書 + 正規表現）だけで完全に機能します。AI は人名・住所などの文脈依存エンティティを補強するレイヤーです。

**初回の AI マスクが遅いのはなぜ？**
約 900MB のモデルが初回使用時に遅延ロードされるためです（通常 1〜3 分）。2 回目以降は即時です。進行状況は 設定 → AI エンジン に表示されます。

**AI モデルはどこに保存されますか？**
実行ファイルの隣の `models/privacy-filter/`（またはアプリのローカルデータ / リソースディレクトリ）。フォルダごと削除すれば完全に除去できます。

**独自の検出ルールを使えますか？**
はい。ライブサンドボックス付きのカスタム YAML ルールと、複数ファイルのインポート/エクスポートを用意しています。[docs/RULES_IMPORT.md](docs/RULES_IMPORT.md) を参照してください。

---

## リリース

ビルド済みインストーラー：[GitHub Releases](https://github.com/AiToByte/SafeMask/releases)

`v*` タグで CI がマルチプラットフォームパッケージ（Windows / macOS / Linux）を生成します。

---

## コントリビュート

1. 小さく焦点を絞った PR を推奨
2. プッシュ前に `cargo test -p SafeMask` と `npm run build` を実行
3. 既存の構成に従う：`core/` は純粋ロジック、`infra/` は OS、`api/` は IPC
4. 新しいテーマ：[docs/THEMES.md](docs/THEMES.md) を参照
5. 新しいルールパック：`RuleGroup` 互換またはルール配列のみの YAML

Issue・PR 歓迎：[AiToByte/SafeMask](https://github.com/AiToByte/SafeMask)

---

## ライセンス

[MIT](LICENSE) © SafeMask / AiToByte

---

<div align="center">

[English](README.md) · [简体中文](README_CN.md) · **日本語** · [한국어](README_KO.md) · [Русский](README_RU.md)

</div>
