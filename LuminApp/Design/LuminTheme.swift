import SwiftUI

enum LuminTheme {
    static let ink = Color(red: 0.10, green: 0.13, blue: 0.18)
    static let muted = Color(red: 0.39, green: 0.43, blue: 0.49)
    static let canvas = Color(red: 0.96, green: 0.95, blue: 0.92)
    static let paper = Color.white
    static let teal = Color(red: 0.06, green: 0.47, blue: 0.43)
    static let tealSoft = Color(red: 0.86, green: 0.94, blue: 0.92)
    static let amber = Color(red: 0.94, green: 0.57, blue: 0.18)
    static let coral = Color(red: 0.86, green: 0.31, blue: 0.24)
}

struct LuminCard<Content: View>: View {
    @ViewBuilder let content: () -> Content

    var body: some View {
        content()
            .padding(20)
            .frame(maxWidth: .infinity, alignment: .leading)
            .background(LuminTheme.paper)
            .clipShape(RoundedRectangle(cornerRadius: 22, style: .continuous))
            .overlay {
                RoundedRectangle(cornerRadius: 22, style: .continuous)
                    .stroke(Color.black.opacity(0.06), lineWidth: 1)
            }
    }
}

struct StatusPill: View {
    let title: String
    let systemImage: String
    var color = LuminTheme.teal

    var body: some View {
        Label(title, systemImage: systemImage)
            .font(.caption.weight(.semibold))
            .foregroundStyle(color)
            .padding(.horizontal, 11)
            .padding(.vertical, 7)
            .background(color.opacity(0.11), in: Capsule())
    }
}

struct PrimaryButtonStyle: ButtonStyle {
    func makeBody(configuration: Configuration) -> some View {
        configuration.label
            .font(.headline)
            .foregroundStyle(.white)
            .padding(.horizontal, 20)
            .frame(minHeight: 52)
            .frame(maxWidth: .infinity)
            .background(configuration.isPressed ? LuminTheme.ink : LuminTheme.teal)
            .clipShape(RoundedRectangle(cornerRadius: 15, style: .continuous))
            .scaleEffect(configuration.isPressed ? 0.98 : 1)
    }
}
