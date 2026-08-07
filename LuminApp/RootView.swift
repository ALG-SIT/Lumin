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
        .background(LuminPageBackground())
        .animation(.spring(response: 0.36, dampingFraction: 1), value: model.role)
        .onChange(of: model.role) { _, role in
            if role == nil { peerService.stop() }
        }
    }
}

private struct RoleSelectionView: View {
    @EnvironmentObject private var model: AppModel
    @Environment(\.accessibilityReduceMotion) private var reduceMotion

    var body: some View {
        GeometryReader { proxy in
            ScrollView {
                VStack(alignment: .leading, spacing: 32) {
                    HStack(spacing: 12) {
                        Image(systemName: "sun.max.fill")
                            .foregroundStyle(LuminTheme.amber)
                        Text("LUMIN")
                            .font(.title3.weight(.black))
                            .tracking(1.6)
                    }
                    .accessibilityElement(children: .combine)
                    .accessibilityLabel("Lumin")

                    VStack(alignment: .leading, spacing: 12) {
                        Text("理解を照らし、\n次の学びにつなげる。")
                            .font(.system(proxy.size.width > 700 ? .largeTitle : .title, design: .rounded, weight: .bold))
                            .tracking(proxy.size.width > 700 ? -1.1 : -0.5)
                            .foregroundStyle(LuminTheme.ink)
                        Text("解答は端末の外へ出さず、必要な気づきだけを教室で共有します。")
                            .font(.title3.weight(.regular))
                            .foregroundStyle(LuminTheme.muted)
                            .fixedSize(horizontal: false, vertical: true)
                    }

                    ViewThatFits(in: .horizontal) {
                        HStack(spacing: 16) { roleCards }
                        VStack(spacing: 14) { roleCards }
                    }
                    .frame(maxWidth: 900)

                    ViewThatFits(in: .horizontal) {
                        HStack(spacing: 20) { trustLabels }
                        VStack(alignment: .leading, spacing: 10) { trustLabels }
                    }
                    .font(.caption.weight(.medium))
                    .foregroundStyle(LuminTheme.muted)

                    GemmaStatusView()
                        .frame(maxWidth: 520)
                }
                .padding(.horizontal, proxy.size.width > 700 ? 40 : 20)
                .padding(.vertical, 28)
                .frame(maxWidth: .infinity, minHeight: proxy.size.height, alignment: .center)
            }
            .background(LuminPageBackground())
        }
    }

    @ViewBuilder
    private var roleCards: some View {
        roleCard(
            title: "先生として始める",
            detail: "小テストを配信し、クラスのつまずきから次の10分を組み立てます。",
            icon: "rectangle.3.group.fill",
            color: LuminTheme.teal
        ) { choose(.teacher) }

        roleCard(
            title: "生徒として参加",
            detail: "自分のペースで解き、正解を見る前に段階的なヒントを受け取ります。",
            icon: "pencil.and.scribble",
            color: LuminTheme.amber
        ) { choose(.student) }
    }

    @ViewBuilder
    private var trustLabels: some View {
        Label("インターネット不要", systemImage: "wifi.slash")
        Label("解答本文は端末内", systemImage: "lock.shield")
        Label("先生が最終判断", systemImage: "person.crop.circle.badge.checkmark")
    }

    private func roleCard(
        title: String,
        detail: String,
        icon: String,
        color: Color,
        action: @escaping () -> Void
    ) -> some View {
        Button(action: action) {
            VStack(alignment: .leading, spacing: 16) {
                LuminIconBadge(systemImage: icon, color: color, size: 54)
                Text(title)
                    .font(.title2.bold())
                    .foregroundStyle(LuminTheme.ink)
                Text(detail)
                    .font(.body)
                    .foregroundStyle(LuminTheme.muted)
                    .multilineTextAlignment(.leading)
                HStack {
                    Text("続ける")
                    Spacer()
                    Image(systemName: "arrow.right")
                }
                .font(.headline)
                .foregroundStyle(color)
            }
            .padding(24)
            .frame(maxWidth: .infinity, minHeight: 248, alignment: .leading)
            .background(.regularMaterial)
            .clipShape(RoundedRectangle(cornerRadius: 24, style: .continuous))
            .overlay {
                RoundedRectangle(cornerRadius: 24, style: .continuous)
                    .stroke(color.opacity(0.2), lineWidth: 1)
            }
            .shadow(color: color.opacity(0.08), radius: 22, y: 9)
        }
        .buttonStyle(LuminPressButtonStyle())
        .accessibilityHint("\(title)画面を開きます")
    }

    private func choose(_ role: AppModel.Role) {
        if reduceMotion {
            model.role = role
        } else {
            withAnimation(.spring(response: 0.36, dampingFraction: 1)) {
                model.role = role
            }
        }
    }
}
