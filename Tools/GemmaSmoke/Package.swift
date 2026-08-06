// swift-tools-version: 6.0
import PackageDescription

let package = Package(
    name: "LuminGemmaSmoke",
    platforms: [.macOS(.v14)],
    dependencies: [
        .package(path: "../../Vendor/LiteRTLM")
    ],
    targets: [
        .executableTarget(
            name: "GemmaSmoke",
            dependencies: [.product(name: "LiteRTLM", package: "litertlm")]
        )
    ]
)
