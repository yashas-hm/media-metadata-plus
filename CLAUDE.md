# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## What This Is

A cross-platform Flutter plugin that reads media metadata (EXIF, video) from JPEG, HEIC, PNG, WebP, TIFF, DNG, NEF,
ARW, CR2, MP4, and MOV files, and extracts video thumbnails. The native layer is written entirely in Rust and
exposed to Dart via `flutter_rust_bridge` v2 — no Swift, Kotlin, or C++ platform code is involved.

Consumers never compile Rust. Each platform ships a **prebuilt** native binary (`.xcframework`, `.so`, `.dll`,
`.jniLibs`) built by CI and fetched at `flutter pub get` / `pod install` / Gradle sync time. See
[`docs/decisions.md`](docs/decisions.md) for why (no Rust toolchain requirement for consumers).

## Commands

```bash
# Install Dart dependencies
flutter pub get

# Generate Dart FFI bindings from Rust API surface (run after changing rust/src/api.rs)
bash scripts/generate.sh

# Rust lint / format (not currently enforced in CI — run before committing Rust changes)
cd rust && cargo fmt && cargo clippy

# Dart lint / format
dart format .
dart analyze
cd example && flutter analyze

# Unit tests (Dart-only, no native library needed)
flutter test
flutter test test/media_metadata_plus_test.dart   # single file
cd rust && cargo test                              # Rust unit tests (container/isobmff.rs etc.)

# Integration tests (exercise the real native library against fixture files)
bash scripts/test.sh --unit                        # unit only
bash scripts/test.sh --integration                 # integration on macOS
bash scripts/test.sh --integration -d linux         # specific device
bash scripts/test.sh                                # both

# Run example app
cd example && flutter run -d macos
```

## Architecture

### Data flow

```
File path (Dart)
  → MediaMetadata.read(path)          # lib/src/models/media_metadata.dart
  → FFI call via generated bindings   # lib/src/rust/frb_generated.dart
  → Rust: read_metadata(path)         # rust/src/api.rs
      → mime::detect()                # magic-byte format detection
      → readers::exif::read()         # JPEG, HEIC, PNG, WebP, TIFF/RAW
      OR readers::video::read()       # MP4, MOV
  → MediaMeta struct → Dart MediaMetadata model
```

### Dart layer (`lib/`)

- `src/models/` — `MediaMetadata`, `GpsCoordinates` (pure Dart, no FFI)
- `src/rust/` — `frb_generated.dart` and generated API (codegen output, gitignored — regenerate with
  `scripts/generate.sh`, do not edit manually)

`lib/media_metadata_plus.dart` is the public barrel — only add exports here.

### Rust layer (`rust/src/`)

- `api.rs` — the FFI surface; only `pub` items here become Dart functions (edit this, then run
  `scripts/generate.sh`)
- `mime.rs` — detects format from the first 16 bytes, not the file extension
- `container/isobmff.rs` — generic ISOBMFF (MP4/MOV/HEIF) box-tree walking, format-agnostic. Has unit tests
  (`cargo test`) — prefer adding coverage here over the slower Flutter integration suite for box-parsing logic
- `readers/exif.rs` — EXIF for JPEG/HEIC/PNG/WebP/TIFF-family via `kamadak-exif`
- `readers/video.rs` — MP4/MOV metadata via the `mp4` crate; falls back to manual `container::isobmff` parsing
  of `mvhd`/`tkhd` for QuickTime files whose audio-track sound description that crate can't parse (see the
  `read()` doc comment). Also owns the iTunes/3GPP metadata-atom reader (camera make/model, GPS, `covr`
  thumbnail extraction) — these are video-file-specific consumers of `container::isobmff`, not generic
  primitives, which is why they live here rather than in `container/`
- `readers/thumbnail.rs` — FFmpeg-backed frame decode/scale/rotate/encode fallback when no `covr` atom exists

### Platform wiring

Each platform directory links the **prebuilt** binary — none contain hand-written native logic:

- macOS / iOS: `Package.swift` `binaryTarget` (Swift Package Manager) pointing at a published GitHub Release
  `.xcframework.zip`, pinned by URL + checksum. The `.podspec` exists for CocoaPods-only projects.
- Android: `build.gradle.kts` linking `.so` files from `src/main/jniLibs/<abi>/`
- Windows / Linux: `CMakeLists.txt` linking a committed `.dll` / `.so`

**Non-obvious gotcha:** because macOS/iOS pin a *published* release, `flutter test -d macos` against a local
Flutter checkout normally exercises last release's Rust code, not your working tree. `scripts/test.sh` works
around this by building a local `.xcframework` and setting `MMP_LOCAL_XCFRAMEWORK` (read by `Package.swift`) to
point at it instead — this only affects local/dev builds; the published/pub.dev/CI-release path is unchanged
when that env var is unset. See [`docs/decisions.md`](docs/decisions.md) for the full story.

### Testing layout

- `test/` — pure-Dart unit tests, no native library, run anywhere
- `example/integration_test/` — exercises the real native library against real fixture files in
  `example/integration_test/media/`. Must live under `example/` (a runnable Flutter app) rather than the plugin
  root, because `integration_test` needs a device/simulator/desktop target to launch — the plugin package itself
  has no runnable app. CI (`.github/workflows/integration_test.yml`) runs this on Linux only, since Linux builds
  its `.so` from source in-workflow and isn't affected by the macOS SPM-pinning gotcha above.

### Key design decisions

- Format detected by **magic bytes**, not extension — extension can be wrong or absent
- HEIC metadata is read from the EXIF box inside the HEIF container without decoding the image (no libheif)
- All timestamps stored as UTC unix epoch milliseconds (`i64`) in Rust, converted to `DateTime` in Dart
- GPS stored as flat lat/lon/alt on the Rust struct, wrapped into `GpsCoordinates` on the Dart side
- MP4/MOV creation time uses the 1904 epoch; offset `2082844800` converts to Unix epoch
- FFmpeg is vendored as a prebuilt, LGPL-only static build (`--disable-gpl --disable-nonfree`) served from a
  GitHub Release, because no existing prebuilt source covers this plugin's full platform matrix under an
  LGPL-only configuration

Full rationale for these and other architectural choices (FFmpeg licensing, prebuilt-binary distribution, SPM
migration pain) is in [`docs/decisions.md`](docs/decisions.md) — read it before revisiting any of them.

## Workflow

After completing a significant feature (new format support, new API method, platform change, breaking change), remind
the user to commit before starting the next task.

## Dart import style

Always use package imports, never relative imports:

```dart
// correct
import 'package:media_metadata_plus/src/models/gps_coordinates.dart';

// wrong
import '../models/gps_coordinates.dart';
import 'gps_coordinates.dart';
```

## Rust crates

```toml
flutter_rust_bridge = "=2.12.0"
kamadak-exif = "0.5"   # EXIF parsing for JPEG, HEIC, PNG, WebP, TIFF-family
mp4 = "0.14"           # MP4/MOV metadata (ISO-only; see readers/video.rs fallback note)
ffmpeg-next = "7"      # video frame decode for thumbnail fallback (LGPL-only build)
jpeg-encoder = "0.7"   # thumbnail JPEG encoding
chrono = "0.4"
rayon = "1"            # parallel batch reads in read_metadata_batch
anyhow = "1"
```
