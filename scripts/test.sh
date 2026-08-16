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
  # macOS SPM's binaryTarget normally pins the *published* release xcframework
  # (see macos/media_metadata_plus/Package.swift), so `flutter test -d macos`
  # would otherwise exercise last release's Rust code, not this working tree.
  # Build a local xcframework from current source and point Package.swift at
  # it via MMP_LOCAL_XCFRAMEWORK instead.
  if [[ "$DEVICE" == "macos" ]]; then
    # ── FFmpeg pre-built cache ──────────────────────────────────────────────
    # ffmpeg-sys-next needs headers/libs matching its pinned major version
    # (see rust/Cargo.toml) — Homebrew's FFmpeg is usually newer.
    FFMPEG_TAG="$(cat "$REPO_ROOT/scripts/ci/ffmpeg_prebuilt_tag")"
    FFMPEG_TARGET="aarch64-apple-darwin"
    FFMPEG_CACHE="$REPO_ROOT/.cache/ffmpeg-${FFMPEG_TAG}"
    FFMPEG_DIR_LOCAL="$FFMPEG_CACHE/$FFMPEG_TARGET"
    RELEASE_BASE="https://github.com/yashas-hm/media-metadata-plus/releases/download/ffmpeg-prebuilt-${FFMPEG_TAG}"

    if [[ ! -d "$FFMPEG_DIR_LOCAL/include" ]]; then
      echo "→ Downloading pre-built FFmpeg $FFMPEG_TAG ($FFMPEG_TARGET)..."
      mkdir -p "$FFMPEG_CACHE"
      curl -fsSL "$RELEASE_BASE/$FFMPEG_TARGET.zip" -o "$FFMPEG_CACHE/$FFMPEG_TARGET.zip"
      unzip -q "$FFMPEG_CACHE/$FFMPEG_TARGET.zip" -d "$FFMPEG_CACHE"
      rm "$FFMPEG_CACHE/$FFMPEG_TARGET.zip"
    fi
    # ───────────────────────────────────────────────────────────────────────

    echo "→ Building local xcframework for integration testing (host arch only)..."
    FFMPEG_DIR="$FFMPEG_DIR_LOCAL" cargo build --release \
      --manifest-path "$REPO_ROOT/rust/Cargo.toml" --target "$FFMPEG_TARGET"

    FW_DIR="$REPO_ROOT/rust/target/$FFMPEG_TARGET/release/media_metadata_plus.framework"
    rm -rf "$FW_DIR"
    mkdir -p "$FW_DIR/Versions/A/Resources"
    cp "$REPO_ROOT/rust/target/$FFMPEG_TARGET/release/libmedia_metadata_plus.dylib" \
      "$FW_DIR/Versions/A/media_metadata_plus"
    cat > "$FW_DIR/Versions/A/Resources/Info.plist" <<'PLIST'
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
  <key>CFBundleExecutable</key>       <string>media_metadata_plus</string>
  <key>CFBundleIdentifier</key>       <string>dev.yashashm.media-metadata-plus</string>
  <key>CFBundleInfoDictionaryVersion</key> <string>6.0</string>
  <key>CFBundlePackageType</key>      <string>FMWK</string>
  <key>CFBundleVersion</key>          <string>1</string>
  <key>CFBundleShortVersionString</key> <string>1.0</string>
</dict>
</plist>
PLIST
    ln -sf A "$FW_DIR/Versions/Current"
    ln -sf "Versions/Current/media_metadata_plus" "$FW_DIR/media_metadata_plus"
    ln -sf "Versions/Current/Resources" "$FW_DIR/Resources"
    install_name_tool -id "@rpath/media_metadata_plus.framework/media_metadata_plus" \
      "$FW_DIR/Versions/A/media_metadata_plus"

    XCFW="$REPO_ROOT/macos/Frameworks/media_metadata_plus.xcframework"
    rm -rf "$XCFW"
    mkdir -p "$REPO_ROOT/macos/Frameworks"
    xcodebuild -create-xcframework -framework "$FW_DIR" -output "$XCFW"
    # SPM's binaryTarget(path:) must be relative to the package root
    # (macos/media_metadata_plus/), not an absolute path.
    export MMP_LOCAL_XCFRAMEWORK="../Frameworks/media_metadata_plus.xcframework"

    # Force SPM to re-resolve against the local xcframework instead of any
    # cached copy of the published release.
    rm -rf "$REPO_ROOT/example/build/macos" "$REPO_ROOT/example/macos/Flutter/ephemeral"

    # example/macos/ is a generated dir (gitignored) — under App Sandbox,
    # Directory.current inside the test app resolves to the sandbox
    # container instead of example/, breaking every fixture path. Disable it
    # for the Debug configuration only (Release entitlements are untouched).
    if [[ -f "$REPO_ROOT/example/macos/Runner/DebugProfile.entitlements" ]]; then
      /usr/libexec/PlistBuddy \
        -c "Set :com.apple.security.app-sandbox false" \
        "$REPO_ROOT/example/macos/Runner/DebugProfile.entitlements"
    fi
  fi

  echo "→ Running integration tests on '$DEVICE'..."
  echo "  Fixtures: example/integration_test/fixtures/"
  cd "$REPO_ROOT/example"
  # The directory must be literally named `integration_test/` — Flutter's
  # tooling hardcodes that name to decide whether to build and launch a real
  # native app versus running as a plain host-side Dart VM test, which never
  # loads the actual compiled dylib.
  flutter test integration_test/ -d "$DEVICE"
  echo ""
fi

echo "✓ Done."
