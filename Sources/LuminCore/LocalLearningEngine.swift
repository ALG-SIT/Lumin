import Foundation

public protocol LearningAnalyzing: Sendable {
    func analyze(answer: String, for question: QuizQuestion) -> AnswerAnalysis
}

/// 通信不要で動くMVP向け分析器。将来はFoundation Models / Gemma実装へ差し替え可能。
public struct RuleBasedLearningEngine: LearningAnalyzing {
    public init() {}

    public func analyze(answer: String, for question: QuizQuestion) -> AnswerAnalysis {
        let normalized = normalize(answer)
        let accepted = question.acceptedAnswers.map(normalize)
        if accepted.contains(normalized) {
            return AnswerAnalysis(isCorrect: true, misconception: nil)
        }

        let matched = question.misconceptionAnswers.first { normalize($0.key) == normalized }
        return AnswerAnalysis(
            isCorrect: false,
            misconception: matched?.value ?? question.genericMisconception
        )
    }

    private func normalize(_ value: String) -> String {
        let halfWidth = value.applyingTransform(.fullwidthToHalfwidth, reverse: false) ?? value
        return halfWidth
            .trimmingCharacters(in: .whitespacesAndNewlines)
            .lowercased()
            .replacingOccurrences(of: " ", with: "")
            .replacingOccurrences(of: "　", with: "")
            .replacingOccurrences(of: "\n", with: "")
            .replacingOccurrences(of: "−", with: "-")
            .replacingOccurrences(of: "–", with: "-")
            .replacingOccurrences(of: "’", with: "'")
            .trimmingCharacters(in: CharacterSet(charactersIn: "。、｡，,．."))
    }
}

public enum ClassAnalytics {
    public static func summarize(_ events: [AnalysisEvent]) -> ClassSummary {
        guard !events.isEmpty else {
            return ClassSummary(
                participantCount: 0,
                responseCount: 0,
                correctRate: 0,
                retrySuccessRate: 0,
                averageHints: 0,
                misconceptions: []
            )
        }

        let participants = Set(events.map(\.participantToken)).count
        let correct = events.filter(\.correct).count
        let retries = events.filter { !$0.correct && $0.hintCount > 0 }
        let retrySuccesses = retries.filter(\.retrySuccess).count
        let grouped = Dictionary(grouping: events.compactMap(\.misconception), by: { $0 })
        let misconceptions = grouped.map { name, values in
            MisconceptionSummary(
                name: name,
                count: values.count,
                share: Double(values.count) / Double(events.count)
            )
        }.sorted { $0.count > $1.count }

        return ClassSummary(
            participantCount: participants,
            responseCount: events.count,
            correctRate: Double(correct) / Double(events.count),
            retrySuccessRate: retries.isEmpty ? 0 : Double(retrySuccesses) / Double(retries.count),
            averageHints: Double(events.reduce(0) { $0 + $1.hintCount }) / Double(events.count),
            misconceptions: misconceptions
        )
    }
}

public enum LessonPlanGenerator {
    public static func generate(from summary: ClassSummary, quiz: Quiz? = nil) -> LessonPlan {
        let defaultFocus = quiz.map { "\($0.topic ?? $0.title)の主要概念" } ?? "この単元の主要概念"
        let top = summary.misconceptions.first?.name ?? defaultFocus
        let percentage = Int((summary.misconceptions.first?.share ?? 0) * 100)
        return LessonPlan(
            focus: "「\(top)」を解きほぐす",
            steps: [
                "0〜2分：正答例と代表的な誤答を見比べ、違いを見つける",
                "2〜5分：判断の根拠をキーワードを使ってペアで説明する",
                "5〜8分：誤答が多かった考え方を使う類題に再挑戦する",
                "8〜10分：答えと理由を共有し、出口問題で理解を確認する"
            ],
            checkQuestion: "今日の要点を一文で説明し、その考え方を使う例を一つ示してください。",
            teacherNote: percentage > 0
                ? "全回答の約\(percentage)%で「\(top)」が見られました。正解を先に示さず、判断の根拠を生徒自身の言葉にさせてください。"
                : "回答が集まったら、上位の誤概念に合わせて内容を再生成してください。"
        )
    }
}
