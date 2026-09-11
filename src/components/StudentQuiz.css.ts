import { style } from "@vanilla-extract/css";

// StudentQuiz.tsx batch (plan checkbox 8, last component batch). Declaration
// values are verbatim copies of the corresponding src/index.css rules.
//
// Shared T3 primitives wired in the TSX: quizHeader, questionPrompt,
// questionConcept (their literal '.question-body &' anchors have been dead
// since T5 hashed question-body; StudentQuiz never had that ancestor),
// progressBar + progressFill, primaryButton, secondaryButton, errorMessage,
// privacyNote. progressBar's literal '.student-quiz &' anchor stops matching
// once student-quiz is hashed, so the descendant-context override is re-homed
// below as progressBarInQuiz (same pattern as the T5 questionPromptInBody).
//
// answerInput is the answer-input share of the SHARED input band (index.css
// L397-408 + L410-415 :focus indigo) merged with the out-of-band own rules
// (L539-542 width/font-size — the band's 1rem font-size is overridden by the
// later own rule, so the merged effective value is encoded) and :disabled
// (L544-546, top-level key). baseInputFocus (the global stylesheet module,
// imported first in the TSX) is attached alongside: its ':focus' purple ties
// at (0,2,0) and loses to the later-imported indigo; its '::placeholder'
// becomes the only placeholder source now that the element pseudo band is
// deleted.
//
// feedback/feedbackCorrect/feedbackIncorrect reproduce the `.feedback` +
// `.feedback.correct` / `.feedback.incorrect` compound via template-literal
// className combinations in the TSX (the branches only render in the
// correct/incorrect states, so the combined string matches the old compound).

// index.css L475-479 + @media 768 L637-641 share + @media 480 L665-668 share.
export const studentQuiz = style({
  maxWidth: "720px",
  margin: "0 auto",
  padding: "var(--space-6)",
  "@media": {
    "(max-width: 768px)": {
      padding: "var(--space-4)",
    },
    "(max-width: 480px)": {
      padding: "var(--space-3)",
    },
  },
});

// index.css L489-492.
export const quizTitle = style({
  fontWeight: "700",
  color: "var(--lumin-ink)",
});

// index.css L494-499.
export const quizProgress = style({
  fontFamily:
    "ui-monospace, SFMono-Regular, Menlo, Monaco, Consolas, monospace",
  fontSize: "0.9375rem",
  fontWeight: "600",
  color: "var(--lumin-text-secondary)",
});

// Re-homed from the T3 shared progressBar primitive: that primitive anchors
// on the literal '.student-quiz &' selector, which stops matching once
// className="student-quiz" becomes the hashed studentQuiz class. The
// descendant-context override (index.css L501-504) is rebuilt here as a real
// VE cross-reference with verbatim values; the primitive itself is still
// wired alongside so its base rule applies, exactly like the original
// base + descendant-rule pair.
export const progressBarInQuiz = style({
  selectors: {
    [`${studentQuiz} &`]: {
      width: "100%",
      marginBottom: "var(--space-5)",
    },
  },
});

// index.css L506-514 (question-card share of the card band) + L516-521 (own
// rules) + @media 480 L677-681 share.
export const questionCard = style({
  padding: "var(--space-5)",
  background: "var(--lumin-card)",
  border: "1px solid var(--lumin-border)",
  borderRadius: "var(--radius-lg)",
  boxShadow: "var(--shadow-sm)",
  display: "flex",
  flexDirection: "column",
  gap: "var(--space-4)",
  marginBottom: "var(--space-4)",
  "@media": {
    "(max-width: 480px)": {
      padding: "12px",
    },
  },
});

// index.css L506-514 (hint-card share of the card band) + L561-563 (own
// rules) + @media 480 L677-681 share.
export const hintCard = style({
  padding: "var(--space-5)",
  background: "var(--lumin-card)",
  border: "1px solid var(--lumin-border)",
  borderRadius: "var(--radius-lg)",
  boxShadow: "var(--shadow-sm)",
  marginBottom: "var(--space-4)",
  "@media": {
    "(max-width: 480px)": {
      padding: "12px",
    },
  },
});

// index.css L506-514 (completion-card share of the card band) + L603-611
// (own rules) + @media 480 L683-685.
export const completionCard = style({
  padding: "var(--space-5)",
  background: "var(--lumin-card)",
  border: "1px solid var(--lumin-border)",
  borderRadius: "var(--radius-lg)",
  boxShadow: "var(--shadow-sm)",
  display: "flex",
  flexDirection: "column",
  alignItems: "center",
  gap: "var(--space-4)",
  textAlign: "center",
  minHeight: "360px",
  justifyContent: "center",
  "@media": {
    "(max-width: 480px)": {
      padding: "16px",
    },
  },
});

// index.css L397-408 (answer-input share of the shared band) + L410-415
// (answer-input share of the :focus indigo group) + L539-542 (own rules;
// font-size 1.25rem is the effective value, the band's 1rem is overridden
// by the later own rule) + L544-546 (:disabled).
export const answerInput = style({
  padding: "var(--space-3)",
  fontSize: "1.25rem",
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
  width: "100%",
  ":disabled": {
    background: "#f3f4f6",
  },
});

// index.css L548-551.
export const feedback = style({
  margin: "0",
  fontWeight: "600",
});

// index.css L553-555 (.feedback.correct share).
export const feedbackCorrect = style({
  color: "var(--lumin-success)",
});

// index.css L557-559 (.feedback.incorrect share).
export const feedbackIncorrect = style({
  color: "var(--lumin-error)",
});

// index.css L565-571.
export const hintHeader = style({
  display: "flex",
  alignItems: "center",
  justifyContent: "space-between",
  gap: "var(--space-3)",
  marginBottom: "var(--space-3)",
});

// index.css L573-579 + ::before L581-583.
export const hintLabel = style({
  display: "inline-flex",
  alignItems: "center",
  gap: "var(--space-2)",
  fontWeight: "700",
  color: "var(--lumin-purple)",
  selectors: {
    "&::before": {
      content: '"💡"',
    },
  },
});

// index.css L585-588.
export const hintGuard = style({
  fontSize: "0.75rem",
  color: "var(--lumin-muted)",
});

// index.css L590-595.
export const hintText = style({
  margin: "0 0 var(--space-4) 0",
  fontSize: "1.125rem",
  lineHeight: "1.5",
  color: "var(--lumin-ink)",
});

// index.css L597-601.
export const hintActions = style({
  display: "flex",
  flexWrap: "wrap",
  gap: "var(--space-3)",
});

// index.css L613-616.
export const completionIcon = style({
  fontSize: "3.5rem",
  color: "var(--lumin-purple)",
});

// index.css L618-623 + @media 480 L687-689.
export const completionTitle = style({
  margin: "0",
  fontSize: "1.75rem",
  fontWeight: "800",
  color: "var(--lumin-ink)",
  "@media": {
    "(max-width: 480px)": {
      fontSize: "1.25rem",
    },
  },
});

// index.css L625-629.
export const completionMessage = style({
  margin: "0",
  color: "var(--lumin-text-secondary)",
  lineHeight: "1.6",
});
