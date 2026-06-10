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
mkdir -p "$FW/Versions/A/Resources"
cp target/macos-universal/release/libmedia_metadata_plus.dylib \
   "$FW/Versions/A/media_metadata_plus"

# Info.plist is required for codesign to recognize this as a valid bundle
cat > "$FW/Versions/A/Resources/Info.plist" <<'PLIST'
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

ln -sf A                                    "$FW/Versions/Current"
ln -sf "Versions/Current/media_metadata_plus" "$FW/media_metadata_plus"
ln -sf "Versions/Current/Resources"          "$FW/Resources"
install_name_tool -id "@rpath/media_metadata_plus.framework/media_metadata_plus" \
  "$FW/Versions/A/media_metadata_plus"

XCFW="$REPO/macos/Frameworks/media_metadata_plus.xcframework"
rm -rf "$XCFW"
mkdir -p "$REPO/macos/Frameworks"
xcodebuild -create-xcframework \
  -framework "$FW" \
  -output "$XCFW"
