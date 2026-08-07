import SwiftUI

struct TeacherDashboardView: View {
    @EnvironmentObject private var model: AppModel
    @EnvironmentObject private var peerService: LocalPeerService
    @State private var selection: Section? = .overview

    private enum Section: String, CaseIterable, Identifiable, Hashable {
        case overview = "理解の現在地"
        case lesson = "次の10分"
        case assistant = "AIと対話"
        case session = "小テスト配信"
        case history = "授業履歴"

        var id: String { rawValue }
        var icon: String {
            switch self {
            case .overview: "chart.bar.xaxis"
            case .lesson: "sparkles.rectangle.stack"
            case .assistant: "bubble.left.and.sparkles"
            case .session: "dot.radiowaves.left.and.right"
            case .history: "clock.arrow.circlepath"
            }
        }
    }

    var body: some View {
        NavigationSplitView {
            sidebar
        } detail: {
            selectedView
            .toolbar {
                ToolbarItem(placement: .automatic) {
                    Button("役割を選び直す", systemImage: "rectangle.portrait.and.arrow.right") {
                        leaveTeacherMode()
                    }
                }
            }
        }
        .onAppear { peerService.startHosting() }
        .onChange(of: peerService.connectedPeers) { _, peers in
            if model.sessionIsLive && !peers.isEmpty {
                if let session = model.activeSession { peerService.sendSession(session) }
            }
        }
        .navigationSplitViewStyle(.balanced)
    }

    private var sidebar: some View {
        VStack(alignment: .leading, spacing: 22) {
            HStack(spacing: 10) {
                Image(systemName: "sun.max.fill")
                    .foregroundStyle(LuminTheme.amber)
                Text("LUMIN")
                    .font(.headline.weight(.black))
                    .tracking(1.5)
            }
            .padding(.horizontal, 10)

            List(selection: $selection) {
                ForEach(Section.allCases) { section in
                    NavigationLink(value: section) {
                        Label(section.rawValue, systemImage: section.icon)
                            .foregroundStyle(selection == section ? LuminTheme.teal : LuminTheme.ink)
                    }
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
        .background(.regularMaterial)
        .navigationSplitViewColumnWidth(min: 220, ideal: 250)
    }

    @ViewBuilder
    private var selectedView: some View {
        switch selection {
        case .overview, nil: TeacherOverviewView()
        case .lesson: LessonPlanEditorView()
        case .assistant: TeacherAIChatView()
        case .session: SessionControlView()
        case .history: SessionHistoryView()
        }
    }

    private func leaveTeacherMode() {
        if let sessionID = model.activeSession?.id {
            peerService.sendSessionEnded(sessionID)
            model.endSession(archive: true)
        }
        peerService.stop()
        model.role = nil
    }
}

private struct TeacherAIChatView: View {
    private struct ChatMessage: Identifiable {
        enum Author: Equatable { case teacher, assistant }

        let id = UUID()
        let author: Author
        let text: String
    }

    @EnvironmentObject private var model: AppModel
    @EnvironmentObject private var gemma: GemmaService
    @State private var messages: [ChatMessage] = []
    @State private var draft = ""
    @State private var isSending = false
    @State private var errorMessage: String?
    @FocusState private var isInputFocused: Bool

    private let suggestions = [
        "今いちばん優先して扱うべきつまずきは？",
        "次の声かけを3つ提案して",
        "理解を確かめる問いを作って"
    ]

    var body: some View {
        VStack(spacing: 0) {
            header
            Divider()

            ScrollViewReader { proxy in
                ScrollView {
                    LazyVStack(spacing: 14) {
                        if messages.isEmpty { welcome }
                        ForEach(messages) { message in
                            messageBubble(message)
                                .id(message.id)
                        }
                        if isSending {
                            HStack(spacing: 8) {
                                ProgressView()
                                Text("学習状況を整理しています…")
                                    .foregroundStyle(LuminTheme.muted)
                                Spacer()
                            }
                            .padding(.horizontal, 4)
                        }
                    }
                    .padding(20)
                }
                .onChange(of: messages.count) { _, _ in
                    if let id = messages.last?.id {
                        withAnimation { proxy.scrollTo(id, anchor: .bottom) }
                    }
                }
            }

            Divider()
            composer
        }
        .background(LuminPageBackground())
        .navigationTitle("AIと対話")
        .alert("AIに質問できませんでした", isPresented: Binding(
            get: { errorMessage != nil },
            set: { if !$0 { errorMessage = nil } }
        )) {
            Button("OK", role: .cancel) {}
        } message: {
            Text(errorMessage ?? "")
        }
    }

    private var header: some View {
        VStack(alignment: .leading, spacing: 8) {
            HStack {
                VStack(alignment: .leading, spacing: 4) {
                    Text("学習状況について相談")
                        .font(.title2.bold())
                    Text("現在の小テストと匿名集計を文脈にして、オンデバイスAIが回答します。")
                        .font(.subheadline)
                        .foregroundStyle(LuminTheme.muted)
                }
                Spacer()
                Image(systemName: "lock.shield.fill")
                    .foregroundStyle(LuminTheme.teal)
            }

            HStack(spacing: 8) {
                StatusPill(title: "(model.summary.responseCount)回答", systemImage: "chart.bar.fill")
                StatusPill(title: gemma.state.title, systemImage: "cpu")
            }
        }
        .padding(20)
        .background(.regularMaterial)
    }

    private var welcome: some View {
        LuminCard {
            VStack(alignment: .leading, spacing: 14) {
                Label("何を相談しますか？", systemImage: "bubble.left.and.sparkles")
                    .font(.headline)
                    .foregroundStyle(LuminTheme.teal)
                Text(model.events.isEmpty
                     ? "回答はまだありません。教材の内容をもとに相談できます。"
                     : "正答率、再挑戦、ヒント利用、つまずきの傾向をもとに相談できます。")
                    .font(.subheadline)
                    .foregroundStyle(LuminTheme.muted)

                if !gemma.state.isReady {
                    GemmaStatusView()
                }

                ForEach(suggestions, id: \.self) { suggestion in
                    Button(suggestion) {
                        draft = suggestion
                        Task { await send() }
                    }
                    .buttonStyle(.bordered)
                    .disabled(isSending || !gemma.state.isReady)
                }
            }
        }
    }

    private func messageBubble(_ message: ChatMessage) -> some View {
        HStack {
            if message.author == .teacher { Spacer(minLength: 42) }
            Text(message.text)
                .font(.body)
                .textSelection(.enabled)
                .padding(14)
                .foregroundStyle(message.author == .teacher ? Color.white : LuminTheme.ink)
                .background(
                    message.author == .teacher ? LuminTheme.teal : LuminTheme.paper,
                    in: RoundedRectangle(cornerRadius: 17, style: .continuous)
                )
            if message.author == .assistant { Spacer(minLength: 42) }
        }
    }

    private var composer: some View {
        VStack(alignment: .leading, spacing: 8) {
            if !gemma.state.isReady && !isSending {
                Text("AIモデルを準備すると質問できます。AIモデル設定を確認してください。")
                    .font(.caption)
                    .foregroundStyle(LuminTheme.coral)
            }
            HStack(alignment: .bottom, spacing: 10) {
                TextField("学習状況について質問", text: $draft, axis: .vertical)
                    .lineLimit(1...5)
                    .textFieldStyle(.plain)
                    .padding(13)
                    .background(LuminTheme.canvas, in: RoundedRectangle(cornerRadius: 14))
                    .focused($isInputFocused)
                    .onSubmit { Task { await send() } }

                Button {
                    Task { await send() }
                } label: {
                    Image(systemName: "arrow.up")
                        .font(.headline.bold())
                        .foregroundStyle(.white)
                        .frame(width: 44, height: 44)
                        .background(LuminTheme.teal, in: Circle())
                }
                .disabled(trimmedDraft.isEmpty || isSending || !gemma.state.isReady)
                .opacity(trimmedDraft.isEmpty || isSending || !gemma.state.isReady ? 0.45 : 1)
                .accessibilityLabel("質問を送信")
            }
            Text("個別の解答本文や氏名はAIへ渡しません。提案は先生が確認して利用してください。")
                .font(.caption2)
                .foregroundStyle(LuminTheme.muted)
        }
        .padding(16)
        .background(.regularMaterial)
    }

    private var trimmedDraft: String {
        draft.trimmingCharacters(in: .whitespacesAndNewlines)
    }

    private func send() async {
        let question = trimmedDraft
        guard !question.isEmpty, !isSending, gemma.state.isReady else { return }

        let priorTurns = messages.suffix(6).map {
            TeacherChatTurn(
                role: $0.author == .teacher ? .teacher : .assistant,
                text: $0.text
            )
        }
        messages.append(ChatMessage(author: .teacher, text: question))
        draft = ""
        isInputFocused = false
        isSending = true
        defer { isSending = false }

        do {
            let answer = try await gemma.answerTeacherQuestion(
                question,
                summary: model.summary,
                quiz: model.activeQuiz,
                priorTurns: priorTurns
            )
            messages.append(ChatMessage(author: .assistant, text: answer))
        } catch {
            errorMessage = error.localizedDescription
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
        .background(LuminPageBackground())
        .navigationTitle("理解の現在地")
    }

    private var pageHeader: some View {
        HStack(alignment: .bottom) {
            VStack(alignment: .leading, spacing: 5) {
                Text(model.activeQuiz.title)
                    .font(.system(.largeTitle, design: .rounded, weight: .bold))
                    .tracking(-0.6)
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
        LazyVGrid(columns: [GridItem(.adaptive(minimum: 190), spacing: 14)], spacing: 14) {
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
                    .font(.system(.largeTitle, design: .rounded, weight: .bold))
                    .tracking(-0.6)

                LuminCard {
                    ViewThatFits(in: .horizontal) {
                        HStack(spacing: 16) {
                            quizPickerLabel
                            Spacer()
                            quizPickerMenu
                        }
                        VStack(alignment: .leading, spacing: 14) {
                            quizPickerLabel
                            quizPickerMenu
                        }
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
                            VStack(alignment: .trailing, spacing: 7) {
                                StatusPill(title: peerService.statusText, systemImage: "antenna.radiowaves.left.and.right")
                                if !peerService.joinCode.isEmpty {
                                    HStack(spacing: 7) {
                                        Text("参加コード")
                                            .font(.caption)
                                            .foregroundStyle(LuminTheme.muted)
                                        Text(peerService.joinCode)
                                            .font(.title3.bold().monospacedDigit())
                                            .textSelection(.enabled)
                                    }
                                    .accessibilityElement(children: .combine)
                                    .accessibilityLabel("参加コード \(peerService.joinCode)")
                                }
                            }
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
                            if let session = model.activeSession {
                                peerService.sendSessionEnded(session.id)
                                model.endSession(archive: true)
                            } else {
                                let session = model.startSession()
                                peerService.sendSession(session)
                            }
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
        .background(LuminPageBackground())
        .navigationTitle("小テスト配信")
    }

    private var quizPickerLabel: some View {
        HStack(spacing: 14) {
            LuminIconBadge(systemImage: "books.vertical.fill", size: 44)
            VStack(alignment: .leading, spacing: 3) {
                Text("教材を選択")
                    .font(.headline)
                Text("授業に合う小テストを選べます")
                    .font(.caption)
                    .foregroundStyle(LuminTheme.muted)
            }
        }
    }

    private var quizPickerMenu: some View {
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
            Label("教材を変更", systemImage: "chevron.up.chevron.down")
        }
        .buttonStyle(.bordered)
        .disabled(model.sessionIsLive)
    }
}

private struct SessionHistoryView: View {
    @EnvironmentObject private var model: AppModel
    @State private var pendingDeletion: SessionArchive?

    var body: some View {
        ScrollView {
            VStack(alignment: .leading, spacing: 22) {
                VStack(alignment: .leading, spacing: 5) {
                    Text("授業履歴")
                        .font(.system(.largeTitle, design: .rounded, weight: .bold))
                        .tracking(-0.6)
                    Text("保存するのは匿名の分析結果と採用した授業案だけです。最新100授業を端末内に保持します。")
                        .foregroundStyle(LuminTheme.muted)
                }

                if let error = model.persistenceError {
                    Label(error, systemImage: "exclamationmark.triangle.fill")
                        .font(.subheadline)
                        .foregroundStyle(LuminTheme.coral)
                }

                if model.history.isEmpty {
                    LuminCard {
                        ContentUnavailableView(
                            "履歴はまだありません",
                            systemImage: "clock.arrow.circlepath",
                            description: Text("小テストの配信を終了すると、匿名集計がこの端末に保存されます。")
                        )
                    }
                } else {
                    ForEach(model.history) { archive in
                        historyCard(archive)
                    }
                }
            }
            .padding(28)
            .frame(maxWidth: 1000)
            .frame(maxWidth: .infinity, alignment: .top)
        }
        .background(LuminPageBackground())
        .navigationTitle("授業履歴")
        .confirmationDialog(
            "この授業履歴を削除しますか？",
            isPresented: Binding(
                get: { pendingDeletion != nil },
                set: { if !$0 { pendingDeletion = nil } }
            ),
            titleVisibility: .visible
        ) {
            Button("端末から削除", role: .destructive) {
                if let id = pendingDeletion?.id { model.deleteArchive(id: id) }
                pendingDeletion = nil
            }
            Button("キャンセル", role: .cancel) { pendingDeletion = nil }
        } message: {
            Text("この操作は取り消せません。生徒の解答本文は元から保存されていません。")
        }
    }

    private func historyCard(_ archive: SessionArchive) -> some View {
        let summary = ClassAnalytics.summarize(archive.events)
        return LuminCard {
            VStack(alignment: .leading, spacing: 14) {
                HStack(alignment: .top) {
                    VStack(alignment: .leading, spacing: 4) {
                        Text(archive.session.quiz.subject)
                            .font(.caption.weight(.bold))
                            .foregroundStyle(LuminTheme.teal)
                        Text(archive.session.quiz.title)
                            .font(.title3.bold())
                        Text(archive.endedAt.formatted(date: .abbreviated, time: .shortened))
                            .font(.caption)
                            .foregroundStyle(LuminTheme.muted)
                    }
                    Spacer()
                    Button("削除", systemImage: "trash", role: .destructive) {
                        pendingDeletion = archive
                    }
                    .labelStyle(.iconOnly)
                    .foregroundStyle(LuminTheme.coral)
                }

                Divider()
                HStack(spacing: 24) {
                    historyMetric("\(summary.participantCount)", "参加端末")
                    historyMetric("\(summary.responseCount)", "回答")
                    historyMetric(summary.correctRate.formatted(.percent.precision(.fractionLength(0))), "初回正答率")
                    historyMetric(summary.retrySuccessRate.formatted(.percent.precision(.fractionLength(0))), "再挑戦成功")
                }

                if let top = summary.misconceptions.first {
                    Label("最多のつまずき：\(top.name)（\(top.count)件）", systemImage: "lightbulb.max.fill")
                        .font(.subheadline.weight(.semibold))
                        .foregroundStyle(LuminTheme.ink)
                }
                if let plan = archive.adoptedPlan {
                    Label("採用した案：\(plan.focus)", systemImage: "checkmark.seal.fill")
                        .font(.subheadline)
                        .foregroundStyle(LuminTheme.teal)
                }
            }
        }
    }

    private func historyMetric(_ value: String, _ label: String) -> some View {
        VStack(alignment: .leading, spacing: 2) {
            Text(value).font(.title3.bold()).monospacedDigit()
            Text(label).font(.caption).foregroundStyle(LuminTheme.muted)
        }
    }
}
