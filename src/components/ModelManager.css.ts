import { globalStyle, style } from "@vanilla-extract/css";
import { modelManager } from "../styles/shared.css.ts";

// ModelManager.tsx batch (plan checkbox 6). Declaration values are verbatim
// copies of the corresponding src/index.css rules (base rule + every
// descendant-combined form), emitted in index.css order so the cascade is
// order-equivalent.
//
// The wrapper class comes from the T3 shared primitive (`modelManager`,
// index.css L543-547 + @media 768 L1563-1567). The primitive intentionally
// excludes the ancestor-form `.model-manager h2` rule (L549-553), which is
// re-homed below with a `${modelManager} &` cross-reference.
//
// progress-bar / progress-fill / error-message are wired from the T3
// primitives; their index.css rules STAY because StudentQuiz / StudentJoin
// still use the literal classes (their deletion is T8's job).

// index.css L549-553 (`.model-manager h2`, ancestor-form rule styling the
// unclassed h2). Re-homed per the T3 primitive note; emits
// `.modelManagerHash h2` — the original selector with the hashed class.
globalStyle(`${modelManager} h2`, {
  margin: "0 0 var(--space-5) 0",
  fontSize: "1.5rem",
  color: "var(--color-ink)",
});

// index.css L265-275.
export const modelCard = style({
  display: "grid",
  gridTemplateColumns: "1fr auto auto",
  alignItems: "center",
  gap: "var(--space-4)",
  padding: "var(--space-5)",
  marginBottom: "var(--space-4)",
  background: "var(--color-paper)",
  borderRadius: "var(--radius-lg)",
  boxShadow: "var(--shadow-sm)",
});

// index.css L277-280 (`.model-card .model-name`). Compound selector
// preserves the original (0,2,0) specificity.
export const modelName = style({
  selectors: {
    [`${modelCard} &`]: {
      fontWeight: "600",
      color: "var(--color-ink)",
    },
  },
});

// index.css L282-289 (`.model-card .model-meta`). Compound selector
// preserves the original (0,2,0) specificity.
export const modelMeta = style({
  selectors: {
    [`${modelCard} &`]: {
      display: "flex",
      flexDirection: "column",
      alignItems: "flex-end",
      gap: "var(--space-1)",
      color: "var(--color-muted)",
      fontSize: "0.875rem",
    },
  },
});

// index.css L291-299.
export const badge = style({
  display: "inline-block",
  padding: "var(--space-1) var(--space-2)",
  borderRadius: "9999px",
  background: "var(--lumin-purple-soft)",
  color: "var(--lumin-purple)",
  fontSize: "0.75rem",
  fontWeight: "700",
});

// index.css L301-308.
export const statusBadge = style({
  display: "inline-flex",
  alignItems: "center",
  gap: "var(--space-1)",
  color: "var(--color-success)",
  fontSize: "0.875rem",
  fontWeight: "600",
});

// index.css L310-314.
export const modelActions = style({
  display: "flex",
  alignItems: "center",
  gap: "var(--space-3)",
});

// index.css L316-325 (`.model-actions button`). The buttons are UNCLASSED in
// ModelManager.tsx, so the element-descendant rules are encoded via
// globalStyle with the interpolated hashed class — emitting exactly the
// original selector shape and preserving the (0,1,1) specificity over the
// `input, button, textarea` element resets.
globalStyle(`${modelActions} button`, {
  background: "var(--lumin-indigo)",
  color: "white",
  border: "none",
  padding: "8px 14px",
  borderRadius: "var(--radius-md)",
  cursor: "pointer",
  fontWeight: "600",
  transition: "background 0.2s ease",
});

// index.css L327-329.
globalStyle(`${modelActions} button:hover:not(:disabled)`, {
  background: "#4338ca",
});

// index.css L331-334.
globalStyle(`${modelActions} button:disabled`, {
  opacity: "0.5",
  cursor: "not-allowed",
});

// index.css L350-354.
export const importSection = style({
  marginTop: "var(--space-6)",
  paddingTop: "var(--space-5)",
  borderTop: "1px solid #e5e7eb",
});

// index.css L356-359 (`.import-section h3`, unclassed h3).
globalStyle(`${importSection} h3`, {
  margin: "0 0 var(--space-3) 0",
  fontSize: "1.125rem",
});

// index.css L361-370 (`.import-section button`, unclassed button).
globalStyle(`${importSection} button`, {
  background: "var(--lumin-purple)",
  color: "white",
  border: "none",
  padding: "10px 18px",
  borderRadius: "var(--radius-md)",
  cursor: "pointer",
  fontWeight: "600",
  transition: "background 0.2s ease",
});

// index.css L372-374.
globalStyle(`${importSection} button:hover:not(:disabled)`, {
  background: "var(--lumin-purple-dark)",
});

// index.css L376-379.
globalStyle(`${importSection} button:disabled`, {
  opacity: "0.5",
  cursor: "not-allowed",
});

// index.css L381-385.
export const importHint = style({
  color: "var(--color-muted)",
  fontSize: "0.875rem",
  marginBottom: "var(--space-3)",
});

// --- Model selection (Gemma 3 / 3n / 4 variants) ---------------------------
// New surface: the list is grouped by family and one variant is the active
// model, so these have no index.css ancestor to mirror.

export const familyGroup = style({
  marginBottom: "var(--space-5)",
});

export const familyHeading = style({
  margin: "0 0 var(--space-3) 0",
  fontSize: "0.8125rem",
  fontWeight: "700",
  letterSpacing: "0.06em",
  textTransform: "uppercase",
  color: "var(--color-muted)",
});

// The active model is outlined rather than filled, so the "installed" and
// "in use" states stay readable at the same time.
export const activeCard = style({
  outline: "2px solid var(--lumin-indigo)",
  outlineOffset: "-2px",
});

export const activeBadge = style({
  display: "inline-block",
  marginLeft: "var(--space-2)",
  padding: "var(--space-1) var(--space-2)",
  borderRadius: "9999px",
  background: "var(--lumin-indigo)",
  color: "white",
  fontSize: "0.75rem",
  fontWeight: "700",
});

export const modelDescription = style({
  marginTop: "var(--space-1)",
  color: "var(--color-muted)",
  fontSize: "0.875rem",
  fontWeight: "400",
});
