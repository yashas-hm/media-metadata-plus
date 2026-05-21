## 0.1.0

* Initial release.
* Read metadata from JPEG, HEIC/HEIF, PNG, WebP, MP4, and MOV files.
* Extracts: capture timestamp, dimensions, camera make/model, GPS coordinates, video duration.
* Format detected by magic bytes — not file extension.
* Powered by Rust via `flutter_rust_bridge` v2. No platform-specific Swift/Kotlin/C++ code.
* Supports macOS, Windows, Linux, iOS, and Android.
