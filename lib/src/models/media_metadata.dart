import 'package:flutter_media_metadata/src/models/gps_coordinates.dart';
import 'package:flutter_media_metadata/src/rust/api.dart';
import 'package:flutter_media_metadata/src/rust/frb_generated.dart';

/// Metadata read from a media file.
///
/// Use [MediaMetadata.read] to load metadata from any supported format:
/// JPEG, HEIC, PNG, WebP, MP4, or MOV.
///
/// ```dart
/// final meta = await MediaMetadata.read('/path/to/photo.heic');
/// print(meta?.capturedAt);   // DateTime?
/// print(meta?.gps?.lat);     // double?
/// print(meta?.duration);     // Duration? — video only
/// ```
class MediaMetadata {
  /// MIME type detected from file content, e.g. `image/heic`, `video/mp4`.
  final String mimeType;

  /// Image or video width in pixels.
  final int? width;

  /// Image or video height in pixels.
  final int? height;

  /// Capture timestamp in UTC, from EXIF DateTimeOriginal or MP4 creation time.
  final DateTime? capturedAt;

  /// Camera manufacturer, e.g. `Apple`.
  final String? cameraMake;

  /// Camera model, e.g. `iPhone 15 Pro`.
  final String? cameraModel;

  /// GPS location embedded in the file. `null` if not present.
  final GpsCoordinates? gps;

  /// Playback duration. Non-null for video files only.
  final Duration? duration;

  const MediaMetadata({
    required this.mimeType,
    this.width,
    this.height,
    this.capturedAt,
    this.cameraMake,
    this.cameraModel,
    this.gps,
    this.duration,
  });

  static bool _initialized = false;

  /// Reads metadata from the file at [filePath].
  ///
  /// Returns `null` if the format is unsupported or the file is corrupt.
  /// Supported formats: JPEG, HEIC/HEIF, PNG, WebP, MP4, MOV.
  static Future<MediaMetadata?> read(String filePath) async {
    if (!_initialized) {
      await RustLib.init();
      _initialized = true;
    }
    try {
      final raw = await readMetadata(path: filePath);
      return MediaMetadata(
        mimeType: raw.mimeType,
        width: raw.width,
        height: raw.height,
        capturedAt: raw.capturedAtMs != null
            ? DateTime.fromMillisecondsSinceEpoch(
                raw.capturedAtMs!.toInt(),
                isUtc: true,
              )
            : null,
        cameraMake: raw.cameraMake,
        cameraModel: raw.cameraModel,
        gps: (raw.latitude != null && raw.longitude != null)
            ? GpsCoordinates(
                lat: raw.latitude!,
                lon: raw.longitude!,
                alt: raw.altitude,
              )
            : null,
        duration: raw.durationMs != null
            ? Duration(milliseconds: raw.durationMs!.toInt())
            : null,
      );
    } catch (_) {
      return null;
    }
  }

  @override
  String toString() => 'MediaMetadata('
      'mimeType: $mimeType, '
      'width: $width, height: $height, '
      'capturedAt: $capturedAt, '
      'camera: $cameraMake $cameraModel, '
      'gps: $gps, duration: $duration)';
}
