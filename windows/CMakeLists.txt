cmake_minimum_required(VERSION 3.14)

set(PROJECT_NAME "media_metadata_plus")
project(${PROJECT_NAME} LANGUAGES CXX)

cmake_policy(VERSION 3.14...3.25)

set(PLUGIN_NAME "media_metadata_plus_plugin")

add_library(${PLUGIN_NAME} SHARED
  "include/media_metadata_plus/media_metadata_plus_plugin_c_api.h"
  "media_metadata_plus_plugin_c_api.cpp"
  "media_metadata_plus_plugin.cpp"
  "media_metadata_plus_plugin.h"
)

apply_standard_settings(${PLUGIN_NAME})

set_target_properties(${PLUGIN_NAME} PROPERTIES
  CXX_VISIBILITY_PRESET hidden)
target_compile_definitions(${PLUGIN_NAME} PRIVATE FLUTTER_PLUGIN_IMPL)

target_include_directories(${PLUGIN_NAME} INTERFACE
  "${CMAKE_CURRENT_SOURCE_DIR}/include")
target_link_libraries(${PLUGIN_NAME} PRIVATE flutter flutter_wrapper_plugin)

# Rust library is compiled and bundled automatically via ffiPlugin: true
set(media_metadata_plus_bundled_libraries
  ""
  PARENT_SCOPE
)
