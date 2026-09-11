import { style } from "@vanilla-extract/css";

// App.tsx batch (plan checkbox 4). Declaration values are verbatim copies of
// the corresponding src/index.css rules (base rule + every descendant-combined
// form + @media portions). Cascade invariant: each class's @media overrides
// live in the same style() call as its base rule, and declaration order
// matches index.css within each rule.

// index.css L78-82.
export const app = style({
  display: "flex",
  flexDirection: "column",
  minHeight: "100vh",
});

// index.css L84-93 + L1587-1590 (@media max-width: 480px).
export const appBar = style({
  display: "flex",
  alignItems: "center",
  justifyContent: "space-between",
  gap: "var(--space-3)",
  padding: "12px 16px",
  background: "var(--lumin-purple)",
  color: "white",
  borderBottom: "1px solid var(--lumin-purple-dark)",
  "@media": {
    "(max-width: 480px)": {
      padding: "10px 12px",
      flexWrap: "wrap",
    },
  },
});

// index.css L95-102 (`.app-bar .brand`).
export const brand = style({
  selectors: {
    [`${appBar} &`]: {
      display: "flex",
      alignItems: "center",
      gap: "var(--space-2)",
      fontWeight: "900",
      letterSpacing: "0.1em",
      color: "white",
    },
  },
});

// index.css L104-108 (`.app-bar .brand-icon`).
export const brandIcon = style({
  selectors: {
    [`${appBar} &`]: {
      width: "24px",
      height: "24px",
      color: "white",
    },
  },
});

// index.css L110-113 (`.app-bar .role-badge`) + L1592-1594 (@media
// max-width: 480px).
export const roleBadge = style({
  selectors: {
    [`${appBar} &`]: {
      fontSize: "0.875rem",
      color: "rgba(255, 255, 255, 0.9)",
    },
  },
  "@media": {
    "(max-width: 480px)": {
      selectors: {
        [`${appBar} &`]: {
          fontSize: "0.75rem",
        },
      },
    },
  },
});

// index.css L115-124, L126-128 (:hover), L130-132 (:active), L1656-1657
// (@media prefers-reduced-motion: reduce portions).
//
// Re-homed from the T3 shared `resetButton` primitive instead of wiring it:
// that primitive anchors on the literal `.app-bar &` selector, which stops
// matching once className="app-bar" becomes the hashed `${appBar}` class. The
// ancestor condition is rebuilt here as a real VE cross-reference so the
// DemoFlow behavior is preserved (it renders .reset-button outside the app
// bar and is intentionally unstyled). This is the T3c "descendant forms
// convert to ${parent} refs at parent-batch time" operation.
export const resetButton = style({
  selectors: {
    [`${appBar} &`]: {
      background: "rgba(255, 255, 255, 0.2)",
      border: "none",
      color: "white",
      padding: "6px 12px",
      borderRadius: "8px",
      cursor: "pointer",
      fontSize: "0.875rem",
      transition: "background 0.2s ease, transform 0.2s ease",
    },
    [`${appBar} &:hover`]: {
      background: "rgba(255, 255, 255, 0.3)",
    },
    [`${appBar} &:active`]: {
      transform: "scale(0.98)",
    },
  },
  "@media": {
    "(prefers-reduced-motion: reduce)": {
      selectors: {
        [`${appBar} &`]: {
          transition: "none",
          transform: "none",
        },
        [`${appBar} &:active`]: {
          transition: "none",
          transform: "none",
        },
      },
    },
  },
});
