#!/usr/bin/env bash
# Generates the flutter_rust_bridge Dart/Rust FFI bindings.
# Run this after any change to rust/src/api.rs.
#
# Output:
#   lib/src/rust/frb_generated.dart   (Dart side — gitignored, loaded by MediaMetadata.read)
#   rust/src/frb_generated.rs         (Rust side — glue code)

set -euo pipefail

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"

# ── FFmpeg pre-built cache ────────────────────────────────────────────────────
# ffmpeg-sys-next v7 requires FFmpeg 7.x headers. Homebrew ships 8+ (avfft.h
# removed), so bindgen fails. Download the same pre-built used by CI.
FFMPEG_TAG="$(cat "$REPO_ROOT/scripts/ci/ffmpeg_prebuilt_tag")"
FFMPEG_TARGET="aarch64-apple-darwin"
FFMPEG_CACHE="$HOME/.cache/media-metadata-plus/ffmpeg-prebuilt"
FFMPEG_DIR_LOCAL="$FFMPEG_CACHE/$FFMPEG_TARGET"
RELEASE_BASE="https://github.com/yashas-hm/media-metadata-plus/releases/download/ffmpeg-prebuilt-${FFMPEG_TAG}"

if [[ ! -d "$FFMPEG_DIR_LOCAL/include" ]]; then
  echo "→ Downloading pre-built FFmpeg $FFMPEG_TAG ($FFMPEG_TARGET)..."
  mkdir -p "$FFMPEG_CACHE"
  curl -fsSL "$RELEASE_BASE/$FFMPEG_TARGET.zip" -o "$FFMPEG_CACHE/$FFMPEG_TARGET.zip"
  unzip -q "$FFMPEG_CACHE/$FFMPEG_TARGET.zip" -d "$FFMPEG_CACHE"
  rm "$FFMPEG_CACHE/$FFMPEG_TARGET.zip"
fi
export FFMPEG_DIR="$FFMPEG_DIR_LOCAL"
# ─────────────────────────────────────────────────────────────────────────────

echo "→ Checking flutter_rust_bridge_codegen..."
if ! command -v flutter_rust_bridge_codegen &>/dev/null; then
  echo "  Not found. Installing via cargo..."
  cargo install flutter_rust_bridge_codegen
fi

echo "→ Running flutter_rust_bridge_codegen generate..."
cd "$REPO_ROOT"
flutter_rust_bridge_codegen generate

echo "→ Running flutter pub get..."
flutter pub get

echo ""
echo "✓ Done. Dart bindings are at lib/src/rust/frb_generated.dart"

# Clean up the downloaded FFmpeg — only headers were needed for codegen
rm -rf "$FFMPEG_CACHE"
