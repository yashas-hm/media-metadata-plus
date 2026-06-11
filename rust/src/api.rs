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

/// Read metadata from multiple files in parallel using Rayon.
/// Each entry is `None` if the file is unsupported or corrupt.
pub fn read_metadata_batch(paths: Vec<String>) -> Vec<Option<MediaMeta>> {
    use rayon::prelude::*;
    paths
        .par_iter()
        .map(|p| read_metadata(p.clone()).ok())
        .collect()
}
