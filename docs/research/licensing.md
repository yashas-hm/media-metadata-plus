# FFmpeg Licensing for `media_metadata_plus`

> This document is for informational and architectural guidance only — not legal advice.  
> Full analysis: `docs/AI/ffmpeg_licensing_analysis.md`

---

## Bottom Line

`media_metadata_plus` can remain **MIT-licensed** while using FFmpeg (LGPL 2.1+), under both dynamic and static linking,
provided FFmpeg is built without GPL components.

---

## FFmpeg License Structure

FFmpeg's core libraries (`libavcodec`, `libavformat`, `libavutil`, `libswscale`) are **LGPL 2.1+** by default.

GPL is triggered only when FFmpeg is compiled with `--enable-gpl` and GPL third-party libraries are linked:

| Component                       | License | Needed for thumbnails?      |
|---------------------------------|---------|-----------------------------|
| Built-in H.264 decoder (`h264`) | LGPL    | Yes                         |
| Built-in HEVC decoder (`hevc`)  | LGPL    | Yes                         |
| Built-in VP9 decoder (`vp9`)    | LGPL    | Yes                         |
| Built-in AV1 decoder (`av1`)    | LGPL    | Yes                         |
| `libx264` (H.264 encoder)       | GPL 2+  | **No** — this is an encoder |
| `libx265` (H.265 encoder)       | GPL 2+  | **No** — this is an encoder |

**Critical insight for thumbnail extraction:** `libx264` and `libx265` are *encoders*. Thumbnail extraction only
*decodes* video frames. The built-in FFmpeg decoders for H.264, H.265, VP9, and AV1 are all LGPL. There is no reason to
enable GPL components.

---

## LGPL Compliance

### Dynamic linking (simpler obligations)

LGPL 2.1 § 6 requires:

1. Provide FFmpeg source or a written offer pointing to it (e.g., a URL to the exact version on ffmpeg.org)
2. Include the LGPL 2.1 license text
3. Allow users to relink — satisfied automatically when the library is dynamically linked (users can swap the `.so`/
   `.dylib`)

The plugin source code remains MIT. LGPL applies only to the FFmpeg shared library. These licenses are orthogonal.

### Static linking

Static linking triggers LGPL's relinking provision: users must be able to substitute a modified FFmpeg and relink. For
an **open-source MIT plugin**, this is satisfied by construction — anyone can clone the repo, supply different `.a`
files, and rebuild. There is no need to distribute object files separately.

The plugin source must remain open-source to preserve this property. If a future commercial closed-source fork is
considered, static LGPL FFmpeg would require distributing object files or moving to dynamic linking.

---

## Recommended Build Configuration

```bash
./configure \
  --enable-static \
  --disable-shared \
  --disable-gpl \
  --disable-nonfree \
  --enable-decoder=h264,hevc,vp9,av1,mpeg4,mjpeg \
  --enable-demuxer=mov,mp4,matroska \
  --enable-protocol=file \
  --enable-swscale
```

**Never enable:** `--enable-libx264`, `--enable-libx265`, `--enable-libfdk-aac`, `--enable-openssl` — these trigger GPL
or non-free escalation.

---

## Attribution Requirements

Add to `NOTICES` or `README.md`:

```
This software uses libraries from the FFmpeg project (https://ffmpeg.org),
licensed under the GNU Lesser General Public License version 2.1 or later.
FFmpeg source code: https://ffmpeg.org/download.html (version X.Y.Z)
```

---

## Decision Matrix

| Question                                   | Answer                                    |
|--------------------------------------------|-------------------------------------------|
| Plugin stays MIT with dynamic LGPL FFmpeg? | **Yes**                                   |
| Plugin stays MIT with static LGPL FFmpeg?  | **Yes** (open-source satisfies relinking) |
| Plugin stays MIT with GPL FFmpeg?          | **No** — binary becomes GPL               |
| H.264 decode without GPL?                  | **Yes** — built-in `h264` decoder is LGPL |
| H.265 decode without GPL?                  | **Yes** — built-in `hevc` decoder is LGPL |
| Need `libx264`/`libx265` for thumbnails?   | **No** — those are encoders only          |