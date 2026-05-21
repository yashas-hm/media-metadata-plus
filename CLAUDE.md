# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## What This Is

A cross-platform Flutter plugin that reads media metadata (EXIF, video) from JPEG, HEIC, MP4, MOV, PNG, and WebP files. The native layer is written entirely in Rust and exposed to Dart via `flutter_rust_bridge` v2. No Swift, Kotlin, or C++ is involved — `ffiPlugin: true` in `pubspec.yaml` tells Flutter to compile and link the Rust library automatically.

## Commands

```bash
# Install Dart dependencies
flutter pub get

# Generate Dart FFI bindings from Rust API surface (run after changing rust/src/api.rs)
bash scripts/generate.sh

# Run tests
flutter test

# Run a single test file
flutter test test/flutter_media_metadata_test.dart

# Run example app on macOS
cd example && flutter run -d macos

# Lint
flutter analyze
```

## Architecture

### Data flow
```
File path (Dart)
  → MediaMetadata.read(path)          # lib/src/api/media_metadata_api.dart
  → FFI call via generated bindings   # lib/src/bridge/frb_generated.dart
  → Rust: read_metadata(path)         # rust/src/api.rs
      → mime::detect()                # magic-byte format detection
      → exif_reader::read()           # JPEG, HEIC, PNG, WebP
      OR video_reader::read()         # MP4, MOV
  → MediaMeta struct → Dart MediaMetadata model
```

### Dart layer (`lib/`)
Feature code lives in `src/<feature>/` subdirectories:
- `src/models/` — `MediaMetadata`, `GpsCoordinates` (pure Dart, no FFI)
- `src/api/` — `MediaMetadata.read()` static method that calls into the bridge (to be added)
- `src/bridge/` — `frb_generated.dart` (codegen output, do not edit manually)

`lib/flutter_media_metadata.dart` is the public barrel — only add exports here.

### Rust layer (`rust/`)
- `src/api.rs` — the FFI surface; only `pub` items here become Dart functions
- `src/mime.rs` — detects format from the first 16 bytes (not file extension)
- `src/exif_reader.rs` — reads EXIF from JPEG, HEIC, PNG, WebP via `kamadak-exif`
- `src/video_reader.rs` — reads duration/dimensions/creation time from MP4/MOV via `mp4` crate

### Key design decisions
- Format detected by **magic bytes**, not extension — extension can be wrong or absent
- HEIC metadata is read from the EXIF box without decoding the image (no libheif)
- All timestamps stored as UTC unix epoch milliseconds (`i64`) in Rust, converted to `DateTime` in Dart
- GPS stored as flat lat/lon/alt on the Rust struct, wrapped into `GpsCoordinates` on the Dart side
- MP4 creation time uses the 1904 epoch; offset `2082844800` converts to Unix epoch

### Platform wiring
Each platform directory contains only a build file that links the compiled Rust artifact. No platform-specific logic lives there.
- macOS / iOS: `.podspec` with `ffiPlugin` vendored framework
- Windows / Linux: `CMakeLists.txt`
- Android: `build.gradle` linking the `.so`

## Dart import style

Always use package imports, never relative imports:

```dart
// correct
import 'package:flutter_media_metadata/src/models/gps_coordinates.dart';

// wrong
import '../models/gps_coordinates.dart';
import 'gps_coordinates.dart';
```

## Rust crates
```toml
kamadak-exif = "0.5"   # EXIF parsing for JPEG, HEIC, PNG, WebP
mp4 = "0.14"           # MP4/MOV metadata
flutter_rust_bridge = "2"
anyhow = "1"
```