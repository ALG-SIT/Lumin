import { useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { isTauriEnvironment } from "../lib/tauri";

export interface DemoFlowProps {
  onReset?: () => void;
}

interface DemoStep {
  label: string;
  detail: string;
}

interface LessonPlanData {
  focus: string;
  steps: string[];
  checkQuestion: string;
  teacherNote: string;
}

interface ClassSummaryData {
  participantCount: number;
  responseCount: number;
  correctRate: number;
  retrySuccessRate: number;
  averageHints: number;
  misconceptions: { name: string; count: number; share: number }[];
}

const DEMO_STEPS: DemoStep[] = [
  { label: "セッション開始", detail: "先生がデモ用クイズを配信しています..." },
  { label: "生徒が参加", detail: "P-101, P-102, P-103... が参加中" },
  { label: "回答収集中", detail: "不正解 → ヒント → 再挑戦の流れを再生中" },
  { label: "集計中", detail: "クラス全体の正答率を計算中" },
  { label: "授業案生成", detail: "誤概念に基づく授業計画を作成中" },
  { label: "デモ完了", detail: "3分デモが完了しました" },
];
function sleep(ms: number) {
  return new Promise((r) => setTimeout(r, ms));
}

export function DemoFlow({ onReset }: DemoFlowProps) {
  const [currentStep, setCurrentStep] = useState(-1);
  const [summary, setSummary] = useState<ClassSummaryData | null>(null);
  const [lessonPlan, setLessonPlan] = useState<LessonPlanData | null>(null);
  const [error, setError] = useState<string | null>(null);

  const runDemo = async () => {
    setError(null);
    if (!isTauriEnvironment()) {
      setError(
        "Luminデスクトップアプリから実行してください(ターミナルで bun run tauri dev)"
      );
      return;
    }
    setCurrentStep(0);

    try {
      // Step 0: Start session
      await invoke<string>("start_demo_session");
      await sleep(1500);

      // Step 1: Students joining
      setCurrentStep(1);
      await sleep(2000);

      // Step 2: Submit demo events (simulates wrong → hint → retry → correct)
      setCurrentStep(2);
      await invoke("submit_demo_events");
      await sleep(2500);

      // Step 3: Show aggregate
      setCurrentStep(3);
      const s = await invoke<ClassSummaryData>("get_demo_quiz_summary");
      setSummary(s);
      await sleep(2000);

      // Step 4: Generate lesson plan
      setCurrentStep(4);
      const p = await invoke<LessonPlanData>("get_demo_lesson_plan");
      setLessonPlan(p);
      await sleep(2500);

      // Step 5: Complete
      setCurrentStep(5);
    } catch (e) {
      console.error("DemoFlow invoke failed", e);
      setError(String(e));
    }
  };

  return (
    <section className="demo-flow" aria-label="3分デモ">
      <h2>3分間デモ</h2>
      <p className="demo-subtitle">
        Luminの全フローを体験できます（実際のネットワーク・AI推論は使用しません）
      </p>

      {currentStep === -1 && (
        <div className="demo-intro">
          {isTauriEnvironment() ? (
            <button
              type="button"
              className="demo-start-button"
              onClick={runDemo}
            >
              デモを始める
            </button>
          ) : (
            <p className="demo-error" role="status">
              このデモはLuminデスクトップアプリでのみ実行できます。
              ブラウザプレビューでは動作しません。
              ターミナルで
              <code> bun run tauri dev </code>
              を実行して起動してください。
            </p>
          )}
          {onReset && (
            <button
              type="button"
              className="reset-button"
              onClick={onReset}
              style={{ marginTop: "0.75rem" }}
            >
              役割を切り替える
            </button>
          )}
        </div>
      )}

      {currentStep >= 0 && (
        <div className="demo-progress">
          {DEMO_STEPS.map((step, i) => (
            <div
              key={i}
              className={`demo-step ${i === currentStep ? "active" : ""} ${i < currentStep ? "done" : ""} ${i > currentStep ? "pending" : ""}`}
            >
              <span className="demo-step-indicator">
                {i < currentStep ? "\u2713" : i === currentStep ? "\u25cf" : "\u25cb"}
              </span>
              <div className="demo-step-text">
                <strong>{step.label}</strong>
                {i === currentStep && (
                  <span className="demo-step-detail">{step.detail}</span>
                )}
              </div>
            </div>
          ))}
        </div>
      )}

      {summary && currentStep >= 3 && (
        <div className="demo-summary">
          <h3>クラス集計</h3>
          <div className="demo-stats">
            <div className="demo-stat">
              <span className="demo-stat-label">参加者</span>
              <span className="demo-stat-value">{summary.participantCount}名</span>
            </div>
            <div className="demo-stat">
              <span className="demo-stat-label">回答数</span>
              <span className="demo-stat-value">{summary.responseCount}件</span>
            </div>
            <div className="demo-stat">
              <span className="demo-stat-label">正答率</span>
              <span className="demo-stat-value">
                {Math.round(summary.correctRate * 100)}%
              </span>
            </div>
            <div className="demo-stat">
              <span className="demo-stat-label">再挑戦成功率</span>
              <span className="demo-stat-value">
                {Math.round(summary.retrySuccessRate * 100)}%
              </span>
            </div>
            <div className="demo-stat">
              <span className="demo-stat-label">平均ヒント数</span>
              <span className="demo-stat-value">
                {summary.averageHints.toFixed(1)}
              </span>
            </div>
          </div>
          {summary.misconceptions.length > 0 && (
            <div className="demo-misconceptions">
              <h4>上位の誤概念</h4>
              <ul>
                {summary.misconceptions.slice(0, 3).map((m) => (
                  <li key={m.name}>
                    {m.name} ({m.count}件 / {Math.round(m.share * 100)}%)
                  </li>
                ))}
              </ul>
            </div>
          )}
        </div>
      )}

      {lessonPlan && currentStep >= 4 && (
        <div className="demo-lesson-plan">
          <h3>生成された授業案</h3>
          <div className="demo-plan-focus">
            <strong>焦点:</strong> {lessonPlan.focus}
          </div>
          <ol className="demo-plan-steps">
            {lessonPlan.steps.map((s, i) => (
              <li key={i}>{s}</li>
            ))}
          </ol>
          <div className="demo-plan-check">
            <strong>確認問題:</strong> {lessonPlan.checkQuestion}
          </div>
          <div className="demo-plan-note">
            <strong>教師メモ:</strong> {lessonPlan.teacherNote}
          </div>
        </div>
      )}

      {error && (
        <div className="demo-error" role="alert">
          <p>エラーが発生しました: {error}</p>
          {isTauriEnvironment() && (
            <button type="button" className="reset-button" onClick={runDemo}>
              もう一度試す
            </button>
          )}
        </div>
      )}

      {currentStep === 5 && (
        <div className="demo-complete">
          <p>3分デモが完了しました。</p>
          {onReset && (
            <button
              type="button"
              className="reset-button"
              onClick={onReset}
              style={{ marginTop: "1rem" }}
            >
              役割を切り替える
            </button>
          )}
        </div>
      )}
    </section>
  );
}
