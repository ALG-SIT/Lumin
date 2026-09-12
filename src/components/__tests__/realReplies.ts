/**
 * Replies captured verbatim from the real on-device models, so the renderer is
 * tested against what it actually has to display rather than against
 * hand-written Markdown.
 *
 * Regenerate with:
 *   cd src-tauri
 *   cargo test teacher_chat_transcript_probe -- --ignored --nocapture --test-threads=1
 */

/** Gemma 4 E2B INT4, "最も多い誤概念は何件ですか。" */
export const GEMMA4_SUMMARY_REPLY = [
  "* **誤概念の件数**: 4件",
  "* **誤概念名**: added numerators",
  "* **割合**: 40%",
].join("\n");

/** Gemma 4 E2B INT4, the follow-up turn: a plan as a nested bullet list. */
export const GEMMA4_FOLLOW_UP_REPLY = [
  "* **次回の授業でやるべきこと**",
  "    * 「added numerators」に関する誤概念について、具体的な練習問題を通じて理解度を確認する。",
  "    * 誤概念の理解を深めるための追加の練習や解説を行う。",
].join("\n");

/**
 * Gemma 4 E4B INT4, the follow-up turn. Same shape as E2B's but spelled with
 * three spaces after the bullet, which is the other marker width the models
 * emit, and with a second nesting level under every heading item.
 */
export const GEMMA4_E4B_FOLLOW_UP_REPLY = [
  "*   **「added numerators」の概念の再確認と演習:**",
  "    *   この誤概念が最も多く見られたため、次の授業で集中的に取り扱うことが推奨されます。",
  "    *   「通分して加算」という正しい手順を、具体的な問題を通して定着させるための演習を計画してください。",
  "*   **再挑戦の成功率（50%）を踏まえたフォローアップ:**",
  "    *   一度間違えた生徒が再挑戦で成功するケースがあるため、間違えた生徒への個別フォローアップや、より段階的な難易度の設定を検討してください。",
].join("\n");

/**
 * Gemma 4 E2B INT4 answering a question that calls for a formula. It mixes
 * every spelling the renderer has to survive: inline `$…$`, a fenced `$$…$$`
 * block spread over three lines, and math nested inside bold inside an
 * ordered list.
 */
export const GEMMA4_MATH_REPLY = [
  "一次関数の傾きを求める式は、2点 $(x_1, y_1)$ と $(x_2, y_2)$ を用いて、以下の式で表されます。",
  "",
  "$$",
  "m = \\frac{y_2 - y_1}{x_2 - x_1}",
  "$$",
  "",
  "ここで、$m$ は一次関数の傾きを表します。",
  "",
  "**説明:**",
  "",
  "1.  **$x_1, y_1$ と $x_2, y_2$**: グラフ上の任意の2つの点とします。",
  "2.  **分子 ($y_2 - y_1$)**: $y$ 座標の変化量（縦の変化）を示します。",
  "3.  **分母 ($x_2 - x_1$)**: $x$ 座標の変化量（横の変化）を示します。",
].join("\n");

/** Gemma 4 E4B INT4, which closes its display block on one line. */
export const GEMMA4_E4B_MATH_REPLY = [
  "一次関数の傾きを求める式は、以下の式で求められます。",
  "",
  "$$\\text{傾き} (m) = \\frac{y_2 - y_1}{x_2 - x_1}$$",
  "",
  "縦軸（$y$軸）の変化量を横軸（$x$軸）の変化量で割ります。",
].join("\n");

/** Gemma 3 1B INT4, which answers with headings and an ordered list. */
export const GEMMA3_PLAN_REPLY = [
  "## 分数の問題解決の練習 - 授業での取り組み",
  "",
  "**考えられる学習目標:**",
  "",
  "*   分数計算の基本的な手順を理解する",
  "*   計算ミスを防ぐための注意点を意識する",
  "",
  "**授業での取り組み (提案):**",
  "",
  "1.  **段階的な問題解決:** まず、生徒に「1/2 + 1/3」という問題を出題します。",
  "2.  **ヒント提供:**  「通分しよう」というヒントを、生徒に提示します。",
].join("\n");
