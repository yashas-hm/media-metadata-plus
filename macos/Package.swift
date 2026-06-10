// swift-tools-version:5.9
import PackageDescription

let package = Package(
    name: "media_metadata_plus",
    platforms: [.macOS(.v10_14)],
    products: [
        .library(name: "media_metadata_plus", targets: ["media_metadata_plus"])
    ],
    targets: [
        .target(
            name: "media_metadata_plus",
            dependencies: ["MediaMetadataPlusRust"],
            path: "Classes",
            resources: [.process("../Resources/PrivacyInfo.xcprivacy")]
        ),
        .binaryTarget(
            name: "MediaMetadataPlusRust",
            url: "https://github.com/yashas-hm/media-metadata-plus/releases/download/v1.1.0/macos_v1.1.0.xcframework.zip",
            checksum: "65a3f25e06b75bcc4a8ea4ec71f3dfb45ac8588d6e8290b9f02f79068362a91e" // macos
        )
    ]
)
