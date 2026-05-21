import 'package:flutter_test/flutter_test.dart';
import 'package:flutter_media_metadata/flutter_media_metadata.dart';

void main() {
  group('JPEG', () {
    test('reads capturedAt from EXIF DateTimeOriginal', () async {
      final meta = await MediaMetadata.read('test/fixtures/photo.jpg');
      expect(meta?.capturedAt, isNotNull);
      expect(meta?.capturedAt?.year, 2021);
    });

    test('reads GPS coordinates', () async {
      final meta = await MediaMetadata.read('test/fixtures/photo_gps.jpg');
      expect(meta?.gps?.lat, closeTo(37.42, 0.01));
      expect(meta?.gps?.lon, closeTo(-122.08, 0.01));
    });

    test('reads camera make and model', () async {
      final meta = await MediaMetadata.read('test/fixtures/photo.jpg');
      expect(meta?.cameraMake, isNotNull);
      expect(meta?.cameraModel, isNotNull);
    });
  });

  group('HEIC', () {
    test('reads capturedAt without full image decode', () async {
      final meta = await MediaMetadata.read('test/fixtures/photo.heic');
      expect(meta?.capturedAt, isNotNull);
      expect(meta?.mimeType, 'image/heic');
    });
  });

  group('MP4', () {
    test('reads duration and dimensions', () async {
      final meta = await MediaMetadata.read('test/fixtures/video.mp4');
      expect(meta?.duration, isNotNull);
      expect(meta?.width, greaterThan(0));
      expect(meta?.height, greaterThan(0));
    });

    test('reads creation time', () async {
      final meta = await MediaMetadata.read('test/fixtures/video.mp4');
      expect(meta?.capturedAt, isNotNull);
    });
  });

  group('MOV', () {
    test('reads creation time', () async {
      final meta = await MediaMetadata.read('test/fixtures/video.mov');
      expect(meta?.capturedAt, isNotNull);
    });
  });

  test('returns null for unsupported format', () async {
    final meta = await MediaMetadata.read('test/fixtures/document.pdf');
    expect(meta, isNull);
  });

  test('returns null for corrupt file', () async {
    final meta = await MediaMetadata.read('test/fixtures/corrupt.jpg');
    expect(meta, isNull);
  });
}
