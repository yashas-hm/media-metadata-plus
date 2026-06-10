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
            checksum: "dad5d129c92e0162f20ce2dc28f3c6b31c80b0369e59a2c9c3ce22b1aec105f8" // macos
        )
    ]
)
