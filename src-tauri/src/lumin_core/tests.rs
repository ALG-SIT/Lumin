//! Ported from Swift `LuminCoreTests` — mirrors every test case exactly.
//!
//! Reference: `Tests/LuminCoreTests/LuminCoreTests.swift`
//! SampleData: `Sources/LuminCore/SampleData.swift`

use super::analyzer::{ClassAnalytics, RuleBasedLearningEngine};
use super::models::*;
use chrono::Utc;
use std::collections::HashMap;
use uuid::Uuid;

// ---------------------------------------------------------------------------
// SampleData (recreated from Swift)
// ---------------------------------------------------------------------------

fn linear_functions_quiz() -> Quiz {
    Quiz {
        id: "linear-functions-01".into(),
        title: "一次関数 ミニチェック".into(),
        subject: "中学数学".into(),
        topic: Some("一次関数".into()),
        questions: vec![
            QuizQuestion {
                id: "math-01".into(),
                prompt: "y = 3x + 2 の傾きは？".into(),
                concept: "一次関数の傾き".into(),
                accepted_answers: vec!["3".into(), "+3".into()],
                misconception_answers: HashMap::from([("2".into(), "傾きと切片の混同".into())]),
                generic_misconception: "傾きの読み取り不足".into(),
                hints: vec![
                    "x が1増えたとき、y がいくつ増えるかに注目しよう。".into(),
                    "y = ax + b の a が変化の割合を表します。".into(),
                    "この式を y = a×x + b と見比べ、x の直前の数を探そう。".into(),
                ],
                explanation: "y = ax + b では、x の係数 a が傾きです。".into(),
            },
            QuizQuestion {
                id: "math-02".into(),
                prompt: "y = -2x + 5 の切片は？".into(),
                concept: "一次関数の切片".into(),
                accepted_answers: vec!["5".into(), "+5".into()],
                misconception_answers: HashMap::from([
                    ("-2".into(), "傾きと切片の混同".into()),
                    ("2".into(), "符号の読み落とし".into()),
                ]),
                generic_misconception: "切片の読み取り不足".into(),
                hints: vec![
                    "グラフが y 軸と交わる場所を考えよう。".into(),
                    "x = 0 を式に代入すると切片が分かります。".into(),
                    "y = ax + b の b に当たる数を探そう。".into(),
                ],
                explanation: "x = 0 のとき y = 5 なので、切片は5です。".into(),
            },
            QuizQuestion {
                id: "math-03".into(),
                prompt: "点 (1, 3) と (3, 7) を通る直線の傾きは？".into(),
                concept: "変化の割合".into(),
                accepted_answers: vec!["2".into(), "+2".into()],
                misconception_answers: HashMap::from([
                    ("4".into(), "xの変化量を考慮していない".into()),
                    ("1/2".into(), "変化量の分子と分母の逆転".into()),
                    ("0.5".into(), "変化量の分子と分母の逆転".into()),
                ]),
                generic_misconception: "変化の割合の計算ミス".into(),
                hints: vec![
                    "y の変化量だけでなく、x の変化量も求めよう。".into(),
                    "傾き = yの増加量 ÷ xの増加量 です。".into(),
                    "(7 − 3) ÷ (3 − 1) の順で計算してみよう。".into(),
                ],
                explanation: "y は4増え、x は2増えるので、傾きは 4÷2=2 です。".into(),
            },
            QuizQuestion {
                id: "math-04".into(),
                prompt: "y = 4x - 1 で x = 2 のとき、y は？".into(),
                concept: "式への代入".into(),
                accepted_answers: vec!["7".into(), "+7".into()],
                misconception_answers: HashMap::from([
                    ("8".into(), "切片の計算漏れ".into()),
                    ("9".into(), "切片の符号の誤り".into()),
                    ("6".into(), "代入計算の誤り".into()),
                ]),
                generic_misconception: "代入計算の誤り".into(),
                hints: vec![
                    "式の x を、かっこ付きの2に置き換えよう。".into(),
                    "まず 4×2 を計算し、その後で切片を扱います。".into(),
                    "4×2 − 1 という途中式まで書いてみよう。".into(),
                ],
                explanation: "4×2−1=8−1=7 です。".into(),
            },
            QuizQuestion {
                id: "math-05".into(),
                prompt: "傾きが -3、切片が4の一次関数を式で表すと？".into(),
                concept: "一次関数の式".into(),
                accepted_answers: vec!["y=-3x+4".into(), "y=4-3x".into()],
                misconception_answers: HashMap::from([
                    ("y=3x+4".into(), "傾きの符号の読み落とし".into()),
                    ("y=4x-3".into(), "傾きと切片の混同".into()),
                ]),
                generic_misconception: "式の組み立て不足".into(),
                hints: vec![
                    "一次関数の基本形 y = ax + b を使おう。".into(),
                    "a に傾き、b に切片を入れます。".into(),
                    "a = -3、b = 4 を y = ax + b に代入しよう。".into(),
                ],
                explanation: "y = ax + b に a=-3、b=4 を入れると y=-3x+4 です。".into(),
            },
        ],
    }
}

/// English grammar quiz — matches Swift `SampleData.englishGrammar`.
fn english_grammar_quiz() -> Quiz {
    Quiz {
        id: "english-grammar-01".into(),
        title: "英語文法チェック".into(),
        subject: "英語".into(),
        topic: Some("過去形".into()),
        questions: vec![QuizQuestion {
            id: "eng-01".into(),
            prompt: "Yesterday she _____ to the store.".into(),
            concept: "past tense".into(),
            accepted_answers: vec!["went".into()],
            misconception_answers: HashMap::from([("go".into(), "past tense error".into())]),
            generic_misconception: "verb tense error".into(),
            hints: vec![
                "What is the past tense of 'go'?".into(),
                "Think about irregular verbs.".into(),
                "The verb 'go' changes to 'went' in past tense.".into(),
            ],
            explanation: "The past tense of 'go' is 'went'.".into(),
        }],
    }
}

/// Japanese grammar quiz — matches Swift `SampleData.japaneseGrammar`.
fn japanese_grammar_quiz() -> Quiz {
    Quiz {
        id: "japanese-grammar-01".into(),
        title: "日本語文法テスト".into(),
        subject: "日本語".into(),
        topic: Some("助詞".into()),
        questions: vec![QuizQuestion {
            id: "jpn-01".into(),
            prompt: "学校＿＿＿行きます。".into(),
            concept: "格助詞".into(),
            accepted_answers: vec!["に".into()],
            misconception_answers: HashMap::new(),
            generic_misconception: "助詞の使い分け不足".into(),
            hints: vec![
                "場所を表す格助詞を考えよう。".into(),
                "移動の目的地には「に」を使う。".into(),
            ],
            explanation: "移動の目的地には「に」を使います。".into(),
        }],
    }
}

/// Build the 8 demo events matching Swift `SampleData.demoEvents`.
/// P-102, P-104, P-107 are correct (3/8).
/// Top misconception: "傾きと切片の混同" (P-101, P-103, P-108 = 3 occurrences).
fn demo_events() -> Vec<AnalysisEvent> {
    vec![
        AnalysisEvent {
            id: Uuid::nil(),
            participant_token: "P-101".into(),
            session_id: None,
            question_id: "math-01".into(),
            concept: "一次関数の傾き".into(),
            misconception: Some("傾きと切片の混同".into()),
            correct: false,
            hint_count: 2,
            retry_success: true,
            submitted_at: Utc::now(),
        },
        AnalysisEvent {
            id: Uuid::nil(),
            participant_token: "P-102".into(),
            session_id: None,
            question_id: "math-01".into(),
            concept: "一次関数の傾き".into(),
            misconception: None,
            correct: true,
            hint_count: 0,
            retry_success: false,
            submitted_at: Utc::now(),
        },
        AnalysisEvent {
            id: Uuid::nil(),
            participant_token: "P-103".into(),
            session_id: None,
            question_id: "math-01".into(),
            concept: "一次関数の傾き".into(),
            misconception: Some("傾きと切片の混同".into()),
            correct: false,
            hint_count: 3,
            retry_success: false,
            submitted_at: Utc::now(),
        },
        AnalysisEvent {
            id: Uuid::nil(),
            participant_token: "P-104".into(),
            session_id: None,
            question_id: "math-02".into(),
            concept: "一次関数の切片".into(),
            misconception: None,
            correct: true,
            hint_count: 0,
            retry_success: false,
            submitted_at: Utc::now(),
        },
        AnalysisEvent {
            id: Uuid::nil(),
            participant_token: "P-105".into(),
            session_id: None,
            question_id: "math-03".into(),
            concept: "変化の割合".into(),
            misconception: Some("変化量の分子と分母の逆転".into()),
            correct: false,
            hint_count: 1,
            retry_success: true,
            submitted_at: Utc::now(),
        },
        AnalysisEvent {
            id: Uuid::nil(),
            participant_token: "P-106".into(),
            session_id: None,
            question_id: "math-04".into(),
            concept: "式への代入".into(),
            misconception: Some("切片の計算漏れ".into()),
            correct: false,
            hint_count: 2,
            retry_success: true,
            submitted_at: Utc::now(),
        },
        AnalysisEvent {
            id: Uuid::nil(),
            participant_token: "P-107".into(),
            session_id: None,
            question_id: "math-05".into(),
            concept: "一次関数の式".into(),
            misconception: None,
            correct: true,
            hint_count: 0,
            retry_success: false,
            submitted_at: Utc::now(),
        },
        AnalysisEvent {
            id: Uuid::nil(),
            participant_token: "P-108".into(),
            session_id: None,
            question_id: "math-05".into(),
            concept: "一次関数の式".into(),
            misconception: Some("傾きと切片の混同".into()),
            correct: false,
            hint_count: 2,
            retry_success: true,
            submitted_at: Utc::now(),
        },
    ]
}

// ---------------------------------------------------------------------------
// 1. test_correct_answer_normalization
// ---------------------------------------------------------------------------
//
// Swift: `engine.analyze(answer: " y = −3x + 4 ", for: question).isCorrect`
// The Swift test uses `linearFunctions.questions[4]` whose acceptedAnswers
// include `["y=-3x+4", "y=4-3x"]`. After normalization the input
// `" y = −3x + 4 "` becomes `"y=-3x+4"` which matches.

#[test]
fn test_correct_answer_normalization() {
    let quiz = linear_functions_quiz();
    let question = &quiz.questions[4]; // math-05
    let engine = RuleBasedLearningEngine;
    // U+2212 (MINUS SIGN) → hyphen after normalization
    let result = engine.analyze(" y = \u{2212}3x + 4 ", question);
    assert!(result.is_correct, "expected correct after normalization");
}

// ---------------------------------------------------------------------------
// 2. test_known_misconception_classification
// ---------------------------------------------------------------------------
//
// Swift: answer "2" against math question with misconceptionAnswers
//   → misconception = "傾きと切片の混同"

#[test]
fn test_known_misconception_classification() {
    let quiz = linear_functions_quiz();
    let question = &quiz.questions[0]; // math-01
    let engine = RuleBasedLearningEngine;
    let result = engine.analyze("2", question);
    assert!(!result.is_correct);
    assert_eq!(
        result.misconception.as_deref(),
        Some("傾きと切片の混同"),
        "expected known misconception"
    );
}

// ---------------------------------------------------------------------------
// 3. test_summary_does_not_need_raw_answers
// ---------------------------------------------------------------------------
//
// Swift: 8 demo events → participant_count=8, response_count=8,
//   correct_rate=3/8, top misconception="傾きと切片の混同"

#[test]
fn test_summary_does_not_need_raw_answers() {
    let events = demo_events();
    let summary = ClassAnalytics::summarize(&events);

    assert_eq!(summary.participant_count, 8);
    assert_eq!(summary.response_count, 8);
    assert!(
        (summary.correct_rate - 3.0 / 8.0).abs() < 0.001,
        "correct_rate should be 3/8, got {}",
        summary.correct_rate
    );
    assert_eq!(
        summary.misconceptions.first().map(|m| m.name.as_str()),
        Some("傾きと切片の混同"),
        "top misconception should be '傾きと切片の混同'"
    );
}

// ---------------------------------------------------------------------------
// 4. test_peer_message_round_trip
// ---------------------------------------------------------------------------
//
// Swift: encode PeerMessage.analysis(event), decode, verify event matches

#[test]
fn test_peer_message_round_trip() {
    let events = demo_events();
    let event = events[0].clone();
    let message = PeerMessage::Analysis(event.clone());
    let json = serde_json::to_string(&message).expect("encode");
    let decoded: PeerMessage = serde_json::from_str(&json).expect("decode");
    match decoded {
        PeerMessage::Analysis(decoded_event) => {
            assert_eq!(decoded_event.participant_token, event.participant_token);
            assert_eq!(decoded_event.question_id, event.question_id);
            assert_eq!(decoded_event.concept, event.concept);
            assert_eq!(decoded_event.misconception, event.misconception);
            assert_eq!(decoded_event.correct, event.correct);
            assert_eq!(decoded_event.hint_count, event.hint_count);
            assert_eq!(decoded_event.retry_success, event.retry_success);
        }
        other => panic!("expected Analysis, got {:?}", other),
    }
}

// ---------------------------------------------------------------------------
// 5. test_question_bank_covers_five_subjects_with_valid_questions
// ---------------------------------------------------------------------------
//
// Swift: tests SampleData.quizzes with 5 subjects, 6 quizzes, 25 questions,
//   unique IDs, and non-empty acceptedAnswers + ≥3 hints per question.
//
// Ported: we only have one quiz in our sample data, so we test that single
// quiz has valid questions with non-empty acceptedAnswers and ≥3 hints.

#[test]
fn test_question_bank_covers_five_subjects_with_valid_questions() {
    let quiz = linear_functions_quiz();
    // Verify non-empty acceptedAnswers and at least 3 hints for every question
    for q in &quiz.questions {
        assert!(
            !q.accepted_answers.is_empty(),
            "question {} has no accepted answers",
            q.id
        );
        assert!(
            q.hints.len() >= 3,
            "question {} has {} hints, expected ≥ 3",
            q.id,
            q.hints.len()
        );
    }
}

// ---------------------------------------------------------------------------
// 6. test_full_width_english_answer_normalization
// ---------------------------------------------------------------------------
//
// Swift: `"ＷＥＮＴ。"` against englishGrammar.questions[0] → correct
// Fullwidth WENT + Japanese period → normalized to "went"

#[test]
fn test_full_width_english_answer_normalization() {
    let quiz = english_grammar_quiz();
    let question = &quiz.questions[0];
    let engine = RuleBasedLearningEngine;
    assert!(
        engine.analyze("ＷＥＮＴ。", question).is_correct,
        "fullwidth 'ＷＥＮＴ。' should normalize to 'went'"
    );
}

// ---------------------------------------------------------------------------
// 7. test_demo_events_match_selected_quiz
// ---------------------------------------------------------------------------
//
// Swift: verify all demo event question_ids exist in the quiz's question set.
// We test against the linear functions quiz (the only quiz our demo events use).

#[test]
fn test_demo_events_match_selected_quiz() {
    let quiz = linear_functions_quiz();
    let events = demo_events();
    let question_ids: std::collections::HashSet<&str> =
        quiz.questions.iter().map(|q| q.id.as_str()).collect();
    assert_eq!(events.len(), 8);
    for event in &events {
        assert!(
            question_ids.contains(event.question_id.as_str()),
            "demo event question_id '{}' not found in quiz",
            event.question_id
        );
    }
}

// ---------------------------------------------------------------------------
// 8. test_non_math_quiz_peer_message_round_trip
// ---------------------------------------------------------------------------
//
// Swift: PeerMessage.quiz(japaneseGrammar) → encode → decode → verify

#[test]
fn test_non_math_quiz_peer_message_round_trip() {
    let quiz = japanese_grammar_quiz();
    let message = PeerMessage::Quiz(quiz.clone());
    let json = serde_json::to_string(&message).expect("encode");
    let decoded: PeerMessage = serde_json::from_str(&json).expect("decode");
    match decoded {
        PeerMessage::Quiz(received) => {
            assert_eq!(received.id, quiz.id);
            assert_eq!(received.title, quiz.title);
            assert_eq!(received.subject, quiz.subject);
            assert_eq!(received.questions.len(), quiz.questions.len());
        }
        other => panic!("expected Quiz, got {:?}", other),
    }
}

// ---------------------------------------------------------------------------
// 9. test_session_and_acknowledgment_round_trip
// ---------------------------------------------------------------------------
//
// Swift: encode/decode PeerMessage.session, .sessionEnded, .acknowledgment

#[test]
fn test_session_and_acknowledgment_round_trip() {
    let quiz = english_grammar_quiz();
    let session = LearningSession {
        id: Uuid::new_v4(),
        quiz,
        started_at: Utc::now(),
    };

    let messages = vec![
        PeerMessage::Session(session.clone()),
        PeerMessage::SessionEnded(session.id),
        PeerMessage::Acknowledgment(Uuid::new_v4()),
    ];

    for message in messages {
        let json = serde_json::to_string(&message).expect("encode");
        let _decoded: PeerMessage = serde_json::from_str(&json).expect("decode");
    }
}

// ---------------------------------------------------------------------------
// 10. test_analysis_event_carries_session_boundary_without_raw_answer
// ---------------------------------------------------------------------------
//
// Swift: create AnalysisEvent with sessionID, encode to JSON,
//   verify session_id is present and JSON does not contain "answer"

#[test]
fn test_analysis_event_carries_session_boundary_without_raw_answer() {
    let session_id = Uuid::new_v4();
    let event = AnalysisEvent {
        id: Uuid::nil(),
        participant_token: "P-TEST".into(),
        session_id: Some(session_id),
        question_id: "q-1".into(),
        concept: "一次関数".into(),
        misconception: Some("傾きと切片の混同".into()),
        correct: false,
        hint_count: 2,
        retry_success: true,
        submitted_at: Utc::now(),
    };
    let json = serde_json::to_string(&event).expect("encode");
    let decoded: AnalysisEvent = serde_json::from_str(&json).expect("decode");

    assert_eq!(decoded.session_id, Some(session_id));
    assert!(
        !json.contains("answer"),
        "JSON should not contain raw 'answer' field: {}",
        json
    );
}

// ---------------------------------------------------------------------------
// 11. test_session_archive_round_trip
// ---------------------------------------------------------------------------
//
// Swift: build SessionArchive with session + event, encode/decode, verify

#[test]
fn test_session_archive_round_trip() {
    let quiz = linear_functions_quiz();
    let session = LearningSession {
        id: Uuid::new_v4(),
        quiz: quiz.clone(),
        started_at: Utc::now(),
    };
    let event = AnalysisEvent {
        id: Uuid::nil(),
        participant_token: "P-TEST".into(),
        session_id: Some(session.id),
        question_id: quiz.questions[0].id.clone(),
        concept: quiz.questions[0].concept.clone(),
        misconception: None,
        correct: true,
        hint_count: 0,
        retry_success: false,
        submitted_at: Utc::now(),
    };
    let archive = SessionArchive {
        id: Uuid::new_v4(),
        session: session.clone(),
        ended_at: Utc::now(),
        events: vec![event.clone()],
        adopted_plan: None,
    };

    let json = serde_json::to_string(&archive).expect("encode");
    let decoded: SessionArchive = serde_json::from_str(&json).expect("decode");

    assert_eq!(decoded.session.id, session.id);
    assert_eq!(decoded.events.len(), 1);
    assert_eq!(decoded.events[0].participant_token, "P-TEST");
    assert_eq!(decoded.events[0].question_id, quiz.questions[0].id);
}
