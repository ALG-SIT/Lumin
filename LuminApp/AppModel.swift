import Foundation

@MainActor
final class AppModel: ObservableObject {
    enum Role: String {
        case teacher
        case student
    }

    @Published var role: Role?
    @Published var activeQuiz = SampleData.linearFunctions
    @Published private(set) var events: [AnalysisEvent] = []
    @Published var adoptedPlan: LessonPlan?
    @Published var sessionIsLive = false

    let participantToken = "P-\(String(UUID().uuidString.prefix(4)).uppercased())"

    var summary: ClassSummary { ClassAnalytics.summarize(events) }
    var availableQuizzes: [Quiz] { SampleData.quizzes }

    func add(_ event: AnalysisEvent) {
        guard !events.contains(where: { $0.id == event.id }) else { return }
        events.append(event)
    }

    func loadDemoEvents() {
        events = SampleData.demoEvents(for: activeQuiz)
    }

    func selectQuiz(_ quiz: Quiz) {
        guard quiz.id != activeQuiz.id else { return }
        activeQuiz = quiz
        resetSession()
    }

    func resetSession() {
        events = []
        adoptedPlan = nil
        sessionIsLive = false
    }
}
