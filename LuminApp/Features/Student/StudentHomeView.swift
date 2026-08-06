import SwiftUI

struct StudentHomeView: View {
    @EnvironmentObject private var model: AppModel
    @EnvironmentObject private var peerService: LocalPeerService
    @State private var hasStarted = false

    var body: some View {
        NavigationStack {
            Group {
                if hasStarted {
                    StudentQuizView(quiz: model.activeQuiz) {
                        hasStarted = false
                    }
                } else {
                    joinView
                }
            }
            .toolbar {
                ToolbarItem(placement: .automatic) {
                    Button("役割を選び直す", systemImage: "chevron.left") {
                        peerService.stop()
                        model.role = nil
                    }
                }
            }
        }
        .onAppear { peerService.startBrowsing() }
    }

    private var joinView: some View {
        ScrollView {
            VStack(alignment: .leading, spacing: 24) {
                VStack(alignment: .leading, spacing: 8) {
                    Text("こんにちは")
                        .font(.caption.weight(.bold))
                        .foregroundStyle(LuminTheme.teal)
                    Text("今日の理解を、\n一緒に見つけよう。")
                        .font(.system(size: 36, weight: .bold, design: .rounded))
                        .foregroundStyle(LuminTheme.ink)
                    Text("あなたの解答本文やヒントの会話は、この端末の中だけで処理されます。")
                        .foregroundStyle(LuminTheme.muted)
                }

                LuminCard {
                    VStack(alignment: .leading, spacing: 16) {
                        HStack {
                            StatusPill(title: peerService.statusText, systemImage: peerService.connectedPeers.isEmpty ? "antenna.radiowaves.left.and.right" : "checkmark.circle.fill")
                            Spacer()
                            Text("参加ID \(model.participantToken)")
                                .font(.caption.monospaced())
                                .foregroundStyle(LuminTheme.muted)
                        }

                        if !peerService.nearbyTeachers.isEmpty && peerService.connectedPeers.isEmpty {
                            Text("見つかった教室")
                                .font(.headline)
                            ForEach(peerService.nearbyTeachers, id: \.self) { teacher in
                                Button {
                                    peerService.connect(to: teacher)
                                } label: {
                                    HStack {
                                        Image(systemName: "person.crop.rectangle.stack.fill")
                                        Text(teacher.displayName)
                                        Spacer()
                                        Text("接続")
                                    }
                                }
                                .buttonStyle(.bordered)
                            }
                        }

                        Divider()

                        VStack(alignment: .leading, spacing: 5) {
                            Text(model.activeQuiz.subject)
                                .font(.caption.weight(.semibold))
                                .foregroundStyle(LuminTheme.teal)
                            Text(model.activeQuiz.title)
                                .font(.title2.bold())
                            Text("全\(model.activeQuiz.questions.count)問・目安5分")
                                .foregroundStyle(LuminTheme.muted)
                        }

                        Button(peerService.connectedPeers.isEmpty ? "デモとして始める" : "小テストを始める") {
                            hasStarted = true
                        }
                        .buttonStyle(PrimaryButtonStyle())
                    }
                }

                Label("教師へ共有されるのは、正誤・誤概念・ヒント回数・再回答結果だけです。", systemImage: "lock.shield.fill")
                    .font(.footnote)
                    .foregroundStyle(LuminTheme.muted)
                    .padding(.horizontal, 8)

                GemmaStatusView()
            }
            .padding(28)
            .frame(maxWidth: 760)
            .frame(maxWidth: .infinity)
        }
        .background(LuminTheme.canvas)
    }
}
