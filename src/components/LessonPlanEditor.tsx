import { invoke } from "@tauri-apps/api/core";
import { useEffect, useRef, useState } from "react";
import { baseInputFocus } from "../styles/global.css.ts";
import {
  errorMessage,
  primaryButton,
  secondaryButton,
} from "../styles/shared.css.ts";
import { editor, manualInput } from "./LessonPlanEditor.css.ts";

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

export function LessonPlanEditor({
  classSummary,
  onAdopted,
}: LessonPlanEditorProps) {
  const [plan, setPlan] = useState<LessonPlan>({
    focus: "",
    steps: ["", "", "", ""],
    checkQuestion: "",
    teacherNote: "",
  });
  const [loading, setLoading] = useState(false);
  const [initializing, setInitializing] = useState(true);
  const edited = useRef(false);
  const [saved, setSaved] = useState(false);
  const [saving, setSaving] = useState(false);
  const [error, setError] = useState<string | null>(null);

  // 初期テンプレートを生成(UIブロックなし・静かに行う/main実装同等)
  // biome-ignore lint/correctness/useExhaustiveDependencies: マウント時の1回だけ初期生成する意図。classSummary を依存に加えると編集中の plan が上書きされる
  useEffect(() => {
    let cancelled = false;
    (async () => {
      try {
        const result = await invoke<LessonPlan>("generate_lesson_plan", {
          classSummaryJson: JSON.stringify(classSummary),
        });
        if (!cancelled && !edited.current) setPlan(result);
      } catch (e) {
        if (!cancelled)
          setError(
            `初期案を生成できませんでした。手動で入力するか、再生成してください: ${e}`,
          );
      } finally {
        if (!cancelled) setInitializing(false);
      }
    })();
    return () => {
      cancelled = true;
    };
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, []);

  const regenerate = async () => {
    setLoading(true);
    setSaved(false);
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
    setSaving(true);
    try {
      await invoke("save_lesson_plan", { planJson: JSON.stringify(plan) });
      onAdopted(plan);
      setSaved(true);
    } catch (e) {
      setError(String(e));
    } finally {
      setSaving(false);
    }
  };

  const updateStep = (i: number, value: string) => {
    const steps = [...plan.steps] as [string, string, string, string];
    steps[i] = value;
    setPlan((p) => ({ ...p, steps }));
  };

  return (
    <div
      className={editor}
      onChange={() => {
        edited.current = true;
        setSaved(false);
      }}
    >
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
          disabled={initializing || loading || saving}
          className={secondaryButton}
        >
          {loading ? "AIが生成中…" : "オンデバイスAIで再生成"}
        </button>
      </div>

      {initializing && (
        <p role="status">
          AIが初期案を生成中です。先に入力した内容は保持されます。
        </p>
      )}
      <fieldset className="lesson-plan-card" disabled={loading || saving}>
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
            /* biome-ignore lint/suspicious/noArrayIndexKey: steps[i] を textarea の値と位置で対応させるため */
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
      </fieldset>

      {saved && <p role="status">授業案を保存しました。</p>}
      {error && <p className={errorMessage}>{error}</p>}

      <div className="lesson-plan-actions">
        <button
          type="button"
          onClick={regenerate}
          disabled={initializing || loading || saving}
          className={secondaryButton}
        >
          {loading ? "生成中…" : "AIで再生成"}
        </button>
        <button
          type="button"
          onClick={adopt}
          disabled={loading || saving}
          className={primaryButton}
        >
          {saving ? "保存中…" : "この案を採用"}
        </button>
      </div>
    </div>
  );
}
