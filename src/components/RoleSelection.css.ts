import { globalStyle, style } from "@vanilla-extract/css";

// RoleSelection.tsx batch (plan checkbox 4). Declaration values are verbatim
// copies of the corresponding src/index.css rules (base rule + every
// descendant-combined form + @media portions + reduced-motion portions).
// Cascade invariant: each class's @media overrides live in the same style()
// call as its base rule, and declaration order matches index.css within each
// rule.

// index.css L389-401 + L1555-1557 (@media max-width: 768px).
export const roleSelection = style({
  display: "flex",
  flexDirection: "column",
  alignItems: "center",
  justifyContent: "center",
  minHeight: "100%",
  padding: "var(--space-6)",
  textAlign: "center",
  background:
    "radial-gradient(circle at 10% 20%, rgba(124, 58, 237, 0.06), transparent 25%), radial-gradient(circle at 90% 80%, rgba(79, 70, 229, 0.05), transparent 25%), var(--lumin-bg)",
  "@media": {
    "(max-width: 768px)": {
      padding: "var(--space-4)",
    },
  },
});

// index.css L403-413.
export const appTitle = style({
  display: "flex",
  alignItems: "center",
  justifyContent: "center",
  gap: "var(--space-3)",
  margin: "0 0 var(--space-3) 0",
  fontSize: "clamp(2.5rem, 8vw, 4rem)",
  fontWeight: "900",
  letterSpacing: "0.05em",
  color: "var(--lumin-ink)",
});

// index.css L415-419 (`.app-title .title-icon`).
export const titleIcon = style({
  selectors: {
    [`${appTitle} &`]: {
      width: "clamp(2rem, 6vw, 3rem)",
      height: "clamp(2rem, 6vw, 3rem)",
      color: "var(--lumin-purple)",
    },
  },
});

// index.css L421-427.
export const tagline = style({
  margin: "0 0 var(--space-6) 0",
  fontSize: "clamp(1.125rem, 3vw, 1.5rem)",
  fontWeight: "500",
  color: "var(--lumin-text-secondary)",
  maxWidth: "640px",
});

// index.css L429-436.
export const roleButtons = style({
  display: "flex",
  flexWrap: "wrap",
  justifyContent: "center",
  gap: "var(--space-4)",
  width: "100%",
  maxWidth: "720px",
});

// index.css L438-452, L454-458 (:hover), L460-462 (:active),
// L1559-1561 (@media max-width: 768px), L1653-1655 (@media
// prefers-reduced-motion: reduce portions — the bare/:hover/:active forms all
// get transition:none + transform:none, so the :active scale is killed under
// reduced motion exactly like the original same-specificity later rule).
export const roleButton = style({
  flex: "1 1 220px",
  display: "flex",
  flexDirection: "column",
  alignItems: "flex-start",
  gap: "var(--space-3)",
  padding: "var(--space-5)",
  textAlign: "left",
  background: "var(--lumin-card)",
  border: "1px solid var(--lumin-border)",
  borderRadius: "var(--radius-lg)",
  boxShadow: "var(--shadow-sm)",
  cursor: "pointer",
  transition:
    "transform 0.2s ease, box-shadow 0.2s ease, border-color 0.2s ease",
  ":hover": {
    borderColor: "var(--lumin-purple)",
    boxShadow: "var(--shadow-purple)",
    transform: "translateY(-2px)",
  },
  ":active": {
    transform: "scale(0.99)",
  },
  "@media": {
    "(max-width: 768px)": {
      flex: "1 1 100%",
    },
    "(prefers-reduced-motion: reduce)": {
      transition: "none",
      transform: "none",
      selectors: {
        "&:hover": {
          transition: "none",
          transform: "none",
        },
        "&:active": {
          transition: "none",
          transform: "none",
        },
      },
    },
  },
});

// index.css L464-471 (`.role-button .role-icon`).
export const roleIcon = style({
  selectors: {
    [`${roleButton} &`]: {
      width: "48px",
      height: "48px",
      padding: "var(--space-2)",
      color: "var(--lumin-purple)",
      background: "var(--lumin-purple-soft)",
      borderRadius: "var(--radius-md)",
    },
  },
});

// index.css L473-478 (`.role-button .role-title`).
export const roleTitle = style({
  selectors: {
    [`${roleButton} &`]: {
      margin: "0",
      fontSize: "1.25rem",
      fontWeight: "700",
      color: "var(--lumin-ink)",
    },
  },
});

// index.css L480-485 (`.role-button .role-detail`).
export const roleDetail = style({
  selectors: {
    [`${roleButton} &`]: {
      margin: "0",
      fontSize: "0.9375rem",
      lineHeight: "1.5",
      color: "var(--lumin-text-secondary)",
    },
  },
});

// index.css L487-494 (`.role-button .role-action`).
export const roleAction = style({
  selectors: {
    [`${roleButton} &`]: {
      display: "flex",
      alignItems: "center",
      gap: "var(--space-2)",
      marginTop: "auto",
      fontWeight: "600",
      color: "var(--lumin-purple)",
    },
  },
});

// index.css L496-499 (`.role-button .role-action svg`). VE selectors must
// target the style's own class in their final compound, so an unclassed
// descendant like this svg cannot be a `selectors` entry — globalStyle with
// interpolated class refs emits the identical selector. Order-independent:
// this rule and roleAction target different elements.
globalStyle(`${roleButton} ${roleAction} svg`, {
  width: "18px",
  height: "18px",
});

// index.css L2588-2598.
export const trustLabels = style({
  display: "flex",
  flexWrap: "wrap",
  justifyContent: "center",
  gap: "var(--space-2)",
  listStyle: "none",
  padding: "0",
  marginTop: "var(--space-4)",
  color: "var(--lumin-text-secondary)",
  fontSize: "0.85rem",
});

// Keep all three entry points discoverable on a phone without oversized cards.
const phone = "(max-width: 640px)";
globalStyle(`${roleSelection}`, {
  "@media": { [phone]: { padding: "12px", justifyContent: "flex-start" } },
});
globalStyle(`${roleButtons}`, {
  "@media": { [phone]: { gap: "12px" } },
});
globalStyle(`${roleButton}`, {
  "@media": {
    [phone]: {
      flex: "1 1 100%",
      display: "grid",
      gridTemplateColumns: "36px minmax(0, 1fr)",
      gap: "6px 12px",
      padding: "12px",
    },
  },
});
globalStyle(`${roleButton} ${roleIcon}`, {
  "@media": { [phone]: { width: "36px", height: "36px", gridRow: "1 / 3" } },
});
globalStyle(`${roleButton} ${roleTitle}`, {
  "@media": { [phone]: { fontSize: "1.0625rem" } },
});
globalStyle(`${roleButton} ${roleDetail}, ${roleButton} ${roleAction}`, {
  "@media": { [phone]: { gridColumn: 2 } },
});
globalStyle(`${trustLabels}`, {
  "@media": { [phone]: { marginTop: 0, gap: "4px", fontSize: "0.75rem" } },
});
