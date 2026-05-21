#include "include/media_metadata_plus/media_metadata_plus_plugin_c_api.h"

#include <flutter/plugin_registrar_windows.h>

#include "media_metadata_plus_plugin.h"

void MediaMetadataPlusPluginCApiRegisterWithRegistrar(
    FlutterDesktopPluginRegistrarRef registrar) {
  media_metadata_plus::MediaMetadataPlusPlugin::RegisterWithRegistrar(
      flutter::PluginRegistrarManager::GetInstance()
          ->GetRegistrar<flutter::PluginRegistrarWindows>(registrar));
}
