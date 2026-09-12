import { keyframes, style } from "@vanilla-extract/css";

// Progress for on-device waits is drawn as a ring placed where the result
// itself will appear. A bar above the content had to appear and disappear,
// which resized the chat area and pushed the lesson plan around; a ring in the
// destination occupies space that is already spoken for.

const spin = keyframes({
  to: { transform: "rotate(360deg)" },
});

export const ring = style({
  display: "flex",
  alignItems: "center",
  gap: "var(--space-3)",
  minWidth: 0,
});

export const ringSvg = style({
  flexShrink: 0,
  width: "28px",
  height: "28px",
  // 12 o'clock start, so a filling ring reads as a clock face.
  transform: "rotate(-90deg)",
});

export const ringTrack = style({
  fill: "none",
  stroke: "var(--lumin-border)",
  strokeWidth: 3,
});

export const ringIndicator = style({
  fill: "none",
  stroke: "var(--lumin-indigo)",
  strokeWidth: 3,
  strokeLinecap: "round",
  transition: "stroke-dashoffset 0.3s ease",
  "@media": {
    "(prefers-reduced-motion: reduce)": {
      transition: "none",
    },
  },
});

/** No fraction to show: a fixed arc sweeps instead of filling. */
export const ringSpinner = style({
  transformOrigin: "50% 50%",
  animationName: spin,
  animationDuration: "1.1s",
  animationTimingFunction: "linear",
  animationIterationCount: "infinite",
  "@media": {
    "(prefers-reduced-motion: reduce)": {
      animation: "none",
      opacity: "0.5",
    },
  },
});

export const ringText = style({
  display: "flex",
  flexDirection: "column",
  gap: "2px",
  minWidth: 0,
});

export const ringLabel = style({
  fontSize: "0.875rem",
  color: "var(--lumin-text-secondary)",
  overflowWrap: "anywhere",
});

export const ringElapsed = style({
  fontSize: "0.75rem",
  color: "var(--lumin-muted)",
  fontVariantNumeric: "tabular-nums",
});
