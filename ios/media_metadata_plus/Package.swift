// swift-tools-version:5.9
import PackageDescription

let package = Package(
    name: "media_metadata_plus",
    platforms: [.iOS(.v13)],
    products: [
        .library(name: "media-metadata-plus", targets: ["media_metadata_plus"])
    ],
    targets: [
        .target(
            name: "media_metadata_plus",
            dependencies: ["MediaMetadataPlusRust"],
            path: "../Classes",
            resources: [.process("../../Resources/PrivacyInfo.xcprivacy")]
        ),
        .binaryTarget(
            name: "MediaMetadataPlusRust",
            url: "https://github.com/yashas-hm/media-metadata-plus/releases/download/1.4.0/ios_1.4.0.xcframework.zip",
            checksum: "605e6d2405fdf96232a490a7a8a095bfd386e3cd0071f624b817e5c5325fba49" // ios
        )
    ]
)