import 'package:flutter_media_metadata/src/models/gps_coordinates.dart';
import 'package:flutter_media_metadata/src/rust/frb_generated.dart';

class MediaMetadata {
  final String mimeType;
  final int? width;
  final int? height;
  final DateTime? capturedAt;
  final String? cameraMake;
  final String? cameraModel;
  final GpsCoordinates? gps;
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

  static Future<MediaMetadata?> read(String filePath) async {
    try {
      final raw = await readMetadata(path: filePath);
      return MediaMetadata(
        mimeType: raw.mimeType,
        width: raw.width,
        height: raw.height,
        capturedAt: raw.capturedAtMs != null
            ? DateTime.fromMillisecondsSinceEpoch(raw.capturedAtMs!, isUtc: true)
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
