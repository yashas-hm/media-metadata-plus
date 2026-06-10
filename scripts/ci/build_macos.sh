#!/usr/bin/env bash
set -euo pipefail
REPO="$(cd "$(dirname "${BASH_SOURCE[0]}")/../../" && pwd)"
cd "$REPO/rust"

cargo build --release --target aarch64-apple-darwin
cargo build --release --target x86_64-apple-darwin

mkdir -p target/macos-universal/release
lipo -create \
  target/aarch64-apple-darwin/release/libmedia_metadata_plus.dylib \
  target/x86_64-apple-darwin/release/libmedia_metadata_plus.dylib \
  -output target/macos-universal/release/libmedia_metadata_plus.dylib

# CocoaPods rejects xcframeworks with raw .dylib slices — wrap in a .framework first
FW="target/macos-universal/release/media_metadata_plus.framework"
mkdir -p "$FW/Versions/A"
cp target/macos-universal/release/libmedia_metadata_plus.dylib \
   "$FW/Versions/A/media_metadata_plus"
ln -sf Versions/A/media_metadata_plus "$FW/media_metadata_plus"
ln -sf A "$FW/Versions/Current"
install_name_tool -id "@rpath/media_metadata_plus.framework/media_metadata_plus" \
  "$FW/Versions/A/media_metadata_plus"

XCFW="$REPO/macos/Frameworks/media_metadata_plus.xcframework"
rm -rf "$XCFW"
mkdir -p "$REPO/macos/Frameworks"
xcodebuild -create-xcframework \
  -framework "$FW" \
  -output "$XCFW"
