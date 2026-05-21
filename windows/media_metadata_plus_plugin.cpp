#include "media_metadata_plus_plugin.h"

#include <flutter/plugin_registrar_windows.h>

namespace media_metadata_plus {

// flutter_rust_bridge handles all native calls via FFI — nothing to register here.
void MediaMetadataPlusPlugin::RegisterWithRegistrar(
    flutter::PluginRegistrarWindows *registrar) {
  registrar->AddPlugin(std::make_unique<MediaMetadataPlusPlugin>());
}

MediaMetadataPlusPlugin::MediaMetadataPlusPlugin() {}
MediaMetadataPlusPlugin::~MediaMetadataPlusPlugin() {}

}  // namespace media_metadata_plus
