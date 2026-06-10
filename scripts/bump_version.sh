#!/usr/bin/env bash
set -euo pipefail

read -rp "New version (e.g. 2.1.0): " NEW_VERSION
NEW_VERSION="${NEW_VERSION#v}"  # strip leading 'v' if provided

REPO="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"

OLD_VERSION=$(grep '^version:' "$REPO/pubspec.yaml" | sed 's/version: //')

if [ "$OLD_VERSION" = "$NEW_VERSION" ]; then
  echo "Already at $NEW_VERSION, nothing to do."
  exit 0
fi

echo "Bumping $OLD_VERSION → $NEW_VERSION"

# macOS sed requires empty string after -i; Linux does not
SED_I=(-i '')
[[ "$(uname)" != "Darwin" ]] && SED_I=(-i)

sed "${SED_I[@]}" "s/${OLD_VERSION}/${NEW_VERSION}/g" \
  "$REPO/pubspec.yaml" \
  "$REPO/rust/Cargo.toml" \
  "$REPO/ios/media_metadata_plus.podspec" \
  "$REPO/macos/media_metadata_plus.podspec" \
  "$REPO/android/build.gradle.kts" \
  "$REPO/macos/media_metadata_plus/Package.swift" \
  "$REPO/ios/media_metadata_plus/Package.swift"

cd "$REPO"
git add \
  pubspec.yaml \
  rust/Cargo.toml \
  ios/media_metadata_plus.podspec \
  macos/media_metadata_plus.podspec \
  android/build.gradle.kts \
  macos/media_metadata_plus/Package.swift \
  ios/media_metadata_plus/Package.swift

git commit -m "chore: bump version to $NEW_VERSION"
git tag "v${NEW_VERSION}"
git push
git push origin "v${NEW_VERSION}"

echo "Done — pushed commit and tag v${NEW_VERSION}"