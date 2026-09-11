import { globalStyle, keyframes, style } from "@vanilla-extract/css";

// TeacherChat.tsx batch (plan checkbox 7). Declaration values are verbatim
// copies of the corresponding src/index.css rules (base rules, pseudo/state
// variants, and the two single-owner @media blocks), emitted in index.css
// order so the cascade is order-equivalent.
//
// The spinner is encoded with the keyframes() API — the '@keyframes' object
// prop inside style() is legacy treat syntax and NOT supported by
// vanilla-extract. The animation parameters (0.8s linear infinite) are
// carried verbatim as longhand properties around animationName.
//
// The single-owner @media blocks (prefers-reduced-motion: reduce and
// max-width: 640px) are folded into the per-class '@media' keys and deleted
// from index.css in the same commit. The mixed prefers-reduced-motion group
// (index.css L692-714) contains no teacher-chat selectors and stays for T9.

// index.css L716-725 + @media 640px L1007-1009.
export const teacherChat = style({
  display: "flex",
  flexDirection: "column",
  minHeight: 0,
  maxWidth: "1000px",
  margin: "0 auto",
  padding: "var(--space-3)",
  gap: "var(--space-3)",
  background: "var(--lumin-bg)",
  "@media": {
    "(max-width: 640px)": {
      padding: "var(--space-3)",
    },
  },
});

// index.css L858-862 (@keyframes teacher-chat-spin { to { transform: rotate(360deg) } }).
const spin = keyframes({
  to: {
    transform: "rotate(360deg)",
  },
});

// index.css L727-737 + @media 640px L1011-1014.
export const teacherChatHeader = style({
  display: "flex",
  alignItems: "flex-start",
  justifyContent: "space-between",
  gap: "var(--space-3)",
  padding: "var(--space-4) var(--space-5)",
  background: "var(--lumin-card)",
  border: "1px solid var(--lumin-border)",
  borderRadius: "var(--radius-lg)",
  boxShadow: "var(--shadow-sm)",
  "@media": {
    "(max-width: 640px)": {
      flexDirection: "column",
      gap: "var(--space-2)",
    },
  },
});

// index.css L739-743 (`.teacher-chat-header h2`, unclassed h2).
globalStyle(`${teacherChatHeader} h2`, {
  margin: "0 0 var(--space-1) 0",
  fontSize: "1.5rem",
  color: "var(--lumin-ink)",
});

// index.css L745-749 (`.teacher-chat-header p`, unclassed p).
globalStyle(`${teacherChatHeader} p`, {
  margin: "0",
  color: "var(--lumin-text-secondary)",
  fontSize: "0.9375rem",
});

// index.css L751-757.
export const teacherChatPrivacy = style({
  display: "flex",
  alignItems: "center",
  gap: "var(--space-2)",
  fontSize: "0.8125rem",
  color: "var(--lumin-muted)",
});

// index.css L759-772 + @media 640px L1020-1022.
export const teacherChatHistory = style({
  flex: "1 1 auto",
  minHeight: "160px",
  height: "clamp(160px, 30dvh, 360px)",
  overscrollBehavior: "contain",
  overflowY: "auto",
  display: "flex",
  flexDirection: "column",
  gap: "var(--space-3)",
  padding: "var(--space-5)",
  background: "var(--lumin-card)",
  border: "1px solid var(--lumin-border)",
  borderRadius: "var(--radius-lg)",
  boxShadow: "var(--shadow-sm)",
  "@media": {
    "(max-width: 640px)": {
      padding: "var(--space-3)",
    },
  },
});

// index.css L774-780.
export const teacherChatWelcome = style({
  padding: "var(--space-5)",
  background: "var(--lumin-indigo-soft)",
  border: "1px solid var(--lumin-indigo-soft)",
  borderRadius: "var(--radius-lg)",
  color: "var(--lumin-ink)",
});

// index.css L782-786 (`.teacher-chat-welcome h3`, unclassed h3).
globalStyle(`${teacherChatWelcome} h3`, {
  margin: "0 0 var(--space-2) 0",
  fontSize: "1.125rem",
  color: "var(--lumin-indigo)",
});

// index.css L788-792 (`.teacher-chat-welcome p`, unclassed p).
globalStyle(`${teacherChatWelcome} p`, {
  margin: "0 0 var(--space-3) 0",
  color: "var(--lumin-text-secondary)",
  fontSize: "0.9375rem",
});

// index.css L794-799 + @media 640px L1016-1018.
export const teacherChatMessage = style({
  display: "flex",
  flexDirection: "column",
  maxWidth: "78%",
  gap: "var(--space-1)",
  "@media": {
    "(max-width: 640px)": {
      maxWidth: "88%",
    },
  },
});

// index.css L801-804 (`.teacher-chat-message.user`). The TSX composes
// `${teacherChatMessage} ${messageUser}`; the same-element compound selector
// preserves the original (0,2,0) specificity.
export const messageUser = style({
  selectors: {
    [`${teacherChatMessage}&`]: {
      alignSelf: "flex-end",
      alignItems: "flex-end",
    },
  },
});

// index.css L806-809 (`.teacher-chat-message.assistant`).
export const messageAssistant = style({
  selectors: {
    [`${teacherChatMessage}&`]: {
      alignSelf: "flex-start",
      alignItems: "flex-start",
    },
  },
});

// index.css L811-817.
export const teacherChatMessageRole = style({
  fontSize: "0.75rem",
  fontWeight: "700",
  color: "var(--lumin-muted)",
  textTransform: "uppercase",
  letterSpacing: "0.03em",
});

// index.css L819-825.
export const teacherChatBubble = style({
  padding: "var(--space-3) var(--space-4)",
  borderRadius: "var(--radius-lg)",
  lineHeight: "1.55",
  whiteSpace: "pre-wrap",
  wordBreak: "break-word",
});

// index.css L827-831 (`.teacher-chat-message.user .teacher-chat-bubble`).
// globalStyle emits the original descendant selector with hashed classes —
// the loading bubble inherits the assistant variant exactly like the source,
// with no per-element variant class needed.
globalStyle(`${teacherChatMessage}${messageUser} ${teacherChatBubble}`, {
  background: "var(--lumin-indigo)",
  color: "#ffffff",
  borderBottomRightRadius: "var(--space-1)",
});

// index.css L833-838 (`.teacher-chat-message.assistant .teacher-chat-bubble`).
globalStyle(`${teacherChatMessage}${messageAssistant} ${teacherChatBubble}`, {
  background: "var(--lumin-canvas)",
  color: "var(--lumin-ink)",
  border: "1px solid var(--lumin-border)",
  borderBottomLeftRadius: "var(--space-1)",
});

// index.css L840-846 + ::before L848-856 (spinner via the keyframes() ref;
// 0.8s linear infinite verbatim) + rm ::before kill L1001-1003.
export const teacherChatThinking = style({
  display: "flex",
  alignItems: "center",
  gap: "var(--space-2)",
  color: "var(--lumin-muted)",
  fontSize: "0.9375rem",
  selectors: {
    "&::before": {
      content: '""',
      width: "14px",
      height: "14px",
      border: "2px solid var(--lumin-border)",
      borderTopColor: "var(--lumin-indigo)",
      borderRadius: "50%",
      animationName: spin,
      animationDuration: "0.8s",
      animationTimingFunction: "linear",
      animationIterationCount: "infinite",
    },
  },
  "@media": {
    "(prefers-reduced-motion: reduce)": {
      selectors: {
        "&::before": {
          animation: "none",
        },
      },
    },
  },
});

// index.css L864-868.
export const teacherChatSuggestions = style({
  display: "flex",
  flexWrap: "wrap",
  gap: "var(--space-2)",
});

// index.css L870-880 + :hover:not(:disabled) L882-885 + :active:not(:disabled)
// L887-889 + :disabled L891-894 + rm group L988-990. Under reduced motion the
// rm `&:active` form (0,2,0) still loses to `:active:not(:disabled)` (0,3,0),
// so the :active scale survives exactly like the source cascade.
export const teacherChatChip = style({
  padding: "var(--space-2) var(--space-3)",
  fontSize: "0.875rem",
  fontWeight: "500",
  color: "var(--lumin-indigo)",
  background: "var(--lumin-card)",
  border: "1px solid var(--lumin-border)",
  borderRadius: "9999px",
  cursor: "pointer",
  transition:
    "background 0.2s ease, border-color 0.2s ease, transform 0.2s ease",
  ":disabled": {
    opacity: "0.5",
    cursor: "not-allowed",
  },
  selectors: {
    "&:hover:not(:disabled)": {
      background: "var(--lumin-indigo-soft)",
      borderColor: "var(--lumin-indigo)",
    },
    "&:active:not(:disabled)": {
      transform: "scale(0.98)",
    },
  },
  "@media": {
    "(prefers-reduced-motion: reduce)": {
      transition: "none",
      transform: "none",
      animation: "none",
      selectors: {
        "&:hover": {
          transition: "none",
          transform: "none",
          animation: "none",
        },
        "&:active": {
          transition: "none",
          transform: "none",
          animation: "none",
        },
      },
    },
  },
});

// index.css L896-905.
export const teacherChatComposer = style({
  display: "flex",
  flexDirection: "column",
  gap: "var(--space-3)",
  padding: "var(--space-4)",
  background: "var(--lumin-card)",
  border: "1px solid var(--lumin-border)",
  borderRadius: "var(--radius-lg)",
  boxShadow: "var(--shadow-sm)",
});

// index.css L907-913.
export const teacherChatError = style({
  padding: "var(--space-3) var(--space-4)",
  background: "#fff1f2",
  color: "var(--lumin-error)",
  borderRadius: "var(--radius-md)",
  fontSize: "0.875rem",
});

// index.css L915-919.
export const teacherChatInputRow = style({
  display: "flex",
  alignItems: "flex-end",
  gap: "var(--space-3)",
});

// index.css L921-935 + :focus L937-940 + :disabled L942-945 + rm group
// L994-995. The component :focus (indigo) imports after the global stylesheet
// module and wins the (0,2,0) tie over baseInputFocus's purple; baseInputFocus
// supplies the ::placeholder the element band used to provide (T6 attach
// pattern). NOTE for future edits of this file: comments must not contain
// CommonJS-looking tokens (dot-suffixed identifiers like the global stylesheet
// module's filename, or call-looking words) — the VE integration sniffs module
// syntax WITHOUT stripping comments, a false positive routes the file-scope
// injection down the wrong branch, and the compiler then fails with "Styles
// were unable to be assigned to a file".
export const teacherChatInput = style({
  flex: "1",
  resize: "none",
  minHeight: "48px",
  maxHeight: "160px",
  padding: "var(--space-3) var(--space-4)",
  fontSize: "1rem",
  lineHeight: "1.5",
  color: "var(--lumin-ink)",
  background: "var(--lumin-canvas)",
  border: "1px solid var(--lumin-border)",
  borderRadius: "var(--radius-md)",
  outline: "none",
  transition: "border-color 0.2s ease, box-shadow 0.2s ease",
  ":focus": {
    borderColor: "var(--lumin-indigo)",
    boxShadow: "0 0 0 3px var(--lumin-indigo-soft)",
  },
  ":disabled": {
    background: "#f3f4f6",
    cursor: "not-allowed",
  },
  "@media": {
    "(prefers-reduced-motion: reduce)": {
      transition: "none",
      transform: "none",
      animation: "none",
      selectors: {
        "&:focus": {
          transition: "none",
          transform: "none",
          animation: "none",
        },
      },
    },
  },
});

// index.css L947-960 + :hover:not(:disabled) L962-964 + :active:not(:disabled)
// L966-968 + :disabled L970-973 + rm group L991-993.
export const teacherChatSend = style({
  display: "inline-flex",
  alignItems: "center",
  justifyContent: "center",
  width: "48px",
  height: "48px",
  flexShrink: 0,
  color: "#ffffff",
  background: "var(--lumin-indigo)",
  border: "none",
  borderRadius: "50%",
  cursor: "pointer",
  transition: "background 0.2s ease, transform 0.2s ease, opacity 0.2s ease",
  ":disabled": {
    opacity: "0.45",
    cursor: "not-allowed",
  },
  selectors: {
    "&:hover:not(:disabled)": {
      background: "#4338ca",
    },
    "&:active:not(:disabled)": {
      transform: "scale(0.96)",
    },
  },
  "@media": {
    "(prefers-reduced-motion: reduce)": {
      transition: "none",
      transform: "none",
      animation: "none",
      selectors: {
        "&:hover": {
          transition: "none",
          transform: "none",
          animation: "none",
        },
        "&:active": {
          transition: "none",
          transform: "none",
          animation: "none",
        },
      },
    },
  },
});

// index.css L975-978 (`.teacher-chat-send svg`, unclassed svg).
globalStyle(`${teacherChatSend} svg`, {
  width: "20px",
  height: "20px",
});

// index.css L980-985.
export const teacherChatFooterNote = style({
  margin: "0",
  fontSize: "0.75rem",
  color: "var(--lumin-muted)",
  textAlign: "center",
});
