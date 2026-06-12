#!/usr/bin/env bash
set -euo pipefail
REPO="$(cd "$(dirname "${BASH_SOURCE[0]}")/../../" && pwd)"
cd "$REPO/rust"

# Build one ABI at a time so we can set FFMPEG_DIR per Rust target triple.
# If FFMPEG_PREBUILT_DIR is not set, cargo finds FFmpeg via pkg-config.
build_abi() {
  local ndk_abi="$1"    # arm64-v8a | armeabi-v7a | x86_64 | x86
  local rust_target="$2" # aarch64-linux-android | armv7-linux-androideabi | …

  if [[ -n "${FFMPEG_PREBUILT_DIR:-}" ]]; then
    FFMPEG_DIR="${FFMPEG_PREBUILT_DIR}/${rust_target}" \
      cargo ndk -t "$ndk_abi" -o "$REPO/android/src/main/jniLibs" build --release
  else
    cargo ndk -t "$ndk_abi" -o "$REPO/android/src/main/jniLibs" build --release
  fi
}

build_abi arm64-v8a   aarch64-linux-android
build_abi armeabi-v7a armv7-linux-androideabi
build_abi x86_64      x86_64-linux-android
build_abi x86         i686-linux-android