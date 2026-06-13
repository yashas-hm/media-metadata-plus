use ffmpeg_next as ffmpeg;

/// Seek into the video, decode one frame, scale it, and encode to JPEG bytes.
///
/// `seek_ms`: if None, seeks to 10% of the stream duration to skip black intros.
/// `max_width`: output is scaled down to this width preserving aspect ratio.
pub fn extract(
    path: &std::path::Path,
    seek_ms: Option<u64>,
    max_width: u32,
) -> anyhow::Result<Vec<u8>> {
    ffmpeg::init()?;

    let path_str = path
        .to_str()
        .ok_or_else(|| anyhow::anyhow!("non-UTF-8 path"))?;
    let mut ictx = ffmpeg::format::input(&path_str)?;

    // Extract stream info and build the decoder in a scoped block so the
    // immutable borrow on `ictx` is released before the mutable seek call.
    let (stream_idx, target_ms, mut decoder, rotation) = {
        let stream = ictx
            .streams()
            .best(ffmpeg::media::Type::Video)
            .ok_or_else(|| anyhow::anyhow!("no video stream found"))?;
        let idx = stream.index();
        let time_base = stream.time_base();

        let duration_ms = if stream.duration() > 0 {
            (stream.duration() as f64 * f64::from(time_base) * 1000.0) as i64
        } else {
            (ictx.duration() as f64 / 1000.0) as i64
        };

        let tgt = seek_ms.map(|ms| ms as i64).unwrap_or(duration_ms / 10);
        let dec = ffmpeg::codec::context::Context::from_parameters(stream.parameters())?
            .decoder()
            .video()?;

        let rot = stream_rotation(&stream);

        (idx, tgt, dec, rot)
    };

    // Pre-input seek is 10–100x faster than post-input seek for long videos
    let ts_us = target_ms * 1000;
    ictx.seek(ts_us, ts_us..)?;

    // Single-thread: we're decoding one frame; no benefit in spawning workers
    decoder.set_threading(ffmpeg::threading::Config::count(1));
    decoder.flush();

    let (out_w, out_h) = scale_dims(decoder.width(), decoder.height(), max_width);
    let mut scaler = ffmpeg::software::scaling::context::Context::get(
        decoder.format(),
        decoder.width(),
        decoder.height(),
        ffmpeg::format::pixel::Pixel::RGB24,
        out_w,
        out_h,
        ffmpeg::software::scaling::flag::Flags::BILINEAR,
    )?;

    for (stream, packet) in ictx.packets() {
        if stream.index() != stream_idx {
            continue;
        }
        decoder.send_packet(&packet)?;
        let mut frame = ffmpeg::util::frame::video::Video::empty();
        if decoder.receive_frame(&mut frame).is_ok() {
            let mut rgb = ffmpeg::util::frame::video::Video::empty();
            scaler.run(&frame, &mut rgb)?;

            // Stride may be wider than the pixel row — copy only the pixel bytes
            let stride = rgb.stride(0);
            let row_bytes = out_w as usize * 3;
            let pixels: Vec<u8> = (0..out_h as usize)
                .flat_map(|row| {
                    rgb.data(0)[row * stride..row * stride + row_bytes]
                        .iter()
                        .copied()
                })
                .collect();

            let (pixels, enc_w, enc_h) = apply_rotation(pixels, out_w, out_h, rotation);

            let mut buf = Vec::new();
            jpeg_encoder::Encoder::new(&mut buf, 85).encode(
                &pixels,
                enc_w as u16,
                enc_h as u16,
                jpeg_encoder::ColorType::Rgb,
            )?;
            return Ok(buf);
        }
    }

    Err(anyhow::anyhow!("no video frame decoded"))
}

/// Read the display rotation angle (degrees CW) from a video stream.
///
/// FFmpeg < 6 stored rotation as a `rotate` key in `AVStream::metadata`.
/// FFmpeg 7+ stores it only as an `AV_PKT_DATA_DISPLAYMATRIX` entry in
/// `AVStream::codecpar::coded_side_data`. We try both.
fn stream_rotation(stream: &ffmpeg::format::stream::Stream<'_>) -> i32 {
    // Older files / older FFmpeg builds
    if let Some(r) = stream
        .metadata()
        .get("rotate")
        .and_then(|v| v.parse::<i32>().ok())
    {
        return r;
    }
    // FFmpeg 7+: rotation lives in the display matrix on the codec parameters
    use ffmpeg_next::ffi::{av_display_rotation_get, AVPacketSideDataType};
    unsafe {
        let par = (*stream.as_ptr()).codecpar;
        if par.is_null() {
            return 0;
        }
        let nb = (*par).nb_coded_side_data as usize;
        let sd = (*par).coded_side_data;
        if sd.is_null() {
            return 0;
        }
        for i in 0..nb {
            let entry = &*sd.add(i);
            if entry.type_ == AVPacketSideDataType::AV_PKT_DATA_DISPLAYMATRIX {
                let angle = av_display_rotation_get(entry.data as *const i32);
                if !angle.is_nan() {
                    // av_display_rotation_get returns CCW degrees; negate for CW
                    return ((-angle).rem_euclid(360.0).round()) as i32;
                }
            }
        }
        0
    }
}

/// Rotate pixel data and return (rotated_pixels, new_width, new_height).
/// For 90°/270° the width and height are swapped in the output.
fn apply_rotation(pixels: Vec<u8>, w: u32, h: u32, rotation: i32) -> (Vec<u8>, u32, u32) {
    match rotation.rem_euclid(360) {
        90 => (rotate_90cw(&pixels, w as usize, h as usize), h, w),
        180 => (rotate_180(pixels), w, h),
        270 => (rotate_270cw(&pixels, w as usize, h as usize), h, w),
        _ => (pixels, w, h),
    }
}

/// 90° clockwise: output dimensions are h × w.
/// Input  (row, col) → output (col, h-1-row).
fn rotate_90cw(pixels: &[u8], w: usize, h: usize) -> Vec<u8> {
    let mut out = vec![0u8; pixels.len()];
    for row in 0..h {
        for col in 0..w {
            let src = (row * w + col) * 3;
            let dst = (col * h + (h - 1 - row)) * 3;
            out[dst..dst + 3].copy_from_slice(&pixels[src..src + 3]);
        }
    }
    out
}

/// 180°: reverse every pixel in place.
fn rotate_180(mut pixels: Vec<u8>) -> Vec<u8> {
    let n = pixels.len() / 3;
    for i in 0..n / 2 {
        let (a, b) = (i * 3, (n - 1 - i) * 3);
        pixels.swap(a, b);
        pixels.swap(a + 1, b + 1);
        pixels.swap(a + 2, b + 2);
    }
    pixels
}

/// 270° clockwise (= 90° counter-clockwise): output dimensions are h × w.
/// Input  (row, col) → output (w-1-col, row).
fn rotate_270cw(pixels: &[u8], w: usize, h: usize) -> Vec<u8> {
    let mut out = vec![0u8; pixels.len()];
    for row in 0..h {
        for col in 0..w {
            let src = (row * w + col) * 3;
            let dst = ((w - 1 - col) * h + row) * 3;
            out[dst..dst + 3].copy_from_slice(&pixels[src..src + 3]);
        }
    }
    out
}

fn scale_dims(src_w: u32, src_h: u32, max_w: u32) -> (u32, u32) {
    if src_w == 0 || src_h == 0 || src_w <= max_w {
        return (src_w, src_h);
    }
    let scale = max_w as f64 / src_w as f64;
    (max_w, (src_h as f64 * scale).round() as u32)
}