#include "flutter_media_metadata_plugin.h"

#include <flutter/plugin_registrar_windows.h>

namespace flutter_media_metadata {

// flutter_rust_bridge handles all native calls via FFI — nothing to register here.
void FlutterMediaMetadataPlugin::RegisterWithRegistrar(
    flutter::PluginRegistrarWindows *registrar) {
  registrar->AddPlugin(std::make_unique<FlutterMediaMetadataPlugin>());
}

FlutterMediaMetadataPlugin::FlutterMediaMetadataPlugin() {}
FlutterMediaMetadataPlugin::~FlutterMediaMetadataPlugin() {}

}  // namespace flutter_media_metadata
