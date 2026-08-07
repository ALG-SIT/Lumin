import Foundation
import Combine
import LiteRTLM

#if canImport(FoundationModels)
import FoundationModels
#endif

struct GemmaGuidance: Sendable {
    let misconception: String
    let hint: String
}

struct TeacherChatTurn: Sendable {
    enum Role: Sendable, Equatable {
        case teacher
        case assistant
    }

    let role: Role
    let text: String
}

@MainActor
final class GemmaService: ObservableObject {
    enum State: Equatable {
        case notInstalled
        case downloading(String)
        case loading
        case ready(modelName: String)
        case generating
        case failed(String)

        var title: String {
            switch self {
            case .notInstalled: "AIモデルを選択してください"
            case .downloading(let name): "\(name)をダウンロード中"
            case .loading: "AIモデルを読み込み中"
            case .ready: "オンデバイスAI"
            case .generating: "AIが考えています"
            case .failed: "AIモデルを開始できません"
            }
        }

        var isReady: Bool {
            if case .ready = self { return true }
            return false
        }
    }

    private enum ActiveBackend: Equatable {
        case appleFoundation
        case gemma
    }

    @Published private(set) var state: State = .notInstalled
    @Published private(set) var lastLatency: TimeInterval?
    @Published private(set) var backendName = "未選択"
    @Published private(set) var selectedProvider: AIProviderChoice
    @Published private(set) var selectedModelID: String
    @Published private(set) var appleAvailabilityText = "確認中"
    @Published private(set) var isAppleFoundationAvailable = false
    @Published private(set) var downloadingModelID: String?
    @Published private(set) var downloadProgress = 0.0

    let modelCatalog = GemmaModelOption.catalog

    private var engine: Engine?
    private var activeBackend: ActiveBackend?
    private var downloader: ModelDownloadClient?
    private let defaults = UserDefaults.standard

    init() {
        selectedProvider = AIProviderChoice(
            rawValue: UserDefaults.standard.string(forKey: "lumin.ai.provider") ?? "automatic"
        ) ?? .automatic
        selectedModelID = UserDefaults.standard.string(forKey: "lumin.ai.gemma-model")
            ?? GemmaModelOption.recommended.id
    }

    var recommendedModelID: String { GemmaModelOption.recommended.id }

    var isUsingAppleFoundation: Bool { activeBackend == .appleFoundation }

    var selectedModelTitle: String {
        if selectedModelID == "custom" { return "読み込んだカスタムモデル" }
        return modelCatalog.first(where: { $0.id == selectedModelID })?.title ?? "Gemma"
    }

    var physicalMemoryDescription: String {
        let bytes = Int64(ProcessInfo.processInfo.physicalMemory)
        return ByteCountFormatter.string(fromByteCount: bytes, countStyle: .memory)
    }

    func prepare() async {
        refreshAppleAvailability()
        await activateSelection()
    }

    func refreshAvailability() async {
        let wasAvailable = isAppleFoundationAvailable
        refreshAppleAvailability()
        guard wasAvailable != isAppleFoundationAvailable,
              selectedProvider != .gemma else { return }
        engine = nil
        activeBackend = nil
        await activateSelection()
    }

    func selectProvider(_ provider: AIProviderChoice) async {
        selectedProvider = provider
        defaults.set(provider.rawValue, forKey: "lumin.ai.provider")
        engine = nil
        activeBackend = nil
        refreshAppleAvailability()
        await activateSelection()
    }

    func selectModel(_ option: GemmaModelOption) async {
        selectedModelID = option.id
        defaults.set(option.id, forKey: "lumin.ai.gemma-model")
        if selectedProvider != .gemma {
            selectedProvider = .gemma
            defaults.set(AIProviderChoice.gemma.rawValue, forKey: "lumin.ai.provider")
        }
        engine = nil
        activeBackend = nil
        await activateGemmaSelection()
    }

    func isModelInstalled(_ option: GemmaModelOption) -> Bool {
        guard let url = try? modelURL(for: option) else { return false }
        if FileManager.default.fileExists(atPath: url.path) { return true }
        return bundledURL(for: option) != nil
    }

    func downloadModel(_ option: GemmaModelOption, token: String) async throws {
        let directory = try modelDirectory()
        let required = Int64(Double(option.downloadBytes) * 1.15)
        let values = try directory.resourceValues(forKeys: [.volumeAvailableCapacityForImportantUsageKey])
        if let available = values.volumeAvailableCapacityForImportantUsage, available < required {
            throw ModelDownloadError.insufficientStorage(required: required)
        }

        try HuggingFaceCredentialStore.save(token)
        var request = URLRequest(url: option.downloadURL)
        request.timeoutInterval = 120
        let normalizedToken = token.trimmingCharacters(in: .whitespacesAndNewlines)
        if !normalizedToken.isEmpty {
            request.setValue("Bearer \(normalizedToken)", forHTTPHeaderField: "Authorization")
        }

        downloadingModelID = option.id
        downloadProgress = 0
        state = .downloading(option.title)
        let stagingURL = directory.appendingPathComponent(".\(option.fileName).download")
        let client = ModelDownloadClient { [weak self] progress in
            Task { @MainActor in self?.downloadProgress = progress }
        }
        downloader = client
        defer {
            downloader = nil
            downloadingModelID = nil
        }

        do {
            _ = try await client.download(request, to: stagingURL)
            let attributes = try FileManager.default.attributesOfItem(atPath: stagingURL.path)
            let actualSize = (attributes[.size] as? NSNumber)?.int64Value ?? 0
            guard actualSize > option.downloadBytes / 2 else {
                try? FileManager.default.removeItem(at: stagingURL)
                throw ModelDownloadError.httpStatus(-1)
            }
            let destination = try modelURL(for: option)
            engine = nil
            if FileManager.default.fileExists(atPath: destination.path) {
                try FileManager.default.removeItem(at: destination)
            }
            try FileManager.default.moveItem(at: stagingURL, to: destination)
            downloadProgress = 1
            await selectModel(option)
        } catch {
            try? FileManager.default.removeItem(at: stagingURL)
            state = .failed(error.localizedDescription)
            throw error
        }
    }

    func cancelDownload() {
        downloader?.cancel()
    }

    func savedHuggingFaceToken() -> String {
        HuggingFaceCredentialStore.load()
    }

    func importModel(from sourceURL: URL) async throws {
        guard sourceURL.pathExtension.lowercased() == "litertlm" else {
            throw GemmaSetupError.unsupportedFile
        }
        let accessing = sourceURL.startAccessingSecurityScopedResource()
        defer { if accessing { sourceURL.stopAccessingSecurityScopedResource() } }

        let destination = try modelDirectory().appendingPathComponent("custom-model.litertlm")
        if FileManager.default.fileExists(atPath: destination.path) {
            try FileManager.default.removeItem(at: destination)
        }
        try FileManager.default.copyItem(at: sourceURL, to: destination)
        selectedModelID = "custom"
        selectedProvider = .gemma
        defaults.set("custom", forKey: "lumin.ai.gemma-model")
        defaults.set(AIProviderChoice.gemma.rawValue, forKey: "lumin.ai.provider")
        engine = nil
        activeBackend = nil
        await loadModel(at: destination, displayName: "カスタムGemma")
    }

    func analyzeIncorrectAnswer(
        _ answer: String,
        question: QuizQuestion,
        hintLevel: Int
    ) async throws -> GemmaGuidance {
        let candidateValues = Array(Set(Array(question.misconceptionAnswers.values) + [question.genericMisconception]))
        let candidatePairs = candidateValues.enumerated().map { (code: "m\($0.offset)", label: $0.element) }
        let candidates = candidatePairs.map { "\($0.code)=\($0.label)" }.joined(separator: " / ")
        let prompt = """
        問題: \(question.prompt)
        正答候補: \(question.acceptedAnswers.joined(separator: ", "))
        生徒の回答: \(answer)
        学習概念: \(question.concept)
        誤概念候補: \(candidates)
        ヒント段階: \(hintLevel)（1は方向だけ、2は使う考え方、3は似た途中式）

        候補コードを一つ選び、正解そのものを示さず、中学生向けの短いヒントを一つ作ってください。
        """

        state = .generating
        let started = Date()
        defer { lastLatency = Date().timeIntervalSince(started) }

        do {
            let guidance: GemmaGuidance
            switch activeBackend {
            case .appleFoundation:
                guidance = try await analyzeWithApple(
                    prompt: prompt,
                    candidatePairs: candidatePairs,
                    question: question,
                    hintLevel: hintLevel
                )
            case .gemma:
                guidance = try await analyzeWithGemma(
                    prompt: prompt,
                    candidatePairs: candidatePairs,
                    question: question,
                    hintLevel: hintLevel
                )
            case nil:
                throw GemmaSetupError.modelNotReady
            }
            restoreReadyState()
            return guidance
        } catch {
            if selectedProvider == .automatic, activeBackend == .appleFoundation,
               await switchToInstalledGemmaFallback() {
                return try await analyzeIncorrectAnswer(answer, question: question, hintLevel: hintLevel)
            }
            restoreReadyState()
            throw error
        }
    }

    func generateLessonPlan(from summary: ClassSummary, quiz: Quiz? = nil) async throws -> LessonPlan {
        let misconceptionText = summary.misconceptions.prefix(4)
            .map { "\($0.name): \(Int($0.share * 100))%" }
            .joined(separator: ", ")
        let prompt = """
        匿名のクラス集計から、授業冒頭10分の案を作ってください。
        教科: \(quiz?.subject ?? "未指定")
        教材: \(quiz?.title ?? "未指定")
        単元: \(quiz?.topic ?? "未指定")
        出題概念: \(quiz?.questions.map(\.concept).joined(separator: "、") ?? "未指定")
        参加端末: \(summary.participantCount)
        初回正答率: \(Int(summary.correctRate * 100))%
        ヒント後の再挑戦成功率: \(Int(summary.retrySuccessRate * 100))%
        主な誤概念: \(misconceptionText.isEmpty ? "回答なし" : misconceptionText)

        4段階で合計10分にし、生徒が説明・比較・再挑戦する活動を含めてください。
        AI案を教師が修正する前提で注意点も付けてください。
        """

        state = .generating
        let started = Date()
        defer { lastLatency = Date().timeIntervalSince(started) }
        do {
            let plan: LessonPlan
            switch activeBackend {
            case .appleFoundation:
                plan = try await lessonPlanWithApple(prompt: prompt)
            case .gemma:
                plan = try await lessonPlanWithGemma(prompt: prompt)
            case nil:
                throw GemmaSetupError.modelNotReady
            }
            restoreReadyState()
            return plan
        } catch {
            if selectedProvider == .automatic, activeBackend == .appleFoundation,
               await switchToInstalledGemmaFallback() {
                return try await generateLessonPlan(from: summary, quiz: quiz)
            }
            restoreReadyState()
            throw error
        }
    }

    func answerTeacherQuestion(
        _ question: String,
        summary: ClassSummary,
        quiz: Quiz,
        priorTurns: [TeacherChatTurn] = []
    ) async throws -> String {
        let misconceptions = summary.misconceptions.prefix(5)
            .map { "\($0.name): \($0.count)件（\(Int($0.share * 100))%）" }
            .joined(separator: "、")
        let history = priorTurns.suffix(6).map { turn in
            let speaker = turn.role == .teacher ? "先生" : "AI"
            return "\(speaker): \(String(turn.text.prefix(500)))"
        }.joined(separator: "\n")
        let prompt = """
        以下の匿名集計と教材情報だけを根拠に、先生の質問へ日本語で簡潔かつ実践的に答えてください。
        集計にない個人の状態は推測せず、データが不足する場合はその旨を明示してください。

        教科: \(quiz.subject)
        教材: \(quiz.title)
        単元: \(quiz.topic ?? quiz.title)
        出題概念: \(quiz.questions.map(\.concept).joined(separator: "、"))
        参加端末: \(summary.participantCount)
        回答数: \(summary.responseCount)
        初回正答率: \(Int(summary.correctRate * 100))%
        ヒント後の再挑戦成功率: \(Int(summary.retrySuccessRate * 100))%
        1回答あたり平均ヒント数: \(summary.averageHints.formatted(.number.precision(.fractionLength(1))))
        主なつまずき: \(misconceptions.isEmpty ? "回答データなし" : misconceptions)

        直近の対話:
        \(history.isEmpty ? "なし" : history)

        先生の質問: \(String(question.prefix(1000)))
        回答は必要に応じて箇条書きを使い、300文字程度までにしてください。
        """

        state = .generating
        let started = Date()
        defer { lastLatency = Date().timeIntervalSince(started) }
        do {
            let answer: String
            switch activeBackend {
            case .appleFoundation:
                answer = try await teacherChatWithApple(prompt: prompt)
            case .gemma:
                answer = try await teacherChatWithGemma(prompt: prompt)
            case nil:
                throw GemmaSetupError.modelNotReady
            }
            restoreReadyState()
            let cleaned = answer.trimmingCharacters(in: .whitespacesAndNewlines)
            guard !cleaned.isEmpty else { throw GemmaSetupError.invalidResponse }
            return cleaned
        } catch {
            if selectedProvider == .automatic, activeBackend == .appleFoundation,
               await switchToInstalledGemmaFallback() {
                return try await answerTeacherQuestion(
                    question,
                    summary: summary,
                    quiz: quiz,
                    priorTurns: priorTurns
                )
            }
            restoreReadyState()
            throw error
        }
    }

    private func activateSelection() async {
        switch selectedProvider {
        case .automatic:
            if isAppleFoundationAvailable {
                activateAppleFoundation()
            } else {
                await activateGemmaSelection()
            }
        case .appleFoundation:
            if isAppleFoundationAvailable {
                activateAppleFoundation()
            } else {
                state = .failed(appleAvailabilityText)
                backendName = "Apple Foundation Models"
            }
        case .gemma:
            await activateGemmaSelection()
        }
    }

    private func activateAppleFoundation() {
        engine = nil
        activeBackend = .appleFoundation
        backendName = "Apple Foundation Model · システム管理"
        state = .ready(modelName: "Apple Foundation Model")
    }

    private func activateGemmaSelection() async {
        if selectedModelID == "custom" {
            let url = try? modelDirectory().appendingPathComponent("custom-model.litertlm")
            if let url, FileManager.default.fileExists(atPath: url.path) {
                await loadModel(at: url, displayName: "カスタムGemma")
                return
            }
        } else if let option = modelCatalog.first(where: { $0.id == selectedModelID }) {
            if let url = try? modelURL(for: option), FileManager.default.fileExists(atPath: url.path) {
                await loadModel(at: url, displayName: option.title)
                return
            }
            if let bundled = bundledURL(for: option) {
                await loadModel(at: bundled, displayName: option.title)
                return
            }
        }
        activeBackend = nil
        backendName = "Gemma · 未ダウンロード"
        state = .notInstalled
    }

    private func loadModel(at url: URL, displayName: String) async {
        state = .loading
        do {
            let cacheURL = try cacheDirectory()
            let newEngine: Engine
#if targetEnvironment(simulator)
            newEngine = try await initializeEngine(modelURL: url, cacheURL: cacheURL, backend: .cpu(threadCount: 4))
            backendName = "\(displayName) · CPU（Simulator）"
#else
            do {
                newEngine = try await initializeEngine(modelURL: url, cacheURL: cacheURL, backend: .gpu)
                backendName = "\(displayName) · Metal GPU"
            } catch {
                newEngine = try await initializeEngine(modelURL: url, cacheURL: cacheURL, backend: .cpu(threadCount: 4))
                backendName = "\(displayName) · CPU"
            }
#endif
            engine = newEngine
            activeBackend = .gemma
            state = .ready(modelName: displayName)
        } catch {
            engine = nil
            activeBackend = nil
            state = .failed(error.localizedDescription)
        }
    }

    private func initializeEngine(modelURL: URL, cacheURL: URL, backend: Backend) async throws -> Engine {
        let config = try EngineConfig(
            modelPath: modelURL.path,
            backend: backend,
            maxNumTokens: 1024,
            cacheDir: cacheURL.path
        )
        let candidate = Engine(engineConfig: config)
        try await candidate.initialize()
        return candidate
    }

    private func analyzeWithGemma(
        prompt: String,
        candidatePairs: [(code: String, label: String)],
        question: QuizQuestion,
        hintLevel: Int
    ) async throws -> GemmaGuidance {
        guard let engine else { throw GemmaSetupError.modelNotReady }
        let conversation = try await makeStructuredConversation(engine: engine)
        let format = try ResponseFormat.json(schema: Self.analysisSchema(codes: candidatePairs.map(\.code)))
        let response = try await conversation.sendMessage(
            Message(prompt + "\nJSON以外は出力しないでください。"),
            maxOutputTokens: 220,
            responseFormat: format
        )
        let decoded = try decodeJSON(GemmaAnalysisPayload.self, from: response.toString)
        return GemmaGuidance(
            misconception: candidatePairs.first(where: { $0.code == decoded.misconception })?.label
                ?? question.genericMisconception,
            hint: sanitizedHint(decoded.hint, question: question, level: hintLevel)
        )
    }

    private func lessonPlanWithGemma(prompt: String) async throws -> LessonPlan {
        guard let engine else { throw GemmaSetupError.modelNotReady }
        let conversation = try await makeStructuredConversation(engine: engine)
        let format = try ResponseFormat.json(schema: Self.lessonPlanSchema)
        let response = try await conversation.sendMessage(
            Message(prompt + "\nJSON以外は出力しないでください。"),
            maxOutputTokens: 420,
            responseFormat: format
        )
        return try decodeJSON(LessonPlan.self, from: response.toString)
    }

    private func teacherChatWithGemma(prompt: String) async throws -> String {
        guard let engine else { throw GemmaSetupError.modelNotReady }
        let sampler = try SamplerConfig(topK: 30, topP: 0.9, temperature: 0.4)
        let conversation = try await engine.createConversation(with: ConversationConfig(
            samplerConfig: sampler,
            automaticToolCalling: false
        ))
        let response = try await conversation.sendMessage(Message(prompt), maxOutputTokens: 420)
        return response.toString
    }

    private func analyzeWithApple(
        prompt: String,
        candidatePairs: [(code: String, label: String)],
        question: QuizQuestion,
        hintLevel: Int
    ) async throws -> GemmaGuidance {
#if canImport(FoundationModels)
        if #available(iOS 26.0, *) {
            let session = LanguageModelSession(instructions: """
            あなたは中学生向けの教科横断チューターです。問題文と概念に沿って誤答を分析し、答えを直接示さない段階的ヒントを一つ返します。
            """)
            let response = try await session.respond(to: prompt, generating: AppleGuidanceOutput.self)
            return GemmaGuidance(
                misconception: candidatePairs.first(where: { $0.code == response.content.misconception })?.label
                    ?? question.genericMisconception,
                hint: sanitizedHint(response.content.hint, question: question, level: hintLevel)
            )
        }
#endif
        throw GemmaSetupError.appleFoundationUnavailable
    }

    private func lessonPlanWithApple(prompt: String) async throws -> LessonPlan {
#if canImport(FoundationModels)
        if #available(iOS 26.0, *) {
            let session = LanguageModelSession(instructions: """
            あなたは中学校の教科横断的な授業設計を支援します。指定された教科・単元に従い、個別の解答本文ではなく匿名集計だけから短い授業案を作ります。
            """)
            let response = try await session.respond(to: prompt, generating: AppleLessonPlanOutput.self)
            return LessonPlan(
                focus: response.content.focus,
                steps: response.content.steps,
                checkQuestion: response.content.checkQuestion,
                teacherNote: response.content.teacherNote
            )
        }
#endif
        throw GemmaSetupError.appleFoundationUnavailable
    }

    private func teacherChatWithApple(prompt: String) async throws -> String {
#if canImport(FoundationModels)
        if #available(iOS 26.0, *) {
            let session = LanguageModelSession(instructions: """
            あなたは中学校の先生を支援する教育アシスタントです。匿名のクラス集計と教材情報だけを根拠にし、個人を推測せず、授業で実行できる提案を返します。
            """)
            let response = try await session.respond(to: prompt)
            return response.content
        }
#endif
        throw GemmaSetupError.appleFoundationUnavailable
    }

    private func makeStructuredConversation(engine: Engine) async throws -> Conversation {
        let sampler = try SamplerConfig(topK: 20, topP: 0.9, temperature: 0.2, seed: 42)
        let config = ConversationConfig(
            samplerConfig: sampler,
            automaticToolCalling: false,
            enableResponseFormat: true
        )
        return try await engine.createConversation(with: config)
    }

    private func refreshAppleAvailability() {
#if canImport(FoundationModels)
        if #available(iOS 26.0, *) {
            switch SystemLanguageModel.default.availability {
            case .available:
                isAppleFoundationAvailable = true
                appleAvailabilityText = "利用可能 · OSモデル（iOS 27では新世代モデル）"
            case .unavailable(.deviceNotEligible):
                isAppleFoundationAvailable = false
                appleAvailabilityText = "この端末はApple Intelligenceに対応していません"
            case .unavailable(.appleIntelligenceNotEnabled):
                isAppleFoundationAvailable = false
                appleAvailabilityText = "設定でApple Intelligenceを有効にしてください"
            case .unavailable(.modelNotReady):
                isAppleFoundationAvailable = false
                appleAvailabilityText = "システムモデルを準備中です"
            @unknown default:
                isAppleFoundationAvailable = false
                appleAvailabilityText = "現在利用できません"
            }
            return
        }
#endif
        isAppleFoundationAvailable = false
        appleAvailabilityText = "iOS 26以降が必要です"
    }

    private func switchToInstalledGemmaFallback() async -> Bool {
        guard let option = modelCatalog.first(where: { $0.id == selectedModelID }),
              isModelInstalled(option) else { return false }
        await activateGemmaSelection()
        return activeBackend == .gemma
    }

    private func restoreReadyState() {
        switch activeBackend {
        case .appleFoundation:
            state = .ready(modelName: "Apple Foundation Model")
        case .gemma:
            state = .ready(modelName: selectedModelTitle)
        case nil:
            state = .notInstalled
        }
    }

    private static func analysisSchema(codes: [String]) -> [String: Any] {
        [
            "type": "object",
            "properties": [
                "misconception": ["type": "string", "enum": codes],
                "hint": ["type": "string", "maxLength": 80]
            ],
            "required": ["misconception", "hint"],
            "additionalProperties": false
        ]
    }

    private static let lessonPlanSchema = """
    {"type":"object","properties":{"focus":{"type":"string","maxLength":60},"steps":{"type":"array","items":{"type":"string","maxLength":100},"minItems":4,"maxItems":4},"checkQuestion":{"type":"string","maxLength":120},"teacherNote":{"type":"string","maxLength":120}},"required":["focus","steps","checkQuestion","teacherNote"],"additionalProperties":false}
    """

    private func sanitizedHint(_ hint: String, question: QuizQuestion, level: Int) -> String {
        let cleaned = hint.trimmingCharacters(in: .whitespacesAndNewlines)
        guard !cleaned.isEmpty else { return question.hints[min(level - 1, question.hints.count - 1)] }
        let normalized = cleaned.replacingOccurrences(of: " ", with: "").lowercased()
        let leaksAnswer = question.acceptedAnswers.contains { accepted in
            let answer = accepted.replacingOccurrences(of: " ", with: "").lowercased()
            return !answer.isEmpty && normalized.contains(answer)
        }
        return leaksAnswer
            ? question.hints[min(level - 1, question.hints.count - 1)]
            : String(cleaned.prefix(80))
    }

    private func decodeJSON<T: Decodable>(_ type: T.Type, from text: String) throws -> T {
        guard let start = text.firstIndex(of: "{"), let end = text.lastIndex(of: "}") else {
            throw GemmaSetupError.invalidResponse
        }
        let json = String(text[start...end])
        guard let data = json.data(using: .utf8) else { throw GemmaSetupError.invalidResponse }
        return try JSONDecoder().decode(type, from: data)
    }

    private func bundledURL(for option: GemmaModelOption) -> URL? {
        let base = (option.fileName as NSString).deletingPathExtension
        return Bundle.main.url(forResource: base, withExtension: "litertlm", subdirectory: "Models")
    }

    private func modelURL(for option: GemmaModelOption) throws -> URL {
        try modelDirectory().appendingPathComponent(option.fileName)
    }

    private func modelDirectory() throws -> URL {
        guard let base = FileManager.default.urls(for: .applicationSupportDirectory, in: .userDomainMask).first else {
            throw GemmaSetupError.storageUnavailable
        }
        let directory = base.appendingPathComponent("Lumin/Models", isDirectory: true)
        try FileManager.default.createDirectory(at: directory, withIntermediateDirectories: true)
        return directory
    }

    private func cacheDirectory() throws -> URL {
        guard let base = FileManager.default.urls(for: .cachesDirectory, in: .userDomainMask).first else {
            throw GemmaSetupError.storageUnavailable
        }
        let directory = base.appendingPathComponent("Lumin/Gemma", isDirectory: true)
        try FileManager.default.createDirectory(at: directory, withIntermediateDirectories: true)
        return directory
    }
}

private struct GemmaAnalysisPayload: Decodable {
    let misconception: String
    let hint: String
}

enum GemmaSetupError: LocalizedError {
    case modelNotReady
    case unsupportedFile
    case storageUnavailable
    case invalidResponse
    case appleFoundationUnavailable

    var errorDescription: String? {
        switch self {
        case .modelNotReady: "AIモデルの準備が完了していません。"
        case .unsupportedFile: "LiteRT-LM形式（.litertlm）のモデルを選んでください。"
        case .storageUnavailable: "モデル保存先を作成できませんでした。"
        case .invalidResponse: "AIの出力を解析できませんでした。"
        case .appleFoundationUnavailable: "Apple Foundation Modelを利用できません。"
        }
    }
}
