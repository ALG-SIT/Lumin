import Foundation
import LiteRTLM

@main
struct LuminGemmaSmoke {
    static func main() async throws {
        let arguments = CommandLine.arguments
        let defaultPath = URL(fileURLWithPath: FileManager.default.currentDirectoryPath)
            .appendingPathComponent("LuminApp/Models/gemma3-1b-it-int4.litertlm")
            .path
        let modelPath = arguments.count > 1 ? arguments[1] : defaultPath

        guard FileManager.default.fileExists(atPath: modelPath) else {
            print("Model not found: \(modelPath)")
            print("Run ./scripts/download-gemma.sh or pass a .litertlm path.")
            throw CocoaError(.fileNoSuchFile)
        }

        let cache = FileManager.default.temporaryDirectory
            .appendingPathComponent("lumin-gemma-smoke-cache", isDirectory: true)
        try FileManager.default.createDirectory(at: cache, withIntermediateDirectories: true)

        print("Loading Gemma on device…")
        let config = try EngineConfig(
            modelPath: modelPath,
            backend: .gpu,
            maxNumTokens: 512,
            cacheDir: cache.path
        )
        let engine = Engine(engineConfig: config)
        try await engine.initialize()

        let sampler = try SamplerConfig(topK: 20, topP: 0.9, temperature: 0.2, seed: 42)
        let conversation = try await engine.createConversation(with: ConversationConfig(
            samplerConfig: sampler,
            automaticToolCalling: false,
            enableResponseFormat: true
        ))
        let prompt = """
        中学生向け数学チューターとして誤答を分析してください。
        問題: y = 3x + 2 の傾きは？
        生徒の回答: 2
        誤概念候補: m0=傾きと切片の混同 / m1=傾きの読み取り不足
        正解を直接示さず、JSONだけを出力してください。
        """
        let format = try ResponseFormat.json(schema: [
            "type": "object",
            "properties": [
                "misconception": ["type": "string", "enum": ["m0", "m1"]],
                "hint": ["type": "string", "maxLength": 80]
            ],
            "required": ["misconception", "hint"],
            "additionalProperties": false
        ])
        let started = Date()
        let response = try await conversation.sendMessage(
            Message(prompt),
            maxOutputTokens: 220,
            responseFormat: format
        )
        let elapsed = Date().timeIntervalSince(started)

        print("Gemma response: \(response.toString)")
        print(String(format: "Inference completed locally in %.2f seconds.", elapsed))
    }
}
