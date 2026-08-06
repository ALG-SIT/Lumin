// swift-tools-version: 6.0
import PackageDescription

let package = Package(
    name: "Lumin",
    platforms: [.macOS(.v14), .iOS(.v17)],
    products: [.library(name: "LuminCore", targets: ["LuminCore"])],
    targets: [
        .target(name: "LuminCore"),
        .testTarget(name: "LuminCoreTests", dependencies: ["LuminCore"])
    ]
)
