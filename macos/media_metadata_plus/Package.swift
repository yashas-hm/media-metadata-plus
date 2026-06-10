// swift-tools-version:5.9
import PackageDescription

let package = Package(
    name: "media_metadata_plus",
    platforms: [.macOS(.v10_14)],
    products: [
        .library(name: "media-metadata-plus", targets: ["media_metadata_plus"])
    ],
    targets: [
        .target(
            name: "media_metadata_plus",
            dependencies: ["MediaMetadataPlusRust"],
            path: "Sources",
            resources: [.process("PrivacyInfo.xcprivacy")]
        ),
        .binaryTarget(
            name: "MediaMetadataPlusRust",
            url: "https://github.com/yashas-hm/media-metadata-plus/releases/download/v2.0.1/macos_v2.0.1.xcframework.zip",
            checksum: "4bbe9b7bf181b400e7ec1e444b957544c3f92df8311d4b933277db49bfbaf6c8" // macos
        )
    ]
)