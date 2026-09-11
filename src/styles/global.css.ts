import { globalStyle, style } from "@vanilla-extract/css";

// Legacy design tokens emitted verbatim from src/index.css :root (L1-47).
// Literal name emission via globalStyle is required: createThemeContract /
// createGlobalTheme unconditionally append a file-scope hash suffix to the
// variable names, so the legacy `var(--lumin-*)` references would break.
// The `as any` cast is required: strict tsc rejects custom-property keys
// with TS2353 against the CSSProperties type.
globalStyle(":root", {
  "--lumin-primary": "var(--lumin-indigo)",
  "--lumin-purple": "#7c3aed",
  "--lumin-purple-light": "#a78bfa",
  "--lumin-purple-dark": "#5b21b6",
  "--lumin-purple-soft": "#ede9fe",
  "--lumin-indigo": "#4f46e5",
  "--lumin-indigo-light": "#818cf8",
  "--lumin-indigo-soft": "#e0e7ff",
  "--lumin-bg": "#fafbff",
  "--lumin-canvas": "#f6f8ff",
  "--lumin-surface": "#ffffff",
  "--lumin-card": "#ffffff",
  "--lumin-ink": "#1f2937",
  "--lumin-text": "#1f2937",
  "--lumin-text-secondary": "#6b7280",
  "--lumin-muted": "#6b7280",
  "--lumin-border": "#e5e7eb",
  "--lumin-success": "#10b981",
  "--lumin-warning": "#f59e0b",
  "--lumin-error": "#ef4444",
  "--lumin-radius": "12px",
  "--lumin-radius-lg": "16px",
  "--lumin-shadow":
    "0 1px 3px rgba(0, 0, 0, 0.1), 0 1px 2px rgba(0, 0, 0, 0.06)",
  "--color-ink": "var(--lumin-ink)",
  "--color-paper": "var(--lumin-card)",
  "--color-muted": "var(--lumin-muted)",
  "--color-teal": "var(--lumin-indigo)",
  "--color-teal-soft": "var(--lumin-indigo-soft)",
  "--color-amber": "var(--lumin-purple)",
  "--color-coral": "var(--lumin-error)",
  "--color-success": "var(--lumin-success)",
  "--radius-sm": "8px",
  "--radius-md": "var(--lumin-radius)",
  "--radius-lg": "var(--lumin-radius-lg)",
  "--space-1": "0.25rem",
  "--space-2": "0.5rem",
  "--space-3": "0.75rem",
  "--space-4": "1rem",
  "--space-5": "1.25rem",
  "--space-6": "1.5rem",
  "--shadow-sm": "0 1px 2px rgba(0, 0, 0, 0.05)",
  "--shadow-md": "0 4px 6px -1px rgba(0, 0, 0, 0.1)",
  "--shadow-purple": "0 9px 22px rgba(124, 58, 237, 0.08)",
  // biome-ignore lint/suspicious/noExplicitAny: vanilla-extract がカスタムプロパティキーを型付けせず tsc が TS2353 を出すため、意図的なキャスト（上記コメント参照）
} as any);

// Element resets translated verbatim from src/index.css.
globalStyle("*", {
  boxSizing: "border-box",
  margin: "0",
  padding: "0",
});

globalStyle("html, body, #root", {
  height: "100%",
});

globalStyle("body", {
  fontFamily:
    "'Noto Sans JP Variable', -apple-system, BlinkMacSystemFont, 'Hiragino Sans', 'Hiragino Kaku Gothic ProN', 'Yu Gothic', sans-serif",
  background: "var(--lumin-bg)",
  color: "var(--lumin-text)",
  minHeight: "100vh",
  lineHeight: "1.5",
  WebkitFontSmoothing: "antialiased",
  MozOsxFontSmoothing: "grayscale",
});

globalStyle("input, button, textarea", {
  fontFamily: "inherit",
});

globalStyle("input, textarea", {
  width: "100%",
  padding: "10px 12px",
  border: "1px solid var(--lumin-border)",
  borderRadius: "8px",
  fontSize: "14px",
  outline: "none",
  color: "var(--lumin-text)",
  background: "var(--lumin-surface)",
  transition: "border-color 0.2s ease, box-shadow 0.2s ease",
});

globalStyle("main", {
  minHeight: 0,
  overflowY: "auto",
  flex: "1",
  padding: "16px",
  maxWidth: "1280px",
  margin: "0 auto",
  width: "100%",
  "@media": {
    "(max-width: 768px)": {
      padding: "12px",
    },
    "(max-width: 480px)": {
      padding: "10px",
    },
  },
});

globalStyle("h1", {
  fontSize: "2em",
  marginBottom: "0.5em",
});

// Pseudo rules cannot live in globalStyle (the API forbids simple/complex
// pseudo selectors). Values verbatim from src/index.css L257-266; the
// input/textarea variants share identical declarations, so one :focus and
// one ::placeholder cover both. Connected in later batches (T6 manual-input
// -> T7 teacher-chat-input -> T8 answer-input/code-input/StudentJoin
// inputs). Do NOT attach to anything yet.
export const baseInputFocus = style({
  ":focus": {
    borderColor: "var(--lumin-purple)",
    boxShadow: "0 0 0 3px rgba(124, 58, 237, 0.1)",
  },
  "::placeholder": {
    color: "var(--lumin-text-secondary)",
  },
});
