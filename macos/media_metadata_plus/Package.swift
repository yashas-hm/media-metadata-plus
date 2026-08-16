// swift-tools-version:5.9
import Foundation
import PackageDescription

// For local dev/CI integration testing against an unreleased Rust build, set
// MMP_LOCAL_XCFRAMEWORK to a local .xcframework path (see scripts/test.sh and
// scripts/ci/build_macos.sh). Unset — the default — resolves the published,
// checksummed release, exactly as pub.dev consumers and the release CI do.
let rustTarget: Target = ProcessInfo.processInfo.environment["MMP_LOCAL_XCFRAMEWORK"].map {
    .binaryTarget(name: "MediaMetadataPlusRust", path: $0)
} ?? .binaryTarget(
    name: "MediaMetadataPlusRust",
    url: "https://github.com/yashas-hm/media-metadata-plus/releases/download/v1.5.0/macos_v1.5.0.xcframework.zip",
    checksum: "942b9e03d2af76c96ab24f9f7b94871a04b42bc5ee7ea85457cf1573881a6cea" // macos
)

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
        rustTarget
    ]
)