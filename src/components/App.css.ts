import { globalStyle, style } from "@vanilla-extract/css";

// App.tsx batch (plan checkbox 4). Declaration values are verbatim copies of
// the corresponding src/index.css rules (base rule + every descendant-combined
// form + @media portions). Cascade invariant: each class's @media overrides
// live in the same style() call as its base rule, and declaration order
// matches index.css within each rule.

// index.css L78-82.
export const app = style({
  display: "flex",
  flexDirection: "column",
  position: "fixed",
  top: "var(--viewport-top, 0px)",
  left: 0,
  width: "100%",
  height: "var(--viewport-height, 100dvh)",
  paddingTop: "env(safe-area-inset-top, 0px)",
  paddingBottom: "env(safe-area-inset-bottom, 0px)",
  paddingLeft: "env(safe-area-inset-left, 0px)",
  paddingRight: "env(safe-area-inset-right, 0px)",
  overflow: "hidden",
  selectors: { ":root[data-keyboard-open] &": { paddingBottom: 0 } },
});

export const appBar = style({
  flexShrink: 0,
  display: "flex",
  alignItems: "center",
  justifyContent: "space-between",
  gap: "8px",
  minHeight: "48px",
  padding: "2px 12px",
  color: "var(--lumin-ink)",
  background: "var(--lumin-bg)",
});

export const workArea = style({
  height: "100%",
  minHeight: 0,
  overflowY: "auto",
  overscrollBehavior: "contain",
  selectors: { "&[hidden]": { display: "none" } },
});

export const iconButton = style({
  display: "inline-flex",
  flexShrink: 0,
  alignItems: "center",
  justifyContent: "center",
  width: "44px",
  height: "44px",
  padding: 0,
  color: "var(--lumin-ink)",
  background: "var(--lumin-card)",
  border: "1px solid var(--lumin-border)",
  borderRadius: "50%",
  cursor: "pointer",
  ":focus-visible": {
    outline: "2px solid var(--lumin-indigo)",
    outlineOffset: "2px",
  },
});
export const actionTrigger = style([
  iconButton,
  { color: "var(--lumin-indigo)" },
]);
export const roleHome = style([
  iconButton,
  {
    width: "80px",
    gap: "6px",
    borderRadius: "22px",
    fontSize: "13px",
    selectors: { "&:not(button)": { visibility: "hidden" } },
  },
]);
export const toolbarTitle = style({
  flex: 1,
  minWidth: 0,
  paddingRight: "36px",
  textAlign: "center",
  fontWeight: 600,
  fontSize: "15px",
});
export const actionDialog = style({
  position: "fixed",
  inset: 0,
  margin: "auto",
  border: "1px solid var(--lumin-border)",
  borderRadius: "28px",
  padding: "16px",
  width: "min(440px, calc(100% - 24px))",
  maxHeight: "calc(100dvh - 80px)",
  overflowY: "auto",
  color: "var(--lumin-ink)",
  background: "var(--lumin-card)",
  selectors: {
    "&[open]": { display: "flex", flexDirection: "column", gap: "16px" },
    "&::backdrop": { background: "rgba(15, 23, 42, 0.35)" },
  },
});
export const dialogHeading = style({
  display: "flex",
  justifyContent: "space-between",
  alignItems: "center",
  gap: "12px",
});
globalStyle(`${dialogHeading} h2`, { margin: 0, fontSize: "1.125rem" });
