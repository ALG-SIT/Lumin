import { keyframes, style } from "@vanilla-extract/css";

// The model is app-wide state, not a screen's state, so its status lives in
// the bar. The chip is a button: the thing you want after seeing "未導入" is
// the place to fix it.

const pulse = keyframes({
  "0%, 100%": { opacity: "1" },
  "50%": { opacity: "0.5" },
});

export const modelChip = style({
  display: "flex",
  flexDirection: "column",
  alignItems: "flex-start",
  gap: "4px",
  minWidth: 0,
  width: "100%",
  minHeight: "60px",
  padding: "12px",
  color: "var(--lumin-ink)",
  background: "var(--lumin-canvas)",
  border: "1px solid var(--lumin-border)",
  borderRadius: "12px",
  cursor: "pointer",
  font: "inherit",
  textAlign: "left",
  ":hover": { background: "var(--lumin-indigo-soft)" },
});

export const modelChipRow = style({
  display: "flex",
  alignItems: "center",
  gap: "var(--space-2)",
  maxWidth: "100%",
});

export const modelChipName = style({
  fontSize: "0.8125rem",
  fontWeight: 600,
  whiteSpace: "nowrap",
  overflow: "hidden",
  textOverflow: "ellipsis",
});

export const modelChipState = style({
  maxWidth: "100%",
  overflow: "hidden",
  textOverflow: "ellipsis",
  fontSize: "0.6875rem",
  whiteSpace: "nowrap",
  color: "var(--lumin-muted)",
});

/** Green when usable, amber while loading, grey when not installed. */
export const modelChipDot = style({
  flexShrink: 0,
  width: "8px",
  height: "8px",
  borderRadius: "50%",
  selectors: {
    '&[data-state="ready"]': { background: "#4ade80" },
    '&[data-state="loading"]': {
      background: "#fbbf24",
      animationName: pulse,
      animationDuration: "1.2s",
      animationTimingFunction: "ease-in-out",
      animationIterationCount: "infinite",
    },
    '&[data-state="missing"]': { background: "#94a3b8" },
    '&[data-state="error"]': { background: "#f87171" },
  },
  "@media": {
    "(prefers-reduced-motion: reduce)": {
      selectors: {
        '&[data-state="loading"]': { animation: "none" },
      },
    },
  },
});

/** Thin determinate bar, shown only while the model is being read in. */
export const modelChipTrack = style({
  width: "100%",
  height: "3px",
  background: "var(--lumin-border)",
  borderRadius: "9999px",
  overflow: "hidden",
});

export const modelChipFill = style({
  height: "100%",
  background: "var(--lumin-indigo)",
  borderRadius: "9999px",
  transition: "width 0.3s ease",
  "@media": {
    "(prefers-reduced-motion: reduce)": { transition: "none" },
  },
});
