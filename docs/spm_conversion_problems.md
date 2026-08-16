# SPM Conversion: Problems Encountered

This document records every distinct problem encountered while adding Swift Package Manager (SPM) support to `media_metadata_plus`. Problems are grouped by domain. Each entry describes what broke, why, and what fixed it.

---

## 1. Core Architectural Constraint — SPM Cannot Compile Rust

**What broke:** Flutter's `ffiPlugin: true` tells CocoaPods to inject a `cargo build` step into the Xcode project. SPM has no equivalent hook. SPM is declarative and sandboxed — it only understands Swift, C, C++, ObjC, and Assembly source targets, and pre-compiled binary `xcframework` targets. There is no mechanism to run arbitrary shell commands during an SPM build.

**Root cause:** CocoaPods generates Xcode projects and allows `prepare_command` and custom build phases. SPM's `Package.swift` is pure data — it describes what to build, not how. Running `cargo build` inside SPM is fundamentally unsupported.

**Fix:** Adopt a pre-built binary distribution model. The CI pipeline cross-compiles the Rust crate and produces `.xcframework` archives, which are uploaded to GitHub Releases. `Package.swift` references them as `binaryTarget(name:url:checksum:)`. Consumers never need Rust installed.

---

## 2. Broken Hybrid State — `hook/build.dart` + `ffiPlugin: true` Coexisting

**What broke:** Before the SPM work began, the plugin had both `hook/build.dart` (native assets via `native_toolchain_rust`) and `ffiPlugin: true` active simultaneously. Both paths tried to compile Rust, both required Rust installed on the consumer's machine, and they conflicted with each other.

**Root cause:** `hook/build.dart` was added experimentally for native assets support but never fully replaced `ffiPlugin: true`. The plugin was published in this broken hybrid state.

**Fix:** Remove `hook/build.dart`, the `hooks` key, and `native_toolchain_rust` from `pubspec.yaml`. Keep `ffiPlugin: true` for Windows and Linux (where pre-built binaries are referenced via CMakeLists). Replace CocoaPods Rust compilation on macOS/iOS with `vendored_frameworks` pointing to pre-built XCFrameworks.

---

## 3. CocoaPods Rejects Raw `.dylib` in XCFramework

**What broke:** The initial `build_macos.sh` used `xcodebuild -create-xcframework -library libmedia_metadata_plus.dylib` (raw dylib slice). CocoaPods rejected this during consumer app builds.

**Root cause:** CocoaPods requires the xcframework to contain a `.framework` bundle, not a raw `.dylib`. The `-library` flag is for static libraries (`.a`). Dynamic libraries on macOS must be wrapped in a versioned `.framework` directory structure before being passed to `xcodebuild`.

**Required framework layout:**
```
media_metadata_plus.framework/
├── media_metadata_plus          ← symlink → Versions/Current/media_metadata_plus
├── Resources                    ← symlink → Versions/Current/Resources
└── Versions/
    ├── Current                  ← symlink → A
    └── A/
        ├── media_metadata_plus  ← the actual dylib
        └── Resources/
            └── Info.plist
```

**Fix:** `build_macos.sh` now creates this full directory structure, sets the install name via `install_name_tool`, and calls `xcodebuild -create-xcframework -framework` instead of `-library`.

---

## 4. Missing `Info.plist` — Codesign Rejects Bundle as Unrecognized Format

**What broke:** Even after wrapping the dylib in a `.framework`, macOS codesign failed:

```
bundle format unrecognized, invalid, or unsuitable
Command CodeSign failed with a nonzero exit code
```

**Root cause:** `codesign` requires `Info.plist` to be present at `Versions/A/Resources/Info.plist`. Without it the directory is not recognized as a valid macOS bundle.

The `Info.plist` must contain at minimum:
- `CFBundleExecutable` — must match the dylib filename
- `CFBundleIdentifier` — reverse-DNS identifier
- `CFBundlePackageType` — `FMWK`
- `CFBundleVersion` — any non-empty string

**Fix:** `build_macos.sh` now creates `Versions/A/Resources/Info.plist` with the required keys before building the xcframework.

---

## 5. Symlinks Not Preserved When Zipping — `Versions/Current` Broken After Download

**What broke:** After CI uploaded the xcframework zip to GitHub Releases and SPM downloaded it, Xcode failed to resolve the framework:

```
Couldn't resolve framework symlink for '.../media_metadata_plus.framework/Versions/Current':
Error Domain=NSPOSIXErrorDomain Code=22 "Invalid argument"
```
```
code object is not signed at all
In subcomponent: .../media_metadata_plus.framework
```

**Root cause:** `zip -r` dereferences symlinks by default, storing `Versions/Current` as a plain directory copy of `Versions/A` rather than as a symlink. When Xcode calls `readlink()` on it at build time, it gets `EINVAL` (not a symlink), framework resolution fails, and codesign cannot sign the broken bundle.

**Fix:** Pass `-y` to `zip` to preserve symlinks:
```bash
zip -r -y "macos_${VERSION}.xcframework.zip" media_metadata_plus.xcframework
zip -r -y "ios_${VERSION}.xcframework.zip"   media_metadata_plus.xcframework
```

---

## 6. Wrong `Package.swift` Location

**What broke:** The initial `Package.swift` files were placed at `macos/Package.swift` and `ios/Package.swift`. The pub.dev package scoring tool reported:

```
Package does not support the Swift Package Manager on macOS
The package does not contain macos/media_metadata_plus/Package.swift.

Package does not support the Swift Package Manager on iOS
The package does not contain ios/media_metadata_plus/Package.swift.
```

**Root cause:** Flutter's SPM tooling expects `Package.swift` inside a subdirectory named after the plugin — `macos/media_metadata_plus/Package.swift` and `ios/media_metadata_plus/Package.swift`. Placing it one level higher makes Flutter ignore it entirely.

**Fix:** Moved both files into the correctly named subdirectories and updated all relative paths inside them (`path: "Classes"` → `path: "../Classes"`, resource paths adjusted accordingly). Updated CI `sed` and `git add` commands to reference the new paths.

---

## 7. `.pubignore` Unanchored Pattern Excluded Generated Dart FFI Bindings

**What broke:** After publishing, pub.dev reported 0 platform support and static analysis failed:

```
ERROR: Target of URI doesn't exist: 'package:media_metadata_plus/src/rust/api.dart'
ERROR: Target of URI doesn't exist: 'package:media_metadata_plus/src/rust/frb_generated.dart'
UNDEFINED_IDENTIFIER: Undefined name 'RustLib'
```

And dartdoc failed:
```
dartdoc failed: fatal error: unable to locate the input directory at '/tmp/pana_.../lib/src/rust'
```

**Root cause:** `.pubignore` contained the pattern `rust/` without a leading `/`. In `.pubignore` (which follows `.gitignore` glob rules), this matches any directory named `rust` at **any depth** in the tree — including `lib/src/rust/`, which contains the generated Dart FFI bindings (`api.dart`, `frb_generated.dart`) that consumers need. The Rust source at the repo root was correctly excluded, but so was the Dart layer.

**Fix:** Anchor all root-level exclusions with a leading `/`:
```
# wrong — matches lib/src/rust/ as well
rust/

# correct — only matches /rust/ at repo root
/rust/
```

The same fix was applied to `/Cargo.lock`, `/scripts/`, and other root-level paths.

---

## 8. SPM Product Name Mismatch

**What broke:** After placing `Package.swift` in the correct location, the consumer app still failed to build with SPM enabled:

```
product 'media-metadata-plus' required by package 'fluttergeneratedpluginswiftpackage'
target 'FlutterGeneratedPluginSwiftPackage' not found in package 'media_metadata_plus'
```

**Root cause:** Flutter's SPM tooling generates `FlutterGeneratedPluginSwiftPackage` which depends on the plugin's product using the plugin name with hyphens (`media-metadata-plus`). The initial `Package.swift` exposed the product as `media_metadata_plus` (underscores) — the Dart/pub convention. SPM product names use hyphens for multi-word names.

**Fix:**
```swift
// wrong
.library(name: "media_metadata_plus", targets: ["media_metadata_plus"])

// correct — product uses hyphens, target uses underscores (valid Swift identifier)
.library(name: "media-metadata-plus", targets: ["media_metadata_plus"])
```

---

## 9. CI Build Script Path Resolution

**What broke:** First CI run failed immediately:

```
scripts/ci/build_macos.sh: line 4: cd: .../scripts/rust: No such file or directory
```

**Root cause:** The scripts used `$(dirname "${BASH_SOURCE[0]}")"/..` to find the repo root. From `scripts/ci/`, one level up is `scripts/` — not the repo root. The Rust source is at `<repo-root>/rust/`, not `scripts/rust/`.

**Fix:** Change `"/..` to `"/../..` in all three build scripts:
```bash
# wrong
REPO="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"

# correct
REPO="$(cd "$(dirname "${BASH_SOURCE[0]}")/../../" && pwd)"
```

---

## 10. Rust Toolchain Working Directory in CI

**What broke:** CI installed Rust cross-compilation targets at the repo root level, then `cargo build` ran inside `rust/` and failed because the targets were not available for the active toolchain.

**Root cause:** `rust/rust-toolchain.toml` pins the channel to `1.95.0`. Running `rustup target add` from the repo root meant rustup used the default (stable) toolchain. When `cargo build` then ran from `rust/` it switched to `1.95.0` (due to `rust-toolchain.toml`), which didn't have the targets installed.

**Fix:** Add `working-directory: rust` to the `rustup target add` step in the GitHub Actions workflow so rustup sees `rust-toolchain.toml` and installs targets into the correct toolchain:
```yaml
- name: Install Rust targets
  working-directory: rust
  run: |
    rustup target add aarch64-apple-darwin
    rustup target add x86_64-apple-darwin
    ...
```

---

## 11. CI Workflow Ordering — Package.swift Commit Before pub.dev Publish

**What broke:** `flutter pub publish --dry-run` printed:

```
Package validation found the following potential issue:
* 2 checked-in files are modified in git.
  Modified files:
  ios/media_metadata_plus/Package.swift
  macos/media_metadata_plus/Package.swift
```

**Root cause:** The workflow stamped the real checksums into `Package.swift` via `sed`, then attempted to publish before committing those changes. `flutter pub publish` validates that the working tree matches the last git commit, so unstaged modifications to tracked files cause a warning that can fail the publish.

**Fix:** Move the `git commit` step for `Package.swift` to *before* the `--dry-run` step, not after it. The tarball then includes the committed, correct `Package.swift`.

---

## 12. GitHub Release Tag Creation for Manual Dispatch

**What broke:** The `workflow_dispatch` path (used for local testing without pub.dev publish) failed because the release creation step needed a tag that didn't exist on the remote for manually-triggered runs.

**Root cause:** Tag-push triggers always create the tag before the workflow runs. Manual dispatch does not — `github.ref_name` is the branch name, not a version tag.

**Fix:** Add an "Ensure tag exists" step that fetches existing tags, checks if the target tag already exists, and creates + pushes it only if absent:
```bash
git fetch --tags
if ! git rev-parse "refs/tags/v${VERSION}" >/dev/null 2>&1; then
  git tag "v${VERSION}"
  git push origin "v${VERSION}"
fi
```

---

## 13. Double `v` Prefix in Release Asset Names

**What broke:** Release assets were named `macos_vv2.0.0.xcframework.zip` instead of `macos_v2.0.0.xcframework.zip`.

**Root cause:** `github.ref_name` for a tag push returns the full tag name including the `v` prefix (e.g. `v2.0.0`). `env.VERSION` was set to `github.ref_name`, so it already contained `v`. Lines in the workflow that built filenames with `v${{ env.VERSION }}` prepended a second `v`.

**Fix:** Use `${{ env.VERSION }}` directly (not `v${{ env.VERSION }}`) in filename construction. For `workflow_dispatch`, require users to include the `v` in the version input.

---

## 14. `dart pub publish` Fails for Flutter Packages

**What broke:**

```
Because media_metadata_plus requires the Flutter SDK, version solving failed.
Flutter users should use `flutter pub` instead of `dart pub`.
```

**Root cause:** The initial publish step used `dart-lang/setup-dart` and called `dart pub publish`. Packages with `sdk: flutter` in `pubspec.yaml` require the Flutter SDK context for dependency resolution. `dart pub` doesn't set that up.

**Fix:** Use `subosito/flutter-action` to install Flutter and call `flutter pub publish`. The `dart-lang/setup-dart` action is still needed for OIDC credential exchange with pub.dev — use both, with `flutter-action` first and `setup-dart` last so its credential setup is not overwritten.

---

## 15. pub.dev OIDC Authentication Failures

**What broke:** `flutter pub publish` in CI opened a browser OAuth flow instead of completing automatically:

```
Pub needs your authorization to upload packages on your behalf.
In a web browser, go to https://accounts.google.com/...
Waiting for your authorization...
```

**Root cause (multiple compounding issues):**

1. **Stale `PUB_TOKEN` secret** — a `PUB_TOKEN` secret existed in GitHub repo settings from a previous manual publish. This credential takes precedence over OIDC. If expired or malformed, pub.dev falls back to interactive browser auth.

2. **Automated publishing not re-configured after publisher transfer** — the package was transferred to a pub.dev publisher after automated publishing was configured. Transferring a package resets the automated publishing configuration silently. It must be re-enabled under the new publisher context.

3. **Missing `id-token: write` on the job** — inheriting the permission from the top-level `permissions` block is not always sufficient. It must be declared explicitly on the `release-and-publish` job.

4. **Missing `environment:` claim** — pub.dev's automated publishing can be configured to require a specific GitHub environment name as an OIDC claim. If set on pub.dev but not declared on the job, the token claim mismatch causes auth failure.

**Fix:**
- Delete the `PUB_TOKEN` secret from GitHub repo settings
- Re-enable automated publishing on pub.dev after any package transfer
- Declare `id-token: write` explicitly on the publish job
- Add `environment: pub.dev` to the job and create the matching environment in GitHub repo settings

---

## 16. `flutter test` Cannot Load the Native Library

**What broke:** Running `flutter test tool/inspect.dart` (a local debugging tool for reading arbitrary media files) failed:

```
Invalid argument(s): Failed to load dynamic library 'media_metadata_plus.framework/media_metadata_plus':
dlopen(media_metadata_plus.framework/media_metadata_plus, 0x0001): tried: ... (no such file)
```

**Root cause:** `flutter test` does not trigger the CocoaPods or Xcode build pipeline. The `.framework` containing the compiled Rust library is only produced by `flutter build macos` or `flutter run`. Running tests from outside the example app (or even from within `example/test/`) does not build the native artifact.

**Fix:** A wrapper shell script (`scripts/inspect.sh`) that:
1. Runs `cargo build` directly to produce the dylib
2. Creates the `.framework` stub directory at the relative path `frb`'s loader expects
3. Copies the dylib into the stub
4. Runs `flutter test` with the `MEDIA_PATH` env var
5. Cleans up the stub on exit

---

## Summary Table

| # | Problem | Domain | Fixed By |
|---|---------|--------|----------|
| 1 | SPM can't run `cargo build` | Architecture | Pre-built XCFramework binary targets |
| 2 | `hook/build.dart` + `ffiPlugin: true` conflict | Architecture | Remove native assets, use vendored frameworks |
| 3 | Raw `.dylib` rejected by CocoaPods | Framework build | Wrap dylib in versioned `.framework` bundle |
| 4 | Missing `Info.plist` — codesign failure | Framework build | Add `Info.plist` to `Versions/A/Resources/` |
| 5 | Symlinks not preserved in ZIP | Framework build | `zip -r -y` flag |
| 6 | `Package.swift` in wrong directory | Project structure | Move to `platform/media_metadata_plus/Package.swift` |
| 7 | `.pubignore` excludes `lib/src/rust/` | Publishing | Anchor patterns with leading `/` |
| 8 | SPM product name uses underscores | SPM integration | Rename product to `media-metadata-plus` (hyphens) |
| 9 | CI build scripts use wrong repo root path | CI | `../..` instead of `..` |
| 10 | Rust targets installed into wrong toolchain | CI | `working-directory: rust` on rustup step |
| 11 | Package.swift commit after publish | CI | Reorder: commit before `--dry-run` |
| 12 | Tag missing for manual dispatch | CI | "Ensure tag exists" step |
| 13 | Double `v` prefix in asset names | CI | Remove extra `v` from filename interpolation |
| 14 | `dart pub` fails for Flutter package | Publishing | Use `flutter pub publish` |
| 15 | OIDC auth failure (multiple causes) | Publishing | Delete stale secret, re-configure after transfer, explicit job permissions |
| 16 | `flutter test` can't load native library | Testing | Wrapper script that manually builds dylib first |
