import 'dart:io';
import 'dart:typed_data';

import 'package:flutter_test/flutter_test.dart';
import 'package:integration_test/integration_test.dart';
import 'package:media_metadata_plus/media_metadata_plus.dart';

// Resolves a fixture path relative to the example directory.
// On desktop, Directory.current is the example/ directory when run via
// `flutter test integration_test/ -d macos`.
String _fixture(String name) =>
    '${Directory.current.path}/integration_test/media/$name';

/// Scan JPEG bytes for the first SOF marker and return (width, height).
(int, int)? _jpegDimensions(Uint8List bytes) {
  int i = 0;
  while (i + 1 < bytes.length) {
    if (bytes[i] != 0xFF) return null;
    final marker = bytes[i + 1];
    if (marker == 0xC0 || marker == 0xC1 || marker == 0xC2) {
      if (i + 8 >= bytes.length) return null;
      final h = (bytes[i + 5] << 8) | bytes[i + 6];
      final w = (bytes[i + 7] << 8) | bytes[i + 8];
      return (w, h);
    }
    if (i + 3 >= bytes.length) break;
    final segLen = (bytes[i + 2] << 8) | bytes[i + 3];
    i += 2 + segLen;
  }
  return null;
}

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

  group('readAll (batch)', () {
    testWidgets('returns results in input order', (_) async {
      final results = await MediaMetadata.readAll([
        _fixture('photo.jpg'),
        _fixture('video.mp4'),
        _fixture('document.txt'),
      ]);
      expect(results, hasLength(3));
      expect(results[0]?.mimeType, 'image/jpeg');
      expect(results[1]?.mimeType, 'video/mp4');
      expect(results[2], isNull); // unsupported
    });

    testWidgets('returns empty list for empty input', (_) async {
      final results = await MediaMetadata.readAll([]);
      expect(results, isEmpty);
    });

    testWidgets('handles mixed valid and corrupt files', (_) async {
      final results = await MediaMetadata.readAll([
        _fixture('photo.jpg'),
        _fixture('corrupt.jpg'),
        _fixture('video.mov'),
      ]);
      expect(results, hasLength(3));
      expect(results[0], isNotNull);
      expect(results[1], isNull);
      expect(results[2], isNotNull);
    });
  });

  group('RAW / TIFF', () {
    // These tests are skipped automatically if the fixture file is absent.
    // Add real RAW files to example/integration_test/media/ and re-run.
    for (final entry in {
      'photo.tiff': 'image/tiff',
      'photo.dng': 'image/tiff',
      'photo.nef': 'image/tiff',
      'photo.arw': 'image/tiff',
      'photo.cr2': 'image/x-canon-cr2',
    }.entries) {
      testWidgets('reads EXIF from ${entry.key}', (_) async {
        final path = _fixture(entry.key);
        if (!File(path).existsSync()) return; // fixture not present — skip
        final meta = await MediaMetadata.read(path);
        expect(meta, isNotNull);
        expect(meta?.mimeType, entry.value);
        expect(meta?.capturedAt, isNotNull);
        expect(meta?.cameraMake, isNotNull);
      });
    }
  });

  testWidgets('returns null for unsupported format', (_) async {
    final meta = await MediaMetadata.read(_fixture('document.txt'));
    expect(meta, isNull);
  });

  testWidgets('returns null for corrupt file', (_) async {
    final meta = await MediaMetadata.read(_fixture('corrupt.jpg'));
    expect(meta, isNull);
  });

  group('thumbnail rotation', () {
    testWidgets('90° rotated video produces portrait thumbnail', (_) async {
      final bytes = await MediaMetadata.generateThumbnail(
        _fixture('video_rotate90.mp4'),
      );
      expect(bytes, isNotNull);
      final dims = _jpegDimensions(bytes!);
      expect(dims, isNotNull, reason: 'output must be a valid JPEG');
      // Stored as 480×270; display rotation 90° CW → portrait 270×480
      expect(dims!.$1, 270, reason: 'width should be 270 after 90° rotation');
      expect(dims.$2, 480, reason: 'height should be 480 after 90° rotation');
    });

    testWidgets('unrotated video preserves landscape dimensions', (_) async {
      final bytes = await MediaMetadata.generateThumbnail(
        _fixture('video.mp4'),
      );
      expect(bytes, isNotNull);
      final dims = _jpegDimensions(bytes!);
      expect(dims, isNotNull);
      expect(dims!.$1, 480, reason: 'width should be 480');
      expect(dims.$2, 270, reason: 'height should be 270');
    });
  });

  group('generateThumbnail', () {
    testWidgets('returns JPEG bytes from embedded covr atom', (_) async {
      final bytes =
          await MediaMetadata.generateThumbnail(_fixture('video_covr.mp4'));
      expect(bytes, isNotNull);
      // JPEG magic bytes: FF D8
      expect(bytes![0], 0xFF);
      expect(bytes[1], 0xD8);
    });

    testWidgets('falls back to FFmpeg frame when no covr atom', (_) async {
      final bytes =
          await MediaMetadata.generateThumbnail(_fixture('video.mp4'));
      expect(bytes, isNotNull);
      // JPEG magic bytes: FF D8
      expect(bytes![0], 0xFF);
      expect(bytes[1], 0xD8);
    });

    testWidgets('returns null for image files', (_) async {
      final bytes =
          await MediaMetadata.generateThumbnail(_fixture('photo.jpg'));
      expect(bytes, isNull);
    });

    testWidgets('writes bytes to savePath when provided', (_) async {
      final dir = Directory.systemTemp.createTempSync('thumb_test_');
      final savePath = '${dir.path}/thumb.jpg';
      try {
        final bytes = await MediaMetadata.generateThumbnail(
          _fixture('video_covr.mp4'),
          savePath: savePath,
        );
        expect(bytes, isNotNull);
        final saved = File(savePath);
        expect(saved.existsSync(), isTrue);
        expect(saved.readAsBytesSync(), bytes);
      } finally {
        dir.deleteSync(recursive: true);
      }
    });

    testWidgets('timeMs parameter is accepted without error', (_) async {
      // timeMs has no effect in Phase 1 (covr atom is position-independent)
      // but must not throw
      final bytes = await MediaMetadata.generateThumbnail(
        _fixture('video_covr.mp4'),
        timeMs: 5000,
      );
      expect(bytes, isNotNull);
    });
  });
}
