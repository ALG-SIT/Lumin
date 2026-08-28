import { globalStyle, style } from "@vanilla-extract/css";

// DemoFlow.tsx batch (plan checkbox 8, last component batch). Declaration
// values are verbatim copies of the corresponding src/index.css rules.
//
// Unclassed descendants (h2/h3/h4/ul/li/strong) are encoded via globalStyle
// with interpolated refs — emits the identical selector shape with the hashed
// class. The reset-button literal is wired to the T3 resetButton primitive:
// every one of its rules is '.app-bar'-conditioned and DemoFlow renders
// outside the app bar, so the hashed class never matches and the button stays
// intentionally unstyled — identical to the literal-class rendering before
// this batch (T4 decision, unchanged).
//
// demo-step state variants: the TSX template combines demoStep +
// demoStepActive/demoStepDone (same-element compounds in index.css). The
// variant consts are declared AFTER demoStep so their same-specificity
// overrides win by source order, mirroring the original (0,2,0)-over-(0,1,0)
// cascade. The indicator color variants are cross-referenced off the variant
// consts ('.demo-step.active .demo-step-indicator' shape); active and done
// are mutually exclusive in the TSX logic, so no ordering conflict exists
// between them.

// index.css L716-720.
export const demoFlow = style({
  maxWidth: "640px",
  margin: "0 auto",
  padding: "var(--space-6)",
});

// index.css L722-726 (.demo-flow h2 — unclassed descendant).
globalStyle(`${demoFlow} h2`, {
  margin: "0 0 var(--space-2)",
  fontSize: "1.5rem",
  color: "var(--lumin-ink)",
});

// index.css L728-732.
export const demoSubtitle = style({
  margin: "0 0 var(--space-5)",
  color: "var(--lumin-text-secondary)",
  fontSize: "0.9rem",
});

// index.css L734-739.
export const demoIntro = style({
  display: "flex",
  flexDirection: "column",
  alignItems: "center",
  gap: "var(--space-3)",
});

// index.css L741-750 + :hover L752-754.
export const demoStartButton = style({
  padding: "0.75rem 2rem",
  fontSize: "1.1rem",
  fontWeight: "600",
  border: "none",
  borderRadius: "var(--radius-md)",
  background: "var(--lumin-primary)",
  color: "#fff",
  cursor: "pointer",
  selectors: {
    "&:hover": {
      opacity: "0.9",
    },
  },
});

// index.css L756-760.
export const demoProgress = style({
  display: "flex",
  flexDirection: "column",
  gap: "var(--space-3)",
});

// index.css L762-772.
export const demoStep = style({
  display: "flex",
  alignItems: "flex-start",
  gap: "var(--space-3)",
  padding: "var(--space-3)",
  borderRadius: "var(--radius-md)",
  background: "var(--lumin-card)",
  border: "1px solid var(--lumin-border)",
  opacity: "0.5",
  transition: "opacity 0.3s",
});

// index.css L774-778 (.demo-step.active). Declared after demoStep so the
// same-specificity overrides win by source order.
export const demoStepActive = style({
  opacity: "1",
  borderColor: "var(--lumin-primary)",
  boxShadow:
    "0 0 0 2px color-mix(in srgb, var(--lumin-primary) 20%, transparent)",
});

// index.css L780-782 (.demo-step.done).
export const demoStepDone = style({
  opacity: "0.7",
});

// index.css L784-790 + L792-794 (.demo-step.active .demo-step-indicator) +
// L796-798 (.demo-step.done .demo-step-indicator).
export const demoStepIndicator = style({
  flexShrink: "0",
  width: "1.5rem",
  textAlign: "center",
  fontSize: "1rem",
  color: "var(--lumin-text-secondary)",
  selectors: {
    [`${demoStepActive} &`]: {
      color: "var(--lumin-primary)",
    },
    [`${demoStepDone} &`]: {
      color: "var(--lumin-success, #22c55e)",
    },
  },
});

// index.css L800-804.
export const demoStepText = style({
  display: "flex",
  flexDirection: "column",
  gap: "var(--space-1)",
});

// index.css L806-809 (.demo-step-text strong — unclassed descendant).
globalStyle(`${demoStepText} strong`, {
  fontSize: "0.95rem",
  color: "var(--lumin-ink)",
});

// index.css L811-814.
export const demoStepDetail = style({
  fontSize: "0.85rem",
  color: "var(--lumin-text-secondary)",
});

// index.css L816-823 (demo-summary share of the band).
export const demoSummary = style({
  marginTop: "var(--space-5)",
  padding: "var(--space-5)",
  background: "var(--lumin-card)",
  border: "1px solid var(--lumin-border)",
  borderRadius: "var(--radius-lg)",
});

// index.css L816-823 (demo-lesson-plan share of the band).
export const demoLessonPlan = style({
  marginTop: "var(--space-5)",
  padding: "var(--space-5)",
  background: "var(--lumin-card)",
  border: "1px solid var(--lumin-border)",
  borderRadius: "var(--radius-lg)",
});

// index.css L825-830 (h3 shares — unclassed descendants).
globalStyle(`${demoSummary} h3`, {
  margin: "0 0 var(--space-3)",
  fontSize: "1.1rem",
  color: "var(--lumin-ink)",
});

globalStyle(`${demoLessonPlan} h3`, {
  margin: "0 0 var(--space-3)",
  fontSize: "1.1rem",
  color: "var(--lumin-ink)",
});

// index.css L832-836.
export const demoStats = style({
  display: "grid",
  gridTemplateColumns: "repeat(auto-fit, minmax(120px, 1fr))",
  gap: "var(--space-3)",
});

// index.css L838-845.
export const demoStat = style({
  display: "flex",
  flexDirection: "column",
  alignItems: "center",
  padding: "var(--space-3)",
  background: "var(--lumin-bg)",
  borderRadius: "var(--radius-md)",
});

// index.css L847-850.
export const demoStatLabel = style({
  fontSize: "0.8rem",
  color: "var(--lumin-text-secondary)",
});

// index.css L852-856.
export const demoStatValue = style({
  fontSize: "1.3rem",
  fontWeight: "700",
  color: "var(--lumin-ink)",
});

// index.css L858-860.
export const demoMisconceptions = style({
  marginTop: "var(--space-4)",
});

// index.css L862-866 / L868-871 / L873-877 (unclassed descendants).
globalStyle(`${demoMisconceptions} h4`, {
  margin: "0 0 var(--space-2)",
  fontSize: "0.95rem",
  color: "var(--lumin-ink)",
});

globalStyle(`${demoMisconceptions} ul`, {
  margin: "0",
  paddingLeft: "1.5rem",
});

globalStyle(`${demoMisconceptions} li`, {
  marginBottom: "var(--space-1)",
  fontSize: "0.9rem",
  color: "var(--lumin-text)",
});

// index.css L879-883.
export const demoPlanFocus = style({
  marginBottom: "var(--space-3)",
  fontSize: "0.95rem",
  color: "var(--lumin-ink)",
});

// index.css L885-888.
export const demoPlanSteps = style({
  margin: "0 0 var(--space-3)",
  paddingLeft: "1.5rem",
});

// index.css L890-894 (unclassed descendant).
globalStyle(`${demoPlanSteps} li`, {
  marginBottom: "var(--space-2)",
  fontSize: "0.9rem",
  color: "var(--lumin-text)",
});

// index.css L896-904 (demo-plan-check share of the band).
export const demoPlanCheck = style({
  marginTop: "var(--space-3)",
  fontSize: "0.9rem",
  color: "var(--lumin-text)",
  padding: "var(--space-3)",
  background: "var(--lumin-bg)",
  borderRadius: "var(--radius-md)",
});

// index.css L896-904 (demo-plan-note share of the band).
export const demoPlanNote = style({
  marginTop: "var(--space-3)",
  fontSize: "0.9rem",
  color: "var(--lumin-text)",
  padding: "var(--space-3)",
  background: "var(--lumin-bg)",
  borderRadius: "var(--radius-md)",
});

// index.css L906-910.
export const demoComplete = style({
  textAlign: "center",
  marginTop: "var(--space-5)",
  color: "var(--lumin-text-secondary)",
});

// index.css L912-920.
export const demoError = style({
  marginTop: "var(--space-4)",
  padding: "var(--space-3)",
  background: "#fef2f2",
  border: "1px solid #fecaca",
  borderRadius: "var(--radius-md)",
  color: "#991b1b",
  fontSize: "0.9rem",
});
