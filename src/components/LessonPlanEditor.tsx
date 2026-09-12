import { invoke } from "@tauri-apps/api/core";
import { useEffect, useRef } from "react";
import { baseInputFocus } from "../styles/global.css.ts";
import {
  errorMessage,
  primaryButton,
  secondaryButton,
} from "../styles/shared.css.ts";
import { ProgressRing } from "./GenerationProgress";
import {
  fallbackNotice,
  lessonEditor,
  manualInput,
  planGenerating,
  planOverlay,
} from "./LessonPlanEditor.css.ts";
import type { LessonPlanDraft, PlanProgress } from "./useLessonPlan";

export type { LessonPlan } from "./useLessonPlan";

const ATTEMPTS = 2;

/** What the generator is doing, said plainly. */
function planLabel(
  progress: PlanProgress | null,
  modelLoading: boolean,
): string {
  // How far the model load has got belongs to the app bar; here it only
  // explains why the plan has not started.
  if (modelLoading) return "モデルの読み込みを待っています…";
  if (!progress) return "AIに指示を送っています…";
  const pass =
    "attempt" in progress && progress.attempt > 1 ? "作り直し: " : "";
  switch (progress.stage) {
    case "prompt":
      return `${pass}集計と教材を読み込み中… (${progress.done}/${progress.total} トークン)`;
    case "generating":
      return `${pass}授業案を作成中… (${progress.generated}/${progress.max} トークン)`;
    case "validating":
      return `${pass}出力の形式を確認中…`;
    case "repairing":
      return `形式が不正だったため作り直します（${progress.reason}）`;
    case "fallback":
      return "AIの案を採用できなかったため、教材に基づく案を使います";
    default:
      return "完了";
  }
}

/**
 * How far along the whole job is, counting both passes.
 *
 * The prompt has a known length, so that part is exact. Writing is measured
 * against the token budget, which is a ceiling rather than a target - a plan
 * usually closes its JSON well before it - so the bar is a lower bound that
 * only ever moves forward. Stages with nothing to measure report null and get
 * a moving stripe instead of an invented number.
 */
function planPercent(
  progress: PlanProgress | null,
  modelLoading: boolean,
): number | null {
  if (modelLoading || !progress) return null;
  const share = (attempt: number, within: number) =>
    ((attempt - 1 + within) / ATTEMPTS) * 100;
  switch (progress.stage) {
    case "prompt":
      return progress.total
        ? share(progress.attempt, (progress.done / progress.total) * 0.2)
        : null;
    case "generating":
      return share(
        progress.attempt,
        0.2 + (progress.generated / progress.max) * 0.8,
      );
    case "validating":
      return share(progress.attempt, 1);
    default:
      return null;
  }
}

export interface LessonPlanEditorProps {
  /**
   * Plan, generation state and controls, owned by the parent.
   *
   * They are not held here because this editor is unmounted whenever another
   * tab is shown, and a generation runs for tens of seconds after that.
   */
  draft: LessonPlanDraft;
  onAdopted: (plan: import("./useLessonPlan").LessonPlan) => void;
}

/**
 * A text field that grows to fit what is in it.
 *
 * Generated text is not written to a box size: a fixed height cut the model's
 * plan mid-sentence and hid the rest behind a scrollbar, and the check
 * question - a single-line input - lost its tail off the right edge. A plan
 * the teacher is about to read has to be readable in full without scrolling
 * inside four separate boxes.
 */
function AutoTextarea({
  value,
  ...props
}: React.ComponentProps<"textarea"> & { value: string }) {
  const ref = useRef<HTMLTextAreaElement>(null);

  // biome-ignore lint/correctness/useExhaustiveDependencies: 値が変わるたびに内容の高さへ合わせ直す
  useEffect(() => {
    const el = ref.current;
    if (!el) return;
    // Shrink first, or the box can only ever grow.
    el.style.height = "auto";
    // jsdom reports no layout; leaving the height alone there is correct.
    if (el.scrollHeight > 0) el.style.height = `${el.scrollHeight}px`;
  }, [value]);

  return <textarea ref={ref} value={value} {...props} />;
}

export function LessonPlanEditor({ draft, onAdopted }: LessonPlanEditorProps) {
  const {
    plan,
    setPlan,
    busy,
    progress,
    modelLoading,
    elapsedMs,
    error,
    setError,
    regenerate,
  } = draft;

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
    <div className={lessonEditor}>
      <div className="lesson-plan-header">
        <div>
          <h2 className="lesson-plan-title">次の10分</h2>
          <p className="lesson-plan-subtitle">
            AIまたは教材に基づく初期案を、先生の判断で仕上げます。
          </p>
        </div>
        <button
          type="button"
          onClick={regenerate}
          disabled={busy}
          className={secondaryButton}
        >
          {busy ? "AIが生成中…" : "オンデバイスAIで再生成"}
        </button>
      </div>

      <div
        className={`lesson-plan-card${busy ? ` ${planGenerating}` : ""}`}
        aria-busy={busy}
      >
        {busy && (
          <div className={planOverlay}>
            <ProgressRing
              ariaLabel="レッスンプランの生成状況"
              label={planLabel(progress, modelLoading)}
              percent={planPercent(progress, modelLoading)}
              elapsedMs={elapsedMs}
            />
          </div>
        )}

        <div className="field">
          <label htmlFor="lesson-focus">学習の焦点</label>
          <AutoTextarea
            id="lesson-focus"
            className={`${manualInput} ${baseInputFocus} lesson-textarea`}
            value={plan.focus}
            onChange={(e) => setPlan((p) => ({ ...p, focus: e.target.value }))}
            placeholder="この授業の主な目標は？"
            rows={1}
          />
        </div>

        <div className="field">
          <label htmlFor="teacher-note">教師メモ</label>
          <AutoTextarea
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
              <AutoTextarea
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
          <AutoTextarea
            id="check-question"
            className={`${manualInput} ${baseInputFocus} lesson-textarea`}
            value={plan.checkQuestion}
            onChange={(e) =>
              setPlan((p) => ({ ...p, checkQuestion: e.target.value }))
            }
            placeholder="最後に確認する質問"
            rows={1}
          />
        </div>
      </div>

      {error && <p className={errorMessage}>{error}</p>}
      {!busy && progress?.stage === "fallback" && (
        <p className={fallbackNotice} role="status">
          AIの出力形式を確認できなかったため、教材に基づく案を表示しています。
          内容を確認してから採用してください。
        </p>
      )}

      <div className="lesson-plan-actions">
        <button
          type="button"
          onClick={regenerate}
          disabled={busy}
          className={secondaryButton}
        >
          {busy ? "生成中…" : "AIで再生成"}
        </button>
        <button
          type="button"
          onClick={adopt}
          disabled={busy}
          className={primaryButton}
        >
          この案を採用
        </button>
      </div>
    </div>
  );
}
