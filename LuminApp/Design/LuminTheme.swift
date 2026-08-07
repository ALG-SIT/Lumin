import SwiftUI
import UIKit

enum LuminTheme {
    static let ink = Color.primary
    static let muted = Color.secondary
    static let canvas = Color(uiColor: .systemGroupedBackground)
    static let paper = Color(uiColor: .secondarySystemGroupedBackground)

    static let teal = Color(uiColor: UIColor { traits in
        traits.userInterfaceStyle == .dark
            ? UIColor(red: 0.20, green: 0.78, blue: 0.70, alpha: 1)
            : UIColor(red: 0.02, green: 0.43, blue: 0.39, alpha: 1)
    })
    static let tealSoft = Color(uiColor: UIColor { traits in
        traits.userInterfaceStyle == .dark
            ? UIColor(red: 0.08, green: 0.24, blue: 0.22, alpha: 1)
            : UIColor(red: 0.87, green: 0.95, blue: 0.93, alpha: 1)
    })
    static let amber = Color(uiColor: UIColor { traits in
        traits.userInterfaceStyle == .dark
            ? UIColor(red: 1.00, green: 0.70, blue: 0.29, alpha: 1)
            : UIColor(red: 0.90, green: 0.49, blue: 0.08, alpha: 1)
    })
    static let coral = Color(uiColor: UIColor { traits in
        traits.userInterfaceStyle == .dark
            ? UIColor(red: 1.00, green: 0.43, blue: 0.38, alpha: 1)
            : UIColor(red: 0.79, green: 0.20, blue: 0.16, alpha: 1)
    })

    static let contentWidth: CGFloat = 1_020
    static let narrowContentWidth: CGFloat = 760
}

struct LuminPageBackground: View {
    var body: some View {
        ZStack {
            LuminTheme.canvas
            LinearGradient(
                colors: [LuminTheme.teal.opacity(0.07), .clear, LuminTheme.amber.opacity(0.045)],
                startPoint: .topLeading,
                endPoint: .bottomTrailing
            )
        }
        .ignoresSafeArea()
        .accessibilityHidden(true)
    }
}

struct LuminCard<Content: View>: View {
    @Environment(\.accessibilityReduceTransparency) private var reduceTransparency
    @ViewBuilder let content: () -> Content

    var body: some View {
        content()
            .padding(20)
            .frame(maxWidth: .infinity, alignment: .leading)
            .background {
                if reduceTransparency {
                    RoundedRectangle(cornerRadius: 22, style: .continuous)
                        .fill(LuminTheme.paper)
                } else {
                    RoundedRectangle(cornerRadius: 22, style: .continuous)
                        .fill(.regularMaterial)
                }
            }
            .clipShape(RoundedRectangle(cornerRadius: 22, style: .continuous))
            .overlay {
                RoundedRectangle(cornerRadius: 22, style: .continuous)
                    .stroke(.white.opacity(0.24), lineWidth: 0.75)
            }
            .shadow(color: .black.opacity(0.055), radius: 18, y: 7)
    }
}

struct LuminSectionHeader: View {
    let eyebrow: String?
    let title: String
    let detail: String

    init(_ title: String, detail: String, eyebrow: String? = nil) {
        self.eyebrow = eyebrow
        self.title = title
        self.detail = detail
    }

    var body: some View {
        VStack(alignment: .leading, spacing: 6) {
            if let eyebrow {
                Text(eyebrow)
                    .font(.caption.weight(.bold))
                    .foregroundStyle(LuminTheme.teal)
                    .textCase(.uppercase)
                    .tracking(0.5)
            }
            Text(title)
                .font(.system(.largeTitle, design: .rounded, weight: .bold))
                .tracking(-0.7)
                .foregroundStyle(LuminTheme.ink)
            Text(detail)
                .font(.body)
                .foregroundStyle(LuminTheme.muted)
                .fixedSize(horizontal: false, vertical: true)
        }
    }
}

struct LuminIconBadge: View {
    let systemImage: String
    var color = LuminTheme.teal
    var size: CGFloat = 50

    var body: some View {
        Image(systemName: systemImage)
            .font(.system(size: size * 0.42, weight: .semibold))
            .foregroundStyle(color)
            .frame(width: size, height: size)
            .background(color.opacity(0.12), in: RoundedRectangle(cornerRadius: size * 0.3, style: .continuous))
            .accessibilityHidden(true)
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
            .background(color.opacity(0.12), in: Capsule())
            .accessibilityElement(children: .combine)
    }
}

struct PrimaryButtonStyle: ButtonStyle {
    @Environment(\.isEnabled) private var isEnabled
    @Environment(\.accessibilityReduceMotion) private var reduceMotion

    func makeBody(configuration: Configuration) -> some View {
        configuration.label
            .font(.headline)
            .foregroundStyle(.white)
            .padding(.horizontal, 20)
            .frame(minHeight: 52)
            .frame(maxWidth: .infinity)
            .background(
                isEnabled
                    ? (configuration.isPressed ? LuminTheme.teal.opacity(0.78) : LuminTheme.teal)
                    : Color.secondary.opacity(0.35),
                in: RoundedRectangle(cornerRadius: 15, style: .continuous)
            )
            .scaleEffect(configuration.isPressed && !reduceMotion ? 0.975 : 1)
            .opacity(isEnabled ? 1 : 0.7)
            .animation(
                reduceMotion ? .linear(duration: 0.01) : .spring(response: 0.30, dampingFraction: 1),
                value: configuration.isPressed
            )
    }
}

struct LuminPressButtonStyle: ButtonStyle {
    @Environment(\.accessibilityReduceMotion) private var reduceMotion

    func makeBody(configuration: Configuration) -> some View {
        configuration.label
            .scaleEffect(configuration.isPressed && !reduceMotion ? 0.985 : 1)
            .brightness(configuration.isPressed ? -0.025 : 0)
            .animation(
                reduceMotion ? .linear(duration: 0.01) : .spring(response: 0.28, dampingFraction: 1),
                value: configuration.isPressed
            )
    }
}
