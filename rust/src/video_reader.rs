use std::io::{Read, Seek, SeekFrom};
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

    let creation_time = read_creation_time(&mp4);

    // GPS and camera metadata from iTunes-style atoms in moov > udta > meta > ilst
    let (latitude, longitude, altitude) = read_gps(path)
        .map(|(lat, lon, alt)| (Some(lat), Some(lon), alt))
        .unwrap_or((None, None, None));

    Ok(MediaMeta {
        mime_type: mime.to_string(),
        width,
        height,
        captured_at_ms: creation_time,
        camera_make: read_itunes_text(path, b"\xa9mak"),
        camera_model: read_itunes_text(path, b"\xa9mod"),
        latitude,
        longitude,
        altitude,
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

fn read_gps(path: &Path) -> Option<(f64, f64, Option<f64>)> {
    let s = read_itunes_text(path, b"\xa9xyz")?;
    parse_iso6709(&s)
}

/// Parse an ISO 6709 coordinate string of the form `+LAT+LON/` or `+LAT+LON+ALT/`.
fn parse_iso6709(s: &str) -> Option<(f64, f64, Option<f64>)> {
    let s = s.trim().trim_end_matches('/');
    if s.is_empty() {
        return None;
    }
    let bytes = s.as_bytes();
    // Find the start of longitude: the second sign character (skip index 0)
    let lon_pos = bytes[1..].iter().position(|&b| b == b'+' || b == b'-')? + 1;
    let lat_str = &s[..lon_pos];
    let rest = &s[lon_pos..];
    // Find optional altitude sign
    let alt_pos = rest.as_bytes()[1..]
        .iter()
        .position(|&b| b == b'+' || b == b'-')
        .map(|p| p + 1);
    let (lon_str, alt_str) = match alt_pos {
        Some(p) => (&rest[..p], Some(&rest[p..])),
        None => (rest, None),
    };
    let lat: f64 = lat_str.parse().ok()?;
    let lon: f64 = lon_str.parse().ok()?;
    let alt = alt_str.and_then(|a| a.parse().ok());
    Some((lat, lon, alt))
}

// ── iTunes atom reader ────────────────────────────────────────────────────────

/// Extract a UTF-8 text value from moov > udta > meta > ilst > <atom_name> > data.
/// atom_name is the raw 4-byte name (e.g. b"\xa9xyz", b"\xa9mak", b"\xa9mod").
fn read_itunes_text(path: &Path, atom_name: &[u8; 4]) -> Option<String> {
    let mut f = std::fs::File::open(path).ok()?;
    let moov = read_top_level_box(&mut f, b"moov")?;
    let udta = find_child_box(&moov, b"udta")?;
    let meta = find_child_box(udta, b"meta")?;
    // meta has a 4-byte version/flags header before its children
    let ilst = find_child_box(meta.get(4..)?, b"ilst")?;
    let atom = find_child_box(ilst, atom_name)?;
    let data = find_child_box(atom, b"data")?;
    // iTunes data box: 4-byte type + 4-byte locale + content
    let text = data.get(8..)?;
    if text.is_empty() {
        return None;
    }
    let s = std::str::from_utf8(text).ok()?.trim().to_string();
    if s.is_empty() { None } else { Some(s) }
}

// ── MP4 box scanner ───────────────────────────────────────────────────────────

/// Scan a file sequentially from the start, seeking past non-target top-level boxes.
/// Returns the content bytes of the named box (excluding its 8-byte header).
fn read_top_level_box(f: &mut std::fs::File, name: &[u8; 4]) -> Option<Vec<u8>> {
    f.seek(SeekFrom::Start(0)).ok()?;
    loop {
        let mut size_bytes = [0u8; 4];
        f.read_exact(&mut size_bytes).ok()?;
        let raw_size = u32::from_be_bytes(size_bytes);

        let mut name_bytes = [0u8; 4];
        f.read_exact(&mut name_bytes).ok()?;

        let (content_size, is_target) = if raw_size == 1 {
            // Extended size: next 8 bytes hold the full box size (including all headers)
            let mut ext = [0u8; 8];
            f.read_exact(&mut ext).ok()?;
            let full = u64::from_be_bytes(ext);
            (full.saturating_sub(16), &name_bytes == name)
        } else if raw_size == 0 {
            // Box extends to end of file
            return if &name_bytes == name {
                let mut content = Vec::new();
                f.read_to_end(&mut content).ok()?;
                Some(content)
            } else {
                None
            };
        } else {
            (raw_size as u64 - 8, &name_bytes == name)
        };

        if is_target {
            let mut content = vec![0u8; content_size as usize];
            f.read_exact(&mut content).ok()?;
            return Some(content);
        }

        f.seek(SeekFrom::Current(content_size as i64)).ok()?;
    }
}

/// Scan a byte slice for a named child box.
/// Returns the child's content bytes (excluding its 8-byte header).
fn find_child_box<'a>(data: &'a [u8], name: &[u8; 4]) -> Option<&'a [u8]> {
    let mut i = 0usize;
    while i + 8 <= data.len() {
        let size = u32::from_be_bytes([data[i], data[i + 1], data[i + 2], data[i + 3]]) as usize;
        if size < 8 || i + size > data.len() {
            break;
        }
        if data[i + 4..i + 8] == *name {
            return Some(&data[i + 8..i + size]);
        }
        i += size;
    }
    None
}
