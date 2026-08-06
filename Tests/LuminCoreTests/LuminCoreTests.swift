import XCTest
@testable import LuminCore

final class LuminCoreTests: XCTestCase {
    private let engine = RuleBasedLearningEngine()

    func testCorrectAnswerNormalization() {
        let question = SampleData.linearFunctions.questions[4]
        XCTAssertTrue(engine.analyze(answer: " y = −3x + 4 ", for: question).isCorrect)
    }

    func testKnownMisconceptionClassification() {
        let question = SampleData.linearFunctions.questions[0]
        let result = engine.analyze(answer: "2", for: question)
        XCTAssertFalse(result.isCorrect)
        XCTAssertEqual(result.misconception, "傾きと切片の混同")
    }

    func testSummaryDoesNotNeedRawAnswers() {
        let summary = ClassAnalytics.summarize(SampleData.demoEvents)
        XCTAssertEqual(summary.participantCount, 8)
        XCTAssertEqual(summary.responseCount, 8)
        XCTAssertEqual(summary.correctRate, 3.0 / 8.0, accuracy: 0.001)
        XCTAssertEqual(summary.misconceptions.first?.name, "傾きと切片の混同")
    }

    func testPeerMessageRoundTrip() throws {
        let message = PeerMessage.analysis(SampleData.demoEvents[0])
        let data = try JSONEncoder().encode(message)
        let decoded = try JSONDecoder().decode(PeerMessage.self, from: data)
        guard case .analysis(let event) = decoded else { return XCTFail("Expected analysis") }
        XCTAssertEqual(event, SampleData.demoEvents[0])
    }

    func testQuestionBankCoversFiveSubjectsWithValidQuestions() {
        XCTAssertEqual(Set(SampleData.quizzes.map(\.subject)).count, 5)
        XCTAssertEqual(SampleData.quizzes.count, 6)
        XCTAssertEqual(SampleData.quizzes.flatMap(\.questions).count, 25)

        let quizzes = SampleData.quizzes
        XCTAssertEqual(Set(quizzes.map(\.id)).count, quizzes.count)
        let questions = quizzes.flatMap(\.questions)
        XCTAssertEqual(Set(questions.map(\.id)).count, questions.count)
        XCTAssertTrue(questions.allSatisfy { !$0.acceptedAnswers.isEmpty && $0.hints.count >= 3 })
    }

    func testFullWidthEnglishAnswerNormalization() {
        let question = SampleData.englishGrammar.questions[0]
        XCTAssertTrue(engine.analyze(answer: "ＷＥＮＴ。", for: question).isCorrect)
    }

    func testDemoEventsMatchSelectedQuiz() {
        let quiz = SampleData.chemicalChanges
        let events = SampleData.demoEvents(for: quiz)
        let questionIDs = Set(quiz.questions.map(\.id))
        XCTAssertEqual(events.count, 8)
        XCTAssertTrue(events.allSatisfy { questionIDs.contains($0.questionID) })
    }

    func testNonMathQuizPeerMessageRoundTrip() throws {
        let quiz = SampleData.japaneseGrammar
        let data = try JSONEncoder().encode(PeerMessage.quiz(quiz))
        let decoded = try JSONDecoder().decode(PeerMessage.self, from: data)
        guard case .quiz(let received) = decoded else { return XCTFail("Expected quiz") }
        XCTAssertEqual(received, quiz)
    }

    func testSessionAndAcknowledgmentRoundTrip() throws {
        let session = LearningSession(quiz: SampleData.englishGrammar)
        let messages: [PeerMessage] = [
            .session(session),
            .sessionEnded(session.id),
            .acknowledgment(UUID())
        ]

        for message in messages {
            let data = try JSONEncoder().encode(message)
            _ = try JSONDecoder().decode(PeerMessage.self, from: data)
        }
    }

    func testAnalysisEventCarriesSessionBoundaryWithoutRawAnswer() throws {
        let sessionID = UUID()
        let event = AnalysisEvent(
            participantToken: "P-TEST",
            sessionID: sessionID,
            questionID: "q-1",
            concept: "一次関数",
            misconception: "傾きと切片の混同",
            correct: false,
            hintCount: 2,
            retrySuccess: true
        )
        let data = try JSONEncoder().encode(event)
        let json = try XCTUnwrap(String(data: data, encoding: .utf8))
        let decoded = try JSONDecoder().decode(AnalysisEvent.self, from: data)

        XCTAssertEqual(decoded.sessionID, sessionID)
        XCTAssertFalse(json.contains("answer"))
    }

    func testSessionArchiveRoundTrip() throws {
        let session = LearningSession(quiz: SampleData.linearFunctions)
        let event = AnalysisEvent(
            participantToken: "P-TEST",
            sessionID: session.id,
            questionID: session.quiz.questions[0].id,
            concept: session.quiz.questions[0].concept,
            misconception: nil,
            correct: true,
            hintCount: 0,
            retrySuccess: false
        )
        let archive = SessionArchive(session: session, events: [event], adoptedPlan: nil)
        let decoded = try JSONDecoder().decode(
            SessionArchive.self,
            from: JSONEncoder().encode(archive)
        )

        XCTAssertEqual(decoded.session.id, session.id)
        XCTAssertEqual(decoded.events, [event])
    }
}
