# FFmpeg Licensing Analysis for `media_metadata_plus`

> **Disclaimer:** This document is for informational and architectural guidance only. It is not legal advice.
> Consult a qualified attorney before making licensing decisions for a commercial product.

---

## 1. FFmpeg License Overview

### The FFmpeg Project's Own License

FFmpeg is a collection of libraries (`libavcodec`, `libavformat`, `libavutil`, `libswscale`, etc.). Each library
is individually licensed under the **GNU Lesser General Public License v2.1 or later (LGPL 2.1+)**. This is true
of the core FFmpeg libraries as built from source using only built-in codecs.

### The GPL Escalation Trigger

FFmpeg can optionally be compiled with third-party encoder/decoder libraries that carry the GPL. The most commonly
encountered are:

| Component | License | Notes |
|-----------|---------|-------|
| `libx264` | GPL 2+ | H.264 **encoder** (not decoder) |
| `libx265` | GPL 2+ (main) | H.265/HEVC encoder; LGPL option exists for limited builds |
| `libvpx` | BSD | VP8/VP9 encoder/decoder — **no GPL contamination** |
| `libfdk-aac` | proprietary / incompatible | Requires separate commercial arrangement |
| `libmp3lame` | LGPL 2+ | MP3 encoder |
| `openssl` / `gnutls` | GPL / LGPL | TLS support |
| Built-in H.264 decoder (`h264`) | LGPL | Bundled with FFmpeg core; not GPL |
| Built-in HEVC decoder (`hevc`) | LGPL | Bundled with FFmpeg core; not GPL |
| Built-in VP9 decoder (`vp9`) | LGPL | Bundled with FFmpeg core; not GPL |

**Critical distinction:** The GPL escalation only applies when FFmpeg is compiled `--enable-gpl` and linked against
GPL-licensed third-party libraries. The GPL escalation applies to the **entire linked binary**, not just the GPL
component — this is the copyleft effect of GPL (not LGPL).

### Default Build Configuration

The official FFmpeg source tree defaults to **LGPL**. GPL mode requires explicit `--enable-gpl` at configure time.
Pre-built binaries sourced from third parties (e.g., the `ffmpeg-kit` project, BtbN Windows builds, Homebrew) vary:

- **Homebrew `ffmpeg`**: GPL build by default (includes libx264, libx265)
- **ffmpeg-kit for mobile**: provides distinct LGPL and GPL variant packages; the LGPL variant excludes GPL codecs
- **Android MediaCodec via FFmpeg wrapper**: often LGPL-only with hardware codec delegation

For a Flutter plugin, you control which FFmpeg build you vendor or depend on, so this is an architectural decision
rather than a fixed constraint.

---

## 2. LGPL Compliance for Dynamic Linking

### What LGPL 2.1 Requires

When you dynamically link an LGPL 2.1 library (e.g., `libavcodec.so`/`libavcodec.dylib`), your obligations under
LGPL 2.1 Section 6 are:

1. **Provide the LGPL library source or a written offer** — you must either ship the FFmpeg source or a 12-month
   written offer to provide it. In practice this is satisfied by pointing to the official FFmpeg source at
   `https://ffmpeg.org/download.html` or vendoring the exact version you built.
2. **Display a copyright/license notice** — end users must be able to determine that FFmpeg is in use and that it
   is LGPL-licensed. A mention in the app's "About" screen or `NOTICES` file satisfies this.
3. **Allow the user to relink** — because it is a *shared* library (`.so`/`.dylib`), users can replace the FFmpeg
   `.so` with their own modified build and relink without rebuilding your application. Dynamic linking satisfies
   this requirement by construction.

### Can the Plugin Remain MIT?

**Yes.** The MIT license on your plugin source code is not affected by dynamic linking against LGPL. The two
licenses apply to different artifacts:

- MIT applies to your plugin's own source code.
- LGPL applies to the FFmpeg shared libraries distributed alongside your binary.

You must distribute the LGPL libraries with their own LGPL license notice intact, but your plugin's source
remains MIT. App developers using your plugin inherit the same arrangement — their app code can be any license,
but they must pass through the FFmpeg LGPL attribution to their end users.

### What Must Be Distributed With the Binary

For a dynamically-linked LGPL FFmpeg distribution:

1. The FFmpeg `.so`/`.dylib` files themselves (or a pointer to where users can obtain the same version).
2. An `LGPL-2.1.txt` (or `LICENSE`) file for FFmpeg.
3. FFmpeg copyright notices (found in `COPYING.LGPLv2.1` and `CREDITS` in the FFmpeg source tree).
4. Ideally, the exact Git commit or release version of FFmpeg you built, so users can reproduce the build.

A `NOTICES` file or a screen in the app listing "This application uses FFmpeg, Copyright (c) the FFmpeg Project,
licensed under LGPL 2.1+" satisfies the notice requirement.

---

## 3. LGPL Compliance for Static Linking

### The Stricter Obligation

Static linking incorporates the LGPL library object code directly into your binary artifact. LGPL 2.1 Section 6
still requires that users be able to substitute a modified FFmpeg and relink. With a static library, the only way
to satisfy this is:

> "Provide the object files or source for your application, so that the user can relink the application with a
> modified version of the Library."

In concrete terms, you must either:

- Distribute the `.o` object files for your plugin (but not necessarily full source), OR
- Distribute the full source of your plugin (which you already do as MIT open-source), AND document the build
  procedure so a user can swap the FFmpeg static library and rebuild.

### Can the Plugin Remain MIT with Static FFmpeg?

**Yes, but with a caveat.** Because `media_metadata_plus` is fully open-source MIT, you already satisfy the
"provide source" requirement of LGPL 2.1. A user can:

1. Clone your repo.
2. Replace the FFmpeg static library with their own modified build.
3. Rerun your build scripts to produce a new plugin binary.

This satisfies LGPL's relinking requirement. However, you must:

- Clearly document the build process and which FFmpeg version/configuration is used.
- Include an FFmpeg `LGPL-2.1` license notice in your repo and any distributed binaries (e.g., in your `LICENSE`
  or a `NOTICES` file at the repo root).
- Not combine static LGPL FFmpeg with GPL codecs unless you escalate your own binary's license to GPL (which
  would conflict with MIT).

### The Practical Meaning of "Allow Users to Relink"

For a Flutter plugin using `flutter_rust_bridge` (as `media_metadata_plus` does), the build chain is:

```
Cargo build → libmedia_metadata_plus.a/.dylib
  → linked into Flutter app via CocoaPods / CMake / Gradle
```

If FFmpeg is statically linked into `libmedia_metadata_plus.a`, then "relinking with modified FFmpeg" means
rebuilding the Rust crate with a different FFmpeg static library. Because your Rust source is MIT-licensed and
publicly available, users can do exactly that. You need to document the `build.rs` / `pkg-config` / environment
variable setup to make this practical.

**Bottom line:** Static linking is viable for an open-source MIT plugin, but it adds a documentation obligation
and prohibits GPL codec inclusion.

---

## 4. GPL Codec Avoidance

### Target Use Case: Video Thumbnail Extraction

To extract a thumbnail from a video file you need to:

1. Parse the container (MP4, MOV, MKV) — demuxing.
2. Locate the nearest keyframe to the requested timestamp.
3. Decode one video frame (H.264, H.265/HEVC, VP9, AV1, etc.).
4. Scale/encode the frame as JPEG or PNG.

### Codec License Status in FFmpeg

| Codec | Built-in FFmpeg | License | Notes |
|-------|----------------|---------|-------|
| H.264 decode | `h264` decoder | **LGPL** | Built-in, no libx264 needed |
| H.265/HEVC decode | `hevc` decoder | **LGPL** | Built-in |
| VP9 decode | `vp9` decoder | **LGPL** | Built-in |
| AV1 decode | `libaom-av1`, `dav1d` | BSD/ISC | No GPL; `dav1d` is preferred |
| H.264 encode | `libx264` | **GPL** | Needed only for re-encoding, not thumbnails |
| H.265 encode | `libx265` (full) | **GPL** | Needed only for re-encoding |
| AAC decode | built-in `aac` | **LGPL** | `libfdk-aac` would be proprietary |

**Key finding for thumbnail extraction:** Decoding H.264, H.265, and VP9 uses only FFmpeg's built-in decoders,
which are LGPL. You do **not** need `libx264`, `libx265`, or any GPL component to decode frames for thumbnail
extraction. An LGPL-only FFmpeg build is fully sufficient.

### What a Safe LGPL-Only Build Looks Like

```bash
./configure \
  --disable-gpl \
  --disable-nonfree \
  --enable-decoder=h264,hevc,vp9,vp8,av1,mpeg4,mjpeg \
  --enable-demuxer=mp4,mov,matroska,avi \
  --enable-protocol=file \
  --disable-encoders \        # not needed for thumbnails
  --disable-muxers \          # not needed for thumbnails
  --enable-encoder=mjpeg,png  # only for JPEG/PNG output
  --disable-network \
  --disable-doc
```

This produces a minimal, LGPL-only FFmpeg build suitable for frame extraction.

---

## 5. Practical Approaches by Other Flutter Packages

### `video_thumbnail` (pub.dev)

`video_thumbnail` (by Fluttercommunity) takes a **platform-native approach** on Android and iOS, and uses
`ffmpeg_kit_flutter` on desktop. It does not bundle FFmpeg itself — it declares `ffmpeg_kit_flutter` as an
optional dependency or asks users to add it. The package's own license is MIT. This sidesteps the question of
static vs. dynamic linking by delegating the FFmpeg dependency to the consumer application.

The consequence is that consumers who want desktop support must add `ffmpeg_kit_flutter` to their `pubspec.yaml`
and accept its licensing obligations themselves.

### `ffmpeg_kit_flutter`

This is the most widely used FFmpeg wrapper in Flutter. It ships:

- **LGPL variant** (`ffmpeg_kit_flutter`) — LGPL-only codecs, MIT-compatible plugin code
- **GPL variant** (`ffmpeg_kit_flutter_full_gpl`) — includes libx264, libx265; full copyleft applies

The LGPL variant is MIT-compatible for the plugin code itself. The package distributes prebuilt FFmpeg frameworks
for iOS/macOS and `.aar`/`.so` for Android, satisfying LGPL binary distribution requirements. The Flutter plugin
source remains Apache 2.0.

### `cross_platform_video_thumbnails`

This package is less widely adopted. It similarly delegates to platform-native APIs on mobile and uses `ffmpeg`
system binaries (via `dart:io Process`) on desktop. This avoids bundling FFmpeg entirely but requires it to be
present on the host OS, making it unsuitable for reliable distribution.

### Commercial App Pattern

Commercial Flutter apps typically:

1. Use `AVFoundation` (iOS/macOS) and `MediaMetadataRetriever` (Android) for thumbnails on mobile.
2. On Windows/Linux where native APIs are weaker, use `ffmpeg_kit_flutter` LGPL variant or a bundled LGPL FFmpeg.
3. Include FFmpeg in a `NOTICES` or `open_source_licenses.html` screen in the app.
4. Never statically link GPL-escalated FFmpeg unless the entire app is GPLv3 or has a commercial FFmpeg license
   (FFmpeg does not offer commercial licenses separately, but some codec vendors like Cisco for OpenH264 do).

---

## 6. Alternative: OS-Native Video APIs

### Platform-by-Platform Analysis

| Platform | Native API | Thumbnail Support | License Impact |
|----------|-----------|------------------|----------------|
| macOS | `AVFoundation` (Objective-C/Swift) | Yes — `AVAssetImageGenerator` | Apple proprietary, no extra obligation |
| iOS | `AVFoundation` | Yes — same API | Apple proprietary |
| Android | `MediaMetadataRetriever` | Yes — `getFrameAtTime()` | Apache 2.0 (AOSP), no extra obligation |
| Windows | Windows Media Foundation | Yes — `IMFSourceReader` + `IMFMediaBuffer` | Microsoft proprietary, no extra obligation |
| Linux | GStreamer | Yes — `gst-plugins-good` | LGPL 2.0+ (libgstreamer) |
| Linux | FFmpeg | Yes | LGPL 2.1+ as above |

### Relevance to `media_metadata_plus`

The current plugin uses **Rust FFI via `flutter_rust_bridge`**, which makes it unusual compared to plugins that
call platform channels into native code. The Rust layer currently uses the `mp4` crate (MIT-licensed) for
container parsing — there is no frame decoding at all today.

If thumbnail extraction were added, the choice is:

**Option A — Stay all-Rust, use FFmpeg via `ffmpeg-sys` or `ffmpeg-next` crate:**
- `ffmpeg-sys-next` on crates.io links against system or bundled FFmpeg
- LGPL obligations apply as described above
- Works on all platforms from Rust code
- Consistent with existing architecture

**Option B — Use platform channels for thumbnail generation, keep Rust for metadata:**
- Dart method channel calls to platform-native thumbnail APIs
- No FFmpeg at all on iOS/macOS/Android/Windows
- Linux would still need GStreamer or FFmpeg (both LGPL)
- Breaks the "all-Rust, no Swift/Kotlin" design principle of this repo

**Option C — Use `ffmpeg_kit_flutter` from Dart, not Rust:**
- Plugin declares `ffmpeg_kit_flutter` as a dependency
- Consumers get the LGPL obligations but you do not bundle FFmpeg yourself
- Easiest licensing path; preserves MIT on plugin code
- Adds a heavy dependency that not all consumers want

---

## 7. Recommended Licensing Strategy

### Can the Plugin Remain MIT?

**Yes, under all practical scenarios:**

| Approach | Plugin stays MIT? | Obligations added |
|----------|------------------|------------------|
| Dynamic link LGPL FFmpeg | Yes | Bundle FFmpeg notice + LGPL text |
| Static link LGPL FFmpeg | Yes (open-source satisfies relink) | Document build process + bundle LGPL notice |
| Use FFmpeg with GPL codecs | **No** — binary becomes GPL | Plugin source would need GPL or commercial license |
| Use platform-native APIs | Yes, fully | None |
| Delegate to `ffmpeg_kit_flutter` | Yes | Consumer accepts LGPL; your plugin is clean |

### Recommended Approach for `media_metadata_plus`

Given that:

1. The plugin is already fully open-source MIT and Rust-based.
2. It currently does no frame decoding (only container/EXIF metadata parsing).
3. The existing architecture avoids native platform code.
4. The `mp4` crate and `kamadak-exif` crate are both MIT/BSD-licensed with no copyleft.

**If adding thumbnail extraction:**

The lowest-friction path that preserves MIT and the Rust-first architecture is:

**Use LGPL-only FFmpeg, statically linked via the `ffmpeg-next` Rust crate, with explicit `--disable-gpl`.**

Steps required:

1. Add `ffmpeg-next` (or `ffmpeg-sys-next`) to `Cargo.toml`. Use feature flags to select only decoders, demuxers,
   and JPEG/PNG encoders needed for thumbnails.
2. Ensure your CI builds FFmpeg with `--disable-gpl --disable-nonfree`.
3. Add a `NOTICES` file to the repo root (and reference it in `README.md`) containing:
   - FFmpeg copyright notice
   - LGPL 2.1 license text (or URL)
   - The FFmpeg version/commit used
4. Document the build procedure in `CLAUDE.md` or `CONTRIBUTING.md` so users can substitute a modified FFmpeg.
5. Keep plugin source as MIT — the two licenses are orthogonal.

**What notice/attribution is required at minimum:**

```
This software uses FFmpeg (https://ffmpeg.org), licensed under the
GNU Lesser General Public License v2.1 or later (LGPL 2.1+).
FFmpeg source code is available at https://ffmpeg.org/download.html.
```

This should appear in:
- `NOTICES` or `THIRD_PARTY_LICENSES` in the repo root.
- The app's About/Licenses screen (consumers are responsible for propagating this).

### Codecs to Explicitly Exclude

If you build FFmpeg yourself, always pass these flags to stay LGPL-safe:

```bash
--disable-gpl
--disable-nonfree
--disable-version3   # if you want strict LGPL 2.1; omit to allow LGPL 3 components
```

Specifically, never enable:
- `--enable-libx264` (GPL)
- `--enable-libx265` (GPL in full build)
- `--enable-libfdk-aac` (non-free)
- `--enable-openssl` (GPL-incompatible; use GnuTLS or built-in if TLS is needed)

The built-in H.264, H.265, VP9, and AV1 decoders in FFmpeg core are all LGPL and sufficient for thumbnail
extraction from common video formats.

---

## Summary Decision Matrix

| Question | Answer |
|----------|--------|
| FFmpeg default license (from source) | LGPL 2.1+ |
| GPL escalation trigger | Compiling with `--enable-gpl` + GPL third-party libs (libx264, libx265, etc.) |
| Can plugin remain MIT with dynamic LGPL FFmpeg? | **Yes** |
| Can plugin remain MIT with static LGPL FFmpeg? | **Yes** (open-source satisfies relink requirement) |
| Can plugin remain MIT with GPL FFmpeg? | **No** — binary becomes GPL |
| H.264 decode without GPL? | **Yes** — built-in `h264` decoder is LGPL |
| H.265 decode without GPL? | **Yes** — built-in `hevc` decoder is LGPL |
| VP9 decode without GPL? | **Yes** — built-in `vp9` decoder is LGPL |
| Do you need `libx264`/`libx265` for thumbnails? | **No** — those are encoders, not needed for decode |
| Minimum attribution required | LGPL notice + FFmpeg copyright in NOTICES file |
| Safest approach to avoid FFmpeg entirely | Platform-native APIs (all platforms except Linux) |
| Recommended approach for this plugin | Static LGPL-only FFmpeg via `ffmpeg-next` Rust crate + NOTICES file |
