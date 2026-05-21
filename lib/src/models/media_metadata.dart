import 'gps_coordinates.dart';

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

  @override
  String toString() => 'MediaMetadata('
      'mimeType: $mimeType, '
      'width: $width, height: $height, '
      'capturedAt: $capturedAt, '
      'camera: $cameraMake $cameraModel, '
      'gps: $gps, duration: $duration)';
}