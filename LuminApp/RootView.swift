import SwiftUI

struct RootView: View {
    @EnvironmentObject private var model: AppModel
    @EnvironmentObject private var peerService: LocalPeerService

    var body: some View {
        Group {
            switch model.role {
            case .teacher:
                TeacherDashboardView()
            case .student:
                StudentHomeView()
            case nil:
                RoleSelectionView()
            }
        }
        .tint(LuminTheme.teal)
        .background(LuminTheme.canvas.ignoresSafeArea())
        .onChange(of: model.role) { _, role in
            if role == nil { peerService.stop() }
        }
    }
}

private struct RoleSelectionView: View {
    @EnvironmentObject private var model: AppModel

    var body: some View {
        GeometryReader { proxy in
            ScrollView {
                VStack(alignment: .leading, spacing: 34) {
                    HStack(spacing: 12) {
                        Image(systemName: "sun.max.fill")
                            .foregroundStyle(LuminTheme.amber)
                        Text("LUMIN")
                            .font(.title3.weight(.black))
                            .tracking(2)
                    }

                    VStack(alignment: .leading, spacing: 14) {
                        Text("理解を照らし、\n次の学びにつなげる。")
                            .font(.system(size: proxy.size.width > 700 ? 52 : 38, weight: .bold, design: .rounded))
                            .foregroundStyle(LuminTheme.ink)
                        Text("解答は端末の外へ出さず、必要な気づきだけを教室で共有します。")
                            .font(.title3)
                            .foregroundStyle(LuminTheme.muted)
                            .fixedSize(horizontal: false, vertical: true)
                    }

                    HStack(spacing: 16) {
                        roleCard(
                            title: "先生として始める",
                            detail: "小テストを配信し、クラスの誤概念から次の10分を組み立てます。",
                            icon: "rectangle.3.group.fill",
                            color: LuminTheme.teal
                        ) { model.role = .teacher }

                        roleCard(
                            title: "生徒として参加",
                            detail: "自分のペースで解き、正解を見ずに段階ヒントを受け取ります。",
                            icon: "pencil.and.scribble",
                            color: LuminTheme.amber
                        ) { model.role = .student }
                    }
                    .frame(maxWidth: 900)

                    HStack(spacing: 20) {
                        Label("インターネット不要", systemImage: "wifi.slash")
                        Label("解答本文は端末内", systemImage: "lock.shield")
                        Label("教師が最終判断", systemImage: "person.crop.circle.badge.checkmark")
                    }
                    .font(.caption.weight(.medium))
                    .foregroundStyle(LuminTheme.muted)

                    GemmaStatusView()
                        .frame(maxWidth: 520)
                }
                .padding(32)
                .frame(maxWidth: .infinity, minHeight: proxy.size.height, alignment: .center)
            }
        }
    }

    private func roleCard(
        title: String,
        detail: String,
        icon: String,
        color: Color,
        action: @escaping () -> Void
    ) -> some View {
        Button(action: action) {
            VStack(alignment: .leading, spacing: 18) {
                Image(systemName: icon)
                    .font(.system(size: 28))
                    .foregroundStyle(color)
                    .frame(width: 56, height: 56)
                    .background(color.opacity(0.12), in: RoundedRectangle(cornerRadius: 16))
                Text(title)
                    .font(.title2.bold())
                    .foregroundStyle(LuminTheme.ink)
                Text(detail)
                    .font(.body)
                    .foregroundStyle(LuminTheme.muted)
                    .multilineTextAlignment(.leading)
                Label("開く", systemImage: "arrow.right")
                    .font(.headline)
                    .foregroundStyle(color)
            }
            .padding(24)
            .frame(maxWidth: .infinity, minHeight: 260, alignment: .leading)
            .background(.white)
            .clipShape(RoundedRectangle(cornerRadius: 24, style: .continuous))
            .overlay {
                RoundedRectangle(cornerRadius: 24, style: .continuous)
                    .stroke(color.opacity(0.18))
            }
        }
        .buttonStyle(.plain)
    }
}
