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
    @Published private(set) var activeSession: LearningSession?
    @Published private(set) var sessionIsLive = false
    @Published private(set) var history: [SessionArchive] = []
    @Published private(set) var persistenceError: String?
    @Published private(set) var participantToken = AppModel.makeParticipantToken()

    private let historyLimit = 100
    private let historyURL: URL?
    private let recoveryURL: URL?

    init() {
        historyURL = Self.makeStorageURL(name: "session-history.json")
        recoveryURL = Self.makeStorageURL(name: "active-session-recovery.json")
        loadHistory()
        recoverInterruptedSession()
    }

    var summary: ClassSummary { ClassAnalytics.summarize(events) }
    var availableQuizzes: [Quiz] { SampleData.quizzes }

    func add(_ event: AnalysisEvent) {
        if let session = activeSession, event.sessionID == session.id {
            guard !events.contains(where: { $0.id == event.id }) else { return }
            events.append(event)
            saveRecoverySnapshot()
            return
        }

        // 終了直前の切断などで遅れて届いた結果も、元の授業履歴へ安全に戻す。
        guard let sessionID = event.sessionID,
              let index = history.firstIndex(where: { $0.session.id == sessionID }),
              !history[index].events.contains(where: { $0.id == event.id }) else { return }
        history[index].events.append(event)
        saveHistory()
    }

    func loadDemoEvents() {
        events = SampleData.demoEvents(for: activeQuiz)
    }

    func selectQuiz(_ quiz: Quiz) {
        guard quiz.id != activeQuiz.id else { return }
        activeQuiz = quiz
        resetSession()
    }

    @discardableResult
    func startSession() -> LearningSession {
        resetSession()
        let session = LearningSession(quiz: activeQuiz)
        activeSession = session
        sessionIsLive = true
        saveRecoverySnapshot()
        return session
    }

    func receiveSession(_ session: LearningSession) {
        guard activeSession?.id != session.id else { return }
        activeSession = session
        activeQuiz = session.quiz
        sessionIsLive = true
        participantToken = Self.makeParticipantToken()
    }

    /// 教師側では匿名集計を履歴へ保存し、生徒側では端末内の一時状態だけを閉じる。
    @discardableResult
    func endSession(archive: Bool) -> SessionArchive? {
        guard let session = activeSession else {
            sessionIsLive = false
            return nil
        }
        let saved = archive ? SessionArchive(
            session: session,
            events: events,
            adoptedPlan: adoptedPlan
        ) : nil
        if let saved {
            history.insert(saved, at: 0)
            if history.count > historyLimit {
                history.removeLast(history.count - historyLimit)
            }
            saveHistory()
        }
        activeSession = nil
        sessionIsLive = false
        removeRecoverySnapshot()
        return saved
    }

    func endReceivedSession(id: UUID) {
        guard activeSession?.id == id else { return }
        activeSession = nil
        sessionIsLive = false
    }

    func deleteArchive(id: UUID) {
        history.removeAll { $0.id == id }
        saveHistory()
    }

    func adoptPlan(_ plan: LessonPlan) {
        adoptedPlan = plan
        saveRecoverySnapshot()
    }

    func resetSession() {
        events = []
        adoptedPlan = nil
        activeSession = nil
        sessionIsLive = false
        removeRecoverySnapshot()
    }

    private static func makeParticipantToken() -> String {
        "P-\(String(UUID().uuidString.prefix(6)).uppercased())"
    }

    private static func makeStorageURL(name: String) -> URL? {
        guard let root = FileManager.default.urls(for: .applicationSupportDirectory, in: .userDomainMask).first else {
            return nil
        }
        return root.appendingPathComponent("Lumin", isDirectory: true)
            .appendingPathComponent(name)
    }

    private func loadHistory() {
        guard let historyURL, FileManager.default.fileExists(atPath: historyURL.path) else { return }
        do {
            let data = try Data(contentsOf: historyURL)
            history = try JSONDecoder().decode([SessionArchive].self, from: data)
                .sorted { $0.endedAt > $1.endedAt }
        } catch {
            persistenceError = "授業履歴を読み込めませんでした: \(error.localizedDescription)"
        }
    }

    private func saveHistory() {
        guard let historyURL else {
            persistenceError = "授業履歴の保存先を作成できませんでした。"
            return
        }
        do {
            try FileManager.default.createDirectory(
                at: historyURL.deletingLastPathComponent(),
                withIntermediateDirectories: true
            )
            let encoder = JSONEncoder()
            encoder.outputFormatting = [.prettyPrinted, .sortedKeys]
            try encoder.encode(history).write(to: historyURL, options: .atomic)
            persistenceError = nil
        } catch {
            persistenceError = "授業履歴を保存できませんでした: \(error.localizedDescription)"
        }
    }

    private func saveRecoverySnapshot() {
        guard role == .teacher, let session = activeSession, let recoveryURL else { return }
        do {
            try FileManager.default.createDirectory(
                at: recoveryURL.deletingLastPathComponent(),
                withIntermediateDirectories: true
            )
            let snapshot = SessionArchive(
                session: session,
                events: events,
                adoptedPlan: adoptedPlan
            )
            try JSONEncoder().encode(snapshot).write(to: recoveryURL, options: .atomic)
        } catch {
            persistenceError = "進行中の授業を保存できませんでした: \(error.localizedDescription)"
        }
    }

    private func recoverInterruptedSession() {
        guard let recoveryURL, FileManager.default.fileExists(atPath: recoveryURL.path) else { return }
        do {
            let recovered = try JSONDecoder().decode(
                SessionArchive.self,
                from: Data(contentsOf: recoveryURL)
            )
            if !history.contains(where: { $0.session.id == recovered.session.id }) {
                history.insert(recovered, at: 0)
                saveHistory()
            }
            try FileManager.default.removeItem(at: recoveryURL)
        } catch {
            persistenceError = "中断された授業を復元できませんでした: \(error.localizedDescription)"
        }
    }

    private func removeRecoverySnapshot() {
        guard let recoveryURL, FileManager.default.fileExists(atPath: recoveryURL.path) else { return }
        try? FileManager.default.removeItem(at: recoveryURL)
    }
}
