import SwiftUI

struct StudentQuizView: View {
    @EnvironmentObject private var model: AppModel
    @EnvironmentObject private var peerService: LocalPeerService
    @EnvironmentObject private var gemma: GemmaService

    let quiz: Quiz
    let sharesResults: Bool
    let onFinish: () -> Void

    @State private var questionIndex = 0
    @State private var answer = ""
    @State private var hintCount = 0
    @State private var firstMisconception: String?
    @State private var initialWasCorrect: Bool?
    @State private var feedback: Feedback?
    @State private var isComplete = false
    @State private var generatedHint: String?
    @State private var isAnalyzing = false
    @State private var lastIncorrectAnswer = ""
    @FocusState private var answerIsFocused: Bool
    @Environment(\.accessibilityReduceMotion) private var reduceMotion

    private let engine = RuleBasedLearningEngine()

    private enum Feedback: Equatable {
        case correct
        case tryAgain
    }

    private var question: QuizQuestion { quiz.questions[questionIndex] }

    var body: some View {
        ScrollView {
            if isComplete {
                completionView
            } else {
                VStack(alignment: .leading, spacing: 22) {
                    progressHeader
                    questionCard
                    if hintCount > 0 {
                        hintCard
                            .transition(.opacity.combined(with: reduceMotion ? .identity : .move(edge: .bottom)))
                    }
                    privacyFooter
                }
                .padding(.horizontal, 20)
                .padding(.vertical, 28)
                .frame(maxWidth: 800)
                .frame(maxWidth: .infinity)
            }
        }
        .background(LuminPageBackground())
        .navigationBarBackButtonHidden()
        .animation(.spring(response: 0.38, dampingFraction: 1), value: hintCount)
        .sensoryFeedback(.success, trigger: feedback == .correct)
        .sensoryFeedback(.warning, trigger: feedback == .tryAgain)
    }

    private var progressHeader: some View {
        VStack(alignment: .leading, spacing: 10) {
            HStack {
                Text(quiz.title)
                    .font(.headline)
                Spacer()
                Text("\(questionIndex + 1) / \(quiz.questions.count)")
                    .font(.subheadline.monospacedDigit().weight(.semibold))
            }
            ProgressView(value: Double(questionIndex + 1), total: Double(quiz.questions.count))
                .tint(LuminTheme.teal)
                .accessibilityLabel("小テストの進捗")
                .accessibilityValue("全\(quiz.questions.count)問中\(questionIndex + 1)問目")
        }
    }

    private var questionCard: some View {
        LuminCard {
            VStack(alignment: .leading, spacing: 20) {
                Text(question.concept)
                    .font(.caption.weight(.bold))
                    .foregroundStyle(LuminTheme.teal)
                    .textCase(.uppercase)
                Text(question.prompt)
                    .font(.system(.title, design: .rounded, weight: .bold))
                    .tracking(-0.35)
                    .foregroundStyle(LuminTheme.ink)

                TextField("答えを入力", text: $answer)
                    .font(.title3)
                    .textFieldStyle(.plain)
                    .padding(16)
                    .background(LuminTheme.canvas, in: RoundedRectangle(cornerRadius: 14))
                    .overlay {
                        RoundedRectangle(cornerRadius: 14, style: .continuous)
                            .stroke(answerIsFocused ? LuminTheme.teal.opacity(0.7) : Color.clear, lineWidth: 2)
                    }
                    .focused($answerIsFocused)
                    .submitLabel(.done)
                    .onSubmit { Task { await submit() } }
                    .disabled(isAnalyzing)

                if feedback == .correct {
                    Label("その考え方で正解です", systemImage: "checkmark.circle.fill")
                        .font(.headline)
                        .foregroundStyle(LuminTheme.teal)
                        .transition(.opacity.combined(with: .scale(scale: 0.97)))
                } else if feedback == .tryAgain {
                    Label("まだ少し違うようです。ヒントを手がかりにもう一度。", systemImage: "arrow.triangle.2.circlepath")
                        .font(.subheadline.weight(.semibold))
                        .foregroundStyle(LuminTheme.coral)
                        .transition(.opacity)
                }

                Button {
                    if feedback == .correct {
                        finishQuestion(retrySuccess: initialWasCorrect == false)
                    } else {
                        Task { await submit() }
                    }
                } label: {
                    if isAnalyzing {
                        HStack { ProgressView().tint(.white); Text("オンデバイスAIが分析中") }
                    } else {
                        Text(feedback == .correct ? nextButtonTitle : "答えを確かめる")
                    }
                }
                .buttonStyle(PrimaryButtonStyle())
                .disabled(isAnalyzing || (answer.trimmingCharacters(in: .whitespacesAndNewlines).isEmpty && feedback != .correct))
            }
        }
    }

    private var hintCard: some View {
        LuminCard {
            VStack(alignment: .leading, spacing: 14) {
                HStack {
                    Image(systemName: "lightbulb.fill")
                        .foregroundStyle(LuminTheme.amber)
                    Text("ヒント \(hintCount)")
                        .font(.headline)
                    Spacer()
                    Text("答えはまだ見せません")
                        .font(.caption)
                        .foregroundStyle(LuminTheme.muted)
                }
                Text(generatedHint ?? question.hints[hintCount - 1])
                    .font(.title3)
                    .foregroundStyle(LuminTheme.ink)

                if hintCount < question.hints.count {
                    Button("もう一つヒントを見る", systemImage: "plus.circle") {
                        Task { await requestNextHint() }
                    }
                    .font(.subheadline.weight(.semibold))
                    .disabled(isAnalyzing)
                } else {
                    Button("この問題を終えて次へ") {
                        finishQuestion(retrySuccess: false)
                    }
                    .font(.subheadline.weight(.semibold))
                    .foregroundStyle(LuminTheme.coral)
                }
            }
        }
    }

    private var privacyFooter: some View {
        HStack(spacing: 8) {
            Image(systemName: "iphone.and.arrow.forward")
            Text(gemma.state.isReady ? "\(gemma.backendName)で分析：解答は外部へ送信されません" : "ローカル分析中：解答は端末外へ送信されません")
        }
        .font(.caption)
        .foregroundStyle(LuminTheme.muted)
        .frame(maxWidth: .infinity)
    }

    private var completionView: some View {
        VStack(spacing: 22) {
            Image(systemName: "sun.max.fill")
                .font(.system(size: 54))
                .foregroundStyle(LuminTheme.amber)
            Text("おつかれさまでした")
                .font(.system(.largeTitle, design: .rounded, weight: .bold))
                .tracking(-0.6)
            Text(sharesResults
                ? "考え直した過程も、学びの大切な一部です。\n先生には匿名の分析結果だけが共有されました。"
                : "考え直した過程も、学びの大切な一部です。\n今回はデモのため、結果は共有されていません。")
                .multilineTextAlignment(.center)
                .foregroundStyle(LuminTheme.muted)
            Button("参加画面へ戻る", action: onFinish)
                .buttonStyle(PrimaryButtonStyle())
                .frame(maxWidth: 360)
        }
        .padding(36)
        .frame(maxWidth: .infinity, minHeight: 580)
    }

    private var nextButtonTitle: String {
        questionIndex == quiz.questions.count - 1 ? "結果を送って終了" : "次の問題へ"
    }

    private func submit() async {
        guard !isAnalyzing else { return }
        let submittedAnswer = answer
        let result = engine.analyze(answer: answer, for: question)
        if initialWasCorrect == nil {
            initialWasCorrect = result.isCorrect
            firstMisconception = result.misconception
        }
        if result.isCorrect {
            withAnimation(.spring(response: 0.32, dampingFraction: 1)) {
                feedback = .correct
            }
            answerIsFocused = false
        } else {
            withAnimation(.easeOut(duration: 0.2)) {
                feedback = .tryAgain
            }
            lastIncorrectAnswer = submittedAnswer
            if hintCount == 0 { hintCount = 1 }
            answer = ""
            guard gemma.state.isReady else { return }
            isAnalyzing = true
            defer { isAnalyzing = false }
            if let guidance = try? await gemma.analyzeIncorrectAnswer(submittedAnswer, question: question, hintLevel: hintCount) {
                firstMisconception = guidance.misconception
                generatedHint = guidance.hint
            }
        }
    }

    private func requestNextHint() async {
        guard hintCount < question.hints.count, !isAnalyzing else { return }
        hintCount += 1
        feedback = nil
        generatedHint = nil
        guard gemma.state.isReady else { return }
        isAnalyzing = true
        defer { isAnalyzing = false }
        if let guidance = try? await gemma.analyzeIncorrectAnswer(
            lastIncorrectAnswer,
            question: question,
            hintLevel: hintCount
        ) {
            generatedHint = guidance.hint
        }
    }

    private func finishQuestion(retrySuccess: Bool) {
        let event = AnalysisEvent(
            participantToken: model.participantToken,
            sessionID: model.activeSession?.id,
            questionID: question.id,
            concept: question.concept,
            misconception: firstMisconception,
            correct: initialWasCorrect ?? false,
            hintCount: hintCount,
            retrySuccess: retrySuccess
        )
        if sharesResults { peerService.sendAnalysis(event) }

        if questionIndex == quiz.questions.count - 1 {
            isComplete = true
        } else {
            questionIndex += 1
            answer = ""
            hintCount = 0
            firstMisconception = nil
            initialWasCorrect = nil
            feedback = nil
            generatedHint = nil
            lastIncorrectAnswer = ""
        }
    }
}
