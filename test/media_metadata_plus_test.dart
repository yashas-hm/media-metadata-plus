import 'package:flutter_test/flutter_test.dart';
import 'package:media_metadata_plus/media_metadata_plus.dart';

void main() {
  group('MediaMetadata', () {
    test('constructs with required fields', () {
      const meta = MediaMetadata(mimeType: 'image/jpeg');
      expect(meta.mimeType, 'image/jpeg');
      expect(meta.width, isNull);
      expect(meta.height, isNull);
      expect(meta.capturedAt, isNull);
      expect(meta.gps, isNull);
      expect(meta.duration, isNull);
    });

    test('constructs with all fields', () {
      final captured = DateTime.utc(2024, 3, 15, 10, 30);
      const gps = GpsCoordinates(lat: 37.42, lon: -122.08, alt: 15.3);
      final meta = MediaMetadata(
        mimeType: 'video/mp4',
        width: 1920,
        height: 1080,
        capturedAt: captured,
        cameraMake: 'Apple',
        cameraModel: 'iPhone 15 Pro',
        gps: gps,
        duration: const Duration(seconds: 42),
      );

      expect(meta.width, 1920);
      expect(meta.height, 1080);
      expect(meta.capturedAt, captured);
      expect(meta.cameraMake, 'Apple');
      expect(meta.cameraModel, 'iPhone 15 Pro');
      expect(meta.gps?.lat, 37.42);
      expect(meta.gps?.lon, -122.08);
      expect(meta.gps?.alt, 15.3);
      expect(meta.duration?.inSeconds, 42);
    });

    test('toString includes key fields', () {
      const meta = MediaMetadata(mimeType: 'image/heic', width: 4032, height: 3024);
      final s = meta.toString();
      expect(s, contains('image/heic'));
      expect(s, contains('4032'));
      expect(s, contains('3024'));
    });
  });

  group('GpsCoordinates', () {
    test('constructs without altitude', () {
      const gps = GpsCoordinates(lat: 51.5074, lon: -0.1278);
      expect(gps.lat, 51.5074);
      expect(gps.lon, -0.1278);
      expect(gps.alt, isNull);
    });

    test('negative lat/lon for south and west', () {
      const gps = GpsCoordinates(lat: -33.8688, lon: 151.2093, alt: -5.0);
      expect(gps.lat, isNegative);
      expect(gps.lon, isPositive);
      expect(gps.alt, -5.0);
    });

    test('toString includes all values', () {
      const gps = GpsCoordinates(lat: 37.42, lon: -122.08, alt: 15.3);
      expect(gps.toString(), contains('37.42'));
      expect(gps.toString(), contains('-122.08'));
      expect(gps.toString(), contains('15.3'));
    });
  });
}
