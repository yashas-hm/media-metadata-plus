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
    let (stream_idx, target_ms, mut decoder) = {
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
        (idx, tgt, dec)
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

            let mut buf = Vec::new();
            jpeg_encoder::Encoder::new(&mut buf, 85).encode(
                &pixels,
                out_w as u16,
                out_h as u16,
                jpeg_encoder::ColorType::Rgb,
            )?;
            return Ok(buf);
        }
    }

    Err(anyhow::anyhow!("no video frame decoded"))
}

fn scale_dims(src_w: u32, src_h: u32, max_w: u32) -> (u32, u32) {
    if src_w == 0 || src_h == 0 || src_w <= max_w {
        return (src_w, src_h);
    }
    let scale = max_w as f64 / src_w as f64;
    (max_w, (src_h as f64 * scale).round() as u32)
}
