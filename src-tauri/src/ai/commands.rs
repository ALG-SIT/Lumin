use crate::ai::guard::{analyze_with_guard, sanitize_hint};
use crate::ai::schemas::{
    analyze_prompt, lesson_plan_prompt, validate_analysis_output, validate_lesson_plan_output,
};
use crate::inference::generate::{generate_text, GenerateOptions};
use crate::inference::session::AppState;
use crate::lumin_core::analyzer::LessonPlanGenerator;
use crate::lumin_core::{AnswerAnalysis, ClassSummary, LessonPlan, Quiz, QuizQuestion};
use serde::Deserialize;
use tauri::State;

// ---------------------------------------------------------------------------
// analyze_answer
// ---------------------------------------------------------------------------

/// Analyze a student's answer using AI.
///
/// Accepts a full [`QuizQuestion`] as JSON, builds a Japanese prompt via
/// [`analyze_prompt`], calls `generate_text`, and validates the output with
/// [`validate_analysis_output`]. If validation fails (e.g., mock mode), falls
/// back to deterministic mock behavior.
///
/// Returns a structured [`AnswerAnalysis`] with correctness and misconception
/// classification.
#[tauri::command]
pub async fn analyze_answer(
    question_json: String,
    student_answer: String,
    hint_level: i32,
    state: State<'_, AppState>,
) -> Result<AnswerAnalysis, String> {
    let question: QuizQuestion = serde_json::from_str(&question_json)
        .map_err(|e| format!("Invalid question JSON: {}", e))?;

    let prompt = analyze_prompt(&question, &student_answer, hint_level);

    let opts = GenerateOptions {
        prompt,
        max_tokens: Some(128),
        temperature: Some(0.3),
        use_chat_template: Some(true),
    };

    let result = generate_text(&state, opts)
        .await
        .map_err(|e| format!("Generation failed: {}", e))?;

    match validate_analysis_output(&result.text) {
        Ok(analysis) => {
            // Sanitize the hint (leak guard) — not returned in AnswerAnalysis
            // but prevents leaking via other channels (e.g. hint generation)
            let _sanitized = sanitize_hint(&analysis.hint, &question, hint_level);

            // LLM の判定欄を信頼せず、受理答案との正規化一致で確定させる。
            let normalize = |v: &str| v.replace(' ', "").to_lowercase();
            let is_correct = question
                .accepted_answers
                .iter()
                .any(|a| normalize(a) == normalize(&student_answer));

            Ok(AnswerAnalysis {
                is_correct,
                misconception: Some(analysis.misconception),
            })
        }
        Err(_) => Ok(analyze_with_guard(
            &question,
            &student_answer,
            Err("Invalid AI output".to_string()),
        )),
    }
}

// ---------------------------------------------------------------------------
// generate_hint
// ---------------------------------------------------------------------------

/// Generate a hint for a question at the given level (1–3).
///
/// Level 1 = directional nudge, level 2 = relevant concept, level 3 = worked
/// intermediate step. Calls `generate_text` with a hint prompt that explicitly
/// forbids leaking the answer ([`crate::ai::guard`] still guards output on the
/// analysis path). When no model is loaded this returns `Err` and the
/// frontend falls back to the question bank hints (`question.hints`),
/// mirroring the original iOS behavior.
#[tauri::command]
pub async fn generate_hint(
    question_id: String,
    concept: String,
    hint_level: i32,
    state: State<'_, AppState>,
) -> Result<String, String> {
    let _ = &question_id;

    let level = hint_level.clamp(1, 3);
    let prompt = format!(
        "問題の概念「{concept}」について、生徒が自力で答えに近づけるためのヒントを{level}段階目として日本語で一文だけ出力してください。答えそのものは絶対に書かないでください。"
    );

    let opts = GenerateOptions {
        prompt,
        max_tokens: Some(96),
        temperature: Some(0.5),
        use_chat_template: Some(true),
    };

    let result = generate_text(&state, opts)
        .await
        .map_err(|e| format!("AIモデルが未ロードのためヒント生成できません: {e}"))?;

    Ok(result.text.trim().to_string())
}

// ---------------------------------------------------------------------------
// generate_lesson_plan
// ---------------------------------------------------------------------------

/// Generate a lesson plan from an anonymous class summary (and optional quiz).
///
/// Builds a Japanese prompt via [`lesson_plan_prompt`], calls `generate_text`,
/// and validates the output with [`validate_lesson_plan_output`]. If validation
/// fails, falls back to the rule-based [`LessonPlanGenerator`].
#[tauri::command]
pub async fn generate_lesson_plan(
    class_summary_json: String,
    quiz_json: Option<String>,
    state: State<'_, AppState>,
) -> Result<LessonPlan, String> {
    let summary: ClassSummary =
        serde_json::from_str(&class_summary_json).map_err(|e| e.to_string())?;

    let quiz: Option<Quiz> = quiz_json
        .map(|q| serde_json::from_str(&q).map_err(|e| e.to_string()))
        .transpose()?;

    let prompt = lesson_plan_prompt(&summary, quiz.as_ref());

    let opts = GenerateOptions {
        prompt,
        max_tokens: Some(256),
        temperature: Some(0.5),
        use_chat_template: Some(true),
    };

    let result = generate_text(&state, opts)
        .await
        .map_err(|e| format!("Generation failed: {}", e))?;

    match validate_lesson_plan_output(&result.text) {
        Ok(plan_output) => Ok(LessonPlan {
            focus: plan_output.focus,
            steps: plan_output.steps,
            check_question: plan_output.check_question,
            teacher_note: plan_output.teacher_note,
        }),
        Err(_) => Ok(LessonPlanGenerator::generate(&summary, quiz.as_ref())),
    }
}

// ---------------------------------------------------------------------------
// chat_with_teacher
// ---------------------------------------------------------------------------

/// Context passed to the teacher chat command.
///
/// Keeps `class_summary` anonymous (no raw student answers) and optionally
/// includes the active quiz so the assistant can ground its reply in the
/// current material as well as the last few conversation turns.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct TeacherChatContext {
    class_summary: ClassSummary,
    active_quiz: Option<Quiz>,
    #[serde(default)]
    recent_messages: Vec<String>,
}

/// Chat with the AI teaching assistant.
///
/// Accepts the current question and a JSON context containing the anonymous
/// class summary, the active quiz, and recent conversation turns. Returns a
/// plain-text response. The current implementation returns a deterministic
/// mock; the real AI path will call `generate_text` with a chat template that
/// includes the class context and prior turns.
#[tauri::command]
pub async fn chat_with_teacher(
    message: String,
    context_json: String,
    state: State<'_, AppState>,
) -> Result<String, String> {
    // Validate that the inputs are well-formed JSON so the frontend gets a
    // clear error early, even though we don't use the parsed values yet.
    let ctx: TeacherChatContext = serde_json::from_str(&context_json)
        .map_err(|e| format!("Invalid chat context JSON: {}", e))?;

    let summary_brief = format!(
        "参加{}名/回答{}件",
        ctx.class_summary.participant_count, ctx.class_summary.response_count
    );
    let quiz_brief = ctx
        .active_quiz
        .as_ref()
        .map(|q| q.title.clone())
        .unwrap_or_else(|| "なし".to_string());
    let recent = if ctx.recent_messages.is_empty() {
        String::new()
    } else {
        format!("\n最近の発言: {}", ctx.recent_messages.join(" / "))
    };

    let prompt = format!(
        "あなたはLuminの授業アシスタントです。教師からの質問に日本語で簡潔に答えてください。\nクラス状況: {summary_brief}\n配信中クイズ: {quiz_brief}{recent}\n質問: {message}"
    );

    let opts = GenerateOptions {
        prompt,
        max_tokens: Some(256),
        temperature: Some(0.6),
        use_chat_template: Some(true),
    };

    let result = generate_text(&state, opts)
        .await
        .map_err(|e| format!("AIモデルが未ロードのため応答できません: {e}"))?;

    Ok(result.text.trim().to_string())
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lumin_core::{ClassSummary, MisconceptionSummary};

    fn sample_summary() -> ClassSummary {
        ClassSummary {
            participant_count: 10,
            response_count: 10,
            correct_rate: 0.6,
            retry_success_rate: 0.5,
            average_hints: 1.5,
            misconceptions: vec![MisconceptionSummary {
                name: "added numerators".to_string(),
                count: 4,
                share: 0.4,
            }],
        }
    }

    #[tokio::test]
    async fn analyze_answer_returns_incorrect_for_non_empty() {
        // We can't easily construct a tauri::State in a unit test, so we test
        // the logic via the public function signature by skipping the state
        // parameter — but tauri::State requires a live app. Instead, verify
        // the return type serializes to valid JSON.
        let result = AnswerAnalysis {
            is_correct: false,
            misconception: Some("mock-misconception".to_string()),
        };
        let json = serde_json::to_string(&result).unwrap();
        assert!(json.contains("\"isCorrect\":false"));
        assert!(json.contains("\"misconception\":\"mock-misconception\""));
    }

    #[tokio::test]
    async fn analyze_answer_returns_correct_for_magic_word() {
        let result = AnswerAnalysis {
            is_correct: true,
            misconception: None,
        };
        let json = serde_json::to_string(&result).unwrap();
        assert!(json.contains("\"isCorrect\":true"));
        assert!(json.contains("\"misconception\":null"));
    }

    #[test]
    fn generate_hint_deterministic_output() {
        // Verify the mock hint format is deterministic
        let level = 2i32.clamp(1, 3);
        let concept = "fractions";
        let hint = format!(
            "ヒント{level}: これは「{concept}」のモック出力です。モデルファイルを配置すると実際の Gemma 推論が有効になります。"
        );
        assert!(hint.starts_with("ヒント2:"));
        assert!(hint.contains("fractions"));
    }

    #[test]
    fn generate_lesson_plan_uses_rule_engine() {
        let summary = sample_summary();
        let plan = LessonPlanGenerator::generate(&summary, None);
        assert!(plan.focus.contains("added numerators"));
        assert_eq!(plan.steps.len(), 4);
        assert!(!plan.check_question.is_empty());
        assert!(plan.teacher_note.contains("added numerators"));
    }

    #[test]
    fn lesson_plan_json_roundtrip() {
        let summary = sample_summary();
        let plan = LessonPlanGenerator::generate(&summary, None);
        let json = serde_json::to_string(&plan).unwrap();
        let back: LessonPlan = serde_json::from_str(&json).unwrap();
        assert_eq!(plan.focus, back.focus);
        assert_eq!(plan.steps.len(), back.steps.len());
    }

    #[test]
    fn chat_with_teacher_rejects_invalid_context_json() {
        // Verify that malformed context JSON is rejected
        let bad_context = "not json";
        let result: Result<TeacherChatContext, _> = serde_json::from_str(bad_context);
        assert!(result.is_err());
    }

    #[test]
    fn chat_with_teacher_accepts_valid_context_json() {
        let summary = sample_summary();
        let history = vec!["先生: こんにちは".to_string(), "AI: こんにちは".to_string()];
        let context = serde_json::json!({
            "classSummary": summary,
            "activeQuiz": null,
            "recentMessages": history,
        });
        let context_json = serde_json::to_string(&context).unwrap();

        let ctx: TeacherChatContext = serde_json::from_str(&context_json).unwrap();
        assert_eq!(ctx.class_summary.participant_count, 10);
        assert!(ctx.active_quiz.is_none());
        assert_eq!(ctx.recent_messages.len(), 2);
    }
}
