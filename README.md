# flutter_media_metadata

A cross-platform Flutter plugin for reading media metadata from JPEG, HEIC, PNG, WebP, MP4, and MOV files. Powered by Rust via `flutter_rust_bridge` v2.

## Supported formats

| Format | Metadata |
|--------|----------|
| JPEG / JPG | Capture time, dimensions, camera, GPS |
| HEIC / HEIF | Capture time, dimensions, camera, GPS |
| PNG | Capture time, dimensions |
| WebP | Capture time, dimensions |
| MP4 | Duration, dimensions, creation time |
| MOV | Duration, dimensions, creation time |

## Platform support

| Platform | Support |
|----------|---------|
| macOS | ✅ |
| Windows | ✅ |
| Linux | ✅ |
| iOS | ✅ |
| Android | ✅ |

## Installation

```yaml
dependencies:
  flutter_media_metadata: ^0.1.0
```

## Usage

```dart
import 'package:flutter_media_metadata/flutter_media_metadata.dart';

final meta = await MediaMetadata.read('/path/to/file.heic');

print(meta?.mimeType);      // "image/heic"
print(meta?.width);         // 4032
print(meta?.height);        // 3024
print(meta?.capturedAt);    // 2024-03-15 10:30:00.000Z
print(meta?.cameraMake);    // "Apple"
print(meta?.cameraModel);   // "iPhone 15 Pro"
print(meta?.gps?.lat);      // 37.42195
print(meta?.gps?.lon);      // -122.08408
print(meta?.gps?.alt);      // 15.3
print(meta?.duration);      // null (images) or Duration for video
```

`MediaMetadata.read()` returns `null` for unsupported formats or corrupt files — no exceptions to handle.

No initialisation call required. The Rust library is loaded automatically on first use.

## Models

### `MediaMetadata`

| Field | Type | Description |
|-------|------|-------------|
| `mimeType` | `String` | Detected MIME type |
| `width` | `int?` | Width in pixels |
| `height` | `int?` | Height in pixels |
| `capturedAt` | `DateTime?` | Capture time in UTC |
| `cameraMake` | `String?` | Camera manufacturer |
| `cameraModel` | `String?` | Camera model |
| `gps` | `GpsCoordinates?` | Location |
| `duration` | `Duration?` | Video duration |

### `GpsCoordinates`

| Field | Type | Description |
|-------|------|-------------|
| `lat` | `double` | Latitude (negative = south) |
| `lon` | `double` | Longitude (negative = west) |
| `alt` | `double?` | Altitude in metres |

## How it works

Format is detected from the first 16 bytes of the file (magic bytes), not the file extension. HEIC metadata is read directly from the EXIF box without decoding the image. The entire native layer is Rust — no Swift, Kotlin, or C++ platform code is involved.

## License

MIT — see [LICENSE](LICENSE).
