# Decisions

Quick-reference index — what was decided and why, one glance each. Full detail is in the linked doc when you need
it.

## Architecture

| Decision | Why |
|---|---|
| Rust-only, no platform channels | One codebase cross-compiles to all 5 platforms. |
| HEIC read via `kamadak-exif`'s EXIF box, not `libheif` | Lighter dependency, no image decode needed for metadata. |
| Format detected by **magic bytes**, never file extension | Extensions can be wrong, missing, or lie. |
| Consumers never compile Rust — prebuilt binaries only | Matches every other native Flutter plugin. Plugin briefly shipped broken with `hook/build.dart` + `ffiPlugin: true` both trying to compile Rust at once; removed. |

## macOS / iOS via SPM

*Full error log: [`spm_conversion_problems.md`](spm_conversion_problems.md)*

- SPM **cannot** run `cargo build` (declarative, sandboxed, binary-targets only) — this is *why* macOS/iOS ship
  a prebuilt `.xcframework`, not a preference.
- `Package.swift`'s `binaryTarget` points at a **checksummed published GitHub Release**. Consequence: local
  `flutter test -d macos` normally runs *last release's* Rust code, not your working tree.
  → Fixed by `MMP_LOCAL_XCFRAMEWORK` (set by `scripts/test.sh`, unset everywhere else — pub.dev/CI/release
  untouched).
- `example/macos/` is gitignored and regenerated, so its App Sandbox entitlement resets to `true` on every
  fresh scaffold, which breaks fixture paths in tests. `scripts/test.sh` disables it for Debug builds each run.
- Integration-test CI runs on **Linux only** — it builds its `.so` from source and isn't affected by any of the
  above. A macOS CI job was considered and rejected as redundant cost.

## FFmpeg

*Full analysis: [`ffmpeg_licensing_analysis.md`](ffmpeg_licensing_analysis.md), [`research/`](research/)*

- Self-built **LGPL-only** static FFmpeg (`--disable-gpl --disable-nonfree`), served from GitHub Releases,
  fetched via `FFMPEG_DIR`.
- **Why self-built:** BtbN doesn't cover macOS/iOS/Android; ffmpeg-kit is archived at v6.0; building from source
  every CI run costs 15–30 min vs. ~3 min for a prebuilt.
- **Why LGPL-only:** GPL codecs (`libx264`/`libx265`) would force the whole binary GPL. Built-in decoders
  (H.264/HEVC/VP9/AV1) are LGPL and sufficient — thumbnails only ever decode, never encode.

## Thumbnails

*Full comparison: [`research/`](research/)*

- Built in-plugin, not a separate package — every existing option was mobile-only, needed a system FFmpeg
  install, or is archived.
- Two-phase: `covr` iTunes atom fast path (no decode) → FFmpeg fallback (seek to 10% duration, pre-input seek,
  decode one frame).

## `mp4` crate limitation

- The crate is ISO-only; QuickTime files with a legacy "Sound Sample Description" audio atom fail its *entire*
  header parse.
- Fallback: a hand-rolled `container::isobmff` parser reads just `mvhd`/`tkhd`, skipping audio entirely.

## Rust source layout

- `container/isobmff.rs` — generic, unit-tested box-tree walking (mirrors `kamadak-exif`'s own internal module
  of the same name).
- `readers/{exif,video,thumbnail}.rs` — format-specific logic built on top of it.

## Deferred — designed, never built

- **EXIF write** — splice a new APP1 segment into JPEG bytes, JPEG-only scope. See
  [`exif_write_plan.md`](exif_write_plan.md).
- **Web/WASM** — bytes-based `readBytes`/`readAllBytes` API. No `web` platform entry in `pubspec.yaml`.
