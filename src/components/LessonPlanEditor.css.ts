import { style } from "@vanilla-extract/css";

// LessonPlanEditor.tsx batch (plan checkbox 6). Declaration values are
// verbatim copies of the corresponding src/index.css rules.
//
// The component's other className literals (lesson-plan-editor,
// lesson-plan-header, lesson-plan-title, lesson-plan-subtitle,
// lesson-plan-card, field, lesson-textarea, steps-section, step-field,
// char-count, lesson-plan-actions) have ZERO index.css rules — they are
// TSX-only hook classes and stay as literal strings. lesson-textarea in
// particular is kept verbatim in the template-literal className
// (census + grep verified: no rule, no test reference).
//
// manual-input is the manual-input share of the SHARED input band
// (index.css L523-534 `.manual-input, .code-input, .answer-input` +
// L536-541 `:focus` group). Only the manual-input declarations are
// translated here; the band itself stays in index.css because
// .code-input (StudentJoin) / .answer-input (StudentQuiz) and
// StudentJoin's own manual-input literals still depend on it — the
// band's deletion is T8's job (last owner).

// index.css L523-534 (manual-input share of the shared band) +
// L536-541 (manual-input share of the :focus indigo group).
// Cascade: baseInputFocus (src/styles/global.css.ts, imported first) is
// attached alongside this class in the TSX. Its ':focus' purple ties at
// (0,2,0) with the ':focus' indigo below and loses to the later-imported
// component CSS — exactly reproducing the old element-vs-class cascade
// (input:focus (0,1,1) lost to .manual-input:focus (0,2,0)). Its
// '::placeholder' (text-secondary) matches the still-present element rule
// value and becomes the only placeholder source once T8/T9 delete the
// element bands.
export const manualInput = style({
  padding: "var(--space-3)",
  fontSize: "1rem",
  color: "var(--lumin-ink)",
  background: "var(--lumin-canvas)",
  border: "1px solid var(--lumin-border)",
  borderRadius: "var(--radius-md)",
  outline: "none",
  transition: "border-color 0.2s ease, box-shadow 0.2s ease",
  ":focus": {
    borderColor: "var(--lumin-indigo)",
    boxShadow: "0 0 0 3px var(--lumin-indigo-soft)",
  },
});
