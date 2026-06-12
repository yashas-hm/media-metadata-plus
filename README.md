<div align="center">
<img src="https://raw.githubusercontent.com/yashas-hm/media-metadata-plus/refs/heads/main/image_asset.png" width="80%">
</div>

A cross-platform Flutter plugin for reading media metadata from images, RAW files, and videos, and extracting video
thumbnails. Powered by Rust via `flutter_rust_bridge` v2.

## Supported formats

| Format          | Metadata                                                                      |
|-----------------|-------------------------------------------------------------------------------|
| JPEG / JPG      | Capture time, modified time, dimensions, camera, GPS                          |
| HEIC / HEIF     | Capture time, modified time, dimensions, camera, GPS                          |
| PNG             | Capture time, modified time, dimensions                                       |
| WebP            | Capture time, modified time, dimensions, camera, GPS                          |
| TIFF            | Capture time, modified time, dimensions, camera, GPS                          |
| DNG / NEF / ARW | Capture time, modified time, dimensions, camera, GPS (reported as `image/tiff`) |
| CR2             | Capture time, modified time, dimensions, camera, GPS                          |
| MP4             | Duration, dimensions, creation time, modified time, GPS, camera               |
| MOV             | Duration, dimensions, creation time, modified time, GPS, camera               |

## Platform support

| Platform | Support |
|----------|---------|
| macOS    | ✅       |
| Windows  | ✅       |
| Linux    | ✅       |
| iOS      | ✅       |
| Android  | ✅       |

## Installation

```bash
flutter pub add media_metadata_plus
```

## Usage

```dart
import 'package:media_metadata_plus/media_metadata_plus.dart';

void main() async {
  final meta = await MediaMetadata.read('/path/to/file.heic');

  print(meta?.mimeType); // "image/heic"
  print(meta?.width); // 4032
  print(meta?.height); // 3024
  print(meta?.capturedAt); // 2024-03-15 10:30:00.000Z
  print(meta?.modifiedAt); // 2024-03-16 08:00:00.000Z
  print(meta?.cameraMake); // "Apple"
  print(meta?.cameraModel); // "iPhone 15 Pro"
  print(meta?.gps?.lat); // 37.42195
  print(meta?.gps?.lon); // -122.08408
  print(meta?.gps?.alt); // 15.3
  print(meta?.duration); // null (images) or Duration for video
}
```

`MediaMetadata.read()` returns `null` for unsupported formats or corrupt files — no exceptions to handle.

No initialisation call required. The Rust library is loaded automatically on first use.

## Video thumbnails

```dart
// Extract a JPEG thumbnail from a video file
final bytes = await MediaMetadata.generateThumbnail('/path/to/video.mp4');
if (bytes != null) {
  final image = Image.memory(bytes);
}

// Seek to a specific position (milliseconds)
final bytes = await MediaMetadata.generateThumbnail(
  '/path/to/video.mp4',
  timeMs: 5000,
);

// Write to disk at the same time
final bytes = await MediaMetadata.generateThumbnail(
  '/path/to/video.mp4',
  savePath: '/tmp/thumb.jpg',
);
```

`generateThumbnail` uses a two-stage approach:

1. **Fast path** — reads the embedded cover-art image from the file's `covr` iTunes atom. No video decoding, covers most iPhone and iPad footage.
2. **FFmpeg fallback** — when no embedded thumbnail is present, decodes one frame at 10 % of the video duration (or `timeMs`). Supports H.264, HEVC, VP9, AV1, and MPEG-4 — covers Android, DSLRs, GoPro, DJI, and any other source.

Output is scaled to a maximum width of 640 px preserving aspect ratio. Returns `null` if no thumbnail can be extracted.

## API

### `MediaMetadata.read(path)`

Reads metadata from a single file. Returns `null` for unsupported formats or corrupt files.

### `MediaMetadata.readAll(paths)`

Reads metadata from multiple files in parallel (Rayon on the Rust side). Returns a `List<MediaMetadata?>` in input order.

### `MediaMetadata.generateThumbnail(path, {int? timeMs, String? savePath})`

Extracts a JPEG thumbnail from a video file. Returns `null` if no thumbnail can be extracted.

## Models

### `MediaMetadata`

| Field         | Type              | Description                              |
|---------------|-------------------|------------------------------------------|
| `mimeType`    | `String`          | Detected MIME type                       |
| `width`       | `int?`            | Width in pixels                          |
| `height`      | `int?`            | Height in pixels                         |
| `capturedAt`  | `DateTime?`       | Capture time in UTC                      |
| `modifiedAt`  | `DateTime?`       | Last-modified time in UTC                |
| `cameraMake`  | `String?`         | Camera manufacturer                      |
| `cameraModel` | `String?`         | Camera model                             |
| `gps`         | `GpsCoordinates?` | Location                                 |
| `duration`    | `Duration?`       | Video duration                           |

### `GpsCoordinates`

| Field | Type      | Description                 |
|-------|-----------|-----------------------------|
| `lat` | `double`  | Latitude (negative = south) |
| `lon` | `double`  | Longitude (negative = west) |
| `alt` | `double?` | Altitude in metres          |

## How it works

Format is detected from the first 16 bytes of the file (magic bytes), not the file extension. HEIC metadata is read
directly from the EXIF box without decoding the image. The entire native layer is Rust — no Swift, Kotlin, or C++
platform code is involved.

## Contributing

Contributions are welcome! Please open an issue first to discuss what you'd like to change. Pull requests should:

- target the `main` branch
- include tests for any new behaviour
- keep the Rust and Dart layers in sync (run `bash scripts/generate.sh` after changing `rust/src/api.rs`)

Please adhere to our [Code of Conduct](CODE_OF_CONDUCT.md) when interacting with the project.

---

## 🔒 Security

If you discover any security vulnerabilities, please report them
via [yashashm.dev@gmail.com](mailto:yashashm.dev@gmail.com). We take security issues seriously and appreciate your
efforts to responsibly disclose them. Read more at [SECURITY](SECURITY.md)

---

## 📜 Code of Conduct

This project is governed by a [Code of Conduct](CODE_OF_CONDUCT.md). By participating, you agree to uphold a welcoming
and inclusive environment.

---

## 📝 License

Project is licensed under the [MIT License](LICENSE).
