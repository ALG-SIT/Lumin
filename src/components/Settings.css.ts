import { globalStyle, style } from "@vanilla-extract/css";

// Settings belong to the app, not to a role: the model the teacher picks is
// the one the students' hints run on. This screen is reachable from the bar
// at any point, including before a role has been chosen.

export const settings = style({
  display: "flex",
  flexDirection: "column",
  flex: "1 1 0",
  minHeight: 0,
  width: "100%",
  maxWidth: "1000px",
  margin: "0 auto",
  padding: "var(--space-5)",
  gap: "var(--space-4)",
  overflowY: "auto",
  "@media": {
    "(max-width: 640px)": {
      padding: "var(--space-3)",
    },
  },
});

export const settingsHeader = style({
  display: "flex",
  alignItems: "flex-start",
  justifyContent: "space-between",
  gap: "var(--space-3)",
});

globalStyle(`${settingsHeader} h2`, {
  margin: "0 0 var(--space-1) 0",
  fontSize: "1.5rem",
  color: "var(--lumin-ink)",
});

globalStyle(`${settingsHeader} p`, {
  margin: 0,
  color: "var(--lumin-text-secondary)",
  fontSize: "0.9375rem",
});

export const settingsSection = style({
  display: "flex",
  flexDirection: "column",
  gap: "var(--space-3)",
  padding: "var(--space-4)",
  background: "var(--lumin-card)",
  border: "1px solid var(--lumin-border)",
  borderRadius: "var(--radius-lg)",
  boxShadow: "var(--shadow-sm)",
});

globalStyle(`${settingsSection} h3`, {
  margin: 0,
  fontSize: "1.0625rem",
  color: "var(--lumin-ink)",
});

export const systemGrid = style({
  display: "grid",
  gridTemplateColumns: "auto 1fr",
  gap: "var(--space-2) var(--space-4)",
  margin: 0,
  fontSize: "0.875rem",
});

globalStyle(`${systemGrid} dt`, {
  color: "var(--lumin-muted)",
});

globalStyle(`${systemGrid} dd`, {
  margin: 0,
  color: "var(--lumin-ink)",
  overflowWrap: "anywhere",
  fontVariantNumeric: "tabular-nums",
});
