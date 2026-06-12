#!/usr/bin/env bash
set -euo pipefail
REPO="$(cd "$(dirname "${BASH_SOURCE[0]}")/../../" && pwd)"
cd "$REPO/rust"

IOS_SDK=$(xcrun --sdk iphoneos --show-sdk-path)
SIM_SDK=$(xcrun --sdk iphonesimulator --show-sdk-path)

# ffmpeg-sys-next uses bindgen which passes the Rust target triple to libclang.
# Rust's iOS simulator target names ('arm64-apple-ios-sim', 'x86_64-apple-ios')
# are not valid clang triples, so we override them via BINDGEN_EXTRA_CLANG_ARGS
# and point libclang at the correct SDK sysroot so system headers resolve.
cargo_build() {
  local target="$1"
  local sdk="$2"
  local clang_target="$3"
  local bindgen_args="--target=${clang_target} -isysroot ${sdk}"

  if [[ -n "${FFMPEG_PREBUILT_DIR:-}" ]]; then
    SDKROOT="$sdk" \
      BINDGEN_EXTRA_CLANG_ARGS="$bindgen_args" \
      IPHONEOS_DEPLOYMENT_TARGET=14.0 \
      FFMPEG_DIR="${FFMPEG_PREBUILT_DIR}/${target}" \
      cargo build --release --target "$target"
  else
    SDKROOT="$sdk" \
      BINDGEN_EXTRA_CLANG_ARGS="$bindgen_args" \
      IPHONEOS_DEPLOYMENT_TARGET=14.0 \
      cargo build --release --target "$target"
  fi
}

cargo_build aarch64-apple-ios     "$IOS_SDK" "arm64-apple-ios14.0"
cargo_build aarch64-apple-ios-sim "$SIM_SDK" "arm64-apple-ios14.0-simulator"
cargo_build x86_64-apple-ios      "$SIM_SDK" "x86_64-apple-ios14.0-simulator"

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