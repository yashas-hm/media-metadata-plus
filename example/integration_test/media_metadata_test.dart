import 'dart:io';

import 'package:flutter_test/flutter_test.dart';
import 'package:media_metadata_plus/media_metadata_plus.dart';
import 'package:integration_test/integration_test.dart';

// Resolves a fixture path relative to the example directory.
// On desktop, Directory.current is the example/ directory when run via
// `flutter test integration_test/ -d macos`.
String _fixture(String name) =>
    '${Directory.current.path}/integration_test/media/$name';

void main() {
  IntegrationTestWidgetsFlutterBinding.ensureInitialized();

  group('JPEG', () {
    testWidgets('reads capturedAt from EXIF DateTimeOriginal', (_) async {
      final meta = await MediaMetadata.read(_fixture('photo.jpg'));
      expect(meta?.capturedAt, isNotNull);
      expect(meta?.capturedAt?.year, 2021);
    });

    testWidgets('reads GPS coordinates', (_) async {
      final meta = await MediaMetadata.read(_fixture('photo_gps.jpg'));
      expect(meta?.gps?.lat, closeTo(37.42, 0.01));
      expect(meta?.gps?.lon, closeTo(-122.08, 0.01));
    });

    testWidgets('reads camera make and model', (_) async {
      final meta = await MediaMetadata.read(_fixture('photo.jpg'));
      expect(meta?.cameraMake, isNotNull);
      expect(meta?.cameraModel, isNotNull);
    });
  });

  group('HEIC', () {
    testWidgets('reads capturedAt without full image decode', (_) async {
      final meta = await MediaMetadata.read(_fixture('photo.heic'));
      expect(meta?.capturedAt, isNotNull);
      expect(meta?.mimeType, 'image/heic');
    });
  });

  group('MP4', () {
    testWidgets('reads duration and dimensions', (_) async {
      final meta = await MediaMetadata.read(_fixture('video.mp4'));
      expect(meta?.duration, isNotNull);
      expect(meta?.width, greaterThan(0));
      expect(meta?.height, greaterThan(0));
    });

    testWidgets('reads creation time', (_) async {
      final meta = await MediaMetadata.read(_fixture('video.mp4'));
      expect(meta?.capturedAt, isNotNull);
    });
  });

  group('MOV', () {
    testWidgets('reads creation time', (_) async {
      final meta = await MediaMetadata.read(_fixture('video.mov'));
      expect(meta?.capturedAt, isNotNull);
    });
  });

  testWidgets('returns null for unsupported format', (_) async {
    final meta = await MediaMetadata.read(_fixture('document.txt'));
    expect(meta, isNull);
  });

  testWidgets('returns null for corrupt file', (_) async {
    final meta = await MediaMetadata.read(_fixture('corrupt.jpg'));
    expect(meta, isNull);
  });
}
