// swift-tools-version:5.9
import PackageDescription

let package = Package(
    name: "SoundAnalysisBridge",
    platforms: [.macOS(.v13)],
    products: [
        .library(
            name: "SoundAnalysisBridge",
            type: .static,
            targets: ["SoundAnalysisBridge"]
        ),
    ],
    targets: [
        .target(
            name: "SoundAnalysisBridge",
            path: "Sources/SoundAnalysisBridge"
        ),
    ]
)
