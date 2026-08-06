import SwiftUI

struct LessonPlanEditorView: View {
    @EnvironmentObject private var model: AppModel
    @EnvironmentObject private var gemma: GemmaService
    @State private var draft = LessonPlanGenerator.generate(from: ClassAnalytics.summarize([]))
    @State private var hasLoaded = false
    @State private var showSaved = false
    @State private var generationError: String?
    @State private var isGenerating = false

    var body: some View {
        ScrollView {
            VStack(alignment: .leading, spacing: 22) {
                HStack(alignment: .bottom) {
                    VStack(alignment: .leading, spacing: 5) {
                        Text("次の10分")
                            .font(.system(size: 30, weight: .bold, design: .rounded))
                        Text("AIの初期案を、先生の判断で仕上げます。")
                            .foregroundStyle(LuminTheme.muted)
                    }
                    Spacer()
                    Button {
                        Task { await regenerate() }
                    } label: {
                        if isGenerating {
                            Label("AIが生成中", systemImage: "sparkles")
                        } else {
                            Label("オンデバイスAIで再生成", systemImage: "sparkles")
                        }
                    }
                    .buttonStyle(.bordered)
                    .disabled(isGenerating)
                }

                if model.events.isEmpty {
                    Label("回答がまだないため、基本テンプレートを表示しています。", systemImage: "info.circle")
                        .font(.subheadline)
                        .foregroundStyle(LuminTheme.muted)
                }

                GemmaStatusView()

                LuminCard {
                    VStack(alignment: .leading, spacing: 18) {
                        Label("Luminの提案", systemImage: "sparkles")
                            .font(.caption.weight(.bold))
                            .foregroundStyle(LuminTheme.teal)

                        TextField("授業の焦点", text: $draft.focus)
                            .font(.title2.bold())
                            .textFieldStyle(.plain)

                        Text(draft.teacherNote)
                            .font(.subheadline)
                            .foregroundStyle(LuminTheme.muted)
                            .padding(14)
                            .frame(maxWidth: .infinity, alignment: .leading)
                            .background(LuminTheme.tealSoft, in: RoundedRectangle(cornerRadius: 14))

                        Text("進め方")
                            .font(.headline)

                        ForEach(draft.steps.indices, id: \.self) { index in
                            HStack(alignment: .top, spacing: 12) {
                                Text("\(index + 1)")
                                    .font(.caption.bold())
                                    .foregroundStyle(.white)
                                    .frame(width: 26, height: 26)
                                    .background(LuminTheme.teal, in: Circle())
                                TextField("ステップ", text: $draft.steps[index], axis: .vertical)
                                    .textFieldStyle(.plain)
                            }
                            .padding(.vertical, 5)
                        }

                        Divider()

                        Text("確認問題")
                            .font(.headline)
                        TextField("確認問題", text: $draft.checkQuestion, axis: .vertical)
                            .textFieldStyle(.plain)
                            .padding(14)
                            .background(LuminTheme.canvas, in: RoundedRectangle(cornerRadius: 14))
                    }
                }

                HStack(spacing: 14) {
                    Button("この案を採用") {
                        model.adoptPlan(draft)
                        showSaved = true
                    }
                    .buttonStyle(PrimaryButtonStyle())

                    if showSaved {
                        Label("先生の修正版として保存しました", systemImage: "checkmark.circle.fill")
                            .font(.subheadline.weight(.semibold))
                            .foregroundStyle(LuminTheme.teal)
                            .transition(.opacity)
                    }
                }
            }
            .padding(28)
            .frame(maxWidth: 1000)
            .frame(maxWidth: .infinity, alignment: .top)
        }
        .background(LuminTheme.canvas)
        .navigationTitle("次の10分")
        .onAppear {
            guard !hasLoaded else { return }
            draft = model.adoptedPlan ?? LessonPlanGenerator.generate(from: model.summary, quiz: model.activeQuiz)
            hasLoaded = true
        }
        .alert("授業案を生成できませんでした", isPresented: Binding(
            get: { generationError != nil },
            set: { if !$0 { generationError = nil } }
        )) {
            Button("OK", role: .cancel) {}
        } message: {
            Text(generationError ?? "")
        }
    }

    private func regenerate() async {
        guard gemma.state.isReady else {
            draft = LessonPlanGenerator.generate(from: model.summary, quiz: model.activeQuiz)
            generationError = "AIモデルが未準備のため、ローカルテンプレートを使用しました。"
            return
        }
        isGenerating = true
        defer { isGenerating = false }
        do {
            draft = try await gemma.generateLessonPlan(from: model.summary, quiz: model.activeQuiz)
        } catch {
            draft = LessonPlanGenerator.generate(from: model.summary, quiz: model.activeQuiz)
            generationError = "\(error.localizedDescription)\nローカルテンプレートへ切り替えました。"
        }
    }
}
