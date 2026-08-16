# Rust FFmpeg Integration for Thumbnail Extraction

> Research date: 2026-06-11. Figures verified against crates.io and GitHub source.

---

## Summary

`ffmpeg-next` v8.1.0 is the correct choice. It wraps the FFmpeg C libraries with safe Rust, supports all required
platforms (macOS, Windows, Linux, iOS arm64, Android arm64/x86_64), and can seek → decode one frame → scale → encode to
JPEG entirely in-memory without spawning external processes.

The primary trade-off is build complexity: FFmpeg must either be installed on the build host or pre-built static `.a`
files must be supplied per-platform.

---

## 1. `ffmpeg-next`

**Crates.io:** https://crates.io/crates/ffmpeg-next  
**GitHub:** https://github.com/zmwangx/rust-ffmpeg  
**Version:** 8.1.0 (March 2026) | **License:** WTFPL | **Downloads:** ~4.1 million (2.05M last 30 days)

The version number tracks FFmpeg's major release (8.1.0 = FFmpeg 8.x). It wraps `ffmpeg-sys-next` (the raw bindgen FFI
layer) with ergonomic Rust types.

### Can it seek → decode → encode to JPEG in memory?

Yes, all three steps are supported.

**Seek:**

```rust
ictx.seek(timestamp_us, timestamp_us..)?;  // avformat_seek_file
decoder.flush();                            // must flush after seek
```

**Decode one frame:**

```rust
decoder.send_packet(&packet)?;
let mut frame = Video::empty();
decoder.receive_frame(&mut frame)?;
```

**Scale to target dimensions:**

```rust
let mut scaler = Context::get(
    decoder.format(), decoder.width(), decoder.height(),
    Pixel::RGB24, out_w, out_h,
    Flags::BILINEAR,
)?;
let mut rgb = Video::empty();
scaler.run(&frame, &mut rgb)?;
// rgb.data(0) is a contiguous RGB24 byte slice
```

**Encode to JPEG (pure-Rust, no FFmpeg encoder needed):**

```rust
// jpeg-encoder crate: 100% pure Rust, 6.3M downloads
let encoder = jpeg_encoder::Encoder::new(Vec::new(), 85);
let jpeg_bytes = encoder.encode(rgb.data(0), out_w as u16, out_h as u16,
                                jpeg_encoder::ColorType::Rgb)?;
```

### Minimal feature set for thumbnail extraction

```toml
[dependencies]
ffmpeg-next = { version = "8", default-features = false,
    features = ["codec", "format", "software-scaling"] }
jpeg-encoder = "0.7"
```

Disabling `device`, `filter`, `software-resampling`, and `postprocessing` significantly reduces compile time and binary
size.

### Build requirements

| Platform             | Tools required                                                                             |
|----------------------|--------------------------------------------------------------------------------------------|
| macOS (dev)          | `brew install ffmpeg pkg-config`                                                           |
| macOS (CI/release)   | Prebuilt static `.a` files + `FFMPEG_DIR` env var                                          |
| Linux                | `apt install clang libavcodec-dev libavformat-dev libavutil-dev libswscale-dev pkg-config` |
| Windows              | LLVM/clang + prebuilt FFmpeg dev headers, `FFMPEG_DIR` env var                             |
| iOS arm64            | macOS build host + `xcrun` (via `build` feature) or ffmpeg-kit xcframework                 |
| Android arm64/x86_64 | `cargo-ndk` + NDK (the build script reads its env vars automatically)                      |

`clang`/`libclang` is required on all platforms for `bindgen` (generates the FFI bindings at compile time).

### Static linking modes

**Mode 1 — System dynamic (default, development only):**  
`pkg-config` locates installed `libavcodec.dylib` etc. Requires FFmpeg at runtime. Not suitable for distribution.

**Mode 2 — Static via `FFMPEG_DIR` (recommended for release):**

```
FFMPEG_DIR=/path/to/prebuilt cargo build --features static
```

The build script links `rustc-link-lib=static=avcodec` etc. No FFmpeg compilation during build. Supply prebuilt `.a`
files per target platform.

**Mode 3 — Build from source (`build` feature):**  
`ffmpeg-sys-next`'s build script clones and compiles FFmpeg during `cargo build`. Adds 15–30 minutes to a clean CI
build. Handles iOS and Android cross-compilation automatically.

### Cross-compilation

**iOS arm64:** The `build` feature calls `xcrun --sdk iphoneos --show-sdk-path`, sets
`--arch arm64 --target-os=darwin --sysroot=<path>`, and passes `-mios-version-min=11.0`. Works only on a macOS build
host.

**Android arm64/x86_64:** The build script reads `cargo-ndk` environment variables (`CC_<target>`,
`CARGO_NDK_SYSROOT_PATH`, `CFLAGS_<target>`). For x86_64 it automatically adds `--disable-asm` to avoid PIC issues.

### CI build time

| Scenario                                            | Approximate time |
|-----------------------------------------------------|------------------|
| System dynamic (macOS)                              | ~2 min           |
| Static via `FFMPEG_DIR` (prebuilt `.a`)             | ~3 min           |
| `build` feature, cold (compiles FFmpeg from source) | 15–30 min        |
| `build` feature, warm cache                         | ~3 min           |

Cache `$CARGO_TARGET_DIR/build/ffmpeg-sys-next-*/out/` across CI runs to avoid the cold-build penalty.

---

## 2. `ffmpeg-sys-next`

**Crates.io:** https://crates.io/crates/ffmpeg-sys-next  
**GitHub:** https://github.com/zmwangx/rust-ffmpeg-sys  
**Version:** 8.1.0 | **Downloads:** ~4.3 million

Raw `extern "C"` FFI declarations auto-generated by `bindgen` from FFmpeg headers. `ffmpeg-next` depends on it as its
underlying layer. You would only reach for `ffmpeg-sys-next` directly if you need an FFmpeg API not yet exposed by the
safe wrapper.

The `static` Cargo feature changes all `rustc-link-lib=avcodec` directives to `rustc-link-lib=static=avcodec` and also
emits the platform-specific system framework/library links needed by a statically linked FFmpeg (e.g., `CoreFoundation`,
`VideoToolbox`, `AudioToolbox` on Apple platforms; `ole32`, `gdi32`, `mfuuid` on Windows).

---

## 3. Alternatives Considered and Rejected

### `ac_ffmpeg` (Angelcam)

52,000 downloads vs `ffmpeg-next`'s 4.1 million. Supports FFmpeg v4–v7 only (no v8). Declining maintenance activity. Not
recommended.

### Pure-Rust H.264/H.265 decoding

**Not viable** as a general-purpose solution for a production Flutter plugin.

- `openh264` crate (408K downloads): Rust bindings to Cisco's H.264 decoder. Bundles C++ source (compiles via `cc`
  crate — no system install needed). Limitations: H.264 only (no H.265/HEVC), raw bitstream input only (you must demux
  the MP4 container yourself — the existing `mp4` crate cannot do this; it would require implementing `ctts`/`stts`/
  `stsc`/`stco` box walking for timestamp-based sample extraction — ~500–1000 lines of custom container code).
- No pure-Rust H.265 decoder with pixel output exists as of 2025.
- `dav1d` bindings: AV1 only, not H.264/H.265.
- `h264-reader`: bitstream parser, not a decoder (no pixel output).

Conclusion: pure-Rust decoding requires implementing a container demuxer from scratch and only covers H.264. Not worth
it when `ffmpeg-next` handles every codec in one API call.

### `video-rs`

Higher-level wrapper over `ffmpeg-next`. Self-described as a work-in-progress. No documented seek support. Unconfirmed
iOS/Android testing. Use `ffmpeg-next` directly for the control required in a Flutter plugin.

---

## 4. Prebuilt Static Library Sources

The `FFMPEG_DIR` approach requires prebuilt `.a` files per platform. Available sources:

| Source                                                         | Platforms                           | Notes                                                                                                                    |
|----------------------------------------------------------------|-------------------------------------|--------------------------------------------------------------------------------------------------------------------------|
| **BtbN/FFmpeg-Builds** (https://github.com/BtbN/FFmpeg-Builds) | Linux x64, Linux arm64, Windows x64 | LGPL and GPL variants, static and shared. macOS/iOS/Android not covered.                                                 |
| **ffmpeg-kit** (https://github.com/arthenica/ffmpeg-kit)       | iOS, Android, macOS                 | Pre-built `.xcframework` (iOS/macOS) and `.aar` (Android). **Archived 2024, last release v6.0 (2023).** FFmpeg 6.0 only. |
| **homebrew**                                                   | macOS x86_64/arm64                  | Dynamic libs only. Development use.                                                                                      |
| **`build` feature**                                            | All targets                         | Compiles FFmpeg from source. Slow but always current.                                                                    |

The archival of `ffmpeg-kit` means iOS and Android prebuilt static libs are no longer maintained. The `build` feature in
`ffmpeg-sys-next` is now the more maintainable path for iOS and Android, at the cost of longer CI builds.

---

## 5. Minimal FFmpeg Build Configuration

For thumbnail extraction, a stripped-down FFmpeg configuration keeps the static library under 10 MB:

```bash
./configure \
  --enable-static \
  --disable-shared \
  --disable-everything \
  --disable-gpl \
  --disable-nonfree \
  --enable-demuxer=mov,mp4,matroska \
  --enable-decoder=h264,hevc,vp9,av1,mpeg4,mjpeg \
  --enable-protocol=file \
  --enable-swscale
```

This covers H.264, H.265/HEVC, VP9, AV1, and MPEG-4 (the video codecs found in real-world MP4/MOV files), while
excluding all encoders, muxers, filters, and network protocols.

---

## 6. Complete Implementation Sketch

```rust
use ffmpeg_next as ffmpeg;

pub fn extract_thumbnail(path: &str, seek_ms: i64, max_width: u32) -> anyhow::Result<Vec<u8>> {
    ffmpeg::init()?;

    let mut ictx = ffmpeg::format::input(path)?;
    let stream = ictx.streams().best(ffmpeg::media::Type::Video)
        .ok_or_else(|| anyhow::anyhow!("no video stream"))?;
    let stream_idx = stream.index();

    // Pre-input seek (much faster than post-input seek for long videos)
    let ts_us = seek_ms * 1000;
    ictx.seek(ts_us, ts_us..)?;

    let mut decoder = ffmpeg::codec::context::Context::from_parameters(stream.parameters())?
        .decoder().video()?;
    decoder.set_threading(ffmpeg::threading::Config::count(1)); // 1 thread per thumbnail
    decoder.flush();

    let (out_w, out_h) = thumbnail_dims(decoder.width(), decoder.height(), max_width);
    let mut scaler = ffmpeg::software::scaling::context::Context::get(
        decoder.format(), decoder.width(), decoder.height(),
        ffmpeg::format::pixel::Pixel::RGB24,
        out_w, out_h,
        ffmpeg::software::scaling::flag::Flags::BILINEAR,
    )?;

    for (stream, packet) in ictx.packets() {
        if stream.index() != stream_idx { continue; }
        decoder.send_packet(&packet)?;
        let mut frame = ffmpeg::util::frame::video::Video::empty();
        if decoder.receive_frame(&mut frame).is_ok() {
            let mut rgb = ffmpeg::util::frame::video::Video::empty();
            scaler.run(&frame, &mut rgb)?;

            let stride = rgb.stride(0);
            let pixels: Vec<u8> = (0..out_h as usize)
                .flat_map(|row| &rgb.data(0)[row*stride..row*stride + out_w as usize * 3])
                .copied()
                .collect();

            let enc = jpeg_encoder::Encoder::new(Vec::new(), 85);
            return Ok(enc.encode(&pixels, out_w as u16, out_h as u16,
                                 jpeg_encoder::ColorType::Rgb)?);
        }
    }
    Err(anyhow::anyhow!("no video frame decoded"))
}

fn thumbnail_dims(src_w: u32, src_h: u32, max_w: u32) -> (u32, u32) {
    if src_w <= max_w { return (src_w, src_h); }
    let scale = max_w as f32 / src_w as f32;
    (max_w, (src_h as f32 * scale).round() as u32)
}
```

---

## 7. Summary Table

| Crate              | Version | Downloads | Platforms | Static link                           | Practical for thumbnail   |
|--------------------|---------|-----------|-----------|---------------------------------------|---------------------------|
| `ffmpeg-next`      | 8.1.0   | 4.1M      | All       | Yes (`static` feature + `FFMPEG_DIR`) | **Yes — recommended**     |
| `ffmpeg-sys-next`  | 8.1.0   | 4.3M      | All       | Yes                                   | Underpins ffmpeg-next     |
| `ac_ffmpeg`        | 0.19.0  | 52K       | Partial   | Yes                                   | Not recommended           |
| `openh264`         | 0.9.3   | 408K      | Partial   | Bundles C++                           | H.264 only, no MP4 demux  |
| `mp4` (in project) | 0.14.0  | 10.8M     | All       | Pure Rust                             | Container only, no decode |
| `jpeg-encoder`     | 0.7.0   | 6.3M      | All       | Pure Rust                             | JPEG encode complement    |