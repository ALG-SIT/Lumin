import { style } from '@vanilla-extract/css';

// Shared primitives pre-declared for the index.css -> vanilla-extract migration
// (plan checkbox 3). Every class here is referenced from >= 2 component files
// (machine-verified, see .omo/evidence/css-migration/task-3/gates.log); the
// input trio manual-input / code-input / answer-input is owned by T6/T8 and is
// intentionally absent.
//
// Declaration values are verbatim copies of the corresponding src/index.css
// rules (base rule + every descendant-combined form that styles the class
// itself + @media overrides). index.css keeps ALL of its rules until T9 —
// these exports are dead until later batches swap TSX className strings, so
// build output and rendering are unchanged by this file.
//
// Encoding conventions for T4-T8 consumers:
// - Pseudo states use top-level keys (':disabled') for simple pseudos and
//   `selectors` for complex ones (':hover:not(:disabled)').
// - Descendant-combined forms (.app-bar .reset-button, .question-body
//   .question-prompt, .student-quiz .progress-bar, .join-code-banner
//   .join-code-value) are encoded as ancestor-conditioned selectors
//   ('.ancestor &') so the primitive renders identically in and out of the
//   original context. When a parent batch migrates, these become `${primitive}`
//   references on the parent side.
// - Where index.css declares the same selector twice, the later rule wins per
//   property (CSS cascade); the merged effective values are recorded below
//   with both source ranges.
// - `.model-manager h2` (index.css L549-553) styles a DESCENDANT, not the
//   class itself, so it is NOT part of the primitive and stays in index.css;
//   the ModelManager batch must re-home it when swapping the wrapper class.

// index.css L839-849 (shared with .join-button/.secondary-button),
// L851-855, L857-860 (:hover), L862-867 (:disabled), L869-872 (:active),
// L1641-1666 (@media prefers-reduced-motion: reduce).
// Reduced-motion transform:none intentionally loses to the :active scale,
// matching the original specificity (.primary-button:active (0,2,0) vs
// .primary-button:active:not(:disabled) (0,3,0)).
export const primaryButton = style({
  padding: 'var(--space-3) var(--space-4)',
  fontSize: '1rem',
  fontWeight: '600',
  border: 'none',
  borderRadius: 'var(--radius-md)',
  cursor: 'pointer',
  transition: 'background 0.2s ease, transform 0.2s ease, opacity 0.2s ease',
  color: '#ffffff',
  background: 'var(--lumin-indigo)',
  ':disabled': {
    opacity: '0.5',
    cursor: 'not-allowed',
  },
  selectors: {
    '&:hover:not(:disabled)': {
      background: '#4338ca',
    },
    '&:active:not(:disabled)': {
      transform: 'scale(0.98)',
    },
  },
  '@media': {
    '(prefers-reduced-motion: reduce)': {
      transition: 'none',
      transform: 'none',
    },
  },
});

// index.css L839-849 (shared base), L862-867 (:disabled), L874-877,
// L879-881 (:hover), L1641-1666 (@media prefers-reduced-motion: reduce).
export const secondaryButton = style({
  padding: 'var(--space-3) var(--space-4)',
  fontSize: '1rem',
  fontWeight: '600',
  border: 'none',
  borderRadius: 'var(--radius-md)',
  cursor: 'pointer',
  transition: 'background 0.2s ease, transform 0.2s ease, opacity 0.2s ease',
  color: 'var(--lumin-indigo)',
  background: 'var(--lumin-indigo-soft)',
  ':disabled': {
    opacity: '0.5',
    cursor: 'not-allowed',
  },
  selectors: {
    '&:hover:not(:disabled)': {
      background: '#c7d2fe',
    },
  },
  '@media': {
    '(prefers-reduced-motion: reduce)': {
      transition: 'none',
      transform: 'none',
    },
  },
});

// index.css L677-684.
export const errorMessage = style({
  marginTop: 'var(--space-4)',
  padding: 'var(--space-3) var(--space-4)',
  background: '#fff1f2',
  color: 'var(--lumin-error)',
  borderRadius: 'var(--radius-md)',
  fontSize: '0.875rem',
});

// index.css L115-124, L126-128 (:hover), L130-132 (:active),
// L1641-1666 (@media prefers-reduced-motion: reduce, which also lists
// `.app-bar .reset-button:active` and therefore kills the :active scale).
// Every original rule is `.app-bar`-conditioned (DemoFlow renders this class
// outside .app-bar and is intentionally unstyled), so the primitive keeps the
// ancestor condition instead of a bare base rule.
export const resetButton = style({
  selectors: {
    '.app-bar &': {
      background: 'rgba(255, 255, 255, 0.2)',
      border: 'none',
      color: 'white',
      padding: '6px 12px',
      borderRadius: '8px',
      cursor: 'pointer',
      fontSize: '0.875rem',
      transition: 'background 0.2s ease, transform 0.2s ease',
    },
    '.app-bar &:hover': {
      background: 'rgba(255, 255, 255, 0.3)',
    },
    '.app-bar &:active': {
      transform: 'scale(0.98)',
    },
  },
  '@media': {
    '(prefers-reduced-motion: reduce)': {
      selectors: {
        '.app-bar &': {
          transition: 'none',
          transform: 'none',
        },
        '.app-bar &:active': {
          transition: 'none',
          transform: 'none',
        },
      },
    },
  },
});

// index.css L883-888.
export const privacyNote = style({
  margin: 'var(--space-4) 0 0 0',
  fontSize: '0.875rem',
  color: 'var(--lumin-muted)',
  textAlign: 'center',
});

// index.css L947-953 (base) + L1403-1408 and L2203-2209 (two top-level
// `.question-body .question-prompt` rules; the later one wins per property and
// adds line-height: 1.5, so the merged effective override is encoded).
export const questionPrompt = style({
  margin: '0',
  fontSize: '1.375rem',
  fontWeight: '700',
  lineHeight: '1.35',
  color: 'var(--lumin-ink)',
  selectors: {
    '.question-body &': {
      margin: '0',
      fontSize: '1rem',
      fontWeight: '600',
      lineHeight: '1.5',
      color: 'var(--lumin-ink)',
    },
  },
});

// index.css L939-945 (base) + L1410-1413 and L2211-2214 (two top-level
// `.question-body .question-concept` rules; the later one wins per property).
export const questionConcept = style({
  fontSize: '0.75rem',
  fontWeight: '700',
  letterSpacing: '0.05em',
  textTransform: 'uppercase',
  color: 'var(--lumin-indigo)',
  selectors: {
    '.question-body &': {
      fontSize: '0.875rem',
      color: 'var(--lumin-text-secondary)',
    },
  },
});

// index.css L634-638.
export const progressFill = style({
  height: '100%',
  background: 'var(--lumin-purple)',
  transition: 'width 0.2s ease',
});

// index.css L626-632 (base) + L917-920 (`.student-quiz .progress-bar`).
export const progressBar = style({
  width: '160px',
  height: '8px',
  background: '#e5e7eb',
  borderRadius: '9999px',
  overflow: 'hidden',
  selectors: {
    '.student-quiz &': {
      width: '100%',
      marginBottom: 'var(--space-5)',
    },
  },
});

// index.css L1428-1434 and L2235-2241 (two top-level rules; the later wins per
// property), L2347-2349 (@media max-width: 640px),
// L2582-2586 (`.join-code-banner .join-code-value`).
export const joinCodeValue = style({
  fontFamily: 'ui-monospace, SFMono-Regular, Menlo, Monaco, Consolas, monospace',
  fontSize: 'clamp(3rem, 10vw, 5rem)',
  fontWeight: '800',
  letterSpacing: '0.15em',
  color: 'var(--lumin-ink)',
  selectors: {
    '.join-code-banner &': {
      fontSize: '1.5rem',
      letterSpacing: '0.35em',
      color: 'var(--lumin-primary)',
    },
  },
  '@media': {
    '(max-width: 640px)': {
      fontSize: '3rem',
    },
  },
});

// index.css L1423-1426 and L2227-2233 (two top-level rules; the later wins per
// property — merged effective values encoded).
export const joinCodeLabel = style({
  fontSize: '0.875rem',
  fontWeight: '700',
  letterSpacing: '0.1em',
  textTransform: 'uppercase',
  color: 'var(--lumin-indigo)',
});

// index.css L686-690.
export const emptyState = style({
  color: 'var(--color-muted)',
  padding: 'var(--space-6)',
  textAlign: 'center',
});

// index.css L543-547 (base) + L1563-1567 (@media max-width: 768px, shared
// selector with .student-join/.student-quiz). `.model-manager h2`
// (L549-553) is an ancestor-form rule and intentionally NOT included.
export const modelManager = style({
  padding: 'var(--space-6)',
  maxWidth: '720px',
  margin: '0 auto',
  '@media': {
    '(max-width: 768px)': {
      padding: 'var(--space-4)',
    },
  },
});

// index.css L897-903 (base) + L1569-1572 (@media max-width: 768px) +
// L1610-1615 (@media max-width: 480px, shared selector with .join-header).
export const quizHeader = style({
  display: 'flex',
  alignItems: 'center',
  justifyContent: 'space-between',
  gap: 'var(--space-3)',
  marginBottom: 'var(--space-3)',
  '@media': {
    '(max-width: 768px)': {
      flexDirection: 'column',
      alignItems: 'flex-start',
    },
    '(max-width: 480px)': {
      flexDirection: 'column',
      alignItems: 'flex-start',
      gap: '8px',
    },
  },
});
