# EXIF Write Support Plan

## Goal

Stamp a JPEG file with GPS coordinates and/or `DateTimeOriginal` from a
Google Takeout sidecar, so the data lives in the file itself (visible to
Photos.app, Finder, and any other EXIF-aware tool).

Scope: **JPEG only** for this iteration. HEIC requires parsing the ISOBMFF
box structure — deferred. PNG/WebP/TIFF can be added later with the same
approach.

---

## Why no new Rust dependency

`kamadak-exif` (already in `Cargo.toml`) provides:
- `exif::Reader::read_raw(Vec<u8>)` — parses raw TIFF bytes into `Field` structs
- `exif::experimental::Writer` — serialises a list of `Field` back to TIFF bytes
- `Field` and `Value` both implement `Clone`

We parse the JPEG APP1 segment manually (the format is simple), hand the
inner TIFF block to `read_raw`, clone + filter the existing fields, inject
new GPS / datetime fields, serialise with `Writer`, and splice the new APP1
back in — without re-encoding a single pixel.

---

## JPEG APP1 layout

```
FF D8               SOI
FF E1  LL LL        APP1 marker + segment length (includes the 2 length bytes)
  45 78 69 66 00 00   "Exif\0\0" prefix (6 bytes)
  [TIFF block]        raw bytes fed to / produced by kamadak-exif
...
FF DA               SOS (scan data — no metadata markers past here)
FF D9               EOI
```

`find_exif_app1` scans from byte 2 until it hits SOS (`0xDA`) or runs out of
data, returning `(start, end)` where `bytes[start..end]` is the full APP1
segment (marker + length + data). If absent, the new APP1 is inserted right
after the SOI.

---

## Rust changes

### `rust/src/api.rs`

New public struct (FRB exposes all `pub` structs):

```rust
pub struct ExifWriteParams {
    pub latitude: Option<f64>,
    pub longitude: Option<f64>,
    pub altitude: Option<f64>,
    pub captured_at_ms: Option<i64>,  // UTC epoch ms
}
```

New public function:

```rust
pub fn write_metadata(path: String, params: ExifWriteParams) -> anyhow::Result<()>
```

Delegates to `crate::exif_writer::write_jpeg`. Returns `Ok(())` if the file
is not a JPEG (so callers don't need to branch on MIME type).

### `rust/src/exif_writer.rs` (new file)

Key steps in `write_jpeg`:

1. Read all bytes from `path`.
2. Bail out silently if not a JPEG (`!bytes.starts_with(&[0xFF, 0xD8])`).
3. `find_exif_app1(&bytes)` → `Option<(start, end)>`.
4. If found, call `exif::Reader::new().read_raw(bytes[start+10..end].to_vec())` to
   clone all existing fields, skipping:
   - All `Context::Gps` tags (when replacing GPS)
   - `Tag::DateTimeOriginal` and `Tag::DateTimeDigitized` (when replacing datetime)
5. Append new GPS fields (`GPSVersionID`, `GPSLatitudeRef`, `GPSLatitude`,
   `GPSLongitudeRef`, `GPSLongitude`, and optionally `GPSAltitudeRef` /
   `GPSAltitude`).
6. Append new `DateTimeOriginal` field formatted as `"YYYY:MM:DD HH:MM:SS"`.
7. Serialise with `exif::experimental::Writer::write(..., false)` (big-endian).
8. Build new APP1: `FF E1` + 2-byte BE length + `Exif\0\0` + TIFF bytes.
9. Splice: replace old APP1 range, or insert after SOI if absent.
10. `std::fs::write(path, new_jpeg)`.

GPS rational encoding (decimal → DMS):
```
deg = floor(|decimal|)
min = floor((|decimal| - deg) × 60)
sec = round((fractional_min × 60) × 1000)   stored as sec/1000
```

### `rust/src/lib.rs`

Add `mod exif_writer;`.

---

## Code generation

After Rust changes compile cleanly:

```bash
cd /path/to/flutter-media-metadata
bash scripts/generate.sh
```

This regenerates:
- `lib/src/rust/frb_generated.dart` — new `writeMetadata(path, params)` binding
- `rust/src/frb_generated.rs` — Rust glue

---

## Dart changes

### `lib/src/rust/api.dart` (auto-generated, do not edit)

Will contain the generated `ExifWriteParams` class and `writeMetadata()`.

### `lib/src/models/media_metadata.dart`

Add a static method:

```dart
static Future<void> write(
  String filePath, {
  double? latitude,
  double? longitude,
  double? altitude,
  DateTime? capturedAt,
}) async {
  await _ensureInit();
  await writeMetadata(
    path: filePath,
    params: ExifWriteParams(
      latitude: latitude,
      longitude: longitude,
      altitude: altitude,
      capturedAtMs:
          capturedAt != null ? capturedAt.millisecondsSinceEpoch : null,
    ),
  );
}
```

---

## Echo Frame integration (`takeout_service.dart`)

In `TakeoutService.apply()`, after `File(pair.mediaPath).copy(destPath)`:

```dart
// Stamp EXIF from sidecar into the copied file
final meta = pair.meta;
if (meta != null) {
  await MediaMetadata.write(
    destPath,
    latitude: meta.latitude,
    longitude: meta.longitude,
    altitude: meta.altitude,
    capturedAt: meta.photoTakenTime,
  );
}

// Set OS modification date so Finder shows the correct timestamp
await File(destPath).setLastModified(capturedAt);
```

`File.setLastModified` is cross-platform Dart — no OS-specific code needed.

---

## What is NOT in scope

- HEIC write (ISOBMFF container is more complex)
- PNG / WebP / TIFF write (different segment formats)
- Setting macOS file *birth* time (requires native `setattrlist` — not worth it,
  modification time is what Photos.app uses for sort)
- Batch write API (single-file write is sufficient for import flow)