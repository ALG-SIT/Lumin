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

/// 1回の授業を識別する境界。再接続しても同じIDを使い、別授業の回答混入を防ぐ。
public struct LearningSession: Codable, Identifiable, Hashable, Sendable {
    public let id: UUID
    public let quiz: Quiz
    public let startedAt: Date

    public init(id: UUID = UUID(), quiz: Quiz, startedAt: Date = Date()) {
        self.id = id
        self.quiz = quiz
        self.startedAt = startedAt
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
    /// nilは旧バージョンおよび大会デモ用データとの互換性のために許容する。
    public let sessionID: UUID?
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
        sessionID: UUID? = nil,
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
        self.sessionID = sessionID
        self.questionID = questionID
        self.concept = concept
        self.misconception = misconception
        self.correct = correct
        self.hintCount = hintCount
        self.retrySuccess = retrySuccess
        self.submittedAt = submittedAt
    }
}

/// 教師が明示的に終了した授業の保存形式。解答本文は含まない。
public struct SessionArchive: Codable, Identifiable, Sendable {
    public let id: UUID
    public let session: LearningSession
    public let endedAt: Date
    public var events: [AnalysisEvent]
    public let adoptedPlan: LessonPlan?

    public init(
        id: UUID = UUID(),
        session: LearningSession,
        endedAt: Date = Date(),
        events: [AnalysisEvent],
        adoptedPlan: LessonPlan?
    ) {
        self.id = id
        self.session = session
        self.endedAt = endedAt
        self.events = events
        self.adoptedPlan = adoptedPlan
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
    case session(LearningSession)
    case sessionEnded(UUID)
    case quiz(Quiz)
    case analysis(AnalysisEvent)
    case acknowledgment(UUID)

    private enum CodingKeys: String, CodingKey { case type, session, sessionID, quiz, analysis, eventID }
    private enum MessageType: String, Codable { case session, sessionEnded, quiz, analysis, acknowledgment }

    public init(from decoder: Decoder) throws {
        let container = try decoder.container(keyedBy: CodingKeys.self)
        switch try container.decode(MessageType.self, forKey: .type) {
        case .session: self = .session(try container.decode(LearningSession.self, forKey: .session))
        case .sessionEnded: self = .sessionEnded(try container.decode(UUID.self, forKey: .sessionID))
        case .quiz: self = .quiz(try container.decode(Quiz.self, forKey: .quiz))
        case .analysis: self = .analysis(try container.decode(AnalysisEvent.self, forKey: .analysis))
        case .acknowledgment: self = .acknowledgment(try container.decode(UUID.self, forKey: .eventID))
        }
    }

    public func encode(to encoder: Encoder) throws {
        var container = encoder.container(keyedBy: CodingKeys.self)
        switch self {
        case .session(let session):
            try container.encode(MessageType.session, forKey: .type)
            try container.encode(session, forKey: .session)
        case .sessionEnded(let sessionID):
            try container.encode(MessageType.sessionEnded, forKey: .type)
            try container.encode(sessionID, forKey: .sessionID)
        case .quiz(let quiz):
            try container.encode(MessageType.quiz, forKey: .type)
            try container.encode(quiz, forKey: .quiz)
        case .analysis(let analysis):
            try container.encode(MessageType.analysis, forKey: .type)
            try container.encode(analysis, forKey: .analysis)
        case .acknowledgment(let eventID):
            try container.encode(MessageType.acknowledgment, forKey: .type)
            try container.encode(eventID, forKey: .eventID)
        }
    }
}
