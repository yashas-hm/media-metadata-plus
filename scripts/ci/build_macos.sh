#!/usr/bin/env bash
set -euo pipefail
REPO="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$REPO/rust"

cargo build --release --target aarch64-apple-darwin
cargo build --release --target x86_64-apple-darwin

mkdir -p target/macos-universal/release
lipo -create \
  target/aarch64-apple-darwin/release/libmedia_metadata_plus.dylib \
  target/x86_64-apple-darwin/release/libmedia_metadata_plus.dylib \
  -output target/macos-universal/release/libmedia_metadata_plus.dylib

install_name_tool -id "@rpath/libmedia_metadata_plus.dylib" \
  target/macos-universal/release/libmedia_metadata_plus.dylib

XCFW="$REPO/macos/Frameworks/media_metadata_plus.xcframework"
rm -rf "$XCFW"
mkdir -p "$REPO/macos/Frameworks"
xcodebuild -create-xcframework \
  -library target/macos-universal/release/libmedia_metadata_plus.dylib \
  -output "$XCFW"
