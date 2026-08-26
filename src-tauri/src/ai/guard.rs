//! Answer-leak guard and rule-based fallback for AI analysis.
//!
//! Mirrors the Swift `GemmaService.sanitizedHint` logic:
//! 1. Normalize both the AI-generated hint and each accepted answer.
//! 2. If any normalized accepted answer is a substring of the normalized hint,
//!    the hint leaks the answer — fall back to `question.hints[level-1]`.
//! 3. If a misconception code is not in the question's known misconceptions,
//!    fall back to `generic_misconception`.
//! 4. If AI generation fails entirely, fall back to `RuleBasedLearningEngine`.

use crate::lumin_core::analyzer::RuleBasedLearningEngine;
use crate::lumin_core::{AnswerAnalysis, QuizQuestion};

// ---------------------------------------------------------------------------
// Hint leak guard
// ---------------------------------------------------------------------------

/// Check if a hint leaks the accepted answer.
///
/// Uses the same normalization as [`RuleBasedLearningEngine`] so fullwidth
/// characters (e.g. `３`) are compared against halfwidth (`3`).
///
/// Returns `Some(fallback_hint)` if a leak is detected, `None` if safe.
pub fn check_hint_leak(hint: &str, question: &QuizQuestion) -> Option<String> {
    let normalized_hint = RuleBasedLearningEngine::normalize(hint);
    for accepted in &question.accepted_answers {
        let normalized_accepted = RuleBasedLearningEngine::normalize(accepted);
        if !normalized_accepted.is_empty() && normalized_hint.contains(&normalized_accepted) {
            return Some(fallback_hint(question, 1));
        }
    }
    None
}

/// Sanitize an AI-generated hint: check for answer leaks and truncate.
///
/// Matches Swift `sanitizedHint` behavior:
/// - Empty or leaking hint → fallback to `question.hints[level-1]`
/// - Safe hint → truncate to 80 characters
pub fn sanitize_hint(hint: &str, question: &QuizQuestion, level: i32) -> String {
    let cleaned = hint.trim();
    if cleaned.is_empty() || check_hint_leak(cleaned, question).is_some() {
        return fallback_hint(question, level);
    }
    // Truncate to 80 chars (matching Swift behavior)
    cleaned.chars().take(80).collect()
}

/// Get the fallback hint at the given level (1-indexed).
///
/// Bounds-safe: clamps to available hints, falls back to first hint or a
/// generic message if the question has no hints at all.
fn fallback_hint(question: &QuizQuestion, level: i32) -> String {
    let idx = (level - 1).max(0) as usize;
    question
        .hints
        .get(idx)
        .cloned()
        .or_else(|| question.hints.first().cloned())
        .unwrap_or_else(|| "ヒントはありません。".to_string())
}

// ---------------------------------------------------------------------------
// Misconception validation
// ---------------------------------------------------------------------------

/// Validate that a misconception is in the question's known misconceptions.
///
/// If the misconception value appears in `misconception_answers` values,
/// it is returned as-is. Otherwise, `generic_misconception` is returned.
pub fn validate_misconception(misconception: &str, question: &QuizQuestion) -> String {
    if question
        .misconception_answers
        .values()
        .any(|v| v == misconception)
    {
        misconception.to_string()
    } else {
        question.generic_misconception.clone()
    }
}

// ---------------------------------------------------------------------------
// Full analyze pipeline
// ---------------------------------------------------------------------------

/// Full analyze pipeline: AI → leak guard → fallback.
///
/// - On AI success: validates the misconception code (falls back to
///   `generic_misconception` if unknown).
/// - On AI failure: falls back to [`RuleBasedLearningEngine::analyze`].
pub fn analyze_with_guard(
    question: &QuizQuestion,
    student_answer: &str,
    ai_output: Result<AnswerAnalysis, String>,
) -> AnswerAnalysis {
    match ai_output {
        Ok(analysis) => {
            // Validate misconception code
            AnswerAnalysis {
                is_correct: analysis.is_correct,
                misconception: analysis
                    .misconception
                    .map(|m| validate_misconception(&m, question)),
            }
        }
        Err(_) => {
            // Fall back to rule-based analysis
            let engine = RuleBasedLearningEngine;
            engine.analyze(student_answer, question)
        }
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod guard_tests {
    use super::*;
    use std::collections::HashMap;

    /// Linear functions question with accepted answer "3" / "+3".
    fn question_with_answers() -> QuizQuestion {
        QuizQuestion {
            id: "lf-q4".into(),
            prompt: "y = 3x + 2 の傾きは？".into(),
            concept: "一次関数".into(),
            accepted_answers: vec!["3".into(), "+3".into()],
            misconception_answers: HashMap::new(),
            generic_misconception: "傾きと切片の混同".into(),
            hints: vec!["y = mx + b の m を探そう".into()],
            explanation: "y = mx + b において m = 3".into(),
        }
    }

    // --- check_hint_leak ---

    #[test]
    fn test_hint_leak_detected_fullwidth_3() {
        let q = question_with_answers();
        // "答えは３です" → normalized contains "3"
        assert!(check_hint_leak("答えは３です", &q).is_some());
    }

    #[test]
    fn test_hint_no_leak() {
        let q = question_with_answers();
        // "傾きと切片を混同しているようです" → no accepted answer substring
        assert!(check_hint_leak("傾きと切片を混同しているようです", &q).is_none());
    }

    #[test]
    fn test_hint_leak_detected_ascii_3() {
        let q = question_with_answers();
        // "答えは3です" → normalized contains "3"
        assert!(check_hint_leak("答えは3です", &q).is_some());
    }

    #[test]
    fn test_hint_leak_returns_fallback() {
        let q = question_with_answers();
        let fallback = check_hint_leak("答えは３です", &q).unwrap();
        assert_eq!(fallback, "y = mx + b の m を探そう");
    }

    #[test]
    fn test_hint_leak_with_spaces() {
        let q = question_with_answers();
        // "答えは 3 です" → after normalization, spaces removed, contains "3"
        assert!(check_hint_leak("答えは 3 です", &q).is_some());
    }

    #[test]
    fn test_hint_leak_empty_accepted_ignored() {
        let q = question_with_answers();
        let mut q2 = q.clone();
        q2.accepted_answers.push("".into());
        // Empty accepted answer should be ignored; "3" still leaks
        assert!(check_hint_leak("答えは３です", &q2).is_some());
    }

    #[test]
    fn test_hint_no_leak_unrelated_text() {
        let q = question_with_answers();
        assert!(check_hint_leak("式の形に注目してください", &q).is_none());
    }

    #[test]
    fn test_hint_leak_plus3_variant() {
        let q = question_with_answers();
        // "+3" is also an accepted answer → "答えは＋３です" leaks
        assert!(check_hint_leak("答えは＋３です", &q).is_some());
    }

    // --- sanitize_hint ---

    #[test]
    fn test_sanitize_hint_leak_replaces() {
        let q = question_with_answers();
        let result = sanitize_hint("答えは３です", &q, 1);
        assert_eq!(result, "y = mx + b の m を探そう");
    }

    #[test]
    fn test_sanitize_hint_no_leak_passthrough() {
        let q = question_with_answers();
        let result = sanitize_hint("傾きと切片を混同しているようです", &q, 1);
        assert_eq!(result, "傾きと切片を混同しているようです");
    }

    #[test]
    fn test_sanitize_hint_empty_uses_fallback() {
        let q = question_with_answers();
        let result = sanitize_hint("", &q, 1);
        assert_eq!(result, "y = mx + b の m を探そう");
    }

    #[test]
    fn test_sanitize_hint_whitespace_only_uses_fallback() {
        let q = question_with_answers();
        let result = sanitize_hint("   ", &q, 1);
        assert_eq!(result, "y = mx + b の m を探そう");
    }

    #[test]
    fn test_sanitize_hint_truncates_to_80() {
        let q = question_with_answers();
        let long_hint = "あ".repeat(100);
        let result = sanitize_hint(&long_hint, &q, 1);
        assert_eq!(result.chars().count(), 80);
    }

    #[test]
    fn test_sanitize_hint_level_selects_fallback() {
        let mut q = question_with_answers();
        q.hints.push("レベル2のヒント".into());
        q.hints.push("レベル3のヒント".into());
        let result = sanitize_hint("答えは３です", &q, 2);
        assert_eq!(result, "レベル2のヒント");
    }

    #[test]
    fn test_sanitize_hint_level_out_of_bounds() {
        let q = question_with_answers();
        // Level 99 doesn't exist → falls back to first hint
        let result = sanitize_hint("答えは３です", &q, 99);
        assert_eq!(result, "y = mx + b の m を探そう");
    }

    // --- validate_misconception ---

    #[test]
    fn test_invalid_misconception_falls_back_to_generic() {
        let q = question_with_answers();
        assert_eq!(
            validate_misconception("不明な誤概念", &q),
            "傾きと切片の混同"
        );
    }

    #[test]
    fn test_valid_misconception_preserved() {
        let mut q = question_with_answers();
        q.misconception_answers
            .insert("2".into(), "傾きと切片の混同".into());
        // The value "傾きと切片の混同" exists in misconception_answers values
        assert_eq!(
            validate_misconception("傾きと切片の混同", &q),
            "傾きと切片の混同"
        );
    }

    #[test]
    fn test_empty_misconception_not_in_values() {
        let q = question_with_answers();
        // Empty string is not in any value → falls back to generic
        assert_eq!(validate_misconception("", &q), "傾きと切片の混同");
    }

    // --- analyze_with_guard ---

    #[test]
    fn test_analyze_with_guard_ai_error_falls_back_to_rule_based() {
        let q = question_with_answers();
        let result = analyze_with_guard(&q, "2", Err("AI failed".to_string()));
        // "2" is not in accepted_answers ["3", "+3"] → incorrect
        assert!(!result.is_correct);
    }

    #[test]
    fn test_analyze_with_guard_ai_error_correct_answer() {
        let q = question_with_answers();
        let result = analyze_with_guard(&q, "3", Err("AI failed".to_string()));
        // "3" matches accepted answer → correct
        assert!(result.is_correct);
        assert!(result.misconception.is_none());
    }

    #[test]
    fn test_analyze_with_guard_valid_misconception() {
        let q = question_with_answers();
        let ai_output = Ok(AnswerAnalysis {
            is_correct: false,
            misconception: Some("傾きと切片の混同".into()),
        });
        let result = analyze_with_guard(&q, "2", ai_output);
        assert!(!result.is_correct);
        assert_eq!(result.misconception.as_deref(), Some("傾きと切片の混同"));
    }

    #[test]
    fn test_analyze_with_guard_invalid_misconception_falls_back() {
        let q = question_with_answers();
        let ai_output = Ok(AnswerAnalysis {
            is_correct: false,
            misconception: Some("不明な誤概念".into()),
        });
        let result = analyze_with_guard(&q, "2", ai_output);
        assert!(!result.is_correct);
        // Falls back to generic_misconception
        assert_eq!(result.misconception.as_deref(), Some("傾きと切片の混同"));
    }

    #[test]
    fn test_analyze_with_guard_correct_answer_no_misconception() {
        let q = question_with_answers();
        let ai_output = Ok(AnswerAnalysis {
            is_correct: true,
            misconception: None,
        });
        let result = analyze_with_guard(&q, "3", ai_output);
        assert!(result.is_correct);
        assert!(result.misconception.is_none());
    }
}
