//! Opt-in real-model diagnostics. Fixtures are synthetic; no classroom data.
use super::*;
use crate::lumin_core::demo_data::demo_quiz_bank;
use crate::lumin_core::MisconceptionSummary;

#[tokio::test]
#[ignore = "requires installed models and LUMIN_AI_EVAL_OUTPUT"]
async fn classroom_model_evaluation() {
    let root = std::env::var("LUMIN_AI_SMOKE_MODEL_DIR").expect("model directory required");
    let output = std::env::var("LUMIN_AI_EVAL_OUTPUT").expect("report path required");
    let state = AppState::new(root.into());
    assert!(
        state.active_model_ready().await,
        "evaluation requires a real model"
    );
    let bank = demo_quiz_bank();
    let quiz = bank[0].clone();
    let q = quiz.questions[0].clone();
    let summary = ClassSummary {
        participant_count: 10,
        response_count: 10,
        correct_rate: 0.6,
        retry_success_rate: 0.5,
        average_hints: 1.5,
        misconceptions: vec![MisconceptionSummary {
            name: "傾きと切片の混同".into(),
            count: 4,
            share: 0.4,
        }],
    };
    let empty = ClassSummary {
        participant_count: 0,
        response_count: 0,
        correct_rate: 0.0,
        retry_success_rate: 0.0,
        average_hints: 0.0,
        misconceptions: vec![],
    };
    let ctx = TeacherChatContext {
        class_summary: summary.clone(),
        active_quiz: Some(quiz.clone()),
        recent_messages: vec![],
    };
    let empty_ctx = TeacherChatContext {
        class_summary: empty.clone(),
        active_quiz: Some(quiz.clone()),
        recent_messages: vec![],
    };
    let all_correct = ClassSummary {
        correct_rate: 1.0,
        misconceptions: vec![],
        ..summary.clone()
    };
    let mut cases = vec![
        (
            "teacher_facts".to_string(),
            teacher_chat_prompt(
                &ctx,
                "最初の問題文をそのまま引用し、初回正答率と最も多い誤概念を教えて。",
            ),
            512,
            None,
        ),
        (
            "teacher_empty".into(),
            teacher_chat_prompt(&empty_ctx, "何が分かっていない？"),
            512,
            None,
        ),
        (
            "analysis_math".into(),
            analyze_prompt(&q, "2", 1),
            128,
            Some((q.clone(), "2".to_string(), 1)),
        ),
        (
            "lesson".into(),
            lesson_plan_prompt(&summary, Some(&quiz)),
            512,
            None,
        ),
        (
            "lesson_empty".into(),
            lesson_plan_prompt(&empty, Some(&quiz)),
            512,
            None,
        ),
    ];
    cases.push((
        "lesson_all_correct".into(),
        lesson_plan_prompt(&all_correct, Some(&quiz)),
        512,
        None,
    ));
    for level in 1..=3 {
        cases.push((
            format!("hint_math_{level}"),
            hint_prompt(&q, "2", level, &[]),
            96,
            Some((q.clone(), "2".into(), level)),
        ));
    }
    let english = bank[4].questions[0].clone();
    let mut wrong: Vec<_> = english.misconception_answers.keys().cloned().collect();
    wrong.sort();
    let answer = wrong.first().cloned().unwrap_or("わからない".into());
    cases.push((
        "analysis_english".into(),
        analyze_prompt(&english, &answer, 1),
        128,
        Some((english.clone(), answer.clone(), 1)),
    ));
    cases.push((
        "hint_english".into(),
        hint_prompt(&english, &answer, 1, &[]),
        96,
        Some((english.clone(), answer, 1)),
    ));
    let selected = std::env::var("LUMIN_AI_EVAL_CASES").unwrap_or_default();
    let mut records = vec![];
    let mut shown_math_hints = Vec::new();
    for (name, mut prompt, limit, question) in cases {
        if !selected.is_empty() && !selected.split(',').any(|id| id == name) {
            continue;
        }
        if name.starts_with("hint_math_") {
            let (q, answer, level) = question.as_ref().unwrap();
            prompt = hint_prompt(q, answer, *level, &shown_math_hints);
        }
        println!("START {name}");
        let attempts = if name.starts_with("lesson") {
            generate_lesson_attempts(&state, &prompt)
                .await
                .expect("lesson generation")
        } else {
            vec![generate_text(
                &state,
                GenerateOptions {
                    prompt: prompt.clone(),
                    max_tokens: Some(limit),
                    temperature: None,
                    use_chat_template: Some(true),
                },
            )
            .await
            .expect("real generation")]
        };
        let result = attempts.last().unwrap().clone();
        assert!(!result.is_mock);
        let raw = result.text.clone();
        let mut record = serde_json::json!({ "case": name, "prompt": prompt, "maxTokens": limit, "generation": result, "attempts": attempts });
        if name.starts_with("analysis") {
            let (q, answer, _) = question.as_ref().unwrap();
            let parsed = validate_analysis_output(&raw);
            record["schemaValid"] = parsed.is_ok().into();
            let allowed = parsed
                .as_ref()
                .ok()
                .is_some_and(|p| resolve_misconception(q, &p.misconception).is_some());
            record["candidateValid"] = allowed.into();
            record["classificationMatchesFixture"] = parsed
                .as_ref()
                .ok()
                .is_some_and(|p| {
                    resolve_misconception(q, &p.misconception)
                        == RuleBasedLearningEngine.analyze(answer, q).misconception
                })
                .into();
            record["delivered"] =
                serde_json::to_value(resolve_analysis(q, answer, real_output(result))).unwrap();
        } else if name.starts_with("hint") {
            let (q, _, level) = question.as_ref().unwrap();
            let delivered = sanitize_hint_with_history(
                &real_output(result).unwrap_or_default(),
                q,
                *level,
                if name.starts_with("hint_math_") {
                    &shown_math_hints
                } else {
                    &[]
                },
            );
            record["guardChanged"] = (delivered != raw.trim()).into();
            if name.starts_with("hint_math_") {
                shown_math_hints.push(delivered.clone());
            }
            record["delivered"] = delivered.into();
        } else if name.starts_with("lesson") {
            let parsed = real_output(result).and_then(|text| validate_lesson_plan_output(&text));
            record["schemaValid"] = parsed.is_ok().into();
            record["validationError"] = parsed.as_ref().err().cloned().into();
            record["delivered"] = match parsed {
                Ok(plan) => serde_json::to_value(finalize_lesson_plan(
                    plan,
                    if name == "lesson_empty" {
                        &empty
                    } else if name == "lesson_all_correct" {
                        &all_correct
                    } else {
                        &summary
                    },
                ))
                .unwrap(),
                Err(_) => serde_json::to_value(LessonPlanGenerator::generate(
                    if name == "lesson_empty" {
                        &empty
                    } else if name == "lesson_all_correct" {
                        &all_correct
                    } else {
                        &summary
                    },
                    Some(&quiz),
                ))
                .unwrap(),
            };
        }
        println!(
            "DONE {name}: {} tokens, {} ms",
            record["generation"]["generated_tokens"], record["generation"]["latency_ms"]
        );
        records.push(record);
        std::fs::write(&output, serde_json::to_string_pretty(&records).unwrap()).unwrap();
    }
    assert!(!records.is_empty(), "case filter matched no cases");
    if std::env::var_os("LUMIN_ORT_PROFILE_DIR").is_some() {
        if let Some(session) = state.session.lock().await.as_mut() {
            println!(
                "Decoder profile: {}",
                session.session.end_profiling().unwrap()
            );
            if let Some(embed) = session.embed_session.as_mut() {
                println!("Embed profile: {}", embed.end_profiling().unwrap());
            }
        }
    }
}
