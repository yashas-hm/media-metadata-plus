#ifndef FLUTTER_PLUGIN_FLUTTER_MEDIA_METADATA_PLUGIN_H_
#define FLUTTER_PLUGIN_FLUTTER_MEDIA_METADATA_PLUGIN_H_

#include <flutter/plugin_registrar_windows.h>

namespace flutter_media_metadata {

class FlutterMediaMetadataPlugin : public flutter::Plugin {
 public:
  static void RegisterWithRegistrar(flutter::PluginRegistrarWindows *registrar);
  FlutterMediaMetadataPlugin();
  ~FlutterMediaMetadataPlugin() override;
  FlutterMediaMetadataPlugin(const FlutterMediaMetadataPlugin &) = delete;
  FlutterMediaMetadataPlugin &operator=(const FlutterMediaMetadataPlugin &) = delete;
};

}  // namespace flutter_media_metadata

#endif  // FLUTTER_PLUGIN_FLUTTER_MEDIA_METADATA_PLUGIN_H_
