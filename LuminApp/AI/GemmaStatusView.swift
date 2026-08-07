import SwiftUI
import UniformTypeIdentifiers

struct GemmaStatusView: View {
    @EnvironmentObject private var gemma: GemmaService
    @State private var showsManager = false

    var body: some View {
        HStack(spacing: 10) {
            statusIcon
            VStack(alignment: .leading, spacing: 2) {
                Text(gemma.state.title)
                    .font(.caption.weight(.bold))
                if case .failed(let message) = gemma.state {
                    Text(message)
                        .font(.caption2)
                        .foregroundStyle(LuminTheme.coral)
                        .lineLimit(2)
                } else if gemma.downloadingModelID != nil {
                    ProgressView(value: gemma.downloadProgress)
                    Text(gemma.downloadProgress.formatted(.percent.precision(.fractionLength(0))))
                        .font(.caption2.monospacedDigit())
                        .foregroundStyle(LuminTheme.muted)
                } else {
                    Text(detailText)
                        .font(.caption2)
                        .foregroundStyle(LuminTheme.muted)
                }
            }
            Spacer()
            Button("AIモデル") { showsManager = true }
                .font(.caption.weight(.semibold))
                .buttonStyle(.bordered)
        }
        .padding(12)
        .background(LuminTheme.paper, in: RoundedRectangle(cornerRadius: 14))
        .sheet(isPresented: $showsManager) {
            AIModelManagerView()
                .environmentObject(gemma)
        }
    }

    @ViewBuilder
    private var statusIcon: some View {
        switch gemma.state {
        case .loading, .generating, .downloading:
            ProgressView().controlSize(.small)
        case .ready:
            Image(systemName: gemma.isUsingAppleFoundation ? "apple.intelligence" : "cpu.fill")
                .foregroundStyle(LuminTheme.teal)
        case .notInstalled, .failed:
            Image(systemName: "cpu").foregroundStyle(LuminTheme.amber)
        }
    }

    private var detailText: String {
        if let latency = gemma.lastLatency {
            return "端末内処理 · 前回 \(latency.formatted(.number.precision(.fractionLength(1))))秒 · \(gemma.backendName)"
        }
        return gemma.state.isReady ? gemma.backendName : "自動選択またはモデルのダウンロードが必要です"
    }
}

private struct AIModelManagerView: View {
    @EnvironmentObject private var gemma: GemmaService
    @Environment(\.dismiss) private var dismiss
    @State private var token = ""
    @State private var errorMessage: String?
    @State private var isImporting = false

    var body: some View {
        NavigationStack {
            ScrollView {
                VStack(alignment: .leading, spacing: 22) {
                    providerSection
                    appleSection
                    gemmaSection
                    credentialSection
                }
                .padding(20)
                .frame(maxWidth: 720)
                .frame(maxWidth: .infinity)
            }
            .background(LuminPageBackground())
            .navigationTitle("オンデバイスAI")
            .toolbar {
                ToolbarItem(placement: .confirmationAction) {
                    Button("完了") { dismiss() }
                }
            }
        }
        .onAppear { token = gemma.savedHuggingFaceToken() }
        .fileImporter(
            isPresented: $isImporting,
            allowedContentTypes: [UTType(filenameExtension: "litertlm") ?? .data],
            allowsMultipleSelection: false
        ) { result in
            Task {
                do {
                    guard let url = try result.get().first else { return }
                    try await gemma.importModel(from: url)
                } catch {
                    errorMessage = error.localizedDescription
                }
            }
        }
        .alert("AIモデルを準備できませんでした", isPresented: Binding(
            get: { errorMessage != nil },
            set: { if !$0 { errorMessage = nil } }
        )) {
            Button("OK", role: .cancel) {}
        } message: {
            Text(errorMessage ?? "")
        }
    }

    private var providerSection: some View {
        VStack(alignment: .leading, spacing: 12) {
            sectionTitle("使用方法", systemImage: "arrow.triangle.branch")
            ForEach(AIProviderChoice.allCases) { provider in
                Button {
                    Task { await gemma.selectProvider(provider) }
                } label: {
                    HStack(alignment: .top, spacing: 12) {
                        Image(systemName: gemma.selectedProvider == provider ? "checkmark.circle.fill" : "circle")
                            .foregroundStyle(gemma.selectedProvider == provider ? LuminTheme.teal : LuminTheme.muted)
                        VStack(alignment: .leading, spacing: 4) {
                            Text(provider.title).font(.headline).foregroundStyle(LuminTheme.ink)
                            Text(provider.detail).font(.caption).foregroundStyle(LuminTheme.muted)
                        }
                        Spacer()
                    }
                    .padding(14)
                    .background(LuminTheme.paper, in: RoundedRectangle(cornerRadius: 14))
                }
                .buttonStyle(.plain)
            }
        }
    }

    private var appleSection: some View {
        VStack(alignment: .leading, spacing: 12) {
            sectionTitle("Apple Foundation Model", systemImage: "apple.intelligence")
            HStack(alignment: .top, spacing: 12) {
                Image(systemName: gemma.isAppleFoundationAvailable ? "checkmark.seal.fill" : "exclamationmark.triangle.fill")
                    .foregroundStyle(gemma.isAppleFoundationAvailable ? LuminTheme.teal : LuminTheme.amber)
                VStack(alignment: .leading, spacing: 5) {
                    Text(gemma.appleAvailabilityText).font(.subheadline.weight(.semibold))
                    Text("Apple Intelligenceが管理するオンデバイスモデルです。iOS 27では新世代モデルが自動的に使われ、アプリ容量を増やしません。")
                        .font(.caption)
                        .foregroundStyle(LuminTheme.muted)
                }
            }
            .padding(14)
            .background(LuminTheme.paper, in: RoundedRectangle(cornerRadius: 14))
        }
    }

    private var gemmaSection: some View {
        VStack(alignment: .leading, spacing: 12) {
            HStack {
                sectionTitle("Gemmaモデル", systemImage: "square.stack.3d.up.fill")
                Spacer()
                Text("端末メモリ \(gemma.physicalMemoryDescription)")
                    .font(.caption)
                    .foregroundStyle(LuminTheme.muted)
            }

            ForEach(gemma.modelCatalog) { option in
                modelCard(option)
            }

            Button("手元の .litertlm を読み込む", systemImage: "folder") {
                isImporting = true
            }
            .buttonStyle(.bordered)
            .disabled(gemma.downloadingModelID != nil)
        }
    }

    private func modelCard(_ option: GemmaModelOption) -> some View {
        let installed = gemma.isModelInstalled(option)
        let recommended = gemma.recommendedModelID == option.id
        let selected = gemma.selectedModelID == option.id && gemma.selectedProvider != .appleFoundation

        return VStack(alignment: .leading, spacing: 12) {
            HStack(alignment: .top) {
                VStack(alignment: .leading, spacing: 4) {
                    HStack(spacing: 7) {
                        Text(option.title).font(.headline)
                        if recommended {
                            Text("この端末に推奨")
                                .font(.caption2.bold())
                                .padding(.horizontal, 7)
                                .padding(.vertical, 3)
                                .background(LuminTheme.tealSoft, in: Capsule())
                                .foregroundStyle(LuminTheme.teal)
                        }
                    }
                    Text(option.subtitle).font(.caption).foregroundStyle(LuminTheme.muted)
                    Text("\(option.formattedSize) · 推奨メモリ \(option.minimumMemoryGB)GB以上")
                        .font(.caption2.monospacedDigit())
                        .foregroundStyle(LuminTheme.muted)
                }
                Spacer()
                if installed {
                    Label("保存済み", systemImage: "checkmark.circle.fill")
                        .font(.caption.weight(.semibold))
                        .foregroundStyle(LuminTheme.teal)
                }
            }

            if gemma.downloadingModelID == option.id {
                ProgressView(value: gemma.downloadProgress)
                HStack {
                    Text(gemma.downloadProgress.formatted(.percent.precision(.fractionLength(0))))
                        .font(.caption.monospacedDigit())
                    Spacer()
                    Button("中止", role: .cancel) { gemma.cancelDownload() }
                        .font(.caption)
                }
            } else {
                Button(selected && gemma.state.isReady ? "使用中" : installed ? "このモデルを使う" : "ダウンロード") {
                    Task {
                        do {
                            if installed {
                                await gemma.selectProvider(.gemma)
                                await gemma.selectModel(option)
                            } else {
                                try await gemma.downloadModel(option, token: token)
                            }
                        } catch {
                            errorMessage = error.localizedDescription
                        }
                    }
                }
                .buttonStyle(.borderedProminent)
                .tint(LuminTheme.teal)
                .disabled((selected && gemma.state.isReady) || gemma.downloadingModelID != nil)
            }
        }
        .padding(16)
        .background(LuminTheme.paper, in: RoundedRectangle(cornerRadius: 16))
        .overlay {
            RoundedRectangle(cornerRadius: 16)
                .stroke(selected ? LuminTheme.teal.opacity(0.5) : .clear, lineWidth: 2)
        }
    }

    private var credentialSection: some View {
        VStack(alignment: .leading, spacing: 10) {
            sectionTitle("Hugging Face認証", systemImage: "key.fill")
            SecureField("hf_… アクセストークン", text: $token)
                .textInputAutocapitalization(.never)
                .autocorrectionDisabled()
                .padding(12)
                .background(LuminTheme.paper, in: RoundedRectangle(cornerRadius: 12))
            Text("Gemmaの利用規約へ同意したアカウントのReadトークンを入力してください。ダウンロード時にKeychainへこの端末内限定で保存します。")
                .font(.caption)
                .foregroundStyle(LuminTheme.muted)
        }
    }

    private func sectionTitle(_ title: String, systemImage: String) -> some View {
        Label(title, systemImage: systemImage)
            .font(.title3.bold())
            .foregroundStyle(LuminTheme.ink)
    }
}
