#!/usr/bin/env bash
set -euo pipefail
REPO="$(cd "$(dirname "${BASH_SOURCE[0]}")/../../" && pwd)"
cd "$REPO/rust"

cargo build --release --target aarch64-apple-ios
cargo build --release --target aarch64-apple-ios-sim
cargo build --release --target x86_64-apple-ios

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
