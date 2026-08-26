# Lumin

> 理解を照らし、次の学びにつなげる。

Lumin は、生徒の解答を端末内で分析して段階的なヒントを返し、解答本文を送らずにクラス全体の誤概念を教師へ共有する、ローカルファースト型の学習支援アプリです。

このリポジトリは `doc/AI_Innovators_Cup_Lumin_構想.md` に基づく大会用 MVP です。Tauri + Rust + React（Bun / Vite）で作られており、1 つのアプリを教師モード・生徒モードに切り替えて利用します。同じローカルネットワーク上の端末同士で HTTP / mDNS を使って教室セッションを構成します。

## 技術スタック

| レイヤー | 技術 | 役割 |
| --- | --- | --- |
| Rust | Tauri 2 | デスクトップ / モバイル向け Rust バックエンド |
| Rust | `serde`, `anyhow`, `thiserror`, `tokio` | IPC、エラー処理、非同期処理 |
| JS ランタイム | Bun | パッケージマネージャ兼実行環境 |
| フロントエンド | React 19 + Vite 8 + TypeScript 5 | UI とビルド |
| Tauri JS | `@tauri-apps/api`, `@tauri-apps/cli` | `invoke` / `listen` / プラットフォームコマンド |
| 推論（予定） | Rust `ort` + Gemma 3 1B INT4 | 端末内 ONNX Runtime 推論 |

## 現在の実装状態

このブランチは **Tauri スキャフォールド** 段階です。以下は既存の Lumin 設計から移植した機能リストで、順次 Rust / React へ実装予定です。

- 数学・国語・理科・社会・英語、6 セット 25 問の選択式教材バンク
- 端末内の正誤判定と誤概念分類
- 正解を直接出さない 3 段階ヒント
- ヒント後の再回答成功記録
- ローカルネットワークによる教室内通信
- 4 桁の参加コードによる教室セッションの照合
- 授業 ID による別授業データの混入防止
- 切断中の分析結果を端末内キューへ保持し、再接続時に受領確認つきで自動再送
- 解答本文を含まない分析イベントのみの送信
- 教師向け正答率・再挑戦成功率・平均ヒント数・誤概念集計
- 回答が集まる前でも試せる大会デモデータ
- 集計から生成する「次回授業の冒頭 10 分案」
- 教師による授業案の編集・採用
- 終了した授業の匿名集計・採用案を端末内へ最大 100 件保存する授業履歴

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

モバイルビルド手順は今後検証予定です。現時点ではデスクトップ開発が推奨です。

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

`bun run tauri dev` は Vite 開発サーバーと Rust バックエンドを同時に起動します。フロントエンドだけ確認したい場合は `bun run dev` で `http://localhost:1420` を開いてください。

### ビルド

```bash
bun run build && bun run tauri build
```

`bun run build` は TypeScript と Vite のビルドを実行します。`bun run tauri build` はそれを組み込んだアプリバンドルを `src-tauri/target/release/bundle` に出力します。

### テスト

```bash
# Rust 側の単体テスト
cargo test --manifest-path src-tauri/Cargo.toml

# 型チェックとフロントエンドビルド
bun run build
```

現時点では本格的なテストスイートは整備中です。`cargo test` はスキャフォールドに含まれる最小限のテストを実行します。

## アーキテクチャの概要

React フロントエンドは Tauri の `invoke` で Rust コマンドを呼び出します。Rust バックエンドは `lumin_core`（教材・分析・集計）、`inference`（`ort` による Gemma 推論）、`network`（教室内 HTTP / mDNS 通信）の 3 つの領域に分けて実装します。教師端末が小さな HTTP サーバーを立て、生徒端末は mDNS で教室を発見して参加コード付きで接続します。分析イベントは端末内で生成され、解答本文を含まずに教師端末へ送信されます。詳細は `docs/architecture/01-overview.md` を参照してください。

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

`AnalysisEvent` は解答本文を保持できない型として定義します。通信終了時はローカルネットワークセッションを切断します。通信が一時的に切れた場合、未送信の `AnalysisEvent` は生徒端末内だけに保持されます。再接続後に再送し、教師端末からイベント ID の受領確認が届いた時点で削除します。教師側の授業履歴も端末内保存で、UI から個別に削除できます。

## オンデバイス AI

推論バックエンドは Rust `ort`（ONNX Runtime）を使い、Gemma 3 1B INT4 を端末内で実行する方向で整備中です。モデルは Hugging Face からダウンロードし、アプリデータディレクトリへ配置します。現時点では推論モジュールはスキャフォールドに含まれておらず、今後 `src-tauri/src/inference/` 以下に追加予定です。Gemma にはインターネット通信機能はなく、推論は端末 CPU / GPU 実行プロバイダで完結します。

## プロジェクト構成

```
.
├── package.json              # bun scripts: dev / build / preview / tauri
├── vite.config.ts            # ポート 1420、Tauri 開発用 HMR 設定
├── tsconfig.json
├── index.html
├── src/
│   ├── App.tsx               # React ルートコンポーネント（現在はスキャフォールド UI）
│   ├── main.tsx
│   └── assets/
├── src-tauri/
│   ├── Cargo.toml            # lumin、Tauri、tokio など
│   ├── tauri.conf.json       # productName、identifier、build 設定
│   ├── build.rs
│   ├── capabilities/default.json
│   └── src/
│       └── lib.rs            # Tauri コマンドとアプリセットアップ
├── scripts/                  # 今後ダウンロード・診断スクリプトを配置
├── docs/
│   └── architecture/         # アーキテクチャ文書
└── dist/                     # Vite ビルド出力
```

## 開発ワークフロー

機能ごとにブランチを切り、PR 経由でマージします。`src-tauri/` に変更を加えた後は以下を実行してください。

```bash
cargo check --manifest-path src-tauri/Cargo.toml
cargo clippy --manifest-path src-tauri/Cargo.toml -- -D warnings
cargo fmt --manifest-path src-tauri/Cargo.toml -- --check
bun run build
```

詳細は `CONTRIBUTING.md`（今後作成予定）を参照してください。

## ライセンス

大会用 MVP。利用モデルのライセンスは各モデル提供者に従います。
