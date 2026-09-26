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
- 設定画面からの使用モデル切り替え（選択は端末内に保存）と、画面上部バーでの状態表示
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
rustc --version  # 1.82+
bun --version    # 1.x
```

Bun は `curl -fsSL https://bun.sh/install | bash` でインストールできます。

### 3) モバイル（オプション）

- **iOS**: Xcode 26+ / `rustup target add aarch64-apple-ios aarch64-apple-ios-sim` / `brew install cocoapods xcodegen libimobiledevice`
- **Android**: Android Studio + SDK + NDK + `cargo install cargo-ndk`

iOS の最小バージョンは 26.0 です。署名は開発者ごとに違うためリポジトリには含めず、Team ID を渡して初期化します。

```bash
export APPLE_DEVELOPMENT_TEAM=XXXXXXXXXX   # 署名証明書の OU フィールド
bun run ios:init                           # Xcode プロジェクトを生成
bun run ios:dev                            # 実機で起動（Mac の Vite 開発サーバーに接続）
bun run ios:build                          # 開発サーバー不要の単体 IPA（要 rustup component add llvm-tools）
```

`ios:build` の IPA はフロントエンドを同梱するため、Mac なしで iPhone / iPad 上で動きます。インストール方法は `docs/ios-build.md` の「Standalone install」を参照してください。

生成される Xcode プロジェクト（`src-tauri/gen/apple/`）と署名設定（`src-tauri/tauri.ios.conf.json`）は追跡対象外です。初回起動時は iOS 側で「設定 → 一般 → VPNとデバイス管理 → デベロッパAPP」から信頼が必要です。

手順と背景は `docs/ios-build.md` と `docs/android-build.md` を参照してください。CI でビルドしているのはデスクトップ 3 プラットフォームのみです。iOSのCoreML推論に加え、Gemma 4 E2BのCPU推論とMacとの教室通信を物理iPhone 17 / iOS 27で確認しています。対象と限界は[追加検証記録](docs/ios-inference-network-ux.md)を参照してください。

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

`bun run tauri dev` は Vite 開発サーバーと Rust バックエンドを同時に起動します。フロントエンドだけ確認したい場合は `bun run dev` で `http://localhost:1420` を開いてください。通常のブラウザではネイティブAPIを使えないため、モデル管理・実推論・教室通信の検証にはTauriアプリを使ってください。

### モデルの取得

アプリ上部バーのモデル表示から「設定」を開いてダウンロードできるほか、CLI でも取得できます。

```bash
bun run download:model                      # Gemma 3 1B INT4（既定）
bun run download:model --variant 1b-int8
bun run download:model --variant 3n-e2b-int4
bun run download:model --variant 4-e2b-int4  # Gemma 4 E2B INT4
bun run download:model --variant 4-e4b-int4  # Gemma 4 E4B INT4
```

導入済みのモデルは設定画面の「使用する」で切り替えます。選択は端末内に保存され、次回起動時も引き継がれます。

モデルは役割をまたいで共有される（先生のチャットや授業案と、生徒に出るヒントは同じモデルで動く）ため、設定は先生・生徒いずれの画面からも、役割を選ぶ前からも開けます。使用中のモデル名・利用可否・メモリへの読み込み進捗は常に上部バーに出ます。

| バリアント | ダウンロード量 | 推奨メモリ |
| --- | --- | --- |
| `1b-int4` | 約 0.8 GB | 8 GB 以上 |
| `1b-int8` | 約 1.0 GB | 8 GB 以上 |
| `3n-e2b-int4` | 約 3.1 GB | 8 GB 以上 |
| `4-e2b-int4` | 約 3.4 GB | 8 GB 以上 |
| `4-e4b-int4` | 約 5.2 GB | 12 GB 以上 |

推奨メモリは端末の物理メモリで、モデルの占有量そのものではありません（OS とアプリの分を含みます）。

E シリーズの「E」は実効パラメータ数を指し、Per-Layer Embeddings（各デコーダ層が語ごとに小さな埋め込みを参照する仕組み）によって実効数が総数より小さくなります。Gemma 4 E2B は実効 2.3B / 埋め込み込み 5.1B、E4B は実効 4.5B / 8B です。Google が公開している 4bit（Q4_0）の推論メモリは **重みのみ** で E2B 2.9 GB、E4B 4.5 GB（コンテキスト長ぶんは別途）です。

ただし本リポジトリの ONNX INT4 ビルドはこれより大きくなります。PLE のテーブルが `embed_tokens` グラフとして書き出されており、デコーダと合わせて 2 グラフを同時にメモリへ載せるため、ディスク上の重みがほぼそのまま常駐するためです。Apple M5 / 16 GB・ONNX Runtime WebGPU (Metal)・64 トークン生成でのピークメモリ実測は E2B 3.7 GB、E4B 5.6 GB でした（macOS の peak memory footprint、debug ビルド）。RSS は外部データファイルを mmap する都合で実行ごとに大きく振れるため、判断には使っていません。上表の推奨メモリはこの実測値に OS とアプリの分を加えたものです。

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

推論バックエンドは Rust `ort`（ONNX Runtime）で Gemma を端末内実行します。対応モデルは Gemma 3 1B（INT4 / INT8）、Gemma 3n E2B INT4、Gemma 4 E2B・E4B INT4 で、どれを使うかは設定画面から選べます。モデルは Hugging Face の `onnx-community` から取得し、全ファイルの SHA-256 を検証したうえでアプリデータディレクトリへ配置します。

対応モデルの一覧は `src-tauri/src/inference/catalog.rs` が単一の情報源です。ダウンロード仕様・配置先・推論の接続はすべてこの定義から導出されるため、モデルの追加はここへ 1 エントリ足すだけで済みます。

Gemma 3 1B は `input_ids` を直接受け取る単一グラフですが、Gemma 3n と Gemma 4 は `embed_tokens` グラフが出力する `inputs_embeds` と `per_layer_inputs` をデコーダへ渡す 2 段構成です。KV キャッシュの層数とヘッド次元はモデルごとに異なる（Gemma 4 は共有 KV 層が露出せず、スライディング窓層だけ次元が倍）ため、定数ではなく ONNX グラフの入力定義から実行時に読み取ります。

チャットテンプレートはファミリーごとに異なり、互換性はありません。Gemma 3 / 3n は `<start_of_turn>` / `<end_of_turn>`、Gemma 4 は `<|turn>` / `<turn|>` を使います。誤ったほうを渡すとマーカーが特殊トークンではなく通常の文字列として扱われ、モデルが応答にマーカーをそのまま書き出します。

先頭の `<bos>` はプロンプト文字列側で付け、トークナイザーの特殊トークン付与は無効にしています。Gemma 3 の `tokenizer.json` は `<bos>` を自動で前置しますが Gemma 4 はしないため、トークナイザー任せにすると Gemma 3 で `<bos>` が二重になり、Gemma 4 では付きません。

Gemma 4 は空の KV キャッシュと 0 始まりの位置で推論します。マスクしたダミーの 1 スロットを前置すると、実モデルで語や記号の重複が生じることを確認したため、Gemma 4 には付けません。Gemma 3 / 3n の既存経路では CoreML のゼロ要素テンソル制限を避けるため、マスク済みスロットを維持しています。デスクトップのGemma 4は標準のONNX Runtime WebGPU EPを使い、Apple SiliconではMetalで実行します。iOSのGemma 4はCoreMLで扱えない演算・空キャッシュを含むため、同じONNX RuntimeのCPU経路をモデル別の標準設定として使用します。モデル一覧にもCPU実行と待ち時間の注意を表示します。実行プロバイダを環境変数で明示した場合はその指定を優先し、失敗時の自動切替は行いません。

### 実機での確認

以下は旧 CoreML 実装での参考計測です。Gemma 4 ではその後、連続推論のクラッシュとダミーキャッシュによる出力の重複が見つかったため、現行実装の速度・回答品質の検証結果としては扱わないでください。現行の GPU 検証は `docs/reviews/2026-09-12-onnx-accelerator-audit.md` を参照してください。

| モデル | 生成 | 所要（モデル読み込み含む） | 端末 |
| --- | --- | --- | --- |
| Gemma 3 1B INT4 | 8 トークン | 6.7 秒 | M5 / 16 GB |
| Gemma 4 E2B INT4 | 8 トークン | 13.9 秒 | M5 / 16 GB |
| Gemma 4 E2B INT4 | 64 トークン | 28.5 秒 | M5 / 16 GB |
| Gemma 4 E4B INT4 | 8 トークン | 16.8 秒 | M4 Max / 64 GB |
| Gemma 4 E4B INT4 | 64 トークン | 37.2 秒 | M4 Max / 64 GB |

実推論のスモークテストは `#[ignore]` 付きで、モデルを配置したうえで次のように実行します。ONNX セッションは同時に生成できないため、実推論テストは `--test-threads=1` で直列実行してください。

```bash
bun run download:model --variant 4-e4b-int4
cargo test --manifest-path src-tauri/Cargo.toml --release \
  gemma4_e4b_long_generation_smoke -- --ignored --nocapture --test-threads=1
```

会話・チャットまわりの実推論テストは次のとおりです。

| テスト | 確認内容 |
| --- | --- |
| `multi_turn_conversation_uses_the_previous_turns` | 直前のターンを参照しないと答えられない質問に正答するか（履歴なしでは正答不能であることも対照確認） |
| `conversation_streaming_matches_the_returned_text` | ストリーミングで届く断片の連結が最終テキストと一致するか |
| `teacher_chat_follow_up_continues_the_conversation` | 教師チャットが集計に基づいて答え、続きの質問で前の質問に答え直さないか |
| `prompt_processing_is_reported_before_the_first_token` | 最初のトークンが出るまでの待ち（プロンプトのキャッシュ読み込み）が、順序どおりに進捗として報告されるか |
| `lesson_plan_reports_the_stages_it_reaches` | レッスンプラン生成が、プロンプト読み込み・生成・検証・作り直しの各段階を実際に報告するか |
| `teacher_chat_transcript_probe` | 実際の応答を出力するだけの確認用（フロントの描画テスト用フィクスチャ採取元） |

```bash
cargo test --manifest-path src-tauri/Cargo.toml \
  multi_turn_conversation_uses_the_previous_turns -- --ignored --nocapture --test-threads=1
```

会話・ストリーミングの2件は導入済みの全バリアント（Gemma 3 1B / Gemma 4 E2B / E4B）を、教師チャットの2件は Gemma 4 系を対象にします。未導入のものは自動的にスキップされます。

採取した実応答は `src/components/__tests__/realReplies.ts` に置き、`MarkdownRealReplies.test.tsx` が実際の Markdown 描画結果（入れ子箇条書きや、モデルごとに異なる箇条書き記号の幅を含む）を検証します。

デスクトップの標準構成は単一の ONNX Runtime とネイティブ WebGPU EP です。Dawn が macOS では Metal、Windows では Direct3D 12、Linux では Vulkan を利用します。Linux x86_64 では CUDA EP も同梱し、未指定時は CUDA、WebGPU、CPU の順にセッションを作成します。選択結果と GPU で初期化できなかった理由は設定画面に表示します。形状計算など未対応の補助演算は ORT の CPU ノードで実行されます。

```bash
bun run prepare:runtime  # 初回に公式バイナリを取得・SHA-256検証（Python 3 はビルド時のみ必要）
bun run tauri dev       # prepare:runtime は dev/build の前にも自動実行
```

同梱版は ONNX Runtime 1.30.0 / WebGPU EP 0.3.0。対象は macOS 14+ ARM64、Linux x86_64 (glibc 2.28+)、Windows x86_64 / ARM64。WSL2 で CUDA を使う場合は、Windows 側 NVIDIA ドライバに加え CUDA 13 と cuDNN 9 の Linux ランタイム（`onnxruntime-gpu[cuda,cudnn]` と同じ組合せ）が必要です。Apple Siliconで実推論確認済み、Windows/Linuxは実機未検証です。署名配布では同梱ネイティブライブラリも署名対象になります。

Ubuntu 24.04 系 WSL2 では NVIDIA CUDA リポジトリを有効にしたうえで、次を一度実行します。

```bash
sudo apt-get install cuda-cudart-13-0 libcublas-13-0 libcudnn9-cuda-13
```

`LUMIN_EXECUTION_PROVIDER=cuda` は CUDA のみを使う診断用指定です。通常は未指定のまま起動し、設定画面で実際に初期化できたプロバイダとフォールバック理由を確認してください。

Linux/WSL2 の WebGPU (Vulkan) と CUDA の診断は次で採取できます。GPU、Vulkan ICD、同梱 ORT ライブラリと不足している CUDA 依存ライブラリだけを出力します。

```bash
bun run diagnose:gpu
```

必要に応じ `LUMIN_EXECUTION_PROVIDER=cuda|tensorrt|directml|coreml|nnapi|xnnpack|cpu` と対応 Cargo feature を明示できます。CUDA 等では対応版 ORT を `ORT_DYLIB_PATH` で指定してください。同梱の標準版は WebGPU 用です。`LUMIN_WEBGPU_LIBRARY` はプラグインの差し替え、`LUMIN_ORT_PROFILE_DIR` は演算配置検証用です。

Gemma にはインターネット通信機能はなく、推論は端末の CPU / GPU 実行プロバイダで完結します。生成は greedy decoding で行います。構造化出力はプロンプトで JSON を指定し、生成後に形式・文字数・候補ラベルを検証します（生成中の JSON Schema 制約ではありません）。途中終了した出力と正答が漏れるヒントは採用せず、採点・ヒント・授業案はルールや教材へ戻します。教師チャットは推論失敗・途中終了をエラーとして表示します。

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
│   ├── components/           # 教師 / 生徒 / 設定・モデル管理 UI と *.css.ts
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
