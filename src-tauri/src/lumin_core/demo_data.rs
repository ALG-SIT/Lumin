#![allow(dead_code)]

use crate::lumin_core::models::{AnalysisEvent, Quiz, QuizQuestion};
use chrono::Utc;
use std::collections::HashMap;
use uuid::Uuid;

/// Returns the full 6-subject demo question bank (25 questions total).
///
/// Ported from Swift `SampleData` and `SampleData.questionBank`:
/// - 数学: 一次関数 (5) + 図形と確率 (4)
/// - 理科: 化学変化 (4)
/// - 国語: 文法 (4)
/// - 英語: 文法 (4)
/// - 社会: 地理・歴史 (4)
pub fn demo_quiz_bank() -> Vec<Quiz> {
    vec![
        linear_functions_quiz(),
        geometry_probability_quiz(),
        chemical_changes_quiz(),
        japanese_grammar_quiz(),
        english_grammar_quiz(),
        social_geography_history_quiz(),
    ]
}

/// Returns the 3-question subset for the 3-minute demo flow.
pub fn demo_quiz_short() -> Quiz {
    Quiz {
        id: "demo-quiz-short".into(),
        title: "一次関数 3分デモ".into(),
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
        ],
    }
}

/// Returns the 8 demo events matching Swift `SampleData.demoEvents`.
pub fn demo_events() -> Vec<AnalysisEvent> {
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

/// Extended demo events for the full bank (25 questions, 10+ participants).
pub fn demo_events_full() -> Vec<AnalysisEvent> {
    let mut events = demo_events();
    let extras = vec![
        (
            "P-201",
            "math-gp-01",
            "確率",
            Some("偶数の個数の数えきっか"),
            false,
            1,
            true,
        ),
        ("P-201", "math-gp-02", "三平方の定理", None, true, 0, false),
        ("P-202", "math-gp-01", "確率", None, true, 0, false),
        (
            "P-202",
            "math-gp-03",
            "三角形の内角",
            Some("既知の角だけを加算"),
            false,
            2,
            true,
        ),
        ("P-301", "science-chem-01", "酸化", None, true, 0, false),
        (
            "P-301",
            "science-chem-03",
            "水の電気分解",
            Some("水素と酸素の順序が逆"),
            false,
            2,
            true,
        ),
        ("P-302", "science-chem-02", "還元", None, true, 0, false),
        (
            "P-401",
            "japanese-01",
            "品詞",
            Some("形容詞と形容動詞の混同"),
            false,
            1,
            true,
        ),
        ("P-401", "japanese-03", "接続語", None, true, 0, false),
        (
            "P-501",
            "english-01",
            "過去形",
            Some("不規則動詞を規則変化させている"),
            false,
            1,
            true,
        ),
        (
            "P-501",
            "english-02",
            "三人称単数現在",
            None,
            true,
            0,
            false,
        ),
        ("P-601", "social-01", "日本の都道府県", None, true, 0, false),
        (
            "P-601",
            "social-03",
            "江戸幕府の成立",
            Some("関ヶ原の戦いとの混同"),
            false,
            2,
            false,
        ),
    ];
    for (token, qid, concept, misc, correct, hints, retry) in extras {
        events.push(AnalysisEvent {
            id: Uuid::nil(),
            participant_token: token.into(),
            session_id: None,
            question_id: qid.into(),
            concept: concept.into(),
            misconception: misc.map(|s| s.into()),
            correct,
            hint_count: hints,
            retry_success: retry,
            submitted_at: Utc::now(),
        });
    }
    events
}

// ---------------------------------------------------------------------------
// Individual quiz builders (ported from Swift QuestionBank.swift + SampleData.swift)
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
                    "この式を y = a\u{00d7}x + b と見比べ、x の直前の数を探そう。".into(),
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
                    "傾き = yの増加量 \u{00f7} xの増加量 です。".into(),
                    "(7 \u{2212} 3) \u{00f7} (3 \u{2212} 1) の順で計算してみよう。".into(),
                ],
                explanation: "y は4増え、x は2増えるので、傾きは 4\u{00f7}2=2 です。".into(),
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
                    "まず 4\u{00d7}2 を計算し、その後で切片を扱います。".into(),
                    "4\u{00d7}2 \u{2212} 1 という途中式まで書いてみよう。".into(),
                ],
                explanation: "4\u{00d7}2\u{2212}1=8\u{2212}1=7 です。".into(),
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

fn geometry_probability_quiz() -> Quiz {
    Quiz {
        id: "geometry-probability-01".into(),
        title: "図形と確率 ミニチェック".into(),
        subject: "中学数学".into(),
        topic: Some("図形・確率".into()),
        questions: vec![
            QuizQuestion {
                id: "math-gp-01".into(),
                prompt: "公平なさいころを1回投げるとき、偶数が出る確率は？".into(),
                concept: "確率".into(),
                accepted_answers: vec!["1/2".into(), "0.5".into(), "50%".into()],
                misconception_answers: HashMap::from([
                    ("1/3".into(), "偶数の個数の数え間違い".into()),
                    ("2/3".into(), "有利な場合と全場合の取り違え".into()),
                ]),
                generic_misconception: "確率の分母・分子の理解不足".into(),
                hints: vec![
                    "出る目を1から6まで書き出そう。".into(),
                    "偶数は2\u{00b7}4\u{00b7}6の3通りです。".into(),
                    "確率は、有利な場合の数\u{00f7}すべての場合の数です。".into(),
                ],
                explanation: "偶数は6通り中3通りなので、3/6=1/2です。".into(),
            },
            QuizQuestion {
                id: "math-gp-02".into(),
                prompt: "直角をはさむ2辺が3cmと4cmの直角三角形。斜辺の長さは？".into(),
                concept: "三平方の定理".into(),
                accepted_answers: vec!["5".into(), "5cm".into()],
                misconception_answers: HashMap::from([
                    ("7".into(), "辺の長さをそのまま加算".into()),
                    ("25".into(), "平方根を取り忘れている".into()),
                ]),
                generic_misconception: "三平方の定理の適用不足".into(),
                hints: vec![
                    "斜辺をcとして a\u{00b2}+b\u{00b2}=c\u{00b2} を使おう。".into(),
                    "3\u{00b2}+4\u{00b2}を計算しよう。".into(),
                    "c\u{00b2}=25。最後にcを求めよう。".into(),
                ],
                explanation: "3\u{00b2}+4\u{00b2}=9+16=25なので、斜辺は\u{221a}25=5cmです。".into(),
            },
            QuizQuestion {
                id: "math-gp-03".into(),
                prompt: "三角形の2つの内角が50\u{00b0}と60\u{00b0}です。残りの角は？".into(),
                concept: "三角形の内角".into(),
                accepted_answers: vec!["70".into(), "70\u{00b0}".into(), "70度".into()],
                misconception_answers: HashMap::from([
                    ("110".into(), "既知の角だけを加算".into()),
                    ("290".into(), "一回転の角度から引いている".into()),
                ]),
                generic_misconception: "三角形の内角の和の理解不足".into(),
                hints: vec![
                    "三角形の内角の和を思い出そう。".into(),
                    "3つの角を足すと180\u{00b0}です。".into(),
                    "180\u{2212}(50+60)を計算しよう。".into(),
                ],
                explanation: "三角形の内角の和は180\u{00b0}なので、180\u{2212}110=70\u{00b0}です。"
                    .into(),
            },
            QuizQuestion {
                id: "math-gp-04".into(),
                prompt: "半径3cmの円の面積を、\u{03c0}を使って表すと？".into(),
                concept: "円の面積".into(),
                accepted_answers: vec![
                    "9\u{03c0}".into(),
                    "9\u{03c0}cm2".into(),
                    "9\u{03c0}cm\u{00b2}".into(),
                    "9\u{03c0}平方cm".into(),
                ],
                misconception_answers: HashMap::from([
                    ("6\u{03c0}".into(), "面積と円周の公式を混同".into()),
                    ("3\u{03c0}".into(), "半径を2乗していない".into()),
                ]),
                generic_misconception: "円の面積公式の理解不足".into(),
                hints: vec![
                    "円の面積は半径\u{00d7}半径\u{00d7}\u{03c0}です。".into(),
                    "半径3を2回かけます。".into(),
                    "3\u{00d7}3\u{00d7}\u{03c0}を計算しよう。".into(),
                ],
                explanation: "3\u{00d7}3\u{00d7}\u{03c0}=9\u{03c0}cm\u{00b2}です。".into(),
            },
        ],
    }
}

fn chemical_changes_quiz() -> Quiz {
    Quiz {
        id: "chemical-changes-01".into(),
        title: "化学変化 ミニチェック".into(),
        subject: "中学理科".into(),
        topic: Some("化学変化と物質".into()),
        questions: vec![
            QuizQuestion {
                id: "science-chem-01".into(),
                prompt: "物質が酸素と結びつく化学変化を何という？".into(),
                concept: "酸化".into(),
                accepted_answers: vec!["酸化".into()],
                misconception_answers: HashMap::from([
                    ("還元".into(), "酸化と還元の混同".into()),
                    ("燃焼".into(), "酸化と激しい酸化の混同".into()),
                ]),
                generic_misconception: "酸化の定義の理解不足".into(),
                hints: vec![
                    "変化の前後で酸素が増えています。".into(),
                    "鉄がさびる変化もこの一種です。".into(),
                    "「酸素と化合する」の最初の漢字を手がかりにしよう。".into(),
                ],
                explanation: "物質が酸素と結びつく変化を酸化といいます。".into(),
            },
            QuizQuestion {
                id: "science-chem-02".into(),
                prompt: "酸化銅から酸素を取り除く化学変化を何という？".into(),
                concept: "還元".into(),
                accepted_answers: vec!["還元".into()],
                misconception_answers: HashMap::from([
                    ("酸化".into(), "酸化と還元の混同".into()),
                    ("分解".into(), "化学変化の分類の混同".into()),
                ]),
                generic_misconception: "還元の定義の理解不足".into(),
                hints: vec![
                    "酸素が結びつく変化とは反対です。".into(),
                    "酸化物から酸素が失われます。".into(),
                    "酸化の反対にあたる二字熟語です。".into(),
                ],
                explanation: "酸化物から酸素を取り除く変化を還元といいます。".into(),
            },
            QuizQuestion {
                id: "science-chem-03".into(),
                prompt: "水を電気分解したとき、水素と酸素の体積比（水素:酸素）は？".into(),
                concept: "水の電気分解".into(),
                accepted_answers: vec!["2:1".into(), "2対1".into(), "2たい1".into()],
                misconception_answers: HashMap::from([
                    ("1:1".into(), "発生する気体の体積が同じだと誤認".into()),
                    ("1:2".into(), "水素と酸素の順序が逆".into()),
                ]),
                generic_misconception: "水の電気分解の量的関係の理解不足".into(),
                hints: vec![
                    "水の化学式H\u{2082}Oに注目しよう。".into(),
                    "水素は酸素の2倍の体積が発生します。".into(),
                    "水素を先に書く比です。".into(),
                ],
                explanation: "水素は酸素の2倍発生するので、体積比は2:1です。".into(),
            },
            QuizQuestion {
                id: "science-chem-04".into(),
                prompt: "密閉した容器内の化学変化で、変化の前後に保たれる物質全体の量は？".into(),
                concept: "質量保存の法則".into(),
                accepted_answers: vec!["質量".into(), "重さ".into()],
                misconception_answers: HashMap::from([
                    ("体積".into(), "質量と体積の混同".into()),
                    ("密度".into(), "質量と密度の混同".into()),
                ]),
                generic_misconception: "質量保存の法則の理解不足".into(),
                hints: vec![
                    "容器の外へ物質は出入りしません。".into(),
                    "物質の見た目や体積は変わることがあります。".into(),
                    "「\u{25cb}\u{25cb}保存の法則」の\u{25cb}\u{25cb}を答えよう。".into(),
                ],
                explanation: "閉じた系では、化学変化の前後で物質全体の質量は変わりません。".into(),
            },
        ],
    }
}

fn japanese_grammar_quiz() -> Quiz {
    Quiz {
        id: "japanese-grammar-01".into(),
        title: "文法・言葉 ミニチェック".into(),
        subject: "中学国語".into(),
        topic: Some("文法と接続関係".into()),
        questions: vec![
            QuizQuestion {
                id: "japanese-01".into(),
                prompt: "「静かな町」の「静かな」の品詞は？".into(),
                concept: "品詞".into(),
                accepted_answers: vec!["形容動詞".into()],
                misconception_answers: HashMap::from([
                    ("形容詞".into(), "形容詞と形容動詞の混同".into()),
                    ("連体詞".into(), "活用する語と連体詞の混同".into()),
                ]),
                generic_misconception: "品詞の識別不足".into(),
                hints: vec![
                    "言い切りの形に直してみよう。".into(),
                    "言い切りは「静かだ」です。".into(),
                    "語幹に「だ」を付けて言い切る品詞です。".into(),
                ],
                explanation: "「静かな」は「静かだ」と活用する形容動詞です。".into(),
            },
            QuizQuestion {
                id: "japanese-02".into(),
                prompt: "「私は図書館で本を読む。」の主語は？".into(),
                concept: "文の成分".into(),
                accepted_answers: vec!["私".into(), "私は".into()],
                misconception_answers: HashMap::from([
                    ("図書館".into(), "場所を表す修飾語との混同".into()),
                    ("本".into(), "目的語との混同".into()),
                ]),
                generic_misconception: "主語の識別不足".into(),
                hints: vec![
                    "「誰が\u{00b7}何が」に当たる部分を探そう。".into(),
                    "本を読むのは誰でしょう。".into(),
                    "助詞「は」が付いた文節に注目しよう。".into(),
                ],
                explanation: "「誰が読むのか」を表す「私は」が主語です。".into(),
            },
            QuizQuestion {
                id: "japanese-03".into(),
                prompt: "「雨が降った。しかし、試合は行われた。」の「しかし」が表す接続関係は？"
                    .into(),
                concept: "接続語".into(),
                accepted_answers: vec!["逆接".into()],
                misconception_answers: HashMap::from([
                    ("順接".into(), "順接と逆接の混同".into()),
                    ("並列".into(), "並列と逆接の混同".into()),
                ]),
                generic_misconception: "接続関係の理解不足".into(),
                hints: vec![
                    "前後の内容は予想どおりにつながっていますか。".into(),
                    "雨なら中止になりそうですが、結果は反対です。".into(),
                    "予想と逆の結果を結ぶ関係です。".into(),
                ],
                explanation: "前の内容から予想される結果と逆の内容を結ぶので、逆接です。".into(),
            },
            QuizQuestion {
                id: "japanese-04".into(),
                prompt: "「まるで宝石のように輝く」で使われている表現技法は？".into(),
                concept: "表現技法".into(),
                accepted_answers: vec!["直喩".into(), "比喩".into(), "明喩".into()],
                misconception_answers: HashMap::from([
                    ("隠喩".into(), "直喩と隠喩の混同".into()),
                    ("擬人法".into(), "比喩表現の種類の混同".into()),
                ]),
                generic_misconception: "比喩表現の識別不足".into(),
                hints: vec![
                    "「ように」という言葉に注目しよう。".into(),
                    "別のものに、目印となる言葉を使ってたとえています。".into(),
                    "「ようだ」「まるで」を使う比喩です。".into(),
                ],
                explanation: "「まるで」「ように」を使って明示的にたとえる直喩です。".into(),
            },
        ],
    }
}

fn english_grammar_quiz() -> Quiz {
    Quiz {
        id: "english-grammar-01".into(),
        title: "英文法 ミニチェック".into(),
        subject: "中学英語".into(),
        topic: Some("時制・比較・語順".into()),
        questions: vec![
            QuizQuestion {
                id: "english-01".into(),
                prompt: "I ( go ) to the park yesterday. goを適切な形にすると？".into(),
                concept: "過去形".into(),
                accepted_answers: vec!["went".into()],
                misconception_answers: HashMap::from([
                    ("goed".into(), "不規則動詞を規則変化させている".into()),
                    ("go".into(), "時制の読み落とし".into()),
                ]),
                generic_misconception: "過去形の理解不足".into(),
                hints: vec![
                    "yesterdayは過去を表します。".into(),
                    "goは不規則動詞です。".into(),
                    "go\u{2013}went\u{2013}goneの2番目を使おう。".into(),
                ],
                explanation: "yesterdayがあるため過去形を使い、goはwentになります。".into(),
            },
            QuizQuestion {
                id: "english-02".into(),
                prompt: "She ___ tennis every Sunday. (play)".into(),
                concept: "三人称単数現在".into(),
                accepted_answers: vec!["plays".into()],
                misconception_answers: HashMap::from([
                    ("play".into(), "三単現のsの付け忘れ".into()),
                    ("played".into(), "習慣と過去の混同".into()),
                ]),
                generic_misconception: "三人称単数現在の理解不足".into(),
                hints: vec![
                    "every Sundayは習慣を表します。".into(),
                    "主語Sheは三人称単数です。".into(),
                    "現在形の動詞playの末尾にsを付けよう。".into(),
                ],
                explanation: "現在の習慣で主語がSheなので、動詞はplaysです。".into(),
            },
            QuizQuestion {
                id: "english-03".into(),
                prompt: "This book is ___ than that one. (interesting)".into(),
                concept: "比較級".into(),
                accepted_answers: vec!["more interesting".into(), "moreinteresting".into()],
                misconception_answers: HashMap::from([
                    ("interestinger".into(), "長い形容詞に-erを付けている".into()),
                    ("most interesting".into(), "比較級と最上級の混同".into()),
                ]),
                generic_misconception: "比較級の作り方の理解不足".into(),
                hints: vec![
                    "thanは2つを比べる合図です。".into(),
                    "interestingは比較的長い形容詞です。".into(),
                    "形容詞の前にmoreを置こう。".into(),
                ],
                explanation: "長い形容詞interestingの比較級はmore interestingです。".into(),
            },
            QuizQuestion {
                id: "english-04".into(),
                prompt: "「彼は今、宿題をしています」を英語にすると？".into(),
                concept: "現在進行形".into(),
                accepted_answers: vec![
                    "he is doing his homework now".into(),
                    "he's doing his homework now".into(),
                    "he is doing homework now".into(),
                    "he's doing homework now".into(),
                ],
                misconception_answers: HashMap::from([
                    (
                        "he does his homework now".into(),
                        "現在形と現在進行形の混同".into(),
                    ),
                    ("he doing his homework now".into(), "be動詞の欠落".into()),
                ]),
                generic_misconception: "現在進行形の語順の理解不足".into(),
                hints: vec![
                    "「今\u{00b7}している」は現在進行形です。".into(),
                    "現在進行形はbe動詞＋動詞ingです。".into(),
                    "主語Heに合うbe動詞isを使おう。".into(),
                ],
                explanation: "現在進行形の形にして、He is doing his homework now.となります。"
                    .into(),
            },
        ],
    }
}

fn social_geography_history_quiz() -> Quiz {
    Quiz {
        id: "social-geography-history-01".into(),
        title: "地理・歴史 ミニチェック".into(),
        subject: "中学社会".into(),
        topic: Some("日本の地理と近世・近代".into()),
        questions: vec![
            QuizQuestion {
                id: "social-01".into(),
                prompt: "日本で最も面積が大きい都道府県は？".into(),
                concept: "日本の都道府県".into(),
                accepted_answers: vec!["北海道".into()],
                misconception_answers: HashMap::from([
                    ("岩手県".into(), "都道府県の面積順位の混同".into()),
                    ("長野県".into(), "都道府県の面積順位の混同".into()),
                ]),
                generic_misconception: "都道府県の地理知識不足".into(),
                hints: vec![
                    "日本列島の北にあります。".into(),
                    "本州ではない大きな島です。".into(),
                    "道庁所在地は札幌市です。".into(),
                ],
                explanation: "日本で最も面積が大きい都道府県は北海道です。".into(),
            },
            QuizQuestion {
                id: "social-02".into(),
                prompt: "日本標準時の基準となる東経135度の子午線が通る兵庫県の市は？".into(),
                concept: "標準時と経度".into(),
                accepted_answers: vec!["明石市".into(), "明石".into()],
                misconception_answers: HashMap::from([
                    ("神戸市".into(), "兵庫県の県庁所在地との混同".into()),
                    ("東京".into(), "首都と標準時子午線の混同".into()),
                ]),
                generic_misconception: "標準時子午線の地理知識不足".into(),
                hints: vec![
                    "兵庫県南部の市です。".into(),
                    "子午線上に天文科学館があります。".into(),
                    "「日本標準時のまち」と呼ばれます。".into(),
                ],
                explanation: "東経135度の日本標準時子午線は兵庫県明石市を通ります。".into(),
            },
            QuizQuestion {
                id: "social-03".into(),
                prompt: "徳川家康が江戸幕府を開いた年は？".into(),
                concept: "江戸幕府の成立".into(),
                accepted_answers: vec!["1603".into(), "1603年".into()],
                misconception_answers: HashMap::from([
                    ("1600".into(), "関ヶ原の戦いとの混同".into()),
                    ("鎌倉時代".into(), "時代区分の混同".into()),
                ]),
                generic_misconception: "江戸幕府成立年の理解不足".into(),
                hints: vec![
                    "関ヶ原の戦いの3年後です。".into(),
                    "17世紀の初めです。".into(),
                    "西暦1600に3を足そう。".into(),
                ],
                explanation: "徳川家康は1603年に征夷大将軍となり、江戸幕府を開きました。".into(),
            },
            QuizQuestion {
                id: "social-04".into(),
                prompt: "明治政府が藩を廃止して府県を置いた政策は？".into(),
                concept: "明治維新".into(),
                accepted_answers: vec!["廃藩置県".into()],
                misconception_answers: HashMap::from([
                    ("版籍奉還".into(), "版籍奉還と廃藩置県の混同".into()),
                    ("大政奉還".into(), "江戸幕府の終結過程との混同".into()),
                ]),
                generic_misconception: "明治初期の政策の理解不足".into(),
                hints: vec![
                    "中央集権化を進めた政策です。".into(),
                    "藩を「廃」し、府県を「置」きました。".into(),
                    "問題文の言葉を四字熟語にまとめよう。".into(),
                ],
                explanation: "藩を廃止し府県を置いた政策を廃藩置県といいます。".into(),
            },
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_quiz_bank_has_six_subjects() {
        let bank = demo_quiz_bank();
        assert_eq!(bank.len(), 6, "expected 6 quizzes in bank");
    }

    #[test]
    fn test_quiz_bank_has_25_plus_questions() {
        let bank = demo_quiz_bank();
        let total: usize = bank.iter().map(|q| q.questions.len()).sum();
        assert!(total >= 25, "expected >= 25 questions, got {total}");
    }

    #[test]
    fn test_all_questions_have_accepted_answers() {
        let bank = demo_quiz_bank();
        for quiz in &bank {
            for q in &quiz.questions {
                assert!(
                    !q.accepted_answers.is_empty(),
                    "question {} missing answers",
                    q.id
                );
            }
        }
    }

    #[test]
    fn test_all_questions_have_3_hints() {
        let bank = demo_quiz_bank();
        for quiz in &bank {
            for q in &quiz.questions {
                assert!(
                    q.hints.len() >= 3,
                    "question {} has {} hints, need >= 3",
                    q.id,
                    q.hints.len()
                );
            }
        }
    }

    #[test]
    fn test_demo_quiz_short_has_3_questions() {
        let quiz = demo_quiz_short();
        assert_eq!(quiz.questions.len(), 3);
    }

    #[test]
    fn test_demo_events_has_8_events() {
        let events = demo_events();
        assert_eq!(events.len(), 8);
    }

    #[test]
    fn test_demo_events_correct_count() {
        let events = demo_events();
        let correct = events.iter().filter(|e| e.correct).count();
        assert_eq!(correct, 3, "P-102, P-104, P-107 should be correct");
    }

    #[test]
    fn test_demo_events_full_has_extra_events() {
        let events = demo_events_full();
        assert!(events.len() > 8, "full should have more than 8 events");
    }

    #[test]
    fn test_all_question_ids_unique() {
        let bank = demo_quiz_bank();
        let mut ids = std::collections::HashSet::new();
        for quiz in &bank {
            for q in &quiz.questions {
                assert!(ids.insert(q.id.clone()), "duplicate question id: {}", q.id);
            }
        }
    }
}
