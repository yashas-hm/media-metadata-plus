# Existing Flutter Video Thumbnail Solutions

> Research date: 2026-06-11. All figures verified against live pub.dev and GitHub source code.

---

## Summary Table

| Package                           | Pub likes | GitHub stars | Android   | iOS       | macOS     | Windows   | Linux     | Web | In-memory bytes | Batch | FFmpeg           | Maintained             |
|-----------------------------------|-----------|--------------|-----------|-----------|-----------|-----------|-----------|-----|-----------------|-------|------------------|------------------------|
| `cross_platform_video_thumbnails` | 4         | 1            | ⚠️ Broken | ⚠️ Broken | ⚠️ System | ⚠️ System | ⚠️ System | ✅  | ✅              | ✅    | System install   | Active (1 contributor) |
| `video_thumbnail`                 | 617       | 218          | ✅ Native | ✅ Native | ❌        | ❌        | ❌        | ❌  | ✅              | ❌    | None (OS APIs)   | Slow (87 open issues)  |
| `media_kit` (screenshot)          | 896       | 1,767        | ✅        | ✅        | ✅        | ✅        | ✅        | ✅  | ✅              | ❌    | Bundled (libmpv) | Active                 |
| `ffmpeg_kit_flutter`              | 471       | 5,834        | ✅        | ✅        | ✅        | ❌        | ❌        | ❌  | ❌ (file only)  | ❌    | Bundled          | **ARCHIVED**           |

---

## 1. `cross_platform_video_thumbnails`

**pub.dev:** https://pub.dev/packages/cross_platform_video_thumbnails  
**GitHub:** https://github.com/Dhia-Bechattaoui/cross_platform_video_thumbnails  
**Version:** 0.1.1 | **License:** MIT | **Last publish:** November 5, 2025

### Metrics

|                 |                                   |
|-----------------|-----------------------------------|
| pub.dev points  | 160/160 (perfect automated score) |
| Likes           | 4                                 |
| Total downloads | ~650                              |
| GitHub stars    | 1                                 |
| GitHub forks    | 0                                 |

### Platform reality (source-code verified)

The package claims 6-platform support and earns a perfect pub.dev score. **Both claims are misleading.**

**pub.dev's 160/160 score only tests documentation quality and static analysis — it does not test runtime behavior.**

| Platform | What the code actually does                                                                                                                                                               | Verdict                                       |
|----------|-------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|-----------------------------------------------|
| Android  | Calls a `'cross_platform_video_thumbnails'` method channel in Dart. **No Kotlin/Java plugin handler exists anywhere in the repo.**                                                        | ❌ Throws `MissingPluginException` at runtime |
| iOS      | Same Dart method channel. **No Swift/ObjC plugin handler exists.**                                                                                                                        | ❌ Throws `MissingPluginException` at runtime |
| Web      | `HTMLVideoElement` + Canvas API in pure Dart. Self-contained.                                                                                                                             | ✅ Works                                      |
| macOS    | `Process.start('ffmpeg', ...)` — spawns a system `ffmpeg` process. Searches `/usr/local/bin/`, `/opt/homebrew/bin/`, `/usr/bin/`. If not found: `ThumbnailException('FFmpeg not found')`. | ⚠️ Requires system FFmpeg install             |
| Windows  | Same process-spawn pattern. Searches `C:\ffmpeg\bin\`, `C:\Program Files\ffmpeg\bin\`, PATH.                                                                                              | ⚠️ Requires system FFmpeg install             |
| Linux    | Same process-spawn pattern.                                                                                                                                                               | ⚠️ Requires system FFmpeg install             |

The package is **not declared as a Flutter plugin** in its `pubspec.yaml` — there is no `flutter.plugin.platforms`
section. The `android/` and `ios/` directories in the repo are an example app, not native plugin implementations.

### API design quality

The API design is good: returns `Uint8List` in memory, supports batch, supports arbitrary timestamp, supports format
selection. If the platform implementations were real, this would be a decent package.

### Verdict

**Not production-ready.** Mobile is broken. Desktop requires a pre-installed FFmpeg binary. The impressive pub score is
a documentation artifact, not a quality signal.

---

## 2. `video_thumbnail`

**pub.dev:** https://pub.dev/packages/video_thumbnail  
**GitHub:** https://github.com/justsoft/video_thumbnail  
**Version:** 0.5.6 | **License:** MIT | **Last publish:** May 14, 2025

### Metrics

|                 |          |
|-----------------|----------|
| pub.dev points  | 140/160  |
| Likes           | 617      |
| Total downloads | ~162,000 |
| GitHub stars    | 218      |
| GitHub forks    | 482      |
| Open issues     | 87       |

### Platform support (source-code verified)

| Platform | Implementation                                                                 | Technology                                                                             |
|----------|--------------------------------------------------------------------------------|----------------------------------------------------------------------------------------|
| Android  | `android/src/main/java/xyz/justsoft/video_thumbnail/VideoThumbnailPlugin.java` | `MediaMetadataRetriever.getFrameAtTime()`, API 27+ uses `getScaledFrameAtTime()`       |
| iOS      | `ios/Classes/VideoThumbnailPlugin.m`                                           | `AVAssetImageGenerator.copyCGImageAtTime:actualTime:error:` with zero tolerance before |
| macOS    | ❌ not implemented                                                             | —                                                                                      |
| Windows  | ❌ not implemented                                                             | —                                                                                      |
| Linux    | ❌ not implemented                                                             | —                                                                                      |
| Web      | ❌ not implemented                                                             | —                                                                                      |

The pub.dev score of 140/160 deducts 20 points for limited platform support — this accurately reflects the code reality.

### Strengths

- **Battle-tested**: 162K downloads, 482 forks, real native implementations
- **No FFmpeg dependency**: uses OS-provided APIs on both platforms
- **In-memory bytes**: `VideoThumbnail.thumbnailData()` returns `Uint8List?`
- **Remote URLs**: supports `http://` and `https://` sources with HTTP headers
- **Arbitrary timestamp**: `timeMs` parameter in milliseconds

### Weaknesses

- **Mobile-only**: no desktop or web support
- **No batch API**: one call per video
- **Slow maintenance**: 87 open issues, no release in 13+ months as of June 2026
- **Unverified publisher** on pub.dev

### Verdict

The most battle-tested option for mobile-only use. Correct and functional on Android and iOS. Useless for any desktop
platform.

---

## 3. `media_kit` (screenshot API)

**pub.dev:** https://pub.dev/packages/media_kit  
**GitHub:** https://github.com/media-kit/media-kit  
**Version:** 1.2.6 | **License:** MIT | **Last push:** May 2026

### Metrics

|                 |          |
|-----------------|----------|
| pub.dev points  | 140/160  |
| Likes           | 896      |
| Total downloads | ~239,000 |
| GitHub stars    | 1,767    |

### Platform support

All 6 platforms: Android 5.0+, iOS 9.0+, macOS 10.9+, Windows 7+, GNU/Linux, Web. This is the only package in this
evaluation that genuinely supports all 6 platforms with bundled native libraries (libmpv + FFmpeg — no separate install
required).

### The screenshot API

```dart
Future<Uint8List?> screenshot
(
{
String
?
format = '
image/jpeg
'
,bool synchronized = true,
bool includeLibassSubtitles = false,
})
```

`screenshot()` captures the **current playback frame only**. There is no timestamp parameter. To extract a thumbnail at
a specific position you must:

1. Create a `Player` instance
2. `await player.open(Media(path))`
3. `await player.seek(Duration(seconds: 5))`
4. `await Future.delayed(...)` — wait for seek to complete
5. `final bytes = await player.screenshot()`
6. `await player.dispose()`

This is not a thumbnail extractor — it is a side-effect of a full video player.

### Required packages

```yaml
media_kit: ^1.2.6
media_kit_video: ^1.2.6          # rendering widget
media_kit_libs_video: ^1.2.6     # bundled libmpv + FFmpeg (~30–60 MB native libs)
```

### Verdict

`media_kit` is genuinely cross-platform with bundled FFmpeg, but it is architected as a video **player** and imposes
full player lifecycle overhead per thumbnail. It is not suitable for batch extraction of 10K+ thumbnails. The 30–60 MB
native library addition is also a significant dependency for a plugin whose only goal is thumbnail extraction.

---

## 4. `ffmpeg_kit_flutter`

**pub.dev:** https://pub.dev/packages/ffmpeg_kit_flutter  
**GitHub:** https://github.com/arthenica/ffmpeg-kit (archived)  
**Version:** 6.0.3 | **License:** LGPL-3.0

### Status: ARCHIVED

The GitHub repository was archived as read-only on **June 23, 2025**. No future updates, bug fixes, or security patches
will be issued. **Do not use for new projects.**

### Platform support

Android, iOS, macOS only. **No Windows or Linux support.** This package does not solve the desktop thumbnail problem.

### Other issues

- Writes to a file path, not in-memory bytes
- LGPL-3.0 requires notice and link obligations
- Locked to FFmpeg 6.0 — no security updates

---

## Key Conclusions

1. **No existing package cleanly solves the all-6-platforms thumbnail extraction problem without requiring a
   pre-installed FFmpeg binary or accepting the overhead of a full video player.**

2. `video_thumbnail` is the correct choice for a mobile-only use case, but `media_metadata_plus` already targets 6
   platforms.

3. `cross_platform_video_thumbnails` looks good on paper and pub.dev metrics but is fundamentally broken at runtime on
   mobile and requires a pre-installed FFmpeg binary on desktop.

4. `media_kit` is the only genuine all-platform option but is the wrong abstraction for thumbnail extraction — it
   requires full player lifecycle overhead and 30–60 MB of native libraries.

5. `ffmpeg_kit_flutter` is discontinued and does not support Windows/Linux.

6. The gap is real and meaningful: a Flutter plugin that extracts thumbnails on all 6 platforms without requiring
   external binary installation does not exist in the ecosystem.