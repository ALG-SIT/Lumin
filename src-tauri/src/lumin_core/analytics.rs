use crate::lumin_core::models::{
    AnalysisEvent, ClassSummary, LessonPlan, MisconceptionSummary, Quiz,
};
use std::collections::HashMap;

// ---------------------------------------------------------------------------
// ClassAnalytics
// ---------------------------------------------------------------------------

pub struct ClassAnalytics;

impl ClassAnalytics {
    /// Summarize analysis events into class-level statistics.
    /// Matches Swift `ClassAnalytics.summarize` behavior.
    pub fn summarize(events: &[AnalysisEvent], _quiz: &Quiz) -> ClassSummary {
        if events.is_empty() {
            return ClassSummary {
                participant_count: 0,
                response_count: 0,
                correct_rate: 0.0,
                retry_success_rate: 0.0,
                average_hints: 0.0,
                misconceptions: Vec::new(),
            };
        }

        let mut participants = std::collections::HashSet::new();
        for e in events {
            participants.insert(&e.participant_token);
        }
        let participant_count = participants.len() as i32;

        let correct = events.iter().filter(|e| e.correct).count();

        // retry candidates: incorrect AND used hints
        let retries: Vec<&AnalysisEvent> = events
            .iter()
            .filter(|e| !e.correct && e.hint_count > 0)
            .collect();
        let retry_successes = retries.iter().filter(|e| e.retry_success).count();

        // Group misconceptions
        let mut grouped: HashMap<&str, Vec<&AnalysisEvent>> = HashMap::new();
        for e in events {
            if let Some(ref m) = e.misconception {
                grouped.entry(m.as_str()).or_default().push(e);
            }
        }
        let mut misconceptions: Vec<MisconceptionSummary> = grouped
            .into_iter()
            .map(|(name, items)| MisconceptionSummary {
                name: name.to_string(),
                count: items.len() as i32,
                share: items.len() as f64 / events.len() as f64,
            })
            .collect();
        misconceptions.sort_by_key(|m| -m.count);

        let total_hints: i32 = events.iter().map(|e| e.hint_count).sum();
        let total = events.len() as f64;

        ClassSummary {
            participant_count,
            response_count: events.len() as i32,
            correct_rate: correct as f64 / total,
            retry_success_rate: if retries.is_empty() {
                0.0
            } else {
                retry_successes as f64 / retries.len() as f64
            },
            average_hints: total_hints as f64 / total,
            misconceptions,
        }
    }
}

// ---------------------------------------------------------------------------
// LessonPlanGenerator
// ---------------------------------------------------------------------------

pub struct LessonPlanGenerator;

impl LessonPlanGenerator {
    /// Generate a lesson plan from class summary.
    /// Fixed 4-step, 10-minute plan matching Swift behavior.
    pub fn generate(summary: &ClassSummary, quiz: &Quiz) -> LessonPlan {
        let default_focus = quiz
            .topic
            .as_deref()
            .or(Some(&quiz.title))
            .map(|t| format!("{t}の主要概念"))
            .unwrap_or_else(|| "この単元の主要概念".to_string());

        let top = summary
            .misconceptions
            .first()
            .map(|m| m.name.as_str())
            .unwrap_or(&default_focus);

        let percentage = ((summary
            .misconceptions
            .first()
            .map(|m| m.share)
            .unwrap_or(0.0))
            * 100.0) as i32;

        let check_question = quiz
            .questions
            .first()
            .map(|q| format!("「{}」について、自分の言葉で説明してください。", q.prompt))
            .unwrap_or_else(|| {
                "今日の要点を一文で説明し、その考え方を使う例を一つ示してください。".to_string()
            });

        let teacher_note = if percentage > 0 {
            format!(
                "全回答の約{percentage}%で「{top}」が見られました。正解を先に示さず、判断の根拠を生徒自身の言葉にさせてください。"
            )
        } else {
            "回答が集まったら、上位の誤概念に合わせて内容を再生成してください。".to_string()
        };

        LessonPlan {
            focus: format!("「{top}」を解きほぐす"),
            steps: vec![
                "0〜2分：正答例と代表的な誤答を見比べ、違いを見つける".to_string(),
                "2〜5分：判断の根判断の根拠をキーワードを使ってペアで説明する".to_string(),
                "5〜8分：誤答が多かった考え方を使う類題に再挑戦する".to_string(),
                "8〜10分：答えと理由を共有し、出口問題で理解を確認する".to_string(),
            ],
            check_question,
            teacher_note,
        }
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;
    use uuid::Uuid;

    // -- Test helpers --

    fn event(
        participant: &str,
        question_id: &str,
        concept: &str,
        misconception: Option<&str>,
        correct: bool,
        hint_count: i32,
        retry_success: bool,
    ) -> AnalysisEvent {
        AnalysisEvent {
            id: Uuid::new_v4(),
            participant_token: participant.into(),
            session_id: None,
            question_id: question_id.into(),
            concept: concept.into(),
            misconception: misconception.map(|s| s.into()),
            correct,
            hint_count,
            retry_success,
            submitted_at: Utc::now(),
        }
    }

    fn sample_quiz() -> Quiz {
        let mut misconception_answers = std::collections::HashMap::new();
        misconception_answers.insert("2".into(), "傾きと切片の混同".into());

        Quiz {
            id: "quiz-sample".into(),
            title: "一次関数 ミニチェック".into(),
            subject: "中学数学".into(),
            topic: Some("一次関数".into()),
            questions: vec![crate::lumin_core::models::QuizQuestion {
                id: "math-01".into(),
                prompt: "y = 3x + 2 の傾きは？".into(),
                concept: "一次関数の傾き".into(),
                accepted_answers: vec!["3".into(), "+3".into()],
                misconception_answers,
                generic_misconception: "傾きの読み取り不足".into(),
                hints: vec![
                    "x が1増えたとき、y がいくつ増えるかに注目しよう。".into(),
                    "y = ax + b の a が変化の割合を表します。".into(),
                    "この式を y = a×x + b と見比べ、x の直前の数を探そう。".into(),
                ],
                explanation: "y = ax + b では、x の係数 a が傾きです。".into(),
            }],
        }
    }

    /// 25 demo events matching the Swift sample pattern for test_summarize_25_sample_events.
    fn demo_events_25() -> Vec<AnalysisEvent> {
        vec![
            // P-1 (3 events)
            event(
                "P-1",
                "math-01",
                "一次関数の傾き",
                Some("傾きと切片の混同"),
                false,
                2,
                true,
            ),
            event("P-1", "math-02", "一次関数の切片", None, true, 0, false),
            event(
                "P-1",
                "math-03",
                "変化の割合",
                Some("変化量の分子と分母の逆転"),
                false,
                1,
                true,
            ),
            // P-2 (3 events)
            event("P-2", "math-01", "一次関数の傾き", None, true, 0, false),
            event(
                "P-2",
                "math-04",
                "式への代入",
                Some("切片の計算漏れ"),
                false,
                3,
                true,
            ),
            event(
                "P-2",
                "math-05",
                "一次関数の式",
                Some("傾きと切片の混同"),
                false,
                2,
                false,
            ),
            // P-3 (3 events)
            event(
                "P-3",
                "math-01",
                "一次関数の傾き",
                Some("傾きと切片の混同"),
                false,
                1,
                true,
            ),
            event("P-3", "math-02", "一次関数の切片", None, true, 0, false),
            event("P-3", "math-03", "変化の割合", None, true, 0, false),
            // P-4 (2 events)
            event(
                "P-4",
                "math-04",
                "式への代入",
                Some("代入計算の誤り"),
                false,
                2,
                true,
            ),
            event("P-4", "math-05", "一次関数の式", None, true, 0, false),
            // P-5 (2 events)
            event("P-5", "math-01", "一次関数の傾き", None, true, 0, false),
            event(
                "P-5",
                "math-02",
                "一次関数の切片",
                Some("傾きと切片の混同"),
                false,
                3,
                false,
            ),
            // P-6 (3 events)
            event(
                "P-6",
                "math-03",
                "変化の割合",
                Some("変化量の分子と分母の逆転"),
                false,
                1,
                true,
            ),
            event("P-6", "math-04", "式への代入", None, true, 0, false),
            event("P-6", "math-05", "一次関数の式", None, true, 0, false),
            // P-7 (2 events)
            event(
                "P-7",
                "math-01",
                "一次関数の傾き",
                Some("傾きと切片の混同"),
                false,
                2,
                true,
            ),
            event("P-7", "math-02", "一次関数の切片", None, true, 0, false),
            // P-8 (2 events)
            event("P-8", "math-03", "変化の割合", None, true, 0, false),
            event(
                "P-8",
                "math-05",
                "一次関数の式",
                Some("傾きの符号の読み落とし"),
                false,
                4,
                true,
            ),
            // P-9 (2 events)
            event(
                "P-9",
                "math-01",
                "一次関数の傾き",
                Some("傾きと切片の混同"),
                false,
                1,
                false,
            ),
            event("P-9", "math-04", "式への代入", None, true, 0, false),
            // P-10 (3 events)
            event(
                "P-10",
                "math-02",
                "一次関数の切片",
                Some("傾きと切片の混同"),
                false,
                3,
                true,
            ),
            event("P-10", "math-03", "変化の割合", None, true, 0, false),
            event(
                "P-10",
                "math-05",
                "一次関数の式",
                Some("傾きの符号の読み落とし"),
                false,
                2,
                true,
            ),
        ]
    }

    // -- Test 1: empty events --

    #[test]
    fn test_summarize_empty_events() {
        let quiz = sample_quiz();
        let summary = ClassAnalytics::summarize(&[], &quiz);
        assert_eq!(summary.participant_count, 0);
        assert_eq!(summary.response_count, 0);
        assert!((summary.correct_rate - 0.0).abs() < f64::EPSILON);
        assert!((summary.retry_success_rate - 0.0).abs() < f64::EPSILON);
        assert!((summary.average_hints - 0.0).abs() < f64::EPSILON);
        assert!(summary.misconceptions.is_empty());
    }

    // -- Test 2: 25 sample events --

    #[test]
    fn test_summarize_25_sample_events() {
        let events = demo_events_25();
        assert_eq!(events.len(), 25);

        let quiz = sample_quiz();
        let summary = ClassAnalytics::summarize(&events, &quiz);

        assert_eq!(summary.participant_count, 10);
        assert_eq!(summary.response_count, 25);

        // 12 correct out of 25
        let expected_correct_rate = 12.0 / 25.0;
        assert!((summary.correct_rate - expected_correct_rate).abs() < 0.001);

        // 13 incorrect with hints > 0 → retry candidates
        // 10 of those have retry_success = true
        let expected_retry_rate = 10.0 / 13.0;
        assert!((summary.retry_success_rate - expected_retry_rate).abs() < 0.001);

        // Total hints: 2+0+1+0+3+2+1+0+0+2+0+0+3+1+0+0+2+0+0+4+1+0+3+0+2 = 27
        let expected_avg_hints = 27.0 / 25.0;
        assert!((summary.average_hints - expected_avg_hints).abs() < 0.001);

        // Top misconception: "傾きと切片の混同" with count 7
        assert_eq!(summary.misconceptions[0].name, "傾きと切片の混同");
        assert_eq!(summary.misconceptions[0].count, 7);
    }

    // -- Test 3: misconception grouping --

    #[test]
    fn test_misconception_grouping() {
        let events = vec![
            event("A", "q1", "c1", Some("M1"), false, 1, false),
            event("B", "q1", "c1", Some("M1"), false, 1, false),
            event("C", "q1", "c1", Some("M2"), false, 1, false),
            event("D", "q1", "c1", Some("M3"), false, 1, false),
            event("E", "q1", "c1", Some("M3"), false, 1, false),
            event("F", "q1", "c1", Some("M3"), false, 1, false),
            event("G", "q1", "c1", None, true, 0, false),
        ];
        let quiz = sample_quiz();
        let summary = ClassAnalytics::summarize(&events, &quiz);

        assert_eq!(summary.misconceptions.len(), 3);
        // Sorted by count descending: M3(3), M1(2), M2(1)
        assert_eq!(summary.misconceptions[0].name, "M3");
        assert_eq!(summary.misconceptions[0].count, 3);
        assert_eq!(summary.misconceptions[1].name, "M1");
        assert_eq!(summary.misconceptions[1].count, 2);
        assert_eq!(summary.misconceptions[2].name, "M2");
        assert_eq!(summary.misconceptions[2].count, 1);

        // Check shares
        assert!((summary.misconceptions[0].share - 3.0 / 7.0).abs() < 0.001);
        assert!((summary.misconceptions[1].share - 2.0 / 7.0).abs() < 0.001);
        assert!((summary.misconceptions[2].share - 1.0 / 7.0).abs() < 0.001);
    }

    // -- Test 4: retry success rate --

    #[test]
    fn test_retry_success_rate() {
        let events = vec![
            // Incorrect + hints + retry success → counted as retry success
            event("A", "q1", "c1", Some("M1"), false, 2, true),
            // Incorrect + hints + no retry success → counted as retry candidate, not success
            event("B", "q1", "c1", Some("M1"), false, 1, false),
            // Correct → not a retry candidate
            event("C", "q1", "c1", None, true, 0, false),
            // Incorrect + no hints → not a retry candidate
            event("D", "q1", "c1", Some("M1"), false, 0, false),
        ];
        let quiz = sample_quiz();
        let summary = ClassAnalytics::summarize(&events, &quiz);

        // 2 retry candidates (A and B), 1 success (A)
        assert!((summary.retry_success_rate - 0.5).abs() < 0.001);
    }

    // -- Test 5: lesson plan four steps --

    #[test]
    fn test_lesson_plan_four_steps() {
        let events = demo_events_25();
        let quiz = sample_quiz();
        let summary = ClassAnalytics::summarize(&events, &quiz);
        let plan = LessonPlanGenerator::generate(&summary, &quiz);

        assert_eq!(plan.steps.len(), 4);
        assert!(plan.focus.contains("傾きと切片の混同"));
    }

    // -- Test 6: lesson plan check question from quiz --

    #[test]
    fn test_lesson_plan_check_question_from_quiz() {
        let events = demo_events_25();
        let quiz = sample_quiz();
        let summary = ClassAnalytics::summarize(&events, &quiz);
        let plan = LessonPlanGenerator::generate(&summary, &quiz);

        assert!(plan.check_question.contains("y = 3x + 2 の傾きは？"));
        assert!(plan.check_question.contains("？"));
    }

    // -- Test 7: average hints calculation --

    #[test]
    fn test_average_hints_calculation() {
        // 10 events with total hints = 25 → average = 2.5
        let events = vec![
            event("P1", "q1", "c1", None, true, 1, false),
            event("P2", "q1", "c1", None, true, 2, false),
            event("P3", "q1", "c1", None, true, 3, false),
            event("P4", "q1", "c1", None, true, 4, false),
            event("P5", "q1", "c1", None, true, 0, false),
            event("P6", "q1", "c1", None, true, 5, false),
            event("P7", "q1", "c1", None, true, 2, false),
            event("P8", "q1", "c1", None, true, 3, false),
            event("P9", "q1", "c1", None, true, 3, false),
            event("P10", "q1", "c1", None, true, 2, false),
        ];
        // Total hints: 1+2+3+4+0+5+2+3+3+2 = 25
        assert_eq!(events.len(), 10);
        let quiz = sample_quiz();
        let summary = ClassAnalytics::summarize(&events, &quiz);

        assert!((summary.average_hints - 2.5).abs() < 0.001);
    }
}
