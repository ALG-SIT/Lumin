import SwiftUI

@main
struct LuminApp: App {
    @Environment(\.scenePhase) private var scenePhase
    @StateObject private var model = AppModel()
    @StateObject private var peerService = LocalPeerService()
    @StateObject private var gemmaService = GemmaService()

    var body: some Scene {
        WindowGroup {
            RootView()
                .environmentObject(model)
                .environmentObject(peerService)
                .environmentObject(gemmaService)
                .task {
                    peerService.onSessionReceived = { session in
                        model.receiveSession(session)
                    }
                    peerService.onSessionEnded = { sessionID in
                        model.endReceivedSession(id: sessionID)
                    }
                    peerService.onQuizReceived = { quiz in
                        model.activeQuiz = quiz
                    }
                    peerService.onAnalysisReceived = { event in
                        model.add(event)
                    }
                    await gemmaService.prepare()
#if DEBUG
                    if ProcessInfo.processInfo.environment["LUMIN_AI_SMOKE_TEST"] == "1" {
                        do {
                            let summary = ClassAnalytics.summarize(SampleData.demoEvents)
                            let plan = try await gemmaService.generateLessonPlan(from: summary, quiz: SampleData.linearFunctions)
                            print("LUMIN_AI_SMOKE_OK backend=\(gemmaService.backendName) focus=\(plan.focus)")
                        } catch {
                            print("LUMIN_AI_SMOKE_FAILED \(error.localizedDescription)")
                        }
                    }
#endif
                }
                .onChange(of: scenePhase) { _, phase in
                    guard phase == .active else { return }
                    Task { await gemmaService.refreshAvailability() }
                }
        }
    }
}
