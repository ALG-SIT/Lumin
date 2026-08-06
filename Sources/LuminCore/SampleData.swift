import Foundation

public enum SampleData {
    public static let linearFunctions = Quiz(
        id: "linear-functions-01",
        title: "一次関数 ミニチェック",
        subject: "中学数学",
        topic: "一次関数",
        questions: [
            QuizQuestion(
                id: "math-01",
                prompt: "y = 3x + 2 の傾きは？",
                concept: "一次関数の傾き",
                acceptedAnswers: ["3", "+3"],
                misconceptionAnswers: ["2": "傾きと切片の混同"],
                genericMisconception: "傾きの読み取り不足",
                hints: [
                    "x が1増えたとき、y がいくつ増えるかに注目しよう。",
                    "y = ax + b の a が変化の割合を表します。",
                    "この式を y = a×x + b と見比べ、x の直前の数を探そう。"
                ],
                explanation: "y = ax + b では、x の係数 a が傾きです。"
            ),
            QuizQuestion(
                id: "math-02",
                prompt: "y = -2x + 5 の切片は？",
                concept: "一次関数の切片",
                acceptedAnswers: ["5", "+5"],
                misconceptionAnswers: ["-2": "傾きと切片の混同", "2": "符号の読み落とし"],
                genericMisconception: "切片の読み取り不足",
                hints: [
                    "グラフが y 軸と交わる場所を考えよう。",
                    "x = 0 を式に代入すると切片が分かります。",
                    "y = ax + b の b に当たる数を探そう。"
                ],
                explanation: "x = 0 のとき y = 5 なので、切片は5です。"
            ),
            QuizQuestion(
                id: "math-03",
                prompt: "点 (1, 3) と (3, 7) を通る直線の傾きは？",
                concept: "変化の割合",
                acceptedAnswers: ["2", "+2"],
                misconceptionAnswers: ["4": "xの変化量を考慮していない", "1/2": "変化量の分子と分母の逆転", "0.5": "変化量の分子と分母の逆転"],
                genericMisconception: "変化の割合の計算ミス",
                hints: [
                    "y の変化量だけでなく、x の変化量も求めよう。",
                    "傾き = yの増加量 ÷ xの増加量 です。",
                    "(7 − 3) ÷ (3 − 1) の順で計算してみよう。"
                ],
                explanation: "y は4増え、x は2増えるので、傾きは 4÷2=2 です。"
            ),
            QuizQuestion(
                id: "math-04",
                prompt: "y = 4x - 1 で x = 2 のとき、y は？",
                concept: "式への代入",
                acceptedAnswers: ["7", "+7"],
                misconceptionAnswers: ["8": "切片の計算漏れ", "9": "切片の符号の誤り", "6": "代入計算の誤り"],
                genericMisconception: "代入計算の誤り",
                hints: [
                    "式の x を、かっこ付きの2に置き換えよう。",
                    "まず 4×2 を計算し、その後で切片を扱います。",
                    "4×2 − 1 という途中式まで書いてみよう。"
                ],
                explanation: "4×2−1=8−1=7 です。"
            ),
            QuizQuestion(
                id: "math-05",
                prompt: "傾きが -3、切片が4の一次関数を式で表すと？",
                concept: "一次関数の式",
                acceptedAnswers: ["y=-3x+4", "y=4-3x"],
                misconceptionAnswers: ["y=3x+4": "傾きの符号の読み落とし", "y=4x-3": "傾きと切片の混同"],
                genericMisconception: "式の組み立て不足",
                hints: [
                    "一次関数の基本形 y = ax + b を使おう。",
                    "a に傾き、b に切片を入れます。",
                    "a = -3、b = 4 を y = ax + b に代入しよう。"
                ],
                explanation: "y = ax + b に a=-3、b=4 を入れると y=-3x+4 です。"
            )
        ]
    )

    public static let demoEvents: [AnalysisEvent] = [
        AnalysisEvent(participantToken: "P-101", questionID: "math-01", concept: "一次関数の傾き", misconception: "傾きと切片の混同", correct: false, hintCount: 2, retrySuccess: true),
        AnalysisEvent(participantToken: "P-102", questionID: "math-01", concept: "一次関数の傾き", misconception: nil, correct: true, hintCount: 0, retrySuccess: false),
        AnalysisEvent(participantToken: "P-103", questionID: "math-01", concept: "一次関数の傾き", misconception: "傾きと切片の混同", correct: false, hintCount: 3, retrySuccess: false),
        AnalysisEvent(participantToken: "P-104", questionID: "math-02", concept: "一次関数の切片", misconception: nil, correct: true, hintCount: 0, retrySuccess: false),
        AnalysisEvent(participantToken: "P-105", questionID: "math-03", concept: "変化の割合", misconception: "変化量の分子と分母の逆転", correct: false, hintCount: 1, retrySuccess: true),
        AnalysisEvent(participantToken: "P-106", questionID: "math-04", concept: "式への代入", misconception: "切片の計算漏れ", correct: false, hintCount: 2, retrySuccess: true),
        AnalysisEvent(participantToken: "P-107", questionID: "math-05", concept: "一次関数の式", misconception: nil, correct: true, hintCount: 0, retrySuccess: false),
        AnalysisEvent(participantToken: "P-108", questionID: "math-05", concept: "一次関数の式", misconception: "傾きと切片の混同", correct: false, hintCount: 2, retrySuccess: true)
    ]
}
