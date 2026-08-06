# Lumin

> 理解を照らし、次の学びにつなげる。

Lumin は、生徒の解答を端末内で分析して段階的なヒントを返し、解答本文を送らずにクラス全体の誤概念を教師へ共有する、ローカルファースト型の学習支援アプリです。

このリポジトリは `doc/AI_Innovators_Cup_Lumin_構想.md` に基づく大会用MVPです。SwiftUIで作られており、1つのアプリを教師モード・生徒モードに切り替えて利用します。

## 実装済み

- 数学・国語・理科・社会・英語、6セット25問の選択式教材バンク
- 端末内の正誤判定と誤概念分類
- 正解を直接出さない3段階ヒント
- ヒント後の再回答成功記録
- MultipeerConnectivityによる暗号化された教室内通信
- 解答本文を含まない分析イベントのみの送信
- 教師向け正答率・再挑戦成功率・平均ヒント数・誤概念集計
- 回答が集まる前でも試せる大会デモデータ
- 集計から生成する「次回授業の冒頭10分案」
- 教師による授業案の編集・採用
- 分析・集計・通信データ形式の自動テスト

## 起動方法

1. Xcodeで `Lumin.xcodeproj` を開きます。
2. 実行先に接続したiPad、iPhone、またはiOS Simulatorを選びます。
3. `Lumin` スキームを実行します。起動画面の「AIモデル」から使用する方式を選択できます。

同じ教室ネットワーク上の2台で、片方を「先生」、もう片方を「生徒」として起動すると相互に検出します。初回はローカルネットワーク利用を許可してください。

端末が1台の場合は、教師モードの「大会デモ用データを読み込む」で集計画面を、生徒モードの「デモとして始める」で回答フローをそれぞれ確認できます。

## テスト

```sh
swift test
```

アプリ本体はXcodeの `Lumin` スキームでビルドできます。現在のターゲットは iOS / iPadOS 17以降です。

## プライバシー境界

生徒端末にだけ残るもの：

- 入力した解答全文
- 回答途中の状態
- 表示したヒント

教師端末へ送るもの：

- セッション内だけの匿名参加トークン
- 問題IDと概念
- 誤概念ラベル
- 初回正誤、ヒント回数、再回答結果

`AnalysisEvent` は解答本文を保持できない型として定義しています。通信終了時はMultipeerConnectivityのセッションを切断します。

## オンデバイスAI

「自動」「Apple Foundation Model」「Gemma」から実行方式を選択できます。自動ではApple Intelligenceのシステムモデルが利用可能なら優先し、利用できない端末では保存済みGemmaへフォールバックします。iOS 27では `SystemLanguageModel.default` を通してOS更新版の新世代モデルが自動的に使われます。

Apple Foundation Modelsの出力には `@Generable` と `@Guide` によるGuided Generationを使用します。GemmaはGoogle LiteRT-LM 0.15.0で実行し、実機ではMetal GPU（失敗時CPU）、SimulatorではXNNPACK CPUを使用します。どちらも誤答時の誤概念分類・段階ヒント・教師向け10分授業案を端末内で生成します。

実装時に参照した公式資料：

- [SystemLanguageModel — Apple Developer](https://developer.apple.com/documentation/foundationmodels/systemlanguagemodel)
- [Foundation Models updates — Apple Developer](https://developer.apple.com/documentation/updates/foundationmodels)
- [Generating content and performing tasks with Foundation Models — Apple Developer](https://developer.apple.com/documentation/foundationmodels/generating-content-and-performing-tasks-with-foundation-models)
- [LiteRT-LM — Google AI Edge](https://github.com/google-ai-edge/LiteRT-LM)

授業案のGemma入力は個々の解答ではなく `ClassSummary` の匿名集計値に限定しています。Gemmaにはインターネット通信機能がなく、推論はMetal GPU（利用できない場合はCPU）で完結します。

## Gemmaモデルの選択とダウンロード

アプリ内のモデル管理画面から端末メモリに応じた推奨モデルを選べます。

- Gemma 3 1B int4：約584MB、一般的なiPhone/iPad向け
- Gemma 3n E2B int4：約3.66GB、8GB以上のメモリを持つ上位端末向け
- Gemma 3n E4B int4：約4.92GB、16GB以上のメモリを持つiPad向け

モデルはApplication Supportへ保存され、ダウンロード前に必要な空き容量を検査します。GoogleのGemma利用規約に同意したHugging Face Readトークンを入力すると、トークンはこの端末限定のKeychainへ保存されます。手元の `.litertlm` ファイルを読み込むこともできます。

この作業環境には開発確認用として `LuminApp/Models/gemma3-1b-it-int4.litertlm` も配置済みです（Git管理外）。別環境で事前配置する場合は次を実行します。

```sh
./scripts/download-gemma.sh
```

モデルをアプリへ同梱しない場合でも、起動画面の「モデルを選択」から `.litertlm` ファイルを端末内へ取り込めます。

Mac上で実推論だけを検証するスモークテストも用意しています。

```sh
swift run --package-path Tools/GemmaSmoke GemmaSmoke
```

初回のみLiteRT-LMの公式ネイティブバイナリを取得します。

## 主な構成

```text
LuminApp/
  AI/                  Gemmaモデル管理・実推論・状態表示
  Features/Student/    生徒の参加・回答・ヒント画面
  Features/Teacher/    集計・配信・授業案編集画面
  Services/            教室内P2P通信
Sources/LuminCore/     モデル、分析、集計、サンプル教材
Tests/LuminCoreTests/  コアロジックのテスト
Vendor/LiteRTLM/       Google公式Swift API v0.15.0の固定ラッパー
Tools/GemmaSmoke/      Mac用の実推論確認ツール
```
