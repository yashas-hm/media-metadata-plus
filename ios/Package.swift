// swift-tools-version:5.9
import PackageDescription

let package = Package(
    name: "media_metadata_plus",
    platforms: [.iOS(.v13)],
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
            url: "https://github.com/yashas-hm/media-metadata-plus/releases/download/v1.1.0/ios_v1.1.0.xcframework.zip",
            checksum: "bed5218a926689e1a495108f694bb23cf0d462b8ca3f416f412a6cc42503204c" // ios
        )
    ]
)
