#!/usr/bin/env bash
# Builds the Rust dylib, wires the framework stub frb expects, then runs the
# inspect test. Pass one or more file paths as arguments.
#
# Usage:
#   bash scripts/inspect.sh /path/to/file.jpg
#   bash scripts/inspect.sh /a.jpg /b.mp4

set -euo pipefail

if [[ $# -eq 0 ]]; then
  echo "Usage: bash scripts/inspect.sh /path/to/file [...]"
  exit 1
fi

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
FRAMEWORK_DIR="$REPO_ROOT/example/media_metadata_plus.framework"

cleanup() { rm -rf "$FRAMEWORK_DIR"; }
trap cleanup EXIT

# Build Rust dylib
echo "→ Building Rust library..."
cargo build --manifest-path "$REPO_ROOT/rust/Cargo.toml" 2>&1

# Create the .framework stub that flutter_rust_bridge looks for
mkdir -p "$FRAMEWORK_DIR"
cp "$REPO_ROOT/rust/target/debug/libmedia_metadata_plus.dylib" \
   "$FRAMEWORK_DIR/media_metadata_plus"

# Join paths with comma for MEDIA_PATH
MEDIA_PATH="$(IFS=','; echo "$*")"

echo "→ Running inspect test..."
cd "$REPO_ROOT/example"
MEDIA_PATH="$MEDIA_PATH" flutter test test/inspect.dart --reporter expanded