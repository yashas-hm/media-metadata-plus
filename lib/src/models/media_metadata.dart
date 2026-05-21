import 'package:media_metadata_plus/src/models/gps_coordinates.dart';
import 'package:media_metadata_plus/src/rust/api.dart';
import 'package:media_metadata_plus/src/rust/frb_generated.dart';

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

  static Future<void> _ensureInit() async {
    if (!_initialized) {
      await RustLib.init();
      _initialized = true;
    }
  }

  /// Reads metadata from the file at [filePath].
  ///
  /// Returns `null` if the format is unsupported or the file is corrupt.
  /// Supported formats: JPEG, HEIC/HEIF, PNG, WebP, MP4, MOV.
  static Future<MediaMetadata?> read(String filePath) async {
    await _ensureInit();
    try {
      return _fromRaw(await readMetadata(path: filePath));
    } catch (_) {
      return null;
    }
  }

  /// Reads metadata from multiple files in parallel (Rayon on the Rust side).
  ///
  /// Each element in the returned list corresponds to the file at the same
  /// index in [filePaths]. An entry is `null` if the file is unsupported or
  /// corrupt. Preserves input order.
  ///
  /// ```dart
  /// final results = await MediaMetadata.readAll(paths);
  /// for (final meta in results) {
  ///   print(meta?.mimeType);
  /// }
  /// ```
  static Future<List<MediaMetadata?>> readAll(List<String> filePaths) async {
    if (filePaths.isEmpty) return [];
    await _ensureInit();
    try {
      final raws = await readMetadataBatch(paths: filePaths);
      return raws.map((raw) => raw == null ? null : _fromRaw(raw)).toList();
    } catch (_) {
      return List.filled(filePaths.length, null);
    }
  }

  static MediaMetadata _fromRaw(MediaMeta raw) => MediaMetadata(
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

  @override
  String toString() => 'MediaMetadata('
      'mimeType: $mimeType, '
      'width: $width, height: $height, '
      'capturedAt: $capturedAt, '
      'camera: $cameraMake $cameraModel, '
      'gps: $gps, duration: $duration)';
}
