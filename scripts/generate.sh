#!/usr/bin/env bash
# Generates the flutter_rust_bridge Dart/Rust FFI bindings.
# Run this after any change to rust/src/api.rs.
#
# Output:
#   lib/src/rust/frb_generated.dart   (Dart side — gitignored, loaded by MediaMetadata.read)
#   rust/src/frb_generated.rs         (Rust side — glue code)

set -euo pipefail

# cargo installs binaries to ~/.cargo/bin which may not be in PATH in all shells
export PATH="$HOME/.cargo/bin:$PATH"

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"

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
