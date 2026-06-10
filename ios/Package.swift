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
            checksum: "4dde9a65416fbba2bf7b6904400e02f7fbe2c6e59f66fbcbdaf288360ed0c56b" // ios
        )
    ]
)
