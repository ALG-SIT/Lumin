# Lumin

> 理解を照らし、次の学びにつなげる。

Lumin は、生徒の解答を端末内で分析して段階的なヒントを返し、解答本文を送らずにクラス全体の誤概念を教師へ共有する、ローカルファースト型の学習支援アプリです。

このリポジトリは `doc/AI_Innovators_Cup_Lumin_構想.md` に基づく大会用 MVP です。Tauri 2 + Rust + React（Bun / Vite）で作られており、1 つのアプリを教師モード・生徒モードに切り替えて利用します。同じローカルネットワーク上の端末同士が DNS-SD（mDNS）で互いを発見し、HTTP で教室セッションを構成します。

Tauri へのリライトは PR #1 で `main` にマージ済みで、`main` が現在の開発ベースです。

## 技術スタック

| レイヤー | 技術 | 役割 |
| --- | --- | --- |
| Rust | Tauri 2 | デスクトップ / モバイル向け Rust バックエンド |
| Rust | `serde`, `anyhow`, `thiserror`, `tokio` | IPC、エラー処理、非同期処理 |
| Rust | `axum`, `tauri-plugin-dns-sd`, `local-ip-address` | 教室内 HTTP サーバーと mDNS 探索 |
| Rust | `ort` 2.0 (ONNX Runtime), `tokenizers`, `ndarray` | 端末内推論 |
| Rust | `reqwest`, `sha2` | モデルのダウンロードと整合性検証 |
| JS ランタイム | Bun | パッケージマネージャ兼実行環境 |
| フロントエンド | React 19 + Vite 8 + TypeScript 5 | UI とビルド |
| スタイル | vanilla-extract | 型付き CSS（`*.css.ts`、ランタイム CSS-in-JS なし） |
| 品質 | Biome 2.5, Vitest 4, Testing Library | Lint / フォーマット / フロントエンドテスト |
| Tauri JS | `@tauri-apps/api`, `@tauri-apps/plugin-dialog`, `@momics/dns-sd-tauri` | `invoke` / `listen` / プラットフォームコマンド |

## 実装状況

以下は `main` に入っている機能です。

**教材と分析（`lumin_core`）**

- 数学・国語・理科・社会・英語、6 セット 25 問のデモ教材バンク
- 端末内の正誤判定と誤概念分類（全角・半角などを吸収する正規化つき）
- 正解を直接出さない 3 段階ヒント
- ヒント後の再回答成功記録
- 教師向け正答率・再挑戦成功率・平均ヒント数・誤概念集計
- 集計から生成する「次回授業の冒頭 10 分案」と、教師による編集・採用

**教室内通信（`network`）**

- 教師端末が `axum` の HTTP サーバーを起動し、生徒端末は `_lumin-class._tcp` を mDNS で発見して参加
- 4 桁の参加コードによる照合（コードは探索情報には載せない）
- 授業 ID による別授業データの混入防止
- 解答本文を含まない分析イベントのみの送信（`AnalysisEvent` は解答本文を保持できない型）
- 切断中の分析結果を端末内キューへ保持し、再接続時に受領確認つきで自動再送・重複排除
- 教師側からの生徒の切断（kick）とセッション終了通知

**オンデバイス AI（`inference` / `ai`）**

- `ort` による ONNX Runtime セッション管理と、Gemma 3 1B / Gemma 3n E2B / Gemma 4 E2B・E4B の生成
- モデル管理画面からの使用モデル切り替え（選択は端末内に保存）
- アプリ内からのモデルダウンロード（進捗イベント・中断・SHA-256 検証）とローカルモデルの取り込み
- 正答漏洩ガードと候補外の誤概念ラベルの拒否、失敗時のルールベースへのフォールバック
- 実行プロバイダとベンチマーク結果、システム情報の表示

**永続化（`persistence`）**

- 終了した授業の匿名集計・採用案を端末内へ最大 100 件保存する授業履歴
- 進行中セッションの復旧用スナップショットと、未送信イベントのキュー（いずれもアトミック書き込み）

**デモ**

- 回答が集まる前でも試せる大会デモデータと 3 分デモフロー

## 前提条件

### 1) システム

Tauri 2 の公式前提を満たしていることを確認してください。

- **macOS / Windows / Linux**: WebKit / WebView2 ランタイム
- **Linux (Ubuntu / WSL)**: `libwebkit2gtk-4.1-dev`, `build-essential`, `libssl-dev`, `pkg-config`, `libgtk-3-dev`, `librsvg2-dev` など

### 2) Rust / Bun

```bash
rustc --version  # 1.77+
bun --version    # 1.x
```

Bun は `curl -fsSL https://bun.sh/install | bash` でインストールできます。

### 3) モバイル（オプション）

- **iOS**: Xcode + `rustup target add aarch64-apple-ios aarch64-apple-ios-sim`
- **Android**: Android Studio + SDK + NDK + `cargo install cargo-ndk`

手順は `docs/ios-build.md` と `docs/android-build.md` を参照してください。CI でビルドしているのはデスクトップ 3 プラットフォームのみで、モバイルビルドは引き続き検証中です。

## 開発の始め方

### インストール

```bash
bun install
```

### 開発サーバー

```bash
# デスクトップ（推奨）
bun run tauri dev

# iOS（macOS + Xcode が必要）
bun run tauri ios dev

# Android（SDK + NDK が必要）
bun run tauri android dev
```

`bun run tauri dev` は Vite 開発サーバーと Rust バックエンドを同時に起動します。フロントエンドだけ確認したい場合は `bun run dev` で `http://localhost:1420` を開いてください。Tauri の外で開いた場合は `isTauriEnvironment()` による判定で IPC を伴う操作が無効化され、画面が固まらないようになっています。

### モデルの取得

アプリの「モデル管理」画面からダウンロードできるほか、CLI でも取得できます。

```bash
bun run download:model                      # Gemma 3 1B INT4（既定）
bun run download:model --variant 1b-int8
bun run download:model --variant 3n-e2b-int4
bun run download:model --variant 4-e2b-int4  # Gemma 4 E2B INT4
bun run download:model --variant 4-e4b-int4  # Gemma 4 E4B INT4
```

導入済みのモデルは「モデル管理」画面の「使用する」で切り替えます。選択は端末内に保存され、次回起動時も引き継がれます。

| バリアント | ダウンロード量 | 推奨メモリ |
| --- | --- | --- |
| `1b-int4` | 約 0.8 GB | 8 GB 以上 |
| `1b-int8` | 約 1.0 GB | 8 GB 以上 |
| `3n-e2b-int4` | 約 3.1 GB | 16 GB 以上 |
| `4-e2b-int4` | 約 3.4 GB | 16 GB 以上 |
| `4-e4b-int4` | 約 5.2 GB | 24 GB 以上 |

Gemma 3n / Gemma 4 はモデルごとのサブディレクトリ（例 `models/gemma-4-e2b-int4/`）へ配置されます。これらは外部データファイル名が共通のため、同じディレクトリに置くと互いを上書きしてしまうためです。

### ビルド

```bash
bun run build && bun run tauri build
```

`bun run build` は TypeScript と Vite のビルドを実行します。`bun run tauri build` はそれを組み込んだアプリバンドルを `src-tauri/target/release/bundle` に出力します。

### テスト・Lint

```bash
bun run test:run   # Vitest（フロントエンド）
bun run check      # Biome（lint + format チェック）
bun run check:fix  # Biome の自動修正
bun run build      # 型チェック + フロントエンドビルド

cargo test --manifest-path src-tauri/Cargo.toml   # Rust 単体テスト
```

## アーキテクチャの概要

React フロントエンドは Tauri の `invoke` で Rust コマンドを呼び出し、ダウンロード進捗や教室の発見は Tauri イベントで受け取ります。Rust バックエンドは `lumin_core`（教材・分析・集計）、`inference` / `ai`（`ort` による Gemma 推論と安全ガード）、`network`（教室内 HTTP / mDNS 通信）、`persistence`（履歴・キュー）に分かれています。教師端末が HTTP サーバーを立てて `_lumin-class._tcp` を広告し、生徒端末は mDNS で教室を発見して参加コード付きで接続します。分析イベントは端末内で生成され、解答本文を含まずに教師端末へ送信されます。詳細は `docs/architecture/01-overview.md` を参照してください。

## プライバシー境界

生徒端末にだけ残るもの：

- 入力した解答全文
- 回答途中の状態
- 表示したヒント

教師端末へ送るもの：

- セッション内だけの匿名参加トークン
- 問題 ID と概念
- 誤概念ラベル
- 初回正誤、ヒント回数、再回答結果

通信終了時はローカルネットワークセッションを切断します。通信が一時的に切れた場合、未送信の `AnalysisEvent` は生徒端末内だけに保持されます。再接続後に再送し、教師端末からイベント ID の受領確認が届いた時点で削除します。教師側の授業履歴も端末内保存で、UI から個別に削除できます。

外部へ出る通信は、モデルファイルを取得するときの Hugging Face へのダウンロードだけです（`tauri.conf.json` の CSP でも `connect-src` を `https://huggingface.co` に限定しています）。

## オンデバイス AI

推論バックエンドは Rust `ort`（ONNX Runtime）で Gemma を端末内実行します。対応モデルは Gemma 3 1B（INT4 / INT8）、Gemma 3n E2B INT4、Gemma 4 E2B・E4B INT4 で、どれを使うかはモデル管理画面から選べます。モデルは Hugging Face の `onnx-community` から取得し、全ファイルの SHA-256 を検証したうえでアプリデータディレクトリへ配置します。

対応モデルの一覧は `src-tauri/src/inference/catalog.rs` が単一の情報源です。ダウンロード仕様・配置先・推論の接続はすべてこの定義から導出されるため、モデルの追加はここへ 1 エントリ足すだけで済みます。

Gemma 3 1B は `input_ids` を直接受け取る単一グラフですが、Gemma 3n と Gemma 4 は `embed_tokens` グラフが出力する `inputs_embeds` と `per_layer_inputs` をデコーダへ渡す 2 段構成です。KV キャッシュの層数とヘッド次元はモデルごとに異なる（Gemma 4 は共有 KV 層が露出せず、スライディング窓層だけ次元が倍）ため、定数ではなく ONNX グラフの入力定義から実行時に読み取ります。

チャットテンプレートはファミリーごとに異なり、互換性はありません。Gemma 3 / 3n は `<start_of_turn>` / `<end_of_turn>`、Gemma 4 は `<|turn>` / `<turn|>` を使います。誤ったほうを渡すとマーカーが特殊トークンではなく通常の文字列として扱われ、モデルが応答にマーカーをそのまま書き出します。

KV キャッシュの先頭には、常にマスクされるゼロ埋めの 1 スロットを置いています。初回ステップでキャッシュが空だと `past_key_values.*` が要素数 0 のテンソルになり、ONNX Runtime の CoreML プロバイダがこれを拒否するためです（マスク済みのキーは softmax 後に寄与しないので出力は変わりません）。

### 実機での確認

Apple M5 / 16 GB / macOS（CoreML）で実推論を確認しています。

| モデル | 生成 | 所要（モデル読み込み含む） |
| --- | --- | --- |
| Gemma 3 1B INT4 | 8 トークン | 6.7 秒 |
| Gemma 4 E2B INT4 | 8 トークン | 13.9 秒 |
| Gemma 4 E2B INT4 | 64 トークン | 28.5 秒 |

Gemma 4 E4B は未検証です（16 GB 機では推奨メモリに届かないため）。実推論のスモークテストは `#[ignore]` 付きで、モデルを配置したうえで次のように実行します。

```bash
bun run download:model --variant 4-e2b-int4
cargo test --manifest-path src-tauri/Cargo.toml --release \
  gemma4_e2b_long_generation_smoke -- --ignored --nocapture
```

実行プロバイダは Cargo feature で切り替えます。Apple Silicon の macOS ビルドでは CoreML が既定で有効で、未対応ノードは ONNX Runtime の CPU プロバイダへフォールバックします。

```bash
cargo build --manifest-path src-tauri/Cargo.toml --features cuda      # NVIDIA
cargo build --manifest-path src-tauri/Cargo.toml --features directml  # Windows
cargo build --manifest-path src-tauri/Cargo.toml --features xnnpack   # CPU 最適化
```

Gemma にはインターネット通信機能はなく、推論は端末の CPU / GPU 実行プロバイダで完結します。生成は JSON Schema 制約つきで行い、正答漏洩チェックと候補外ラベルの拒否を通し、不正な場合はルールベースの安全な出力へフォールバックします。

## プロジェクト構成

```
.
├── package.json              # bun scripts: dev / build / test / check / download:model / tauri
├── biome.json                # Biome 2.5 の lint / format 設定
├── vite.config.ts            # ポート 1420、vanilla-extract、Tauri 開発用 HMR 設定
├── vitest.config.ts
├── Cargo.toml                # src-tauri を含む Cargo ワークスペース
├── src/
│   ├── App.tsx               # 役割選択とモード切り替え
│   ├── components/           # 教師 / 生徒 / モデル管理 UI と *.css.ts
│   │   └── __tests__/        # Vitest + Testing Library
│   ├── styles/               # vanilla-extract のトークンと共有スタイル
│   └── lib/tauri.ts          # Tauri ホスト内かどうかの実行時判定
├── src-tauri/
│   ├── Cargo.toml            # Tauri、axum、ort、tokenizers など
│   ├── tauri.conf.json       # productName、identifier、CSP、bundle 設定
│   ├── capabilities/default.json
│   └── src/
│       ├── lib.rs            # Tauri コマンド登録とアプリセットアップ
│       ├── lumin_core/       # 教材・分析・集計・デモデータ
│       ├── inference/        # ort セッション、生成、トークナイザ、DL、ベンチ
│       ├── ai/               # 構造化出力スキーマと正答漏洩ガード
│       ├── network/          # HTTP サーバー、DNS-SD、参加コード認証、再送
│       ├── session.rs        # 教室セッションのコマンド
│       └── persistence.rs    # 履歴・スナップショット・保留キュー
├── scripts/                  # モデルダウンロードスクリプト
├── docs/                     # アーキテクチャ、スパイク、モバイルビルド手順
├── doc/                      # 大会構想メモ（旧 Swift 版の実装メモを含む）
└── .github/workflows/ci.yml  # frontend / rust / tauri build マトリクス
```

## 開発ワークフロー

`main` への直接 push は行わず、機能ごとにブランチを切って PR 経由でマージします。CI は macOS / Windows / Linux で `bun run build`、`cargo check` / `clippy` / `fmt`、および Tauri のバンドルビルドを実行します。ローカルでは以下を通してから PR を出してください。

```bash
cargo check --manifest-path src-tauri/Cargo.toml
cargo clippy --manifest-path src-tauri/Cargo.toml -- -D warnings
cargo fmt --manifest-path src-tauri/Cargo.toml -- --check
bun run check
bun run test:run
bun run build
```

ブランチ命名、コミット規約、PR の進め方は `CONTRIBUTING.md` を参照してください。

## ライセンス

大会用 MVP。利用モデルのライセンスは各モデル提供者に従います。
