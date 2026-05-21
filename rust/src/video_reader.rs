use std::path::Path;

use crate::api::MediaMeta;

pub fn read(path: &Path, mime: &str) -> anyhow::Result<MediaMeta> {
    let f = std::fs::File::open(path)?;
    let size = f.metadata()?.len();
    let mp4 = mp4::Mp4Reader::read_header(f, size)?;

    let track = mp4
        .tracks()
        .values()
        .find(|t| t.track_type().ok() == Some(mp4::TrackType::Video));

    let (width, height, duration_ms) = match track {
        Some(t) => (
            Some(t.width() as u32),
            Some(t.height() as u32),
            Some(t.duration().as_millis() as u64),
        ),
        None => (None, None, None),
    };

    Ok(MediaMeta {
        mime_type: mime.to_string(),
        width,
        height,
        captured_at_ms: read_creation_time(&mp4),
        camera_make: None,
        camera_model: None,
        latitude: None,
        longitude: None,
        altitude: None,
        duration_ms,
    })
}

fn read_creation_time(mp4: &mp4::Mp4Reader<std::fs::File>) -> Option<i64> {
    let raw = mp4.moov.mvhd.creation_time;
    if raw == 0 {
        return None;
    }
    // MP4 epoch is 1904-01-01; offset to Unix epoch is 2082844800 seconds
    let unix_secs = raw.saturating_sub(2082844800) as i64;
    Some(unix_secs * 1000)
}
