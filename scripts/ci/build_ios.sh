#!/usr/bin/env bash
set -euo pipefail
REPO="$(cd "$(dirname "${BASH_SOURCE[0]}")/../../" && pwd)"
cd "$REPO/rust"

# If FFMPEG_PREBUILT_DIR is set (CI), point each build at the matching target dir.
# Otherwise cargo finds FFmpeg via pkg-config (local: brew install ffmpeg pkg-config).
cargo_build() {
  local target="$1"
  if [[ -n "${FFMPEG_PREBUILT_DIR:-}" ]]; then
    IPHONEOS_DEPLOYMENT_TARGET=14.0 \
      FFMPEG_DIR="${FFMPEG_PREBUILT_DIR}/${target}" cargo build --release --target "$target"
  else
    IPHONEOS_DEPLOYMENT_TARGET=14.0 cargo build --release --target "$target"
  fi
}

cargo_build aarch64-apple-ios
cargo_build aarch64-apple-ios-sim
cargo_build x86_64-apple-ios

mkdir -p target/ios-sim-universal/release
lipo -create \
  target/aarch64-apple-ios-sim/release/libmedia_metadata_plus.a \
  target/x86_64-apple-ios/release/libmedia_metadata_plus.a \
  -output target/ios-sim-universal/release/libmedia_metadata_plus.a

XCFW="$REPO/ios/Frameworks/media_metadata_plus.xcframework"
rm -rf "$XCFW"
mkdir -p "$REPO/ios/Frameworks"
xcodebuild -create-xcframework \
  -library target/aarch64-apple-ios/release/libmedia_metadata_plus.a \
  -library target/ios-sim-universal/release/libmedia_metadata_plus.a \
  -output "$XCFW"