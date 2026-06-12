pub struct MediaMeta {
    pub mime_type: String,
    pub width: Option<u32>,
    pub height: Option<u32>,
    pub captured_at_ms: Option<i64>,
    pub modified_at_ms: Option<i64>,
    pub camera_make: Option<String>,
    pub camera_model: Option<String>,
    pub latitude: Option<f64>,
    pub longitude: Option<f64>,
    pub altitude: Option<f64>,
    pub duration_ms: Option<u64>,
}

pub fn read_metadata(path: String) -> anyhow::Result<MediaMeta> {
    let path = std::path::Path::new(&path);
    let mime = crate::mime::detect(path)?;

    match mime.as_str() {
        "image/jpeg"
        | "image/heic"
        | "image/heif"
        | "image/png"
        | "image/webp"
        // TIFF-based: generic TIFF, DNG, NEF, ARW, CR2
        | "image/tiff"
        | "image/x-canon-cr2" => crate::exif_reader::read(path, &mime),
        "video/mp4" | "video/quicktime" => crate::video_reader::read(path, &mime),
        _ => Err(anyhow::anyhow!("unsupported format: {mime}")),
    }
}

/// Extract a thumbnail from a video file, returning raw JPEG/PNG bytes.
///
/// For MP4 and MOV files, reads the embedded cover-art image from the file's
/// `covr` iTunes atom without decoding any video frames. `time_ms` is accepted
/// for API compatibility but has no effect in this implementation — the `covr`
/// atom is a single fixed image independent of playback position.
///
/// If `save_path` is provided the bytes are also written to that path
/// (parent directories are created automatically).
///
/// Returns an error if no embedded thumbnail is present or the format is
/// unsupported.
pub fn extract_video_thumbnail(
    path: String,
    time_ms: Option<u64>,
    save_path: Option<String>,
) -> anyhow::Result<Vec<u8>> {
    let path = std::path::Path::new(&path);
    let mime = crate::mime::detect(path)?;

    let bytes = match mime.as_str() {
        "video/mp4" | "video/quicktime" => {
            // Fast path: embedded cover-art atom (no decode required)
            if let Some(b) = crate::video_reader::read_covr_thumbnail(path) {
                b
            } else {
                // Fallback: seek-and-decode via FFmpeg
                crate::thumbnail_reader::extract(path, time_ms, 640)?
            }
        }
        _ => anyhow::bail!("thumbnail extraction not supported for {mime}"),
    };

    if let Some(dest) = save_path {
        let dest = std::path::Path::new(&dest);
        if let Some(parent) = dest.parent() {
            std::fs::create_dir_all(parent)?;
        }
        std::fs::write(dest, &bytes)?;
    }

    Ok(bytes)
}

/// Read metadata from multiple files in parallel using Rayon.
/// Each entry is `None` if the file is unsupported or corrupt.
pub fn read_metadata_batch(paths: Vec<String>) -> Vec<Option<MediaMeta>> {
    use rayon::prelude::*;
    paths
        .par_iter()
        .map(|p| read_metadata(p.clone()).ok())
        .collect()
}
