import { keyframes, style } from "@vanilla-extract/css";
import { appBar } from "./App.css.ts";

// The model is app-wide state, not a screen's state, so its status lives in
// the bar. The chip is a button: the thing you want after seeing "未導入" is
// the place to fix it.

const pulse = keyframes({
  "0%, 100%": { opacity: "1" },
  "50%": { opacity: "0.5" },
});

export const modelChip = style({
  selectors: {
    [`${appBar} &`]: {
      display: "flex",
      flexDirection: "column",
      alignItems: "flex-start",
      gap: "3px",
      minWidth: 0,
      maxWidth: "min(42vw, 320px)",
      padding: "5px 12px",
      background: "rgba(255, 255, 255, 0.16)",
      border: "1px solid rgba(255, 255, 255, 0.28)",
      borderRadius: "9999px",
      color: "white",
      cursor: "pointer",
      font: "inherit",
      textAlign: "left",
      transition: "background 0.2s ease",
    },
    [`${appBar} &:hover`]: {
      background: "rgba(255, 255, 255, 0.28)",
    },
  },
  "@media": {
    "(prefers-reduced-motion: reduce)": {
      selectors: {
        [`${appBar} &`]: { transition: "none" },
      },
    },
  },
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
  fontSize: "0.6875rem",
  whiteSpace: "nowrap",
  color: "rgba(255, 255, 255, 0.85)",
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
    '&[data-state="missing"]': { background: "rgba(255, 255, 255, 0.5)" },
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
  background: "rgba(255, 255, 255, 0.25)",
  borderRadius: "9999px",
  overflow: "hidden",
});

export const modelChipFill = style({
  height: "100%",
  background: "white",
  borderRadius: "9999px",
  transition: "width 0.3s ease",
  "@media": {
    "(prefers-reduced-motion: reduce)": { transition: "none" },
  },
});
