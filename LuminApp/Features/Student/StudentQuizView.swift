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
                    if hintCount > 0 { hintCard }
                    privacyFooter
                }
                .padding(28)
                .frame(maxWidth: 800)
                .frame(maxWidth: .infinity)
            }
        }
        .background(LuminTheme.canvas)
        .navigationBarBackButtonHidden()
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
            ProgressView(value: Double(questionIndex), total: Double(quiz.questions.count))
                .tint(LuminTheme.teal)
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
                    .font(.system(size: 28, weight: .bold, design: .rounded))
                    .foregroundStyle(LuminTheme.ink)

                TextField("答えを入力", text: $answer)
                    .font(.title3)
                    .textFieldStyle(.plain)
                    .padding(16)
                    .background(LuminTheme.canvas, in: RoundedRectangle(cornerRadius: 14))
                    .submitLabel(.done)
                    .onSubmit { Task { await submit() } }
                    .disabled(isAnalyzing)

                if feedback == .correct {
                    Label("その考え方で正解です", systemImage: "checkmark.circle.fill")
                        .font(.headline)
                        .foregroundStyle(LuminTheme.teal)
                } else if feedback == .tryAgain {
                    Label("まだ少し違うようです。ヒントを手がかりにもう一度。", systemImage: "arrow.triangle.2.circlepath")
                        .font(.subheadline.weight(.semibold))
                        .foregroundStyle(LuminTheme.coral)
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
                .opacity(answer.trimmingCharacters(in: .whitespacesAndNewlines).isEmpty && feedback != .correct ? 0.45 : 1)
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
                .font(.system(size: 34, weight: .bold, design: .rounded))
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
            feedback = .correct
        } else {
            feedback = .tryAgain
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
