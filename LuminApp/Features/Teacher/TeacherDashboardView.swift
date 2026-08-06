import SwiftUI

struct TeacherDashboardView: View {
    @EnvironmentObject private var model: AppModel
    @EnvironmentObject private var peerService: LocalPeerService
    @State private var selection: Section = .overview

    private enum Section: String, CaseIterable, Identifiable {
        case overview = "理解の現在地"
        case lesson = "次の10分"
        case session = "小テスト配信"

        var id: String { rawValue }
        var icon: String {
            switch self {
            case .overview: "chart.bar.xaxis"
            case .lesson: "sparkles.rectangle.stack"
            case .session: "dot.radiowaves.left.and.right"
            }
        }
    }

    var body: some View {
        NavigationSplitView {
            VStack(alignment: .leading, spacing: 22) {
                HStack(spacing: 10) {
                    Image(systemName: "sun.max.fill")
                        .foregroundStyle(LuminTheme.amber)
                    Text("LUMIN")
                        .font(.headline.weight(.black))
                        .tracking(1.5)
                }
                .padding(.horizontal, 10)

                List {
                    ForEach(Section.allCases) { section in
                        Button {
                            selection = section
                        } label: {
                            Label(section.rawValue, systemImage: section.icon)
                                .foregroundStyle(selection == section ? LuminTheme.teal : LuminTheme.ink)
                        }
                        .buttonStyle(.plain)
                        .listRowBackground(selection == section ? LuminTheme.tealSoft : Color.clear)
                    }
                }
                .scrollContentBackground(.hidden)
                .listStyle(.sidebar)

                VStack(alignment: .leading, spacing: 8) {
                    StatusPill(
                        title: peerService.statusText,
                        systemImage: "antenna.radiowaves.left.and.right"
                    )
                    Text("生徒の解答本文は受信しません")
                        .font(.caption2)
                        .foregroundStyle(LuminTheme.muted)
                }
                .padding(10)
            }
            .padding(.vertical, 18)
            .background(LuminTheme.canvas)
            .navigationSplitViewColumnWidth(min: 220, ideal: 250)
        } detail: {
            Group {
                switch selection {
                case .overview: TeacherOverviewView()
                case .lesson: LessonPlanEditorView()
                case .session: SessionControlView()
                }
            }
            .toolbar {
                ToolbarItem(placement: .automatic) {
                    Button("役割を選び直す", systemImage: "rectangle.portrait.and.arrow.right") {
                        peerService.stop()
                        model.role = nil
                    }
                }
            }
        }
        .onAppear { peerService.startHosting() }
        .onChange(of: peerService.connectedPeers) { _, peers in
            if model.sessionIsLive && !peers.isEmpty {
                peerService.sendQuiz(model.activeQuiz)
            }
        }
    }
}

private struct TeacherOverviewView: View {
    @EnvironmentObject private var model: AppModel

    private var summary: ClassSummary { model.summary }

    var body: some View {
        ScrollView {
            VStack(alignment: .leading, spacing: 22) {
                pageHeader

                if model.events.isEmpty {
                    emptyState
                } else {
                    metricGrid
                    misconceptionChart
                    recentSignals
                }
            }
            .padding(28)
            .frame(maxWidth: 1100)
            .frame(maxWidth: .infinity, alignment: .top)
        }
        .background(LuminTheme.canvas)
        .navigationTitle("理解の現在地")
    }

    private var pageHeader: some View {
        HStack(alignment: .bottom) {
            VStack(alignment: .leading, spacing: 5) {
                Text(model.activeQuiz.title)
                    .font(.system(size: 30, weight: .bold, design: .rounded))
                Text("正答率ではなく、なぜ迷ったかを見ます。")
                    .foregroundStyle(LuminTheme.muted)
            }
            Spacer()
            if model.sessionIsLive {
                StatusPill(title: "LIVE", systemImage: "circle.fill", color: LuminTheme.coral)
            }
        }
    }

    private var metricGrid: some View {
        HStack(spacing: 14) {
            metricCard(value: "\(summary.participantCount)", label: "参加端末", note: "匿名トークン")
            metricCard(value: summary.correctRate.formatted(.percent.precision(.fractionLength(0))), label: "初回正答率", note: "全\(summary.responseCount)回答")
            metricCard(value: summary.retrySuccessRate.formatted(.percent.precision(.fractionLength(0))), label: "再挑戦成功", note: "ヒント利用後")
            metricCard(value: summary.averageHints.formatted(.number.precision(.fractionLength(1))), label: "平均ヒント", note: "1回答あたり")
        }
    }

    private func metricCard(value: String, label: String, note: String) -> some View {
        LuminCard {
            VStack(alignment: .leading, spacing: 7) {
                Text(value)
                    .font(.system(size: 32, weight: .bold, design: .rounded))
                    .foregroundStyle(LuminTheme.ink)
                Text(label)
                    .font(.subheadline.weight(.semibold))
                Text(note)
                    .font(.caption)
                    .foregroundStyle(LuminTheme.muted)
            }
        }
    }

    private var misconceptionChart: some View {
        LuminCard {
            VStack(alignment: .leading, spacing: 18) {
                HStack {
                    VStack(alignment: .leading, spacing: 3) {
                        Text("つまずきの傾向")
                            .font(.title3.bold())
                        Text("受信した最小化データから集計")
                            .font(.caption)
                            .foregroundStyle(LuminTheme.muted)
                    }
                    Spacer()
                    Image(systemName: "lock.shield.fill")
                        .foregroundStyle(LuminTheme.teal)
                }

                ForEach(summary.misconceptions.prefix(5)) { item in
                    VStack(alignment: .leading, spacing: 7) {
                        HStack {
                            Text(item.name)
                                .font(.subheadline.weight(.semibold))
                            Spacer()
                            Text("\(item.count)件 · \(item.share.formatted(.percent.precision(.fractionLength(0))))")
                                .font(.caption.monospacedDigit())
                                .foregroundStyle(LuminTheme.muted)
                        }
                        GeometryReader { geometry in
                            ZStack(alignment: .leading) {
                                Capsule().fill(LuminTheme.canvas)
                                Capsule().fill(item == summary.misconceptions.first ? LuminTheme.coral : LuminTheme.amber)
                                    .frame(width: max(8, geometry.size.width * item.share))
                            }
                        }
                        .frame(height: 10)
                    }
                }
            }
        }
    }

    private var recentSignals: some View {
        LuminCard {
            VStack(alignment: .leading, spacing: 14) {
                Text("最近のシグナル")
                    .font(.title3.bold())
                ForEach(model.events.suffix(5).reversed()) { event in
                    HStack(spacing: 12) {
                        Image(systemName: event.correct ? "checkmark.circle.fill" : "exclamationmark.circle.fill")
                            .foregroundStyle(event.correct ? LuminTheme.teal : LuminTheme.coral)
                        VStack(alignment: .leading, spacing: 2) {
                            Text(event.concept)
                                .font(.subheadline.weight(.semibold))
                            Text(event.misconception ?? "初回で理解")
                                .font(.caption)
                                .foregroundStyle(LuminTheme.muted)
                        }
                        Spacer()
                        Text(event.participantToken)
                            .font(.caption.monospaced())
                            .foregroundStyle(LuminTheme.muted)
                    }
                    if event.id != model.events.suffix(5).first?.id { Divider() }
                }
            }
        }
    }

    private var emptyState: some View {
        LuminCard {
            VStack(spacing: 18) {
                Image(systemName: "chart.bar.doc.horizontal")
                    .font(.system(size: 42))
                    .foregroundStyle(LuminTheme.teal)
                Text("回答を待っています")
                    .font(.title2.bold())
                Text("生徒が回答すると、解答本文を含まない分析結果だけがここに届きます。")
                    .foregroundStyle(LuminTheme.muted)
                    .multilineTextAlignment(.center)
                Button("大会デモ用データを読み込む") { model.loadDemoEvents() }
                    .buttonStyle(.borderedProminent)
            }
            .frame(maxWidth: .infinity, minHeight: 300)
        }
    }
}

private struct SessionControlView: View {
    @EnvironmentObject private var model: AppModel
    @EnvironmentObject private var peerService: LocalPeerService

    var body: some View {
        ScrollView {
            VStack(alignment: .leading, spacing: 22) {
                Text("小テスト配信")
                    .font(.system(size: 30, weight: .bold, design: .rounded))

                LuminCard {
                    HStack(spacing: 16) {
                        Image(systemName: "books.vertical.fill")
                            .font(.title2)
                            .foregroundStyle(LuminTheme.teal)
                            .frame(width: 42, height: 42)
                            .background(LuminTheme.tealSoft, in: RoundedRectangle(cornerRadius: 12))
                        VStack(alignment: .leading, spacing: 3) {
                            Text("教材を選択")
                                .font(.headline)
                            Text("5教科・6セットから授業に合う小テストを選べます")
                                .font(.caption)
                                .foregroundStyle(LuminTheme.muted)
                        }
                        Spacer()
                        Menu {
                            ForEach(SampleData.subjects, id: \.self) { subject in
                                Section(subject) {
                                    ForEach(model.availableQuizzes.filter { $0.subject == subject }) { quiz in
                                        Button {
                                            model.selectQuiz(quiz)
                                        } label: {
                                            if quiz.id == model.activeQuiz.id {
                                                Label(quiz.title, systemImage: "checkmark")
                                            } else {
                                                Text(quiz.title)
                                            }
                                        }
                                    }
                                }
                            }
                        } label: {
                            Label("変更", systemImage: "chevron.up.chevron.down")
                        }
                        .buttonStyle(.bordered)
                        .disabled(model.sessionIsLive)
                    }
                }

                LuminCard {
                    VStack(alignment: .leading, spacing: 18) {
                        HStack {
                            VStack(alignment: .leading, spacing: 4) {
                                Text(model.activeQuiz.subject)
                                    .font(.caption.weight(.bold))
                                    .foregroundStyle(LuminTheme.teal)
                                Text(model.activeQuiz.title)
                                    .font(.title2.bold())
                                Text("\(model.activeQuiz.questions.count)問 · \(model.activeQuiz.topic ?? model.activeQuiz.title)")
                                    .foregroundStyle(LuminTheme.muted)
                            }
                            Spacer()
                            StatusPill(title: peerService.statusText, systemImage: "antenna.radiowaves.left.and.right")
                        }

                        Divider()

                        ForEach(Array(model.activeQuiz.questions.enumerated()), id: \.element.id) { index, question in
                            HStack(alignment: .top, spacing: 12) {
                                Text("\(index + 1)")
                                    .font(.caption.bold())
                                    .foregroundStyle(.white)
                                    .frame(width: 26, height: 26)
                                    .background(LuminTheme.teal, in: Circle())
                                VStack(alignment: .leading, spacing: 3) {
                                    Text(question.prompt)
                                        .font(.subheadline.weight(.semibold))
                                    Text(question.concept)
                                        .font(.caption)
                                        .foregroundStyle(LuminTheme.muted)
                                }
                            }
                        }

                        Button(model.sessionIsLive ? "配信を終了" : "この小テストを配信") {
                            model.sessionIsLive.toggle()
                            if model.sessionIsLive { peerService.sendQuiz(model.activeQuiz) }
                        }
                        .buttonStyle(PrimaryButtonStyle())
                    }
                }

                LuminCard {
                    HStack(alignment: .top, spacing: 14) {
                        Image(systemName: "wifi.router.fill")
                            .font(.title2)
                            .foregroundStyle(LuminTheme.teal)
                        VStack(alignment: .leading, spacing: 5) {
                            Text("教室内だけで接続")
                                .font(.headline)
                            Text("同じWi‑Fiまたは近距離のApple端末を自動検出し、暗号化された通信で接続します。インターネット接続は使いません。")
                                .font(.subheadline)
                                .foregroundStyle(LuminTheme.muted)
                        }
                    }
                }
            }
            .padding(28)
            .frame(maxWidth: 1000)
            .frame(maxWidth: .infinity, alignment: .top)
        }
        .background(LuminTheme.canvas)
        .navigationTitle("小テスト配信")
    }
}
