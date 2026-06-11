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
    let modification_time = read_modification_time(&mp4);

    // GPS and camera metadata — see read_itunes_text for the paths tried
    let (latitude, longitude, altitude) = read_gps(path)
        .map(|(lat, lon, alt)| (Some(lat), Some(lon), alt))
        .unwrap_or((None, None, None));

    Ok(MediaMeta {
        mime_type: mime.to_string(),
        width,
        height,
        captured_at_ms: creation_time,
        modified_at_ms: modification_time,
        camera_make: read_itunes_text(path, b"\xa9mak"),
        camera_model: read_itunes_text(path, b"\xa9mod"),
        latitude,
        longitude,
        altitude,
        duration_ms,
    })
}

fn read_creation_time(mp4: &mp4::Mp4Reader<std::fs::File>) -> Option<i64> {
    mp4_timestamp(mp4.moov.mvhd.creation_time)
}

fn read_modification_time(mp4: &mp4::Mp4Reader<std::fs::File>) -> Option<i64> {
    mp4_timestamp(mp4.moov.mvhd.modification_time)
}

fn mp4_timestamp(raw: u64) -> Option<i64> {
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

// ── iTunes / 3GPP atom reader ─────────────────────────────────────────────────

/// Extract a UTF-8 text value for atom_name, trying three locations in order:
///   1. moov > udta > meta > ilst > <atom> > data  (iTunes/Apple)
///   2. moov > udta > <atom>                        (3GPP, many Android cameras)
///   3. moov > meta > ilst > <atom> > data          (iTunes without udta wrapper)
fn read_itunes_text(path: &Path, atom_name: &[u8; 4]) -> Option<String> {
    let mut f = std::fs::File::open(path).ok()?;
    let moov = read_top_level_box(&mut f, b"moov")?;

    if let Some(udta) = find_child_box(&moov, b"udta") {
        // Path 1: iTunes — udta > meta > ilst > <atom> > data
        if let Some(s) = read_ilst_atom(udta, atom_name) {
            return Some(s);
        }
        // Path 2: 3GPP — <atom> directly under udta with (len, lang, text) header
        if let Some(atom) = find_child_box(udta, atom_name) {
            if let Some(s) = parse_3gpp_text(atom) {
                return Some(s);
            }
        }
    }

    // Path 3: meta > ilst directly under moov (no udta wrapper)
    if let Some(meta) = find_child_box(&moov, b"meta") {
        // meta is a FullBox: skip 4-byte version/flags header
        if let Some(s) = read_ilst_text(meta.get(4..).unwrap_or(meta), atom_name) {
            return Some(s);
        }
    }

    None
}

/// Read from: <container> > meta > ilst > <atom> > data  (meta is a FullBox).
fn read_ilst_atom(container: &[u8], atom_name: &[u8; 4]) -> Option<String> {
    let meta = find_child_box(container, b"meta")?;
    // meta is a FullBox: 4-byte version/flags before its children
    read_ilst_text(meta.get(4..)?, atom_name)
}

/// Read from: <meta-children> > ilst > <atom> > data.
fn read_ilst_text(meta_children: &[u8], atom_name: &[u8; 4]) -> Option<String> {
    let ilst = find_child_box(meta_children, b"ilst")?;
    let atom = find_child_box(ilst, atom_name)?;
    let data = find_child_box(atom, b"data")?;
    // iTunes data box: 4-byte type indicator + 4-byte locale + UTF-8 content
    let text = data.get(8..)?;
    utf8_nonempty(text)
}

/// Parse a 3GPP text atom payload: uint16 text-length, uint16 language, UTF-8 text.
fn parse_3gpp_text(data: &[u8]) -> Option<String> {
    if data.len() >= 4 {
        let declared_len = u16::from_be_bytes([data[0], data[1]]) as usize;
        // Validate: declared length must fit within the remaining bytes.
        if declared_len > 0 && 4 + declared_len <= data.len() {
            if let Some(s) = utf8_nonempty(&data[4..4 + declared_len]) {
                return Some(s);
            }
        }
        // Fallback: skip the 4-byte header and read whatever is left.
        if let Some(s) = utf8_nonempty(&data[4..]) {
            return Some(s);
        }
    }
    None
}

fn utf8_nonempty(bytes: &[u8]) -> Option<String> {
    if bytes.is_empty() {
        return None;
    }
    let s = std::str::from_utf8(bytes).ok()?.trim().to_string();
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
