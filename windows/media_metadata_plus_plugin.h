#ifndef FLUTTER_PLUGIN_MEDIA_METADATA_PLUS_PLUGIN_H_
#define FLUTTER_PLUGIN_MEDIA_METADATA_PLUS_PLUGIN_H_

#include <flutter/plugin_registrar_windows.h>

namespace media_metadata_plus {

class MediaMetadataPlusPlugin : public flutter::Plugin {
 public:
  static void RegisterWithRegistrar(flutter::PluginRegistrarWindows *registrar);
  MediaMetadataPlusPlugin();
  ~MediaMetadataPlusPlugin() override;
  MediaMetadataPlusPlugin(const MediaMetadataPlusPlugin &) = delete;
  MediaMetadataPlusPlugin &operator=(const MediaMetadataPlusPlugin &) = delete;
};

}  // namespace media_metadata_plus

#endif  // FLUTTER_PLUGIN_MEDIA_METADATA_PLUS_PLUGIN_H_
