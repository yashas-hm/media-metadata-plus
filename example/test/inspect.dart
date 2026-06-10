// Run with:
//   cd example && MEDIA_PATH=/path/to/file.jpg flutter test test/inspect.dart --reporter expanded
//
// Multiple files (comma-separated):
//   cd example && MEDIA_PATH=/a.jpg,/b.mp4 flutter test test/inspect.dart --reporter expanded

import 'dart:io';

import 'package:flutter_test/flutter_test.dart';
import 'package:media_metadata_plus/media_metadata_plus.dart';

void main() {
  final env = '/Users/yashas/Downloads/Takeout/Google Photos/Carolina brewery/20250201_210013.jpg';

  if (env.isEmpty) {
    test('inspect — no MEDIA_PATH set', () {
      fail(
        'Set MEDIA_PATH before running:\n'
        '  cd example && MEDIA_PATH=/path/to/file.jpg flutter test test/inspect.dart --reporter expanded',
      );
    });
    return;
  }

  final paths = env.split(',').map((p) => p.trim()).where((p) => p.isNotEmpty).toList();

  for (final path in paths) {
    test('inspect: $path', () async {
      final file = File(path);
      expect(file.existsSync(), isTrue, reason: 'File not found: $path');

      final meta = await MediaMetadata.read(path);
      expect(meta, isNotNull, reason: 'Unsupported format or corrupt file: $path');

      _print(path, meta!);
    });
  }
}

void _print(String path, MediaMetadata meta) {
  final lines = [
    '',
    '  file      : $path',
    '  mimeType  : ${meta.mimeType}',
    if (meta.width != null || meta.height != null)
      '  dimensions: ${meta.width} × ${meta.height}',
    if (meta.capturedAt != null) '  capturedAt: ${meta.capturedAt}',
    if (meta.cameraMake != null || meta.cameraModel != null)
      '  camera    : ${[meta.cameraMake, meta.cameraModel].whereType<String>().join(' ')}',
    if (meta.gps != null) ...[
      '  gps.lat   : ${meta.gps!.lat}',
      '  gps.lon   : ${meta.gps!.lon}',
      if (meta.gps!.alt != null) '  gps.alt   : ${meta.gps!.alt} m',
    ],
    if (meta.duration != null) '  duration  : ${_fmtDuration(meta.duration!)}',
    '',
  ];
  // ignore: avoid_print
  print(lines.join('\n'));
}

String _fmtDuration(Duration d) {
  final h = d.inHours;
  final m = d.inMinutes.remainder(60).toString().padLeft(2, '0');
  final s = d.inSeconds.remainder(60).toString().padLeft(2, '0');
  final ms = d.inMilliseconds.remainder(1000).toString().padLeft(3, '0');
  return h > 0 ? '$h:$m:$s.$ms' : '$m:$s.$ms';
}