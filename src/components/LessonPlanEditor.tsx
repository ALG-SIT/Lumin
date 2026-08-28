import { useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { baseInputFocus } from "../styles/global.css.ts";
import { errorMessage, primaryButton, secondaryButton } from "../styles/shared.css.ts";
import { manualInput } from "./LessonPlanEditor.css.ts";

export interface MisconceptionSummary {
  concept: string;
  count: number;
}

export interface ClassSummary {
  participantCount: number;
  responseCount: number;
  correctRate: number;
  retrySuccessRate: number;
  averageHints: number;
  misconceptions: MisconceptionSummary[];
}

export interface LessonPlan {
  focus: string;
  steps: [string, string, string, string];
  checkQuestion: string;
  teacherNote: string;
}

export interface LessonPlanEditorProps {
  classSummary: ClassSummary;
  onAdopted: (plan: LessonPlan) => void;
}

export function LessonPlanEditor({ classSummary, onAdopted }: LessonPlanEditorProps) {
  const [plan, setPlan] = useState<LessonPlan>({
    focus: "",
    steps: ["", "", "", ""],
    checkQuestion: "",
    teacherNote: "",
  });
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  // 初期テンプレートを生成(UIブロックなし・静かに行う/main実装同等)
  useEffect(() => {
    let cancelled = false;
    (async () => {
      try {
        const result = await invoke<LessonPlan>("generate_lesson_plan", {
          classSummaryJson: JSON.stringify(classSummary),
        });
        if (!cancelled) setPlan(result);
      } catch {
        // 初期生成失敗は手動再生成に譲る
      }
    })();
    return () => {
      cancelled = true;
    };
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, []);

  const regenerate = async () => {
    setLoading(true);
    setError(null);
    try {
      const result = await invoke<LessonPlan>("generate_lesson_plan", {
        classSummaryJson: JSON.stringify(classSummary),
      });
      setPlan(result);
    } catch (e) {
      setError(String(e));
    } finally {
      setLoading(false);
    }
  };

  const adopt = async () => {
    setError(null);
    try {
      await invoke("save_lesson_plan", { planJson: JSON.stringify(plan) });
      onAdopted(plan);
    } catch (e) {
      setError(String(e));
    }
  };

  const updateStep = (i: number, value: string) => {
    const steps = [...plan.steps] as [string, string, string, string];
    steps[i] = value;
    setPlan((p) => ({ ...p, steps }));
  };

  return (
    <div className="lesson-plan-editor">
      <div className="lesson-plan-header">
        <div>
          <h2 className="lesson-plan-title">次の10分</h2>
          <p className="lesson-plan-subtitle">
            AIの初期案を、先生の判断で仕上げます。
          </p>
        </div>
        <button
          type="button"
          onClick={regenerate}
          disabled={loading}
          className={secondaryButton}
        >
          {loading ? "AIが生成中…" : "オンデバイスAIで再生成"}
        </button>
      </div>

      <div className="lesson-plan-card">
        <div className="field">
          <label htmlFor="lesson-focus">学習の焦点</label>
          <input
            id="lesson-focus"
            className={`${manualInput} ${baseInputFocus}`}
            value={plan.focus}
            onChange={(e) => setPlan((p) => ({ ...p, focus: e.target.value }))}
            placeholder="この授業の主な目標は？"
          />
        </div>

        <div className="field">
          <label htmlFor="teacher-note">教師メモ</label>
          <textarea
            id="teacher-note"
            className={`${manualInput} ${baseInputFocus} lesson-textarea`}
            value={plan.teacherNote}
            onChange={(e) =>
              setPlan((p) => ({ ...p, teacherNote: e.target.value }))
            }
            placeholder="教師向けの補足メモ"
          />
        </div>

        <div className="steps-section">
          <h3>授業の展開（4段階）</h3>
          {plan.steps.map((step, i) => (
            <div key={i} className="step-field">
              <label htmlFor={`step-${i}`}>段階 {i + 1}</label>
              <textarea
                id={`step-${i}`}
                className={`${manualInput} ${baseInputFocus} lesson-textarea`}
                value={step}
                onChange={(e) => updateStep(i, e.target.value)}
                placeholder={`段階${i + 1}の説明`}
                maxLength={100}
              />
              <span className="char-count">{step.length}/100</span>
            </div>
          ))}
        </div>

        <div className="field">
          <label htmlFor="check-question">確認質問</label>
          <input
            id="check-question"
            className={`${manualInput} ${baseInputFocus}`}
            value={plan.checkQuestion}
            onChange={(e) =>
              setPlan((p) => ({ ...p, checkQuestion: e.target.value }))
            }
            placeholder="最後に確認する質問"
          />
        </div>
      </div>

      {error && <p className={errorMessage}>{error}</p>}

      <div className="lesson-plan-actions">
        <button
          type="button"
          onClick={regenerate}
          disabled={loading}
          className={secondaryButton}
        >
          {loading ? "生成中…" : "AIで再生成"}
        </button>
        <button
          type="button"
          onClick={adopt}
          disabled={loading}
          className={primaryButton}
        >
          この案を採用
        </button>
      </div>
    </div>
  );
}
