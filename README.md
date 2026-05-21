<div align="center">
<img src="https://raw.githubusercontent.com/yashas-hm/media-metadata-plus/refs/heads/main/assets/git_image.png" width="80%">
</div>

A cross-platform Flutter plugin for reading media metadata from images, RAW files, and videos. Powered by Rust via
`flutter_rust_bridge` v2.

## Supported formats

| Format          | Metadata                                                         |
|-----------------|------------------------------------------------------------------|
| JPEG / JPG      | Capture time, dimensions, camera, GPS                            |
| HEIC / HEIF     | Capture time, dimensions, camera, GPS                            |
| PNG             | Capture time, dimensions                                         |
| WebP            | Capture time, dimensions, camera, GPS                            |
| TIFF            | Capture time, dimensions, camera, GPS                            |
| DNG / NEF / ARW | Capture time, dimensions, camera, GPS (reported as `image/tiff`) |
| CR2             | Capture time, dimensions, camera, GPS                            |
| MP4             | Duration, dimensions, creation time, GPS, camera                 |
| MOV             | Duration, dimensions, creation time, GPS, camera                 |

## Platform support

| Platform | Support |
|----------|---------|
| macOS    | ✅       |
| Windows  | ✅       |
| Linux    | ✅       |
| iOS      | ✅       |
| Android  | ✅       |

## Installation

```yaml
dependencies:
  media_metadata_plus: ^0.3.0
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

## Models

### `MediaMetadata`

| Field         | Type              | Description         |
|---------------|-------------------|---------------------|
| `mimeType`    | `String`          | Detected MIME type  |
| `width`       | `int?`            | Width in pixels     |
| `height`      | `int?`            | Height in pixels    |
| `capturedAt`  | `DateTime?`       | Capture time in UTC |
| `cameraMake`  | `String?`         | Camera manufacturer |
| `cameraModel` | `String?`         | Camera model        |
| `gps`         | `GpsCoordinates?` | Location            |
| `duration`    | `Duration?`       | Video duration      |

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
