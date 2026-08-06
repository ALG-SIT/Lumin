import Foundation

enum AIProviderChoice: String, CaseIterable, Identifiable {
    case automatic
    case appleFoundation
    case gemma

    var id: String { rawValue }

    var title: String {
        switch self {
        case .automatic: "自動"
        case .appleFoundation: "Apple Foundation Model"
        case .gemma: "Gemma"
        }
    }

    var detail: String {
        switch self {
        case .automatic: "Appleモデルが使える場合は優先し、それ以外は選択したGemmaを使います。"
        case .appleFoundation: "Apple Intelligenceのシステムモデルを使います。追加ダウンロードは不要です。"
        case .gemma: "Google LiteRT-LMで、ダウンロードしたモデルを完全オフライン実行します。"
        }
    }
}

struct GemmaModelOption: Identifiable, Hashable {
    let id: String
    let title: String
    let subtitle: String
    let fileName: String
    let downloadURL: URL
    let downloadBytes: Int64
    let minimumMemoryGB: Int

    var formattedSize: String {
        ByteCountFormatter.string(fromByteCount: downloadBytes, countStyle: .file)
    }

    static let catalog: [GemmaModelOption] = [
        GemmaModelOption(
            id: "gemma3-1b-int4",
            title: "Gemma 3 1B",
            subtitle: "軽量・高速。ほとんどのiPhone/iPad向け",
            fileName: "gemma3-1b-it-int4.litertlm",
            downloadURL: URL(string: "https://huggingface.co/litert-community/Gemma3-1B-IT/resolve/main/gemma3-1b-it-int4.litertlm?download=true")!,
            downloadBytes: 584_417_280,
            minimumMemoryGB: 4
        ),
        GemmaModelOption(
            id: "gemma3n-e2b-int4",
            title: "Gemma 3n E2B",
            subtitle: "品質と速度のバランス。新しい上位端末向け",
            fileName: "gemma-3n-E2B-it-int4.litertlm",
            downloadURL: URL(string: "https://huggingface.co/google/gemma-3n-E2B-it-litert-lm/resolve/main/gemma-3n-E2B-it-int4.litertlm?download=true")!,
            downloadBytes: 3_660_000_000,
            minimumMemoryGB: 8
        ),
        GemmaModelOption(
            id: "gemma3n-e4b-int4",
            title: "Gemma 3n E4B",
            subtitle: "高品質。大容量メモリのiPad向け",
            fileName: "gemma-3n-E4B-it-int4.litertlm",
            downloadURL: URL(string: "https://huggingface.co/google/gemma-3n-E4B-it-litert-lm/resolve/main/gemma-3n-E4B-it-int4.litertlm?download=true")!,
            downloadBytes: 4_920_000_000,
            minimumMemoryGB: 16
        )
    ]

    static var recommended: GemmaModelOption {
#if targetEnvironment(simulator)
        return catalog[0]
#else
        let memoryGB = Int(ProcessInfo.processInfo.physicalMemory / 1_073_741_824)
        return catalog.last(where: { memoryGB >= $0.minimumMemoryGB }) ?? catalog[0]
#endif
    }
}
