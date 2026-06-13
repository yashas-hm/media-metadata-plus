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
            url: "https://github.com/yashas-hm/media-metadata-plus/releases/download/v1.4.2/macos_v1.4.2.xcframework.zip",
            checksum: "ab2760de9f43c97dda8ae5b6cbf7fdf521ff6a5d386f174a4fe9e964408d0cf5" // macos
        )
    ]
)