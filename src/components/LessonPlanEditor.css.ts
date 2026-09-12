import { globalStyle, keyframes, style } from "@vanilla-extract/css";

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

export const lessonEditor = style({
  display: "flex",
  flexDirection: "column",
  gap: "12px",
  padding: "12px",
  minWidth: 0,
});

globalStyle(`${lessonEditor} .lesson-plan-header`, {
  display: "flex",
  alignItems: "center",
  justifyContent: "space-between",
  gap: "12px",
  flexWrap: "wrap",
});
globalStyle(`${lessonEditor} h2, ${lessonEditor} h3, ${lessonEditor} p`, {
  margin: 0,
});
globalStyle(`${lessonEditor} .lesson-plan-card`, {
  display: "grid",
  gridTemplateColumns: "repeat(2, minmax(0, 1fr))",
  gap: "12px",
  "@media": { "(max-width: 640px)": { gridTemplateColumns: "minmax(0, 1fr)" } },
});
globalStyle(`${lessonEditor} .field, ${lessonEditor} .step-field`, {
  display: "flex",
  flexDirection: "column",
  gap: "4px",
  minWidth: 0,
});
globalStyle(`${lessonEditor} .steps-section`, {
  gridColumn: "1 / -1",
  display: "grid",
  gridTemplateColumns: "repeat(2, minmax(0, 1fr))",
  gap: "8px 12px",
  "@media": { "(max-width: 640px)": { gridTemplateColumns: "minmax(0, 1fr)" } },
});
globalStyle(`${lessonEditor} .steps-section h3`, { gridColumn: "1 / -1" });
globalStyle(`${lessonEditor} textarea`, {
  resize: "vertical",
  // The fields size themselves to their content (see AutoTextarea), so this is
  // only a floor for an empty one. It used to be 64px, which both wasted space
  // on short fields and still cut long generated text off.
  minHeight: "44px",
  // A grown field must not also scroll: the height already fits the text, and
  // a stray scrollbar would hide the last line again.
  overflowY: "hidden",
  padding: "8px",
  fontSize: "14px",
  lineHeight: 1.5,
});
globalStyle(`${lessonEditor} .char-count`, {
  alignSelf: "flex-end",
  fontSize: "11px",
  color: "var(--lumin-muted)",
});
globalStyle(`${lessonEditor} .lesson-plan-actions`, {
  display: "flex",
  justifyContent: "flex-end",
  gap: "8px",
  position: "sticky",
  bottom: 0,
  background: "var(--lumin-bg)",
  padding: "8px 0",
  flexWrap: "wrap",
});

// Generating a plan has no output to show until it is finished and validated,
// so the card itself carries the signal: a light sweep across the fields says
// work is happening even when the exact progress is not knowable.
const shimmer = keyframes({
  "0%": { backgroundPosition: "200% 0" },
  "100%": { backgroundPosition: "-200% 0" },
});

export const planGenerating = style({
  position: "relative",
  selectors: {
    // The sweep rides over the card without blocking what is underneath, so a
    // previous plan stays readable while the next one is produced.
    "&::after": {
      content: '""',
      position: "absolute",
      inset: 0,
      pointerEvents: "none",
      borderRadius: "inherit",
      backgroundImage:
        "linear-gradient(100deg, transparent 35%, var(--lumin-indigo-soft) 50%, transparent 65%)",
      backgroundSize: "200% 100%",
      animationName: shimmer,
      animationDuration: "1.8s",
      animationTimingFunction: "linear",
      animationIterationCount: "infinite",
      opacity: "0.9",
    },
  },
  "@media": {
    "(prefers-reduced-motion: reduce)": {
      selectors: {
        // No sweep, but the card still reads as busy rather than as finished.
        "&::after": {
          animation: "none",
          backgroundImage: "none",
          background: "var(--lumin-indigo-soft)",
          opacity: "0.25",
        },
      },
    },
  },
});

// The ring sits over the card, where the plan itself will appear, so starting
// and finishing a generation never moves anything on the page.
export const planOverlay = style({
  position: "absolute",
  top: "50%",
  left: "50%",
  transform: "translate(-50%, -50%)",
  zIndex: 1,
  display: "flex",
  alignItems: "center",
  maxWidth: "calc(100% - var(--space-6, 32px))",
  padding: "var(--space-3) var(--space-4)",
  background: "var(--lumin-card)",
  border: "1px solid var(--lumin-border)",
  borderRadius: "var(--radius-lg)",
  boxShadow: "var(--shadow-sm)",
  // Readable over an existing plan without taking the fields away from the
  // teacher: a regeneration does not have to block editing.
  pointerEvents: "none",
});
