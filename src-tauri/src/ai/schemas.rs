//! JSON schemas and prompt templates for AI analysis, hints, and lesson plans.
//!
//! Mirrors the Swift `GemmaService` schemas (analysisSchema / lessonPlanSchema)
//! so the Rust backend produces output the frontend already knows how to parse.
//!
//! All prompts are in Japanese and end with the guard
//! `JSON以上は出力しないでください` ("Do not output anything other than JSON")
//! to keep the model's response machine-parseable.

use crate::lumin_core::{ClassSummary, Quiz, QuizQuestion};
use serde::{Deserialize, Serialize};

// ---------------------------------------------------------------------------
// Analysis output schema
// ---------------------------------------------------------------------------

/// Structured output for [`analyze_prompt`].
///
/// Mirrors Swift `analysisSchema`:
/// ```json
/// {
///   "misconception": "m0",
///   "hint": "通分について考えましょう"
/// }
/// ```
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct AnalysisOutput {
    /// Misconception code (e.g. "m0", "m1") or a short label.
    pub misconception: String,
    /// Hint text, max 80 characters.
    pub hint: String,
}

// ---------------------------------------------------------------------------
// Lesson plan output schema
// ---------------------------------------------------------------------------

/// Structured output for [`lesson_plan_prompt`].
///
/// Mirrors Swift `lessonPlanSchema`:
/// ```json
/// {
///   "focus": "通分の必要性",
///   "steps": ["步骤1", "步骤2", "步骤3", "步骤4"],
///   "checkQuestion": "確認問題",
///   "teacherNote": "教師への注意点"
/// }
/// ```
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct LessonPlanOutput {
    /// Lesson focus, max 60 characters.
    pub focus: String,
    /// Exactly 4 lesson steps, each max 100 characters.
    pub steps: Vec<String>,
    /// Check question, max 120 characters.
    pub check_question: String,
    /// Teacher note, max 120 characters.
    pub teacher_note: String,
}

// ---------------------------------------------------------------------------
// Prompt builders
// ---------------------------------------------------------------------------

/// Build the analysis prompt for a student's answer.
///
/// The prompt:
/// - Includes the question, student answer, and concept
/// - Builds misconception candidates as `code=label` pairs (like Swift)
/// - Asks for a JSON output with `misconception` code and `hint`
/// - Does NOT reveal accepted answers
/// - Ends with the JSON-only guard
pub fn analyze_prompt(question: &QuizQuestion, student_answer: &str, hint_level: i32) -> String {
    // Build candidate misconception codes like Swift: "m0=label / m1=label"
    let mut candidate_labels: Vec<String> = question
        .misconception_answers
        .values()
        .cloned()
        .collect::<std::collections::HashSet<_>>()
        .into_iter()
        .collect();
    candidate_labels.sort();

    // Always include the generic misconception as the last candidate
    if !question.generic_misconception.is_empty()
        && !candidate_labels.contains(&question.generic_misconception)
    {
        candidate_labels.push(question.generic_misconception.clone());
    }

    let candidate_pairs: Vec<String> = candidate_labels
        .iter()
        .enumerate()
        .map(|(i, label)| format!("m{i}={label}"))
        .collect();
    let candidates = candidate_pairs.join(" / ");

    let level = hint_level.clamp(1, 3);
    let level_desc = match level {
        1 => "1（方向だけ）",
        2 => "2（使う考え方）",
        _ => "3（似た途中式）",
    };

    format!(
        r#"問題: {}
生徒の回答: {}
学習概念: {}
誤概念候補: {}
ヒント段階: {}

候補コードを一つ選び、正解そのものを示さず、中学生向けの短いヒントを一つ作ってください。
JSON以上は出力しないでください。
出力形式: {{"misconception": "候補コード", "hint": "ヒント（80文字以内）"}}"#,
        question.prompt, student_answer, question.concept, candidates, level_desc
    )
}

/// Build the lesson plan prompt from a class summary and optional quiz.
///
/// The prompt:
/// - Includes anonymous class statistics
/// - Includes quiz metadata (subject, title, topic, concepts) if available
/// - Asks for a 4-step 10-minute lesson plan
/// - Ends with the JSON-only guard
pub fn lesson_plan_prompt(summary: &ClassSummary, quiz: Option<&Quiz>) -> String {
    let misconception_text = if summary.misconceptions.is_empty() {
        "回答なし".to_string()
    } else {
        summary
            .misconceptions
            .iter()
            .take(4)
            .map(|m| format!("{}: {}%", m.name, (m.share * 100.0).round() as i32))
            .collect::<Vec<_>>()
            .join(", ")
    };

    let subject = quiz.map(|q| q.subject.as_str()).unwrap_or("未指定");
    let title = quiz.map(|q| q.title.as_str()).unwrap_or("未指定");
    let topic = quiz.and_then(|q| q.topic.as_deref()).unwrap_or("未指定");
    let concepts = quiz
        .map(|q| {
            let cs: Vec<&str> = q.questions.iter().map(|qq| qq.concept.as_str()).collect();
            if cs.is_empty() {
                "未指定".to_string()
            } else {
                cs.join("、")
            }
        })
        .unwrap_or_else(|| "未指定".to_string());

    let correct_pct = (summary.correct_rate * 100.0).round() as i32;
    let retry_pct = (summary.retry_success_rate * 100.0).round() as i32;

    format!(
        r#"匿名のクラス集計から、授業冒頭10分の案を作ってください。
教科: {subject}
教材: {title}
単元: {topic}
出題概念: {concepts}
参加端末: {participants}
初回正答率: {correct_pct}%
ヒント後の再挑戦成功率: {retry_pct}%
主な誤概念: {misconceptions}

4段階で合計10分にし、生徒が説明・比較・再挑戦する活動を含めてください。
AI案を教師が修正する前提で注意点も付けてください。
JSON以上は出力しないでください。
出力形式: {{"focus": "焦点（60文字以内）", "steps": ["步骤1", "步骤2", "步骤3", "步骤4"], "checkQuestion": "確認問題（120文字以内）", "teacherNote": "教師への注意点（120文字以内）"}}"#,
        subject = subject,
        title = title,
        topic = topic,
        concepts = concepts,
        participants = summary.participant_count,
        correct_pct = correct_pct,
        retry_pct = retry_pct,
        misconceptions = misconception_text
    )
}

// ---------------------------------------------------------------------------
// Validation
// ---------------------------------------------------------------------------

/// Parse and validate analysis output JSON.
///
/// Rejects:
/// - Invalid JSON
/// - Empty `misconception` or `hint`
/// - `hint` exceeding 80 characters
pub fn validate_analysis_output(json: &str) -> Result<AnalysisOutput, String> {
    let output: AnalysisOutput =
        serde_json::from_str(json).map_err(|e| format!("Invalid JSON: {}", e))?;

    if output.misconception.trim().is_empty() {
        return Err("misconception field is empty".to_string());
    }
    if output.hint.trim().is_empty() {
        return Err("hint field is empty".to_string());
    }
    if output.hint.chars().count() > 80 {
        return Err(format!(
            "Hint exceeds 80 characters (got {})",
            output.hint.chars().count()
        ));
    }

    Ok(output)
}

/// Parse and validate lesson plan output JSON.
///
/// Rejects:
/// - Invalid JSON
/// - Empty `focus`, `check_question`, or `teacher_note`
/// - `steps` not having exactly 4 items
/// - Any step being empty
/// - `focus` exceeding 60 characters
/// - Any step exceeding 100 characters
/// - `check_question` or `teacher_note` exceeding 120 characters
pub fn validate_lesson_plan_output(json: &str) -> Result<LessonPlanOutput, String> {
    let output: LessonPlanOutput =
        serde_json::from_str(json).map_err(|e| format!("Invalid JSON: {}", e))?;

    if output.focus.trim().is_empty() {
        return Err("focus field is empty".to_string());
    }
    if output.focus.chars().count() > 60 {
        return Err(format!(
            "focus exceeds 60 characters (got {})",
            output.focus.chars().count()
        ));
    }
    if output.steps.len() != 4 {
        return Err(format!(
            "steps must have exactly 4 items (got {})",
            output.steps.len()
        ));
    }
    for (i, step) in output.steps.iter().enumerate() {
        if step.trim().is_empty() {
            return Err(format!("steps[{}] is empty", i));
        }
        if step.chars().count() > 100 {
            return Err(format!(
                "steps[{}] exceeds 100 characters (got {})",
                i,
                step.chars().count()
            ));
        }
    }
    if output.check_question.trim().is_empty() {
        return Err("checkQuestion field is empty".to_string());
    }
    if output.check_question.chars().count() > 120 {
        return Err(format!(
            "checkQuestion exceeds 120 characters (got {})",
            output.check_question.chars().count()
        ));
    }
    if output.teacher_note.trim().is_empty() {
        return Err("teacherNote field is empty".to_string());
    }
    if output.teacher_note.chars().count() > 120 {
        return Err(format!(
            "teacherNote exceeds 120 characters (got {})",
            output.teacher_note.chars().count()
        ));
    }

    Ok(output)
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lumin_core::{MisconceptionSummary, QuizQuestion};
    use std::collections::HashMap;

    fn sample_question() -> QuizQuestion {
        let mut misconceptions = HashMap::new();
        misconceptions.insert("2/5".into(), "added numerators".into());
        misconceptions.insert("2/6".into(), "added denominators".into());
        QuizQuestion {
            id: "q1".into(),
            prompt: "1/2 + 1/3 は？".into(),
            concept: "分数の足し算".into(),
            accepted_answers: vec!["5/6".into()],
            misconception_answers: misconceptions,
            generic_misconception: "分数の計算ミス".into(),
            hints: vec!["通分しましょう".into()],
            explanation: "3/6 + 2/6 = 5/6".into(),
        }
    }

    fn sample_summary() -> ClassSummary {
        ClassSummary {
            participant_count: 10,
            response_count: 10,
            correct_rate: 0.6,
            retry_success_rate: 0.5,
            average_hints: 1.5,
            misconceptions: vec![MisconceptionSummary {
                name: "added numerators".into(),
                count: 4,
                share: 0.4,
            }],
        }
    }

    // --- Prompt tests ---

    #[test]
    fn analyze_prompt_includes_question_and_concept() {
        let q = sample_question();
        let prompt = analyze_prompt(&q, "2/5", 2);
        assert!(prompt.contains("1/2 + 1/3 は？"));
        assert!(prompt.contains("生徒の回答: 2/5"));
        assert!(prompt.contains("分数の足し算"));
        assert!(prompt.contains("JSON以上は出力しないでください"));
    }

    #[test]
    fn analyze_prompt_does_not_leak_accepted_answers() {
        let q = sample_question();
        let prompt = analyze_prompt(&q, "2/5", 1);
        // The prompt must NOT contain the accepted answer "5/6"
        assert!(!prompt.contains("5/6"));
    }

    #[test]
    fn analyze_prompt_includes_misconception_candidates() {
        let q = sample_question();
        let prompt = analyze_prompt(&q, "2/5", 1);
        // Should include candidate codes like m0=..., m1=...
        assert!(prompt.contains("m0=") || prompt.contains("m1="));
        // Should include the generic misconception
        assert!(prompt.contains("分数の計算ミス"));
    }

    #[test]
    fn analyze_prompt_clamps_hint_level() {
        let q = sample_question();
        let prompt = analyze_prompt(&q, "2/5", 5);
        // Level 5 should clamp to 3
        assert!(prompt.contains("3（似た途中式）"));
    }

    #[test]
    fn lesson_plan_prompt_includes_summary_stats() {
        let s = sample_summary();
        let prompt = lesson_plan_prompt(&s, None);
        assert!(prompt.contains("参加端末: 10"));
        assert!(prompt.contains("初回正答率: 60%"));
        assert!(prompt.contains("再挑戦成功率: 50%"));
        assert!(prompt.contains("added numerators: 40%"));
        assert!(prompt.contains("JSON以上は出力しないでください"));
    }

    #[test]
    fn lesson_plan_prompt_includes_quiz_metadata() {
        let s = sample_summary();
        let q = Quiz {
            id: "quiz1".into(),
            title: "分数テスト".into(),
            subject: "算数".into(),
            topic: Some("分数".into()),
            questions: vec![sample_question()],
        };
        let prompt = lesson_plan_prompt(&s, Some(&q));
        assert!(prompt.contains("教科: 算数"));
        assert!(prompt.contains("教材: 分数テスト"));
        assert!(prompt.contains("単元: 分数"));
        assert!(prompt.contains("出題概念: 分数の足し算"));
    }

    #[test]
    fn lesson_plan_prompt_handles_no_misconceptions() {
        let mut s = sample_summary();
        s.misconceptions = vec![];
        let prompt = lesson_plan_prompt(&s, None);
        assert!(prompt.contains("主な誤概念: 回答なし"));
    }

    // --- Validation tests ---

    #[test]
    fn validate_analysis_output_accepts_valid_json() {
        let json = r#"{"misconception": "m0", "hint": "通分について考えましょう"}"#;
        let result = validate_analysis_output(json);
        assert!(result.is_ok());
        let output = result.unwrap();
        assert_eq!(output.misconception, "m0");
        assert_eq!(output.hint, "通分について考えましょう");
    }

    #[test]
    fn validate_analysis_output_rejects_invalid_json() {
        let json = "not json";
        let result = validate_analysis_output(json);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Invalid JSON"));
    }

    #[test]
    fn validate_analysis_output_rejects_empty_misconception() {
        let json = r#"{"misconception": "", "hint": "hint"}"#;
        let result = validate_analysis_output(json);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("misconception field is empty"));
    }

    #[test]
    fn validate_analysis_output_rejects_empty_hint() {
        let json = r#"{"misconception": "m0", "hint": ""}"#;
        let result = validate_analysis_output(json);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("hint field is empty"));
    }

    #[test]
    fn validate_analysis_output_rejects_long_hint() {
        let long_hint = "あ".repeat(81);
        let json = format!(r#"{{"misconception": "m0", "hint": "{}"}}"#, long_hint);
        let result = validate_analysis_output(&json);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Hint exceeds 80 characters"));
    }

    #[test]
    fn validate_analysis_output_accepts_80_char_hint() {
        let hint = "あ".repeat(80);
        let json = format!(r#"{{"misconception": "m0", "hint": "{}"}}"#, hint);
        let result = validate_analysis_output(&json);
        assert!(result.is_ok());
    }

    #[test]
    fn validate_lesson_plan_output_accepts_valid_json() {
        let json = r#"{
            "focus": "通分の必要性",
            "steps": ["步骤1", "步骤2", "步骤3", "步骤4"],
            "checkQuestion": "1/2 + 1/3 は？",
            "teacherNote": "通分の重要性を強調"
        }"#;
        let result = validate_lesson_plan_output(json);
        assert!(result.is_ok());
        let output = result.unwrap();
        assert_eq!(output.focus, "通分の必要性");
        assert_eq!(output.steps.len(), 4);
    }

    #[test]
    fn validate_lesson_plan_output_rejects_invalid_json() {
        let json = "not json";
        let result = validate_lesson_plan_output(json);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Invalid JSON"));
    }

    #[test]
    fn validate_lesson_plan_output_rejects_wrong_step_count() {
        let json = r#"{
            "focus": "通分",
            "steps": ["步骤1", "步骤2", "步骤3"],
            "checkQuestion": "問題",
            "teacherNote": "注意点"
        }"#;
        let result = validate_lesson_plan_output(json);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .contains("steps must have exactly 4 items"));
    }

    #[test]
    fn validate_lesson_plan_output_rejects_empty_focus() {
        let json = r#"{
            "focus": "",
            "steps": ["1", "2", "3", "4"],
            "checkQuestion": "問題",
            "teacherNote": "注意点"
        }"#;
        let result = validate_lesson_plan_output(json);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("focus field is empty"));
    }

    #[test]
    fn validate_lesson_plan_output_rejects_empty_step() {
        let json = r#"{
            "focus": "通分",
            "steps": ["1", "", "3", "4"],
            "checkQuestion": "問題",
            "teacherNote": "注意点"
        }"#;
        let result = validate_lesson_plan_output(json);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("steps[1] is empty"));
    }

    #[test]
    fn validate_lesson_plan_output_rejects_long_focus() {
        let long_focus = "あ".repeat(61);
        let json = format!(
            r#"{{"focus": "{}", "steps": ["1", "2", "3", "4"], "checkQuestion": "問題", "teacherNote": "注意点"}}"#,
            long_focus
        );
        let result = validate_lesson_plan_output(&json);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("focus exceeds 60 characters"));
    }

    #[test]
    fn validate_lesson_plan_output_rejects_long_step() {
        let long_step = "あ".repeat(101);
        let json = format!(
            r#"{{"focus": "通分", "steps": ["{}", "2", "3", "4"], "checkQuestion": "問題", "teacherNote": "注意点"}}"#,
            long_step
        );
        let result = validate_lesson_plan_output(&json);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .contains("steps[0] exceeds 100 characters"));
    }

    #[test]
    fn validate_lesson_plan_output_rejects_long_check_question() {
        let long_q = "あ".repeat(121);
        let json = format!(
            r#"{{"focus": "通分", "steps": ["1", "2", "3", "4"], "checkQuestion": "{}", "teacherNote": "注意点"}}"#,
            long_q
        );
        let result = validate_lesson_plan_output(&json);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .contains("checkQuestion exceeds 120 characters"));
    }

    #[test]
    fn validate_lesson_plan_output_rejects_long_teacher_note() {
        let long_note = "あ".repeat(121);
        let json = format!(
            r#"{{"focus": "通分", "steps": ["1", "2", "3", "4"], "checkQuestion": "問題", "teacherNote": "{}"}}"#,
            long_note
        );
        let result = validate_lesson_plan_output(&json);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .contains("teacherNote exceeds 120 characters"));
    }

    // --- Serialization tests ---

    #[test]
    fn analysis_output_serializes_to_camel_case() {
        let output = AnalysisOutput {
            misconception: "m0".into(),
            hint: "ヒント".into(),
        };
        let json = serde_json::to_string(&output).unwrap();
        assert!(json.contains("\"misconception\":\"m0\""));
        assert!(json.contains("\"hint\":\"ヒント\""));
    }

    #[test]
    fn lesson_plan_output_serializes_to_camel_case() {
        let output = LessonPlanOutput {
            focus: "通分".into(),
            steps: vec!["1".into(), "2".into(), "3".into(), "4".into()],
            check_question: "問題".into(),
            teacher_note: "注意点".into(),
        };
        let json = serde_json::to_string(&output).unwrap();
        assert!(json.contains("\"checkQuestion\":\"問題\""));
        assert!(json.contains("\"teacherNote\":\"注意点\""));
    }

    #[test]
    fn analysis_output_deserializes_from_camel_case() {
        let json = r#"{"misconception": "m0", "hint": "ヒント"}"#;
        let output: AnalysisOutput = serde_json::from_str(json).unwrap();
        assert_eq!(output.misconception, "m0");
        assert_eq!(output.hint, "ヒント");
    }

    #[test]
    fn lesson_plan_output_deserializes_from_camel_case() {
        let json = r#"{
            "focus": "通分",
            "steps": ["1", "2", "3", "4"],
            "checkQuestion": "問題",
            "teacherNote": "注意点"
        }"#;
        let output: LessonPlanOutput = serde_json::from_str(json).unwrap();
        assert_eq!(output.focus, "通分");
        assert_eq!(output.check_question, "問題");
        assert_eq!(output.teacher_note, "注意点");
    }
}
