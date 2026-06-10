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
            path: "../Classes",
            resources: [.process("../../Resources/PrivacyInfo.xcprivacy")]
        ),
        .binaryTarget(
            name: "MediaMetadataPlusRust",
            url: "https://github.com/yashas-hm/media-metadata-plus/releases/download/v2.0.0/ios_v2.0.0.xcframework.zip",
            checksum: "a6e6619ed721f74d97bcc47cdda43992b512a59aa012782d299eef698b5b13f4" // ios
        )
    ]
)