import 'package:file_picker/file_picker.dart';
import 'package:flutter/material.dart';
import 'package:media_metadata_plus/media_metadata_plus.dart';

void main() {
  runApp(const App());
}

class App extends StatelessWidget {
  const App({super.key});

  @override
  Widget build(BuildContext context) {
    return MaterialApp(
      title: 'media_metadata_plus',
      theme: ThemeData(colorSchemeSeed: Colors.indigo, useMaterial3: true),
      home: const HomePage(),
    );
  }
}

class HomePage extends StatefulWidget {
  const HomePage({super.key});

  @override
  State<HomePage> createState() => _HomePageState();
}

class _HomePageState extends State<HomePage> {
  MediaMetadata? _meta;
  bool _loading = false;

  Future<void> _pick() async {
    final result = await FilePicker.platform.pickFiles(type: FileType.media);
    if (result == null) return;

    setState(() => _loading = true);
    final meta = await MediaMetadata.read(result.files.single.path!);
    setState(() {
      _meta = meta;
      _loading = false;
    });
  }

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      appBar: AppBar(title: const Text('Media Metadata')),
      body: Center(
        child: _loading
            ? const CircularProgressIndicator()
            : _meta == null
            ? ElevatedButton.icon(
                onPressed: _pick,
                icon: const Icon(Icons.file_open),
                label: const Text('Pick a file'),
              )
            : _MetaView(
                meta: _meta!,
                onReset: () => setState(() => _meta = null),
              ),
      ),
    );
  }
}

class _MetaView extends StatelessWidget {
  const _MetaView({required this.meta, required this.onReset});

  final MediaMetadata meta;
  final VoidCallback onReset;

  @override
  Widget build(BuildContext context) {
    final rows = [
      _row('Format', meta.mimeType),
      if (meta.width != null && meta.height != null)
        _row('Dimensions', '${meta.width} × ${meta.height}'),
      if (meta.capturedAt != null)
        _row('Captured', meta.capturedAt!.toLocal().toString()),
      if (meta.cameraMake != null) _row('Make', meta.cameraMake!),
      if (meta.cameraModel != null) _row('Model', meta.cameraModel!),
      if (meta.gps != null)
        _row(
          'GPS',
          '${meta.gps!.lat.toStringAsFixed(5)}, '
              '${meta.gps!.lon.toStringAsFixed(5)}'
              '${meta.gps!.alt != null ? ', ${meta.gps!.alt!.toStringAsFixed(1)} m' : ''}',
        ),
      if (meta.duration != null)
        _row('Duration', _formatDuration(meta.duration!)),
    ];

    return SingleChildScrollView(
      padding: const EdgeInsets.all(24),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          ...rows,
          const SizedBox(height: 24),
          TextButton.icon(
            onPressed: onReset,
            icon: const Icon(Icons.refresh),
            label: const Text('Pick another'),
          ),
        ],
      ),
    );
  }

  Widget _row(String label, String value) => Padding(
    padding: const EdgeInsets.symmetric(vertical: 6),
    child: Row(
      crossAxisAlignment: CrossAxisAlignment.start,
      children: [
        SizedBox(
          width: 100,
          child: Text(
            label,
            style: const TextStyle(fontWeight: FontWeight.bold),
          ),
        ),
        Expanded(child: Text(value)),
      ],
    ),
  );

  String _formatDuration(Duration d) {
    final h = d.inHours;
    final m = d.inMinutes.remainder(60).toString().padLeft(2, '0');
    final s = d.inSeconds.remainder(60).toString().padLeft(2, '0');
    return h > 0 ? '$h:$m:$s' : '$m:$s';
  }
}
