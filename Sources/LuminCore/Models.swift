import Foundation

public struct QuizQuestion: Codable, Identifiable, Hashable, Sendable {
    public let id: String
    public let prompt: String
    public let concept: String
    public let acceptedAnswers: [String]
    public let misconceptionAnswers: [String: String]
    public let genericMisconception: String
    public let hints: [String]
    public let explanation: String

    public init(
        id: String,
        prompt: String,
        concept: String,
        acceptedAnswers: [String],
        misconceptionAnswers: [String: String],
        genericMisconception: String,
        hints: [String],
        explanation: String
    ) {
        self.id = id
        self.prompt = prompt
        self.concept = concept
        self.acceptedAnswers = acceptedAnswers
        self.misconceptionAnswers = misconceptionAnswers
        self.genericMisconception = genericMisconception
        self.hints = hints
        self.explanation = explanation
    }
}

public struct Quiz: Codable, Identifiable, Hashable, Sendable {
    public let id: String
    public let title: String
    public let subject: String
    /// 教師画面で表示する単元。旧形式のQuizをデコードできるようoptionalにしている。
    public let topic: String?
    public let questions: [QuizQuestion]

    public init(
        id: String,
        title: String,
        subject: String,
        topic: String? = nil,
        questions: [QuizQuestion]
    ) {
        self.id = id
        self.title = title
        self.subject = subject
        self.topic = topic
        self.questions = questions
    }
}

public struct AnswerAnalysis: Equatable, Sendable {
    public let isCorrect: Bool
    public let misconception: String?

    public init(isCorrect: Bool, misconception: String?) {
        self.isCorrect = isCorrect
        self.misconception = misconception
    }
}

/// 生徒の解答本文を含まない、教師端末へ共有可能な最小データ。
public struct AnalysisEvent: Codable, Identifiable, Hashable, Sendable {
    public let id: UUID
    public let participantToken: String
    public let questionID: String
    public let concept: String
    public let misconception: String?
    public let correct: Bool
    public let hintCount: Int
    public let retrySuccess: Bool
    public let submittedAt: Date

    public init(
        id: UUID = UUID(),
        participantToken: String,
        questionID: String,
        concept: String,
        misconception: String?,
        correct: Bool,
        hintCount: Int,
        retrySuccess: Bool,
        submittedAt: Date = Date()
    ) {
        self.id = id
        self.participantToken = participantToken
        self.questionID = questionID
        self.concept = concept
        self.misconception = misconception
        self.correct = correct
        self.hintCount = hintCount
        self.retrySuccess = retrySuccess
        self.submittedAt = submittedAt
    }
}

public struct MisconceptionSummary: Identifiable, Equatable, Sendable {
    public var id: String { name }
    public let name: String
    public let count: Int
    public let share: Double

    public init(name: String, count: Int, share: Double) {
        self.name = name
        self.count = count
        self.share = share
    }
}

public struct ClassSummary: Equatable, Sendable {
    public let participantCount: Int
    public let responseCount: Int
    public let correctRate: Double
    public let retrySuccessRate: Double
    public let averageHints: Double
    public let misconceptions: [MisconceptionSummary]

    public init(
        participantCount: Int,
        responseCount: Int,
        correctRate: Double,
        retrySuccessRate: Double,
        averageHints: Double,
        misconceptions: [MisconceptionSummary]
    ) {
        self.participantCount = participantCount
        self.responseCount = responseCount
        self.correctRate = correctRate
        self.retrySuccessRate = retrySuccessRate
        self.averageHints = averageHints
        self.misconceptions = misconceptions
    }
}

public struct LessonPlan: Codable, Equatable, Sendable {
    public var focus: String
    public var steps: [String]
    public var checkQuestion: String
    public var teacherNote: String

    public init(focus: String, steps: [String], checkQuestion: String, teacherNote: String) {
        self.focus = focus
        self.steps = steps
        self.checkQuestion = checkQuestion
        self.teacherNote = teacherNote
    }
}

public enum PeerMessage: Codable, Sendable {
    case quiz(Quiz)
    case analysis(AnalysisEvent)

    private enum CodingKeys: String, CodingKey { case type, quiz, analysis }
    private enum MessageType: String, Codable { case quiz, analysis }

    public init(from decoder: Decoder) throws {
        let container = try decoder.container(keyedBy: CodingKeys.self)
        switch try container.decode(MessageType.self, forKey: .type) {
        case .quiz: self = .quiz(try container.decode(Quiz.self, forKey: .quiz))
        case .analysis: self = .analysis(try container.decode(AnalysisEvent.self, forKey: .analysis))
        }
    }

    public func encode(to encoder: Encoder) throws {
        var container = encoder.container(keyedBy: CodingKeys.self)
        switch self {
        case .quiz(let quiz):
            try container.encode(MessageType.quiz, forKey: .type)
            try container.encode(quiz, forKey: .quiz)
        case .analysis(let analysis):
            try container.encode(MessageType.analysis, forKey: .type)
            try container.encode(analysis, forKey: .analysis)
        }
    }
}
