#!/usr/bin/env bash
# Runs the full test suite.
#
# Unit tests run anywhere (no native library needed).
# Integration tests compile the Rust library and run on macOS (or a specified device).
#
# Usage:
#   bash scripts/test.sh                         # unit + integration on macOS
#   bash scripts/test.sh --unit                  # unit tests only
#   bash scripts/test.sh --integration           # integration on macOS
#   bash scripts/test.sh --integration -d <id>   # specific device

set -euo pipefail

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
MODE="all"
DEVICE="macos"

while [[ $# -gt 0 ]]; do
  case $1 in
    --unit)        MODE="unit"; shift ;;
    --integration) MODE="integration"; shift ;;
    -d)            DEVICE="$2"; shift 2 ;;
    *) echo "Unknown argument: $1"; exit 1 ;;
  esac
done

if [[ "$MODE" == "unit" || "$MODE" == "all" ]]; then
  echo "→ Running unit tests..."
  cd "$REPO_ROOT"
  flutter test
  echo ""
fi

if [[ "$MODE" == "integration" || "$MODE" == "all" ]]; then
  # flutter test on macOS doesn't trigger Cargo via ffiPlugin.
  # Build the dylib manually and place it where FRB's debug loader expects it:
  #   ~/Library/Containers/<bundle-id>/Data/rust/target/release/lib<name>.dylib
  if [[ "$DEVICE" == "macos" ]]; then
    BUNDLE_ID="com.example.example"
    DYLIB_NAME="libmedia_metadata_plus.dylib"
    SANDBOX_DIR="$HOME/Library/Containers/$BUNDLE_ID/Data/rust/target/release"
    DYLIB_SRC="$REPO_ROOT/rust/target/release/$DYLIB_NAME"

    # ── FFmpeg pre-built cache ──────────────────────────────────────────────
    # ffmpeg-sys-next v7 requires FFmpeg 7.x headers+libs. Homebrew ships 8+.
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
    # ───────────────────────────────────────────────────────────────────────

    echo "→ Building Rust library (release)..."
    cargo build --release --manifest-path "$REPO_ROOT/rust/Cargo.toml"

    echo "→ Staging dylib for FRB debug loader..."
    mkdir -p "$SANDBOX_DIR"
    cp "$DYLIB_SRC" "$SANDBOX_DIR/$DYLIB_NAME"

    rm -rf "$FFMPEG_CACHE"
  fi

  echo "→ Running integration tests on '$DEVICE'..."
  echo "  Fixtures: example/integration_test/media/"
  cd "$REPO_ROOT/example"
  flutter test integration_test/ -d "$DEVICE"
  echo ""
fi

echo "✓ Done."
