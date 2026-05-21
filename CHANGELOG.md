## 0.3.0

* Added TIFF support — reads capture time, dimensions, camera make/model, and GPS.
* Added CR2 (Canon RAW) support — detected by magic bytes, full EXIF extraction.
* Added DNG, NEF (Nikon), and ARW (Sony) support — all TIFF-based; reported as `image/tiff`.
* No new native dependencies — all RAW formats handled by the existing `kamadak-exif` reader.

## 0.2.0

* Added `MediaMetadata.readAll(List<String> paths)` — reads multiple files in parallel via Rayon.
* WebP: dimension fallback from VP8X / VP8L / VP8 bitstream when EXIF is absent.
* WebP files without EXIF now return a `MediaMetadata` (was `null`).
* Video GPS: reads `©xyz` atom (ISO 6709) from MP4/MOV user data.
* Video camera: reads `©mak` / `©mod` iTunes atoms for make and model.

## 0.1.0

* Initial release.
* Read metadata from JPEG, HEIC/HEIF, PNG, WebP, MP4, and MOV files.
* Extracts: capture timestamp, dimensions, camera make/model, GPS coordinates, video duration.
* Format detected by magic bytes — not file extension.
* Powered by Rust via `flutter_rust_bridge` v2. No platform-specific Swift/Kotlin/C++ code.
* Supports macOS, Windows, Linux, iOS, and Android.
