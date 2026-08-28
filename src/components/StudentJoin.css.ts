import { style } from '@vanilla-extract/css';

// StudentJoin.tsx batch (plan checkbox 8, last component batch). Declaration
// values are verbatim copies of the corresponding src/index.css rules.
//
// This file owns the StudentJoin share of the SHARED input band (index.css
// L397-408 `.manual-input, .code-input, .answer-input` + L410-415 `:focus`
// indigo group): manualInput (verbatim duplicate of the LessonPlanEditor
// module's manualInput — VE hashes scope class names, so duplication is the
// safe form) and codeInput (band share + focus + the out-of-band monospace/
// center rules L417-421). The answer-input share lives in StudentQuiz.css.ts.
// baseInputFocus (the global stylesheet module, imported first in the TSX) is
// attached alongside both in the TSX: its ':focus' purple ties at (0,2,0)
// and loses to the later-imported indigo, reproducing the old element-vs-class
// cascade; its '::placeholder' becomes the only placeholder source now that
// the element pseudo band is deleted.
//
// joinButton is the join-button share of the former button band (base +
// color/hover/disabled/active + reduced-motion). Its reduced-motion
// transform:none intentionally loses to the :active scale, matching the
// original specificity (.join-button:active (0,2,0) vs
// .join-button:active:not(:disabled) (0,3,0)).
// joinReset is single-owner: its reduced-motion ':active' override had EQUAL
// specificity to the base ':active' scale and won by source order, so it is
// re-encoded as a same-specificity selector inside the media block.

// index.css L277-281 + @media 768 L637-641 share + @media 480 L665-668 share.
export const studentJoin = style({
  maxWidth: '720px',
  margin: '0 auto',
  padding: 'var(--space-6)',
  '@media': {
    '(max-width: 768px)': {
      padding: 'var(--space-4)',
    },
    '(max-width: 480px)': {
      padding: 'var(--space-3)',
    },
  },
});

// index.css L283-289 + @media 480 L670-675 share (with .quiz-header).
export const joinHeader = style({
  display: 'flex',
  alignItems: 'center',
  justifyContent: 'space-between',
  gap: 'var(--space-3)',
  marginBottom: 'var(--space-4)',
  '@media': {
    '(max-width: 480px)': {
      flexDirection: 'column',
      alignItems: 'flex-start',
      gap: '8px',
    },
  },
});

// index.css L291-295.
export const joinTitle = style({
  margin: '0',
  fontSize: '1.5rem',
  color: 'var(--lumin-ink)',
});

// index.css L297-306 + :hover L308-310 + :active L312-314 + reduced-motion
// group lines L693-695 (transition none at (0,1,0); ':active' transform none
// re-encoded at (0,2,0) because the original tied with the scale and won by
// source order).
export const joinReset = style({
  padding: '0.5em 1em',
  fontSize: '0.875rem',
  color: 'var(--lumin-purple)',
  background: 'var(--lumin-purple-soft)',
  border: '1px solid transparent',
  borderRadius: 'var(--radius-md)',
  cursor: 'pointer',
  transition: 'background 0.2s ease, transform 0.2s ease',
  selectors: {
    '&:hover': {
      background: '#ddd6fe',
    },
    '&:active': {
      transform: 'scale(0.98)',
    },
  },
  '@media': {
    '(prefers-reduced-motion: reduce)': {
      transition: 'none',
      selectors: {
        '&:active': {
          transform: 'none',
        },
      },
    },
  },
});

// index.css L316-319.
export const joinSubtitle = style({
  margin: '0 0 var(--space-5) 0',
  color: 'var(--lumin-text-secondary)',
});

// index.css L321-327 + @media 480 L677-681 share (with .question-card/
// .hint-card).
export const joinCard = style({
  padding: 'var(--space-5)',
  background: 'var(--lumin-card)',
  border: '1px solid var(--lumin-border)',
  borderRadius: 'var(--radius-lg)',
  boxShadow: 'var(--shadow-sm)',
  '@media': {
    '(max-width: 480px)': {
      padding: '12px',
    },
  },
});

// index.css L329-333.
export const joinStatus = style({
  margin: '0 0 var(--space-4) 0',
  color: 'var(--lumin-text-secondary)',
  textAlign: 'center',
});

// index.css L335-339 (StudentJoin is the only TSX user).
export const sectionLabel = style({
  margin: '0 0 var(--space-3) 0',
  fontSize: '1rem',
  color: 'var(--lumin-ink)',
});

// index.css L341-345.
export const teacherSection = style({
  marginBottom: 'var(--space-5)',
  paddingBottom: 'var(--space-5)',
  borderBottom: '1px solid var(--lumin-border)',
});

// index.css L347-351.
export const teacherList = style({
  display: 'flex',
  flexDirection: 'column',
  gap: 'var(--space-3)',
});

// index.css L353-362 + @media 640 L654-657.
export const teacherCard = style({
  display: 'grid',
  gridTemplateColumns: '1fr auto auto',
  alignItems: 'center',
  gap: 'var(--space-3)',
  padding: 'var(--space-3)',
  background: 'var(--lumin-canvas)',
  border: '1px solid var(--lumin-border)',
  borderRadius: 'var(--radius-md)',
  '@media': {
    '(max-width: 640px)': {
      gridTemplateColumns: '1fr',
      alignItems: 'stretch',
    },
  },
});

// index.css L364-369.
export const teacherInfo = style({
  display: 'flex',
  flexDirection: 'column',
  gap: 'var(--space-1)',
  minWidth: '0',
});

// index.css L371-377.
export const teacherName = style({
  fontWeight: '600',
  color: 'var(--lumin-ink)',
  whiteSpace: 'nowrap',
  overflow: 'hidden',
  textOverflow: 'ellipsis',
});

// index.css L379-383.
export const teacherMeta = style({
  fontSize: '0.75rem',
  color: 'var(--lumin-muted)',
  fontFamily: 'ui-monospace, SFMono-Regular, Menlo, Monaco, Consolas, monospace',
});

// index.css L385-389.
export const manualJoin = style({
  display: 'flex',
  flexDirection: 'column',
  gap: 'var(--space-3)',
});

// index.css L391-395 + @media 640 L650-652.
export const manualFields = style({
  display: 'grid',
  gridTemplateColumns: '1fr 1fr 1fr',
  gap: 'var(--space-3)',
  '@media': {
    '(max-width: 640px)': {
      gridTemplateColumns: '1fr',
    },
  },
});

// index.css L397-408 (manual-input share of the shared band) + L410-415
// (manual-input share of the :focus indigo group). Verbatim duplicate of the
// LessonPlanEditor module's manualInput — separate VE scope hash, same values.
export const manualInput = style({
  padding: 'var(--space-3)',
  fontSize: '1rem',
  color: 'var(--lumin-ink)',
  background: 'var(--lumin-canvas)',
  border: '1px solid var(--lumin-border)',
  borderRadius: 'var(--radius-md)',
  outline: 'none',
  transition: 'border-color 0.2s ease, box-shadow 0.2s ease',
  ':focus': {
    borderColor: 'var(--lumin-indigo)',
    boxShadow: '0 0 0 3px var(--lumin-indigo-soft)',
  },
});

// index.css L397-408 (code-input share of the shared band) + L410-415
// (code-input share of the :focus indigo group) + L417-421 (out-of-band
// monospace/center rules).
export const codeInput = style({
  padding: 'var(--space-3)',
  fontSize: '1rem',
  color: 'var(--lumin-ink)',
  background: 'var(--lumin-canvas)',
  border: '1px solid var(--lumin-border)',
  borderRadius: 'var(--radius-md)',
  outline: 'none',
  transition: 'border-color 0.2s ease, box-shadow 0.2s ease',
  ':focus': {
    borderColor: 'var(--lumin-indigo)',
    boxShadow: '0 0 0 3px var(--lumin-indigo-soft)',
  },
  fontFamily: 'ui-monospace, SFMono-Regular, Menlo, Monaco, Consolas, monospace',
  textAlign: 'center',
  letterSpacing: '0.15em',
});

// index.css L423-433 (join-button share of the button band base) +
// L435-439 (color/background) + L441-444 (:hover) + L446-451 share
// (:disabled) + L453-456 (:active) + reduced-motion group lines L696-698.
export const joinButton = style({
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
