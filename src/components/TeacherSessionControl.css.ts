import { globalStyle, style } from "@vanilla-extract/css";

// TeacherSessionControl.tsx batch (plan checkbox 5). Declaration values are
// verbatim copies of the corresponding src/index.css rules (base rule + every
// descendant-combined form + @media portions + reduced-motion portions).
// Cascade invariant: each class's @media overrides live in the same style()
// call as its base rule, and declaration order matches index.css within each
// rule.
//
// DOUBLE-DEFINED session-control section: index.css defines this section TWICE
// at top level (copy 1: L1052-1348, copy 2: L1788-2141). The copies share the
// same selectors, so the cascade resolves per-property with copy 2 winning
// every duplicated declaration; copy-1-only declarations survive. Each const
// below encodes the merged effective result, with both source ranges noted.
// Encoding the merged values (instead of two same-selector rules) is exactly
// equivalent: CSS cascade resolution for duplicate selectors is per-property
// later-wins.

// index.css L1052-1056 (copy 1) + L1788-1795 (copy 2) + L2144-2146
// (@media max-width: 640px, copy-2-era block).
export const sessionControl = style({
  display: "flex",
  flexDirection: "column",
  gap: "var(--space-5)",
  maxWidth: "900px",
  margin: "0 auto",
  padding: "var(--space-6)",
  "@media": {
    "(max-width: 640px)": {
      padding: "var(--space-4)",
    },
  },
});

// index.css L1058-1063 (copy 1) + L1797-1802 (copy 2; font-size/weight win).
export const sessionTitle = style({
  margin: "0",
  fontSize: "clamp(1.5rem, 4vw, 2rem)",
  fontWeight: "800",
  color: "var(--lumin-ink)",
});

// index.css L1065-1071 (copy 1) + L1838-1844 (copy 2; identical values).
export const sessionCard = style({
  background: "var(--lumin-card)",
  border: "1px solid var(--lumin-border)",
  borderRadius: "var(--radius-lg)",
  boxShadow: "var(--shadow-sm)",
  padding: "var(--space-5)",
});

// index.css L1073-1078 (copy 1) + L1846-1851 (copy 2; gap/margin win) +
// L2152-2154 (@media max-width: 640px).
export const sessionCardHeader = style({
  display: "flex",
  alignItems: "center",
  gap: "var(--space-4)",
  marginBottom: "var(--space-5)",
  "@media": {
    "(max-width: 640px)": {
      alignItems: "flex-start",
    },
  },
});

// index.css L1080-1087 (copy 1; padding is copy-1-only and survives) +
// L1853-1863 (copy 2; display/alignment/flex-shrink/color/background win).
export const sessionIconBadge = style({
  display: "inline-flex",
  alignItems: "center",
  justifyContent: "center",
  width: "48px",
  height: "48px",
  flexShrink: 0,
  padding: "var(--space-2)",
  color: "var(--lumin-indigo)",
  background: "var(--lumin-indigo-soft)",
  borderRadius: "var(--radius-md)",
});

// index.css L1089-1092 (copy 1: 100%) + L1865-1868 (copy 2: 26px wins).
// Unclassed descendant svg => globalStyle with interpolated refs.
globalStyle(`${sessionIconBadge} svg`, {
  width: "26px",
  height: "26px",
});

// index.css L1094-1099 (copy 1) + L1870-1875 (copy 2; identical values).
export const sessionCardTitle = style({
  margin: "0",
  fontSize: "1.125rem",
  fontWeight: "700",
  color: "var(--lumin-ink)",
});

// index.css L1101-1105 (copy 1) + L1877-1881 (copy 2; identical values).
export const sessionCardSubtitle = style({
  margin: "var(--space-1) 0 0 0",
  fontSize: "0.875rem",
  color: "var(--lumin-text-secondary)",
});

// index.css L1107-1112 (copy 1) + L1883-1888 (copy 2; gap/margin win).
export const subjectGroups = style({
  display: "flex",
  flexDirection: "column",
  gap: "var(--space-5)",
  marginBottom: "var(--space-5)",
});

// index.css L1114-1118 (copy 1) + L1890-1894 (copy 2; gap wins).
export const subjectGroup = style({
  display: "flex",
  flexDirection: "column",
  gap: "var(--space-3)",
});

// index.css L1120-1127 (copy 1) + L1896-1903 (copy 2; font-size wins).
export const subjectLabel = style({
  margin: "0",
  fontSize: "0.75rem",
  fontWeight: "700",
  color: "var(--lumin-indigo)",
  textTransform: "uppercase",
  letterSpacing: "0.05em",
});

// index.css L1129-1133 (copy 1) + L1905-1909 (copy 2; identical values) +
// L2148-2150 (@media max-width: 640px).
export const quizPicker = style({
  display: "grid",
  gridTemplateColumns: "repeat(auto-fill, minmax(200px, 1fr))",
  gap: "var(--space-3)",
  "@media": {
    "(max-width: 640px)": {
      gridTemplateColumns: "1fr",
    },
  },
});

// index.css L1135-1146 (copy 1) + L1911-1922 (copy 2; gap/padding/border/
// transition win) + L1148-1150 + L1924-1927 (:hover merged) + L2162-2163
// (@media prefers-reduced-motion: reduce).
export const quizCard = style({
  display: "flex",
  flexDirection: "column",
  gap: "var(--space-2)",
  padding: "var(--space-4)",
  textAlign: "left",
  background: "var(--lumin-canvas)",
  border: "1px solid var(--lumin-border)",
  borderRadius: "var(--radius-md)",
  cursor: "pointer",
  transition:
    "border-color 0.2s ease, background 0.2s ease, transform 0.2s ease",
  ":hover": {
    borderColor: "var(--lumin-indigo)",
    background: "var(--lumin-indigo-soft)",
  },
  "@media": {
    "(prefers-reduced-motion: reduce)": {
      transition: "none",
      transform: "none",
      selectors: {
        "&:hover": {
          transition: "none",
          transform: "none",
        },
      },
    },
  },
});

// index.css L1152-1155 (copy 1) + L1929-1933 (copy 2 wins). Compound
// same-element selector preserves the original .quiz-card.selected
// specificity (0,2,0) so it keeps beating :hover (0,1,1) exactly like the
// source cascade.
export const quizCardSelected = style({
  selectors: {
    [`${quizCard}&`]: {
      borderColor: "var(--lumin-indigo)",
      background: "var(--lumin-indigo-soft)",
      boxShadow: "0 0 0 2px var(--lumin-indigo)",
    },
  },
});

// index.css L1157-1160 (copy 1) + L1935-1938 (copy 2; font-weight wins).
export const quizCardTitle = style({
  fontWeight: "700",
  color: "var(--lumin-ink)",
});

// index.css L1162-1165 (copy 1) + L1940-1943 (copy 2; color wins).
export const quizCardMeta = style({
  fontSize: "0.875rem",
  color: "var(--lumin-text-secondary)",
});

// index.css L1167-1169 (copy 1) + L1945-1947 (copy 2; identical values).
export const sessionStartButton = style({
  width: "100%",
});

// index.css L1171-1173 (copy 1) + L1949-1953 (copy 2; margin wins, adds
// padding-bottom + border-bottom).
export const selectedQuizHeader = style({
  marginBottom: "var(--space-5)",
  paddingBottom: "var(--space-4)",
  borderBottom: "1px solid var(--lumin-border)",
});

// index.css L1175-1181 (copy 1) + L1955-1963 (copy 2; adds display +
// margin-bottom).
export const selectedQuizSubject = style({
  display: "inline-block",
  marginBottom: "var(--space-2)",
  fontSize: "0.75rem",
  fontWeight: "700",
  letterSpacing: "0.05em",
  textTransform: "uppercase",
  color: "var(--lumin-indigo)",
});

// index.css L1183-1188 (copy 1) + L1965-1970 (copy 2; margin/size/weight win).
export const selectedQuizTitle = style({
  margin: "0",
  fontSize: "1.5rem",
  fontWeight: "800",
  color: "var(--lumin-ink)",
});

// index.css L1190-1194 (copy 1; font-size is copy-1-only and survives) +
// L1972-1975 (copy 2; margin/color win).
export const selectedQuizMeta = style({
  margin: "var(--space-2) 0 0 0",
  fontSize: "0.875rem",
  color: "var(--lumin-text-secondary)",
});

// index.css L1196-1203 (copy 1) + L1977-1984 (copy 2; gap wins).
export const questionList = style({
  display: "flex",
  flexDirection: "column",
  gap: "var(--space-4)",
  listStyle: "none",
  padding: "0",
  margin: "0",
});

// index.css L1205-1212 (copy 1; padding/background/border/radius are
// copy-1-only and survive) + L1986-1990 (copy 2; align-items wins).
export const questionListItem = style({
  display: "flex",
  alignItems: "flex-start",
  gap: "var(--space-3)",
  padding: "var(--space-3)",
  background: "var(--lumin-canvas)",
  border: "1px solid var(--lumin-border)",
  borderRadius: "var(--radius-md)",
});

// index.css L1214-1226 (copy 1) + L1992-2004 (copy 2; display/color/background
// win).
export const questionNumber = style({
  display: "inline-flex",
  alignItems: "center",
  justifyContent: "center",
  width: "28px",
  height: "28px",
  flexShrink: 0,
  fontSize: "0.875rem",
  fontWeight: "700",
  color: "#ffffff",
  background: "var(--lumin-indigo)",
  borderRadius: "50%",
});

// index.css L1228-1232 (copy 1) + L2006-2010 (copy 2; gap wins).
export const questionBody = style({
  display: "flex",
  flexDirection: "column",
  gap: "var(--space-1)",
});

// Re-homed from the T3 shared `questionPrompt` primitive: that primitive
// anchors on the literal `.question-body &` selector, which stops matching
// once className="question-body" becomes the hashed `${questionBody}` class.
// The descendant-context override (index.css L1234-1239 copy 1 + L2012-2018
// copy 2, later wins per property and adds line-height: 1.5) is rebuilt here
// as a real VE cross-reference with verbatim merged values. The primitive
// itself is still wired alongside so its base rule applies, exactly like the
// original base + descendant-rule pair.
export const questionPromptInBody = style({
  selectors: {
    [`${questionBody} &`]: {
      margin: "0",
      fontSize: "1rem",
      fontWeight: "600",
      lineHeight: "1.5",
      color: "var(--lumin-ink)",
    },
  },
});

// Re-homed from the T3 shared `questionConcept` primitive for the same reason
// (index.css L1241-1244 copy 1 + L2020-2023 copy 2, later wins per property).
export const questionConceptInBody = style({
  selectors: {
    [`${questionBody} &`]: {
      fontSize: "0.875rem",
      color: "var(--lumin-text-secondary)",
    },
  },
});

// index.css L1246-1252 (copy 1) + L2025-2034 (copy 2; gap wins, adds padding/
// background/border).
export const joinCodeCard = style({
  display: "flex",
  flexDirection: "column",
  alignItems: "center",
  gap: "var(--space-3)",
  textAlign: "center",
  padding: "var(--space-6)",
  background:
    "linear-gradient(135deg, var(--lumin-indigo-soft), var(--lumin-purple-soft))",
  border: "1px solid var(--lumin-indigo-soft)",
});

// index.css L1267-1271 (copy 1) + L2052-2056 (copy 2; color wins).
export const joinCodeHint = style({
  margin: "0",
  fontSize: "0.875rem",
  color: "var(--lumin-text-secondary)",
});

// index.css L1273-1275 (copy 1) + L2058-2060 (copy 2; margin wins).
export const connectedStudentsHeader = style({
  marginBottom: "var(--space-4)",
});

// index.css L1277-1284 (copy 1) + L2062-2069 (copy 2; gap wins).
export const studentList = style({
  display: "flex",
  flexDirection: "column",
  gap: "var(--space-3)",
  margin: "0",
  padding: "0",
  listStyle: "none",
});

// index.css L1286-1294 (copy 1) + L2071-2080 (copy 2; padding wins, adds
// border).
export const studentRow = style({
  display: "flex",
  alignItems: "center",
  justifyContent: "space-between",
  gap: "var(--space-3)",
  padding: "var(--space-3) var(--space-4)",
  background: "var(--lumin-canvas)",
  border: "1px solid var(--lumin-border)",
  borderRadius: "var(--radius-md)",
});

// index.css L2082-2085 (copy 2 only). Unclassed descendant span =>
// globalStyle with interpolated refs.
globalStyle(`${studentRow} span`, {
  fontWeight: "600",
  color: "var(--lumin-ink)",
});

// index.css L1296-1305 (copy 1) + L2087-2097 (copy 2; padding/weight/
// background/border/radius win) + L1307-1309 + L2099-2101 (:hover merged) +
// L2166-2167 (@media prefers-reduced-motion: reduce).
export const studentKickButton = style({
  padding: "0.4em 0.9em",
  fontSize: "0.875rem",
  fontWeight: "600",
  color: "var(--lumin-error)",
  background: "#fff1f2",
  border: "1px solid transparent",
  borderRadius: "var(--radius-md)",
  cursor: "pointer",
  transition: "background 0.2s ease",
  ":hover": {
    background: "#ffe4e6",
  },
  "@media": {
    "(prefers-reduced-motion: reduce)": {
      transition: "none",
      transform: "none",
      selectors: {
        "&:hover": {
          transition: "none",
          transform: "none",
        },
      },
    },
  },
});

// index.css L1311-1313 (copy 1) + L2103-2105 (copy 2; identical values).
export const sessionEndButton = style({
  width: "100%",
});

// index.css L1315-1317 (copy 1) + L2107-2109 (copy 2; margin-top: 0 wins).
export const sessionError = style({
  marginTop: "0",
});

// index.css L1319-1321 (copy 1) + L2111-2114 (copy 2; adds border-style).
export const sessionInfoCard = style({
  background: "var(--lumin-canvas)",
  borderStyle: "dashed",
});

// index.css L1323-1327 (copy 1) + L2116-2120 (copy 2; gap wins).
export const sessionInfoRow = style({
  display: "flex",
  alignItems: "flex-start",
  gap: "var(--space-4)",
});

// index.css L1329-1334 (copy 1: 24px) + L2122-2127 (copy 2: 28px wins).
// Unclassed descendant svg => globalStyle with interpolated refs.
globalStyle(`${sessionInfoRow} svg`, {
  width: "28px",
  height: "28px",
  flexShrink: 0,
  color: "var(--lumin-indigo)",
});

// index.css L1336-1341 (copy 1) + L2129-2134 (copy 2; margin wins).
export const sessionInfoTitle = style({
  margin: "0 0 var(--space-1) 0",
  fontSize: "1rem",
  fontWeight: "700",
  color: "var(--lumin-ink)",
});

// index.css L1343-1348 (copy 1) + L2136-2141 (copy 2; margin/line-height win).
export const sessionInfoText = style({
  margin: "0",
  fontSize: "0.875rem",
  lineHeight: "1.6",
  color: "var(--lumin-text-secondary)",
});

// Shared primitive wiring notes (T3 inventory):
// - primaryButton / secondaryButton / errorMessage / emptyState: no ancestor
//   conditioning on T5-owned classes; wired directly in the TSX.
// - questionPrompt / questionConcept: re-homed overrides above
//   (questionPromptInBody / questionConceptInBody) because their literal
//   `.question-body &` anchors stop matching once question-body is hashed.
//   StudentQuiz renders question-prompt/question-concept WITHOUT any
//   question-body ancestor, so its literal-class rendering is unaffected.
// - joinCodeLabel / joinCodeValue: wired directly; joinCodeValue's literal
//   `.join-code-banner &` form is re-homed in TeacherDashboard.css.ts (the
//   only join-code-banner usage lives there).
// - quizHeader / modelManager: NOT wired — their real usages live in
//   StudentQuiz.tsx / ModelManager.tsx (later batches), not in this batch's
//   files.
