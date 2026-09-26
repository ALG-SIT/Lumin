import { globalStyle, style } from "@vanilla-extract/css";

// Styling for rendered Markdown replies. The chat bubble sets pre-wrap so a
// plain reply keeps its line breaks; inside rendered Markdown the block
// elements own the spacing instead, so the container resets it and the leaf
// text nodes opt back in.
export const markdown = style({
  whiteSpace: "normal",
  display: "flex",
  flexDirection: "column",
  gap: "var(--space-2)",
});

globalStyle(`${markdown} > *:first-child`, {
  marginTop: 0,
});

globalStyle(`${markdown} > *:last-child`, {
  marginBottom: 0,
});

globalStyle(`${markdown} p`, {
  margin: 0,
  whiteSpace: "pre-wrap",
});

globalStyle(`${markdown} h3, ${markdown} h4, ${markdown} h5, ${markdown} h6`, {
  margin: "var(--space-2) 0 0 0",
  lineHeight: 1.35,
  fontWeight: 700,
});

globalStyle(`${markdown} h3`, { fontSize: "1.0625rem" });
globalStyle(`${markdown} h4`, { fontSize: "1rem" });
globalStyle(`${markdown} h5, ${markdown} h6`, { fontSize: "0.9375rem" });

globalStyle(`${markdown} ul, ${markdown} ol`, {
  margin: 0,
  paddingLeft: "1.4em",
  display: "flex",
  flexDirection: "column",
  gap: "var(--space-1)",
});

globalStyle(`${markdown} li`, {
  whiteSpace: "pre-wrap",
});

globalStyle(`${markdown} strong`, {
  fontWeight: 700,
});

globalStyle(`${markdown} a`, {
  color: "var(--lumin-indigo)",
  textDecoration: "underline",
});

// Inline code inside a reply.
export const markdownCode = style({
  padding: "0.1em 0.35em",
  fontSize: "0.875em",
  fontFamily:
    "ui-monospace, SFMono-Regular, Menlo, Consolas, 'Noto Sans Mono', monospace",
  background: "var(--lumin-canvas)",
  border: "1px solid var(--lumin-border)",
  borderRadius: "var(--radius-sm, 4px)",
  wordBreak: "break-word",
});

export const markdownCodeBlock = style({
  margin: 0,
  padding: "var(--space-3)",
  // Long lines scroll inside the block rather than widening the bubble.
  overflowX: "auto",
  fontSize: "0.8125rem",
  lineHeight: 1.5,
  fontFamily:
    "ui-monospace, SFMono-Regular, Menlo, Consolas, 'Noto Sans Mono', monospace",
  background: "var(--lumin-canvas)",
  border: "1px solid var(--lumin-border)",
  borderRadius: "var(--radius-md)",
});

export const markdownBlockquote = style({
  margin: 0,
  padding: "var(--space-2) var(--space-3)",
  borderLeft: "3px solid var(--lumin-indigo-soft)",
  color: "var(--lumin-text-secondary)",
});

export const markdownHr = style({
  width: "100%",
  margin: "var(--space-1) 0",
  border: "none",
  borderTop: "1px solid var(--lumin-border)",
});

export const markdownTable = style({
  width: "100%",
  display: "block",
  overflowX: "auto",
  borderCollapse: "collapse",
  fontSize: "0.9375rem",
});

globalStyle(`${markdownTable} th, ${markdownTable} td`, {
  padding: "var(--space-1) var(--space-2)",
  border: "1px solid var(--lumin-border)",
  textAlign: "left",
  verticalAlign: "top",
});

globalStyle(`${markdownTable} th`, {
  background: "var(--lumin-canvas)",
  fontWeight: 700,
});

// KaTeX supplies the typesetting; these only place it in the reply.
export const markdownMath = style({
  // A long expression scrolls inside itself rather than widening the bubble.
  display: "inline-block",
  maxWidth: "100%",
  overflowX: "auto",
  verticalAlign: "middle",
});

export const markdownMathBlock = style({
  display: "block",
  width: "100%",
  overflowX: "auto",
  padding: "var(--space-1) 0",
});
