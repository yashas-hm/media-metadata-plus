## 1.4.0

* Added `MediaMetadata.generateThumbnail(path, {int? timeMs, String? savePath})` — extracts a JPEG thumbnail from MP4 and MOV files.
  * **Fast path:** reads the embedded cover-art image from the `covr` iTunes atom — no video decoding, zero overhead. Covers most iPhone and iPad footage.
  * **FFmpeg fallback:** when no embedded thumbnail is present, seeks to 10 % of the video duration (or a caller-supplied `timeMs`) and decodes one frame via FFmpeg. Supports H.264, HEVC, VP9, AV1, and MPEG-4 — covers Android devices, DSLRs, GoPro, DJI, and any video without an embedded thumbnail.
* Thumbnail output is scaled to a maximum width of 640 px, preserving aspect ratio.
* Optional `savePath` — if provided, the JPEG bytes are written to that path (parent directories are created automatically).
* Returns `null` if the file is unsupported or no frame can be extracted — no exception to handle.

## 1.3.0

* Added `modifiedAt` field (`DateTime?`) — last-modified timestamp in UTC. Source is the EXIF `DateTime` tag for images and `mvhd.modification_time` for MP4/MOV.
* Improved video camera make/model coverage: now tries three atom paths in order — iTunes (`moov > udta > meta > ilst`), 3GPP (`moov > udta > ©mak/©mod` with length/language header), and iTunes-without-udta (`moov > meta > ilst`). Covers Apple, Android, and most third-party cameras.

## 1.2.1

* Fixed rust reader orientation calculation

## 1.2.0

* Added Info.plist and Resources symlink to macOS framework for codesign
* Pre-built native binaries are now shipped with the package — consumers no longer need Rust or any native toolchain installed.
* Added Swift Package Manager (SPM) support for iOS and macOS via `Package.swift` with pre-built xcframework binary targets.
* Fixed `.pubignore` — `lib/src/rust/` (generated Dart FFI bindings) was incorrectly excluded from the published package, causing static analysis errors and 0 platform support on pub.dev.
* Moved `Package.swift` to `macos/media_metadata_plus/` and `ios/media_metadata_plus/` — correct location expected by Flutter's Swift Package Manager tooling.

## 1.0.2

* Updated README.

## 1.0.1

* Updated README.

## 1.0.0

First stable release. All core features complete and publish-ready.

* Supports JPEG, HEIC/HEIF, PNG, WebP, TIFF, DNG, NEF, ARW, CR2, MP4, and MOV.
* Extracts: capture timestamp, dimensions, camera make/model, GPS coordinates, video duration.
* `MediaMetadata.readAll()` for parallel batch reads via Rayon.
* Format detected by magic bytes — not file extension.
* Video GPS (`©xyz`) and camera (`©mak`/`©mod`) from MP4/MOV user data atoms.
* WebP without EXIF returns dimensions from bitstream.
* Powered by Rust via `flutter_rust_bridge` v2. No platform-specific Swift/Kotlin/C++.
* Supports macOS, Windows, Linux, iOS, and Android.

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
