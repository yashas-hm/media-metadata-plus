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
            url: "https://github.com/yashas-hm/media-metadata-plus/releases/download/v1.5.1/ios_v1.5.1.xcframework.zip",
            checksum: "e48ad5cd003e40704021990797714210190f77469527c0cad27e84542734de69" // ios
        )
    ]
)