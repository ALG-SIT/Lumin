import { style } from "@vanilla-extract/css";

// TeacherDashboard.tsx batch (plan checkbox 5). Declaration values are
// verbatim copies of the corresponding src/index.css rules (base rule + every
// descendant-combined form + @media portions). Cascade invariant: each class's
// @media overrides live in the same style() call as its base rule, and
// declaration order matches index.css within each rule.

// index.css L87-92 + L1352-1356 (@media max-width: 768px) + L1414-1417
// (@media max-width: 480px). 768 is emitted before 480 to preserve cascade.
export const teacherLayout = style({
  display: "grid",
  gridTemplateColumns: "200px minmax(0, 1fr)",
  gap: "16px",
  height: "100%",
  minHeight: 0,
  "@media": {
    "(max-width: 768px)": {
      gridTemplateColumns: "1fr",
      gridTemplateRows: "auto minmax(0, 1fr)",
      height: "100%",
      minHeight: 0,
    },
    "(max-width: 480px)": {
      gridTemplateRows: "auto minmax(0, 1fr)",
      gap: "8px",
    },
  },
});

// index.css L94-99 + L1358-1365 (@media max-width: 768px).
export const teacherSidebar = style({
  display: "flex",
  flexDirection: "column",
  gap: "4px",
  paddingTop: "8px",
  "@media": {
    "(max-width: 768px)": {
      flexDirection: "row",
      overflowX: "auto",
      paddingTop: "0",
      paddingBottom: "8px",
      borderBottom: "1px solid var(--lumin-border)",
      gap: "6px",
    },
  },
});

// index.css L101-112 (`.teacher-sidebar button`) + L114-116 (:hover) +
// L1367-1372 (@media max-width: 768px). The original styling reached the
// sidebar buttons through an element-descendant rule; the buttons now carry
// this class instead (including the previously unclassed reset button), which
// applies the identical declarations to exactly the same elements.
export const sidebarButton = style({
  minHeight: "44px",
  textAlign: "left",
  padding: "12px 14px",
  borderRadius: "8px",
  border: "none",
  background: "transparent",
  color: "var(--lumin-text)",
  cursor: "pointer",
  fontSize: "14px",
  fontWeight: "500",
  transition: "background 0.15s ease, color 0.15s ease",
  ":hover": {
    background: "rgba(124, 58, 237, 0.08)",
  },
  "@media": {
    "(max-width: 768px)": {
      whiteSpace: "nowrap",
      fontSize: "13px",
      padding: "8px 12px",
      flexShrink: 0,
    },
  },
});

// index.css L118-121 (`.teacher-sidebar button.active`). Compound
// same-element selector preserves the original specificity relationship where
// .active (later rule) beats :hover.
export const sidebarButtonActive = style({
  selectors: {
    [`${sidebarButton}&`]: {
      background: "var(--lumin-purple)",
      color: "white",
    },
  },
});

// index.css L123-127.
export const teacherContent = style({
  display: "flex",
  flexDirection: "column",
  minHeight: 0,
  flex: 1,
  minWidth: 0,
  overflowY: "auto",
});

// index.css L130-136.
export const card = style({
  background: "var(--lumin-surface)",
  borderRadius: "var(--lumin-radius)",
  boxShadow: "var(--lumin-shadow)",
  padding: "16px",
  marginBottom: "16px",
});

// index.css L2380-2389.
export const joinCodeBanner = style({
  display: "flex",
  alignItems: "center",
  gap: "var(--space-3)",
  padding: "var(--space-3) var(--space-4)",
  marginBottom: "var(--space-4)",
  background: "var(--lumin-indigo-soft)",
  border: "1px solid var(--lumin-primary)",
  borderRadius: "var(--radius-md)",
});

// Re-homed from the T3 shared `joinCodeValue` primitive: that primitive
// anchors on the literal `.join-code-banner &` selector, which stops matching
// once className="join-code-banner" becomes the hashed `${joinCodeBanner}`
// class. The banner-context override (index.css L2391-2395) is rebuilt here as
// a real VE cross-reference with verbatim values. The primitive itself is
// still wired alongside so its base rule applies, exactly like the original
// base + descendant-rule pair (its (0,2,0) override also keeps beating the
// primitive's @media max-width: 640px font-size, matching the source cascade).
export const joinCodeValueBanner = style({
  selectors: {
    [`${joinCodeBanner} &`]: {
      fontSize: "1.5rem",
      letterSpacing: "0.35em",
      color: "var(--lumin-primary)",
    },
  },
});

// index.css L878-882.
export const teacherDashboard = style({
  display: "flex",
  flexDirection: "column",
  gap: "var(--space-5)",
});

// index.css L884-891.
export const teacherDashboardHeader = style({
  display: "flex",
  alignItems: "flex-start",
  justifyContent: "space-between",
  gap: "var(--space-3)",
  paddingBottom: "var(--space-4)",
  borderBottom: "1px solid var(--lumin-border)",
});

// index.css L893-898.
export const teacherDashboardTitle = style({
  margin: "0",
  fontSize: "1.75rem",
  fontWeight: "800",
  color: "var(--lumin-ink)",
});

// index.css L900-903.
export const teacherDashboardSubtitle = style({
  margin: "var(--space-1) 0 0 0",
  color: "var(--lumin-text-secondary)",
});

// index.css L213-218 + L1378-1380 (@media max-width: 768px).
export const metricGrid = style({
  display: "grid",
  gridTemplateColumns: "repeat(auto-fit, minmax(140px, 1fr))",
  gap: "8px",
  marginBottom: "24px",
  "@media": {
    "(max-width: 768px)": {
      gridTemplateColumns: "repeat(2, 1fr)",
    },
  },
});

// index.css L220-226 + L1437-1439 (@media max-width: 480px).
export const metricCard = style({
  background: "var(--lumin-surface)",
  borderRadius: "var(--lumin-radius)",
  padding: "16px",
  textAlign: "center",
  boxShadow: "var(--lumin-shadow)",
  "@media": {
    "(max-width: 480px)": {
      padding: "12px",
    },
  },
});

// index.css L228-233 + L1441-1443 (@media max-width: 480px).
export const metricValue = style({
  display: "block",
  fontSize: "28px",
  fontWeight: "700",
  color: "var(--lumin-purple)",
  "@media": {
    "(max-width: 480px)": {
      fontSize: "22px",
    },
  },
});

// index.css L235-240.
export const metricLabel = style({
  display: "block",
  fontSize: "12px",
  color: "var(--lumin-text-secondary)",
  marginTop: "4px",
});

// index.css L905-910.
export const metricNote = style({
  display: "block",
  fontSize: "0.75rem",
  color: "var(--lumin-muted)",
  marginTop: "var(--space-1)",
});

// index.css L912-919.
export const dashboardCard = style({
  background: "var(--lumin-card)",
  border: "1px solid var(--lumin-border)",
  borderRadius: "var(--radius-lg)",
  boxShadow: "var(--shadow-sm)",
  padding: "var(--space-5)",
  marginBottom: "var(--space-4)",
});

// index.css L921-923.
export const dashboardCardHeader = style({
  marginBottom: "var(--space-4)",
});

// index.css L925-930.
export const dashboardCardTitle = style({
  margin: "0",
  fontSize: "1.125rem",
  fontWeight: "700",
  color: "var(--lumin-ink)",
});

// index.css L932-936.
export const dashboardCardSubtitle = style({
  margin: "var(--space-1) 0 0 0",
  fontSize: "0.875rem",
  color: "var(--lumin-text-secondary)",
});

// index.css L938-941.
export const dashboardEmpty = style({
  color: "var(--lumin-muted)",
  fontSize: "0.9375rem",
});

// index.css L243-248.
export const barRow = style({
  display: "flex",
  alignItems: "center",
  gap: "8px",
  marginBottom: "8px",
});

// index.css L943-948.
export const barMeta = style({
  minWidth: 0,
  "@media": { "(max-width: 640px)": { width: "40%" } },
  width: "180px",
  display: "flex",
  flexDirection: "column",
  gap: "2px",
});

// index.css L250-257 (base) + L950-952 (`.bar-meta .bar-label`, width: auto)
// + L1382-1384 (@media max-width: 768px, width: 100px). The ancestor-conditioned
// width:auto (0,2,0) keeps beating the @media width (0,1,0) exactly like the
// source cascade.
export const barLabel = style({
  width: "180px",
  fontSize: "13px",
  color: "var(--lumin-text)",
  whiteSpace: "nowrap",
  overflow: "hidden",
  textOverflow: "ellipsis",
  selectors: {
    [`${barMeta} &`]: {
      width: "auto",
    },
  },
  "@media": {
    "(max-width: 768px)": {
      width: "100px",
    },
  },
});

// index.css L259-265.
export const barTrack = style({
  flex: 1,
  height: "20px",
  background: "var(--lumin-border)",
  borderRadius: "10px",
  overflow: "hidden",
});

// index.css L267-272. The element's dynamic width/backgroundColor stay as
// inline styles in the TSX (out of scope everywhere).
export const barFill = style({
  height: "100%",
  background:
    "linear-gradient(90deg, var(--lumin-purple), var(--lumin-indigo))",
  borderRadius: "10px",
  transition: "width 0.3s",
});

// index.css L274-280.
export const barCount = style({
  width: "32px",
  textAlign: "right",
  fontSize: "13px",
  fontWeight: "600",
  color: "var(--lumin-text)",
});

// index.css L954-964.
export const signalCard = style({
  display: "grid",
  gridTemplateColumns: "auto 1fr auto",
  alignItems: "center",
  gap: "var(--space-3)",
  padding: "var(--space-3)",
  background: "var(--lumin-canvas)",
  border: "1px solid var(--lumin-border)",
  borderRadius: "var(--radius-md)",
  marginBottom: "var(--space-2)",
});

// index.css L966-975.
export const signalBadge = style({
  display: "inline-flex",
  alignItems: "center",
  justifyContent: "center",
  width: "28px",
  height: "28px",
  borderRadius: "50%",
  fontWeight: "700",
  color: "white",
});

// index.css L977-979 (`.signal-badge.correct`). Compound same-element selector
// preserves the original (0,2,0) override over the base rule.
export const signalBadgeCorrect = style({
  selectors: {
    [`${signalBadge}&`]: {
      background: "var(--lumin-success)",
    },
  },
});

// index.css L981-983 (`.signal-badge.incorrect`).
export const signalBadgeIncorrect = style({
  selectors: {
    [`${signalBadge}&`]: {
      background: "var(--lumin-error)",
    },
  },
});

// index.css L985-990.
export const signalBody = style({
  display: "flex",
  flexDirection: "column",
  gap: "2px",
  minWidth: 0,
});

// index.css L992-998.
export const signalConcept = style({
  fontWeight: "600",
  color: "var(--lumin-ink)",
  whiteSpace: "nowrap",
  overflow: "hidden",
  textOverflow: "ellipsis",
});

// index.css L1000-1003.
export const signalMisconception = style({
  fontSize: "0.875rem",
  color: "var(--lumin-muted)",
});

// index.css L1005-1009.
export const signalToken = style({
  fontSize: "0.75rem",
  color: "var(--lumin-muted)",
  fontFamily:
    "ui-monospace, SFMono-Regular, Menlo, Monaco, Consolas, monospace",
});

// index.css L1011-1024.
export const teacherEmptyState = style({
  display: "flex",
  flexDirection: "column",
  alignItems: "center",
  justifyContent: "center",
  gap: "var(--space-4)",
  minHeight: "360px",
  textAlign: "center",
  padding: "var(--space-6)",
  background: "var(--lumin-card)",
  border: "1px solid var(--lumin-border)",
  borderRadius: "var(--radius-lg)",
  boxShadow: "var(--shadow-sm)",
});

// index.css L1026-1029.
export const teacherEmptyIcon = style({
  fontSize: "3.5rem",
  color: "var(--lumin-purple)",
});

// index.css L1031-1036.
export const teacherEmptyTitle = style({
  margin: "0",
  fontSize: "1.5rem",
  fontWeight: "700",
  color: "var(--lumin-ink)",
});

// index.css L1038-1042.
export const teacherEmptyText = style({
  margin: "0",
  color: "var(--lumin-text-secondary)",
  maxWidth: "480px",
});

// index.css L1044-1050.
export const demoActions = style({
  display: "flex",
  flexDirection: "column",
  alignItems: "center",
  gap: "var(--space-3)",
  paddingTop: "var(--space-4)",
});

// Shared primitive wiring notes (T3 inventory):
// - joinCodeLabel / joinCodeValue: wired directly in the TSX (both usages of
//   these classes live in this batch's files). joinCodeValue's literal
//   `.join-code-banner &` form is re-homed above as joinCodeValueBanner.
// - primaryButton / secondaryButton / privacyNote: no ancestor conditioning on
//   T5-owned classes; wired directly.
// - modelManager: NOT wired — its real usage lives in ModelManager.tsx (later
//   batch). `.model-manager-panel` (this file) has no index.css rules.
// - quizHeader: NOT wired — its real usage lives in StudentQuiz.tsx (T8).
