# Ubuntu CI の RPM 作成時間の調査

調査日: 2026-09-30。作業ツリー: `18ddf08e305848514b9acf70827b9e76915ffccd`。

## 結論

RPM 作成の遅延は、Tauri CLI 2.11.4 が使用する `rpm` 0.16.0 のハッシュ処理と、大きな ONNX Runtime ライブラリの組み合わせで説明できる。

`rpm` の `Sha256Writer::write()` は、圧縮器が実際に受け取ったバイト数を確認する前に、渡されたバッファ全体を SHA-256 に追加する。圧縮器が一部分だけ受け取ると、`write_all()` が残りを再送し、残り全体のハッシュ計算を何度も繰り返す。

実際に同梱される約272MBの CUDA ライブラリでは、gzip によって **約741GB分、元サイズの約2,724倍** のハッシュ計算が発生することを再現した。単に gzip の圧縮処理が遅いという問題ではなく、未処理データの再ハッシュが主因と判断する。

短期的な回避策は RPM の圧縮を無効にすること。同じライブラリ一式を同梱した検証用 RPM は、この環境で **11.00秒** で生成できた。根本対策は、実際に書き込んだ部分だけをハッシュするよう RPM 生成ライブラリを修正した CLI を使用すること。

## CI ログによる確認

GitHub の完了済みジョブログを API で取得し、ログのタイムスタンプから工程ごとの時間を求めた。

| Run / Ubuntu job | Rust release コンパイル | DEB 作成 | RPM 作成 |
| --- | ---: | ---: | ---: |
| [36281036659 / 108512749636](https://github.com/ALG-SIT/Lumin/actions/runs/36281036659/job/108512749636) | 1分15秒 | 8.78秒 | **41分1.89秒** |
| [36281033252 / 108512738952](https://github.com/ALG-SIT/Lumin/actions/runs/36281033252/job/108512738952) | 1分12秒 | 9.42秒 | **56分22.87秒** |

最初の Run の時刻は以下のとおり。UTC 表記。

```text
2026-09-27T00:00:39.3104763Z Finished `release` profile ... in 1m 15s
2026-09-27T00:00:39.4105946Z Bundling Lumin_0.1.0_amd64.deb
2026-09-27T00:00:48.1923702Z Bundling Lumin-0.1.0-1.x86_64.rpm
2026-09-27T00:41:50.0824977Z Finished 2 bundles at:
```

この Run の `Build Tauri app bundle` ステップ全体は42分34秒。ほとんどを RPM 作成が占める。キャッシュの復元や Rust コンパイルの改善だけでは、この遅延は解消しない。

ログ取得には次のコマンドを使用した。この環境では `gh run view --log` が空の結果を返したため、ジョブログの API を直接使用した。

```sh
gh api repos/ALG-SIT/Lumin/actions/jobs/108512749636/logs
gh api repos/ALG-SIT/Lumin/actions/jobs/108512738952/logs
```

## リポジトリ設定と入力ファイル

- `.github/workflows/ci.yml` は Ubuntu で `bun run tauri build --bundles deb,rpm` を実行する。
- `bun.lock` の Tauri CLI は 2.11.4。CI ログでも同じバージョンを確認した。
- 調査時点の `src-tauri/tauri.linux.conf.json` は `runtime/` 全体を bundle resource として同梱し、RPM 圧縮方式の指定はなかった。
- `scripts/prepare_onnx_runtime.py` は CPU、WebGPU、CUDA、TensorRT のネイティブライブラリを取得する。
- Linux 用 lock file、準備スクリプト、Linux 用 Tauri 設定は、調査した2件の CI のコミットと作業ツリーで一致している。

準備スクリプトを `--output /tmp/lumin-rpm-investigation/runtime` で実行し、公式アーカイブの SHA-256 検証を通した実ファイルを計測に使用した。

| ファイル | バイト数 | 概算サイズ（10進 MB） |
| --- | ---: | ---: |
| `libonnxruntime_providers_cuda.so` | 272,049,904 | 272.05 MB |
| `libonnxruntime.so` | 29,062,976 | 29.06 MB |
| `libonnxruntime_providers_webgpu.so` | 15,779,056 | 15.78 MB |
| `libonnxruntime_providers_tensorrt.so` | 990,656 | 0.99 MB |
| 全リソース合計（ライセンスなどを含む） | 318,914,888 | 318.91 MB |

## ソースコードで確認した原因

Tauri 2.11.4 の RPM bundler は `rpm` 0.16.0 を使い、圧縮設定の指定がなければ `Gzip(6)` を選ぶ。

出典: [Tauri RPM bundler](https://github.com/tauri-apps/tauri/blob/tauri-cli-v2.11.4/crates/tauri-bundler/src/bundle/linux/rpm.rs#L64)、[CLI の Cargo.lock](https://github.com/tauri-apps/tauri/blob/tauri-cli-v2.11.4/Cargo.lock)。Context7 でも [RPM 配布ドキュメント](https://v2.tauri.app/distribute/rpm/)を確認し、具体的な挙動はこのバージョンの実装を確認した。

`rpm` 0.16.0 の `Sha256Writer` は次の順序で処理する。

```rust
fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
    self.hasher.update(buf);
    self.writer.write(buf)
}
```

出典: [rpm 0.16.0 のハッシュ処理、513行目以降](https://github.com/rpm-rs/rpm/blob/11f3e1644c37c77abd173b2caef7225bc7ae96e9/src/rpm/headers/types.rs#L513)。このコミットは公開 crate の `.cargo_vcs_info.json` で確認した。

パッケージ生成側は、この writer を圧縮器の手前に置き、各ファイルの内容全体を `write_all(&content)` へ渡す。

出典: [writer の構築](https://github.com/rpm-rs/rpm/blob/11f3e1644c37c77abd173b2caef7225bc7ae96e9/src/rpm/builder.rs#L651)、[ファイルの書き込み](https://github.com/rpm-rs/rpm/blob/11f3e1644c37c77abd173b2caef7225bc7ae96e9/src/rpm/builder.rs#L726)。

`write()` が `n < buf.len()` を返すことは正常な動作だが、この実装はその場合にも全バッファをハッシュする。修正する場合の要点は次の順序になる。

```rust
fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
    let n = self.writer.write(buf)?;
    self.hasher.update(&buf[..n]);
    Ok(n)
}
```

## この環境での実測

### ハッシュ量の計測

`flate2 = 1.1.1`、`miniz_oxide = 0.8.8`、`sha2 = 0.10.8`、`zstd = 0.13.2` を使用した Rust の計測プログラムで、CUDA ライブラリ全体を圧縮器へ渡し、返された書き込みバイト数と呼び出し回数を記録した。これらのバージョンは Tauri CLI のソース側 Cargo.lock に合わせている。

| 圧縮方式 | `write()` 回数 | 元の処理でハッシュ対象となる累計バイト数 | 入力の何倍か | 受け取った部分だけハッシュする処理の実測時間 |
| --- | ---: | ---: | ---: | ---: |
| gzip level 6 | 6,550 | 740,934,580,509 | 2,723.52倍 | 12.64秒 |
| zstd level 3 | 1,840 | 231,671,717,120 | 851.58倍 | 2.90秒 |

累計バイト数の計測では、重複するハッシュ計算を実行せず、その処理へ渡る長さだけを加算した。右端の時間は圧縮と正しいハッシュ処理を実行した時間であり、修正済み CLI によるアプリ全体の RPM 作成時間ではない。

gzip の元の処理を10回だけ実行した測定でも、消費した入力は約1.77MBなのに、ハッシュ対象は約2.71GBになった。zstd は圧縮自体が速くても、未修正のハッシュ処理による重複計算を解消しない。

### 実際の Tauri CLI による RPM 生成

`/tmp` に独立した最小 Cargo プロジェクトを用意し、npm から取得した Tauri CLI 2.11.4 の `tauri bundle` を使用した。実際の Linux runtime 一式を resource に指定し、アプリの代わりに `/bin/true` を既存バイナリとして置いた。CLI のバイナリ識別用変数に関する警告は出るが、RPM 生成は実行される。

| 条件 | 結果 |
| --- | --- |
| 圧縮指定なし（gzip level 6） | 30秒で打ち切り、未完了。CPU user time 29.29秒 |
| zstd level 3 | 186.83秒で打ち切り、未完了。CPU user time 185.62秒 |
| 圧縮なし | 完了。初回11.66秒、再実行11.00秒 |

圧縮なしの RPM は **318,959,673バイト（約319MB）**。巨大な圧縮済みファイルのダウンロードを待っている状況ではなく、CPU を使う処理として遅延を再現できた。CI とこの環境の CPU 性能は異なるため、秒数をそのまま CI の予測値にすることはできない。

### ハッシュ不一致も再現

TensorRT ライブラリ1個（990,656バイト）だけを含めた小さな検証プロジェクトで、既定の gzip 圧縮を使って RPM を生成した。RPM は0.63秒で生成できたが、ヘッダーの未圧縮ペイロードの SHA-256 と、ペイロードを実際に展開した SHA-256 は一致しなかった。

```text
RPM ヘッダー tag 5097:
bf47670eed3507135f77737c3d70b0c1af697618938c5cecabc744b12634bb87
実際の未圧縮ペイロード:
217545c714433f3698eaad33483e8d937c64e5384ef476740dc9bb70a83daad5
```

圧縮後のペイロードの SHA-256（tag 5092）は一致した。圧縮なしで生成した約319MBの RPM は、tag 5092 と tag 5097 の両方が実データの SHA-256 と一致した。この問題が実際の対象ディストリビューションでのインストールにどう影響するかは、今回調査していない。

## 対策案

### 短期: RPM 圧縮を無効にする

`src-tauri/tauri.linux.conf.json` の `bundle` に次の設定を追加した。`resources` は維持し、CUDA を含む runtime 一式の同梱と DEB / RPM 両方の生成を継続する。

```json
{
  "bundle": {
    "linux": {
      "rpm": {
        "compression": { "type": "none" }
      }
    }
  }
}
```

実際の CLI でこの設定を読み込み、RPM 生成完了を確認した。既存の `resources` 設定と併用できる。RPM 自体のサイズは増えるため、配布時の容量とダウンロード時間を評価する必要がある。GitHub Actions のアップロード工程についても、変更後の実測が必要。

### 根本対策: RPM 生成側の修正を含む CLI を使用する

実際に書き込まれた `buf[..n]` だけをハッシュする修正が必要。修正を含む CLI へ更新するか、CLI / bundler 側を修正してビルドする方法が考えられる。修正済みリリースの特定と、その CLI での実アプリのビルドは今回行っていない。

**アプリ側の Cargo.lock や `[patch.crates-io]` だけを変更しても、npm パッケージに同梱された Tauri CLI の RPM 生成処理は変更されない。** 対策対象は CLI のビルド時依存関係になる。

zstd の採用は、このハッシュ処理を修正した後に容量・速度・対象環境の対応を比較する候補。今回の結果から、未修正の CLI に対して zstd を指定するだけの対策は推奨しない。

CUDA ライブラリを外す方法も入力サイズを大きく減らせるが、アプリの GPU 機能に影響するため、今回の回避策には含めていない。

## 再現用の作業ファイルと範囲

調査用のファイルは `/tmp/lumin-rpm-investigation/` に置いた。`/tmp` のため永続保存は保証されない。

- `runtime/`: lock file で検証した Linux runtime。
- `cli/node_modules/.bin/tauri`: npm の Tauri CLI 2.11.4。
- `probe/Cargo.toml`, `probe/src/main.rs`: 圧縮器の部分書き込みとハッシュ量の計測プログラム。`count` は重複ハッシュ量を数え、`correct` は正しい範囲をハッシュする。
- `bundle/`, `bundle-none/`, `bundle-small/`: 独立した検証用プロジェクト。

```sh
# runtime の準備（リポジトリのルートから実行）
python3 scripts/prepare_onnx_runtime.py --output /tmp/lumin-rpm-investigation/runtime

# 計測プログラム
cargo build --release --manifest-path /tmp/lumin-rpm-investigation/probe/Cargo.toml
/tmp/lumin-rpm-investigation/probe/target/release/rpm-write-probe \
  /tmp/lumin-rpm-investigation/runtime/libonnxruntime_providers_cuda.so gzip count

# bundle-none/ から実行
/tmp/lumin-rpm-investigation/cli/node_modules/.bin/tauri bundle --bundles rpm \
  --config '{"bundle":{"linux":{"rpm":{"compression":{"type":"none"}}}}}'
```

初期調査では実アプリの Rust / フロントエンド全体の再ビルド、CI の変更・再実行、対象ディストリビューションへのインストールは行っていない。`perf` による CPU スタック計測はカーネルの権限制限で実行できず、ソースコード、書き込みバイト数の計測、実際の CLI による再現を根拠とした。回避策適用後の検証結果は以下に追記する。

## 回避策適用後の実アプリ検証

2026-09-30、`origin/main` の `18ddf08` から独立した作業ツリーで検証した。Bun 1.4.2、Rust 1.98.1、Tauri CLI 2.11.4 を使用し、CLI や依存関係の更新は行っていない。

以下はすべて成功した。

```sh
bun install --frozen-lockfile
python3 scripts/prepare_onnx_runtime.py
cargo check --manifest-path src-tauri/Cargo.toml
cargo clippy --manifest-path src-tauri/Cargo.toml -- -D warnings
cargo fmt --manifest-path src-tauri/Cargo.toml -- --check
bun run check
bun run test:run
bun run build
cargo test --manifest-path src-tauri/Cargo.toml
CARGO_TARGET_DIR="$PWD/target" bun run tauri build --bundles deb,rpm
```

フロントエンドは88件成功、Rust は219件成功・19件 ignored。vendored `tao` の既存の非推奨 API 警告と、フロントエンドの chunk サイズ警告は出るが、上記コマンドは終了コード0で完了した。

| 実アプリのローカル測定 | 結果 |
| --- | ---: |
| `tauri build` 全体（runtime 準備・frontend・初回 release コンパイル込み） | 269.36秒 |
| Rust release コンパイル（Cargo の表示） | 3分55秒 |
| DEB 工程 | 約15.41秒 |
| RPM 工程 | **約11.14秒** |
| DEB ファイルサイズ | 256,556,466バイト（約257MB） |
| RPM ファイルサイズ | **355,018,793バイト（約355MB）** |

工程時間はログの `Bundling ...deb`、`Bundling ...rpm`、`Finished 2 bundles` の出力を50ms間隔で監視し、観測時刻の差から算出した概算。全体時間は `/usr/bin/time -p` による実測。初期調査の約319MBは最小検証用バイナリを含む RPM であり、実アプリのサイズとは異なる。CPU、ビルドキャッシュ、実行環境によって所要時間は変わる。

生成した RPM の lead / signature header / main header を解析し、以下を確認した。

- 圧縮器タグ1125は存在せず、ペイロードは直接 `070701` で始まる非圧縮 newc CPIO。ペイロード形式タグ1124は `cpio`。
- SHA-256 アルゴリズムのタグ5093は8。ペイロードの SHA-256 は `0b261042687ea60a97561cf5424a1137832d90d4132bf241916aa4f79008fb25` で、タグ5092と5097の両方に一致。
- CPIO 内の `runtime/` 配下を読み取り、準備済みの13ファイル（合計318,914,888バイト）すべてについて、対応するパスが1個存在し、サイズと SHA-256 が元ファイルと一致。CUDA / TensorRT / WebGPU / CPU ライブラリ、ライセンス等の欠落はない。

DEB と RPM の両ファイルが生成された。対象ディストリビューションへのインストール試験と、インストールしたアプリの起動・推論試験は実施していない。この検証はパッケージ生成と内容・ペイロード整合性の確認に限る。根本的な writer の修正と修正済み CLI への移行は別作業とする。
