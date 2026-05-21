#include "include/flutter_media_metadata/flutter_media_metadata_plugin_c_api.h"

#include <flutter/plugin_registrar_windows.h>

#include "flutter_media_metadata_plugin.h"

void FlutterMediaMetadataPluginCApiRegisterWithRegistrar(
    FlutterDesktopPluginRegistrarRef registrar) {
  flutter_media_metadata::FlutterMediaMetadataPlugin::RegisterWithRegistrar(
      flutter::PluginRegistrarManager::GetInstance()
          ->GetRegistrar<flutter::PluginRegistrarWindows>(registrar));
}
