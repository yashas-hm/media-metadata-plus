use std::path::Path;

use crate::api::MediaMeta;
use crate::container::isobmff::{find_all_child_boxes, find_child_box, read_top_level_box};

pub fn read(path: &Path, mime: &str) -> anyhow::Result<MediaMeta> {
    let f = std::fs::File::open(path)?;
    let size = f.metadata()?.len();

    let (width, height, duration_ms, creation_time, modification_time) =
        match mp4::Mp4Reader::read_header(f, size) {
            Ok(mp4) => {
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

                (
                    width,
                    height,
                    duration_ms,
                    read_creation_time(&mp4),
                    read_modification_time(&mp4),
                )
            }
            // Some QuickTime .mov files describe their audio track with a
            // legacy "Sound Sample Description" (version 1/2, with extra
            // fields and a nested `wave` atom) that the `mp4` crate's
            // ISO-only mp4a parser can't handle, failing the entire header
            // parse even though we only need the video track and
            // movie-level timing. Fall back to a minimal manual box walk
            // for just those fields; if that also fails, surface the
            // original (more informative) crate error.
            Err(crate_err) => match read_moov_fallback(path) {
                Ok(fields) => fields,
                Err(_) => return Err(crate_err.into()),
            },
        };

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

/// Manual fallback for files whose `moov` box the `mp4` crate can't fully
/// parse (see the QuickTime sound-description note in `read`). Walks just
/// enough of the box tree to recover movie-level timing from `mvhd` and the
/// video track's pixel dimensions from its `tkhd`, without touching audio.
type FallbackFields = (
    Option<u32>,
    Option<u32>,
    Option<u64>,
    Option<i64>,
    Option<i64>,
);

fn read_moov_fallback(path: &Path) -> anyhow::Result<FallbackFields> {
    let mut f = std::fs::File::open(path)?;
    let moov = read_top_level_box(&mut f, b"moov").ok_or_else(|| anyhow::anyhow!("no moov box"))?;

    let mvhd = find_child_box(&moov, b"mvhd").ok_or_else(|| anyhow::anyhow!("no mvhd box"))?;
    let (creation_time, modification_time, duration_ms) =
        parse_mvhd(mvhd).ok_or_else(|| anyhow::anyhow!("malformed mvhd box"))?;

    let mut width = None;
    let mut height = None;
    for trak in find_all_child_boxes(&moov, b"trak") {
        let is_video = find_child_box(trak, b"mdia")
            .and_then(|mdia| find_child_box(mdia, b"hdlr"))
            .and_then(|hdlr| hdlr.get(8..12))
            .map(|handler_type| handler_type == b"vide")
            .unwrap_or(false);
        if !is_video {
            continue;
        }
        if let Some(tkhd) = find_child_box(trak, b"tkhd") {
            if let Some(dims) = parse_tkhd_dimensions(tkhd) {
                (width, height) = (Some(dims.0), Some(dims.1));
            }
        }
        break;
    }

    Ok((width, height, duration_ms, creation_time, modification_time))
}

/// Parse `moov > mvhd` (a FullBox): returns (creation_time, modification_time, duration_ms).
fn parse_mvhd(mvhd: &[u8]) -> Option<(Option<i64>, Option<i64>, Option<u64>)> {
    let version = *mvhd.first()?;
    let (creation_raw, modification_raw, timescale, duration_raw) = if version == 1 {
        (
            u64::from_be_bytes(mvhd.get(4..12)?.try_into().ok()?),
            u64::from_be_bytes(mvhd.get(12..20)?.try_into().ok()?),
            u32::from_be_bytes(mvhd.get(20..24)?.try_into().ok()?),
            u64::from_be_bytes(mvhd.get(24..32)?.try_into().ok()?),
        )
    } else {
        (
            u32::from_be_bytes(mvhd.get(4..8)?.try_into().ok()?) as u64,
            u32::from_be_bytes(mvhd.get(8..12)?.try_into().ok()?) as u64,
            u32::from_be_bytes(mvhd.get(12..16)?.try_into().ok()?),
            u32::from_be_bytes(mvhd.get(16..20)?.try_into().ok()?) as u64,
        )
    };

    let duration_ms = (timescale > 0).then(|| duration_raw.saturating_mul(1000) / timescale as u64);
    Some((
        mp4_timestamp(creation_raw),
        mp4_timestamp(modification_raw),
        duration_ms,
    ))
}

/// Parse `trak > tkhd` (a FullBox) for pixel width/height (fixed-point 16.16).
fn parse_tkhd_dimensions(tkhd: &[u8]) -> Option<(u32, u32)> {
    let version = *tkhd.first()?;
    // version 1: creation(8)+modification(8)+track_ID(4)+reserved(4)+duration(8) = 32
    // version 0: creation(4)+modification(4)+track_ID(4)+reserved(4)+duration(4) = 20
    let version_block_len = if version == 1 { 32 } else { 20 };
    // reserved(8) + layer(2) + alternate_group(2) + volume(2) + reserved(2) + matrix(36) = 52
    let dims_offset = 4 + version_block_len + 52;
    let width_raw = u32::from_be_bytes(tkhd.get(dims_offset..dims_offset + 4)?.try_into().ok()?);
    let height_raw = u32::from_be_bytes(
        tkhd.get(dims_offset + 4..dims_offset + 8)?
            .try_into()
            .ok()?,
    );
    Some((width_raw >> 16, height_raw >> 16))
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

/// Read raw bytes from: <meta-children> > ilst > <atom> > data.
fn read_ilst_bytes(meta_children: &[u8], atom_name: &[u8; 4]) -> Option<Vec<u8>> {
    let ilst = find_child_box(meta_children, b"ilst")?;
    let atom = find_child_box(ilst, atom_name)?;
    let data = find_child_box(atom, b"data")?;
    // iTunes data box: 4-byte type indicator + 4-byte locale + payload
    let bytes = data.get(8..)?;
    if bytes.is_empty() {
        None
    } else {
        Some(bytes.to_vec())
    }
}

/// Extract the embedded cover-art JPEG/PNG from the `covr` iTunes atom.
///
/// Tries two paths (matching the same structure used by `read_itunes_text`):
///   1. moov > udta > meta > ilst > covr > data  (iTunes/Apple)
///   2. moov > meta > ilst > covr > data          (no udta wrapper)
pub fn read_covr_thumbnail(path: &std::path::Path) -> Option<Vec<u8>> {
    let mut f = std::fs::File::open(path).ok()?;
    let moov = read_top_level_box(&mut f, b"moov")?;

    // Path 1: moov > udta > meta > ilst > covr > data
    if let Some(udta) = find_child_box(&moov, b"udta") {
        if let Some(meta) = find_child_box(udta, b"meta") {
            if let Some(bytes) = read_ilst_bytes(meta.get(4..).unwrap_or(meta), b"covr") {
                return Some(bytes);
            }
        }
    }

    // Path 2: moov > meta > ilst > covr > data
    if let Some(meta) = find_child_box(&moov, b"meta") {
        if let Some(bytes) = read_ilst_bytes(meta.get(4..).unwrap_or(meta), b"covr") {
            return Some(bytes);
        }
    }

    None
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
    if s.is_empty() {
        None
    } else {
        Some(s)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mp4_timestamp_treats_zero_as_absent() {
        assert_eq!(mp4_timestamp(0), None);
    }

    #[test]
    fn mp4_timestamp_converts_1904_epoch_to_unix_millis() {
        // 100 seconds after the Unix epoch, expressed in the MP4 (1904) epoch.
        assert_eq!(mp4_timestamp(2_082_844_800 + 100), Some(100_000));
    }

    /// Build an `mvhd` FullBox body (version 0 or 1) with the given raw field values.
    fn build_mvhd(
        version: u8,
        creation: u64,
        modification: u64,
        timescale: u32,
        duration: u64,
    ) -> Vec<u8> {
        let mut buf = vec![version, 0, 0, 0];
        if version == 1 {
            buf.extend_from_slice(&creation.to_be_bytes());
            buf.extend_from_slice(&modification.to_be_bytes());
            buf.extend_from_slice(&timescale.to_be_bytes());
            buf.extend_from_slice(&duration.to_be_bytes());
        } else {
            buf.extend_from_slice(&(creation as u32).to_be_bytes());
            buf.extend_from_slice(&(modification as u32).to_be_bytes());
            buf.extend_from_slice(&timescale.to_be_bytes());
            buf.extend_from_slice(&(duration as u32).to_be_bytes());
        }
        buf
    }

    #[test]
    fn parse_mvhd_version0() {
        let mvhd = build_mvhd(0, 2_082_844_800 + 100, 2_082_844_800 + 200, 1000, 2500);
        let (creation, modification, duration_ms) = parse_mvhd(&mvhd).unwrap();
        assert_eq!(creation, Some(100_000));
        assert_eq!(modification, Some(200_000));
        assert_eq!(duration_ms, Some(2500));
    }

    #[test]
    fn parse_mvhd_version1() {
        let mvhd = build_mvhd(1, 2_082_844_800 + 300, 2_082_844_800 + 400, 48_000, 96_000);
        let (creation, modification, duration_ms) = parse_mvhd(&mvhd).unwrap();
        assert_eq!(creation, Some(300_000));
        assert_eq!(modification, Some(400_000));
        assert_eq!(duration_ms, Some(2_000));
    }

    #[test]
    fn parse_mvhd_zero_timescale_yields_no_duration() {
        let mvhd = build_mvhd(0, 0, 0, 0, 0);
        let (creation, modification, duration_ms) = parse_mvhd(&mvhd).unwrap();
        assert_eq!(creation, None);
        assert_eq!(modification, None);
        assert_eq!(duration_ms, None);
    }

    #[test]
    fn parse_mvhd_rejects_truncated_box() {
        assert_eq!(parse_mvhd(&[0, 0, 0]), None);
    }

    /// Build a `tkhd` FullBox body (version 0 or 1) with the given pixel dimensions
    /// (fixed-point 16.16, so plain integers are shifted left by 16 bits).
    fn build_tkhd(version: u8, width: u32, height: u32) -> Vec<u8> {
        let version_block_len = if version == 1 { 32 } else { 20 };
        let mut buf = vec![version, 0, 0, 0];
        buf.extend(std::iter::repeat(0u8).take(version_block_len));
        buf.extend(std::iter::repeat(0u8).take(52)); // reserved/layer/volume/matrix
        buf.extend_from_slice(&(width << 16).to_be_bytes());
        buf.extend_from_slice(&(height << 16).to_be_bytes());
        buf
    }

    #[test]
    fn parse_tkhd_dimensions_version0() {
        let tkhd = build_tkhd(0, 480, 270);
        assert_eq!(parse_tkhd_dimensions(&tkhd), Some((480, 270)));
    }

    #[test]
    fn parse_tkhd_dimensions_version1() {
        let tkhd = build_tkhd(1, 640, 360);
        assert_eq!(parse_tkhd_dimensions(&tkhd), Some((640, 360)));
    }

    #[test]
    fn parse_tkhd_dimensions_rejects_truncated_box() {
        assert_eq!(parse_tkhd_dimensions(&[0, 0, 0]), None);
    }

    fn assert_close(actual: Option<(f64, f64, Option<f64>)>, lat: f64, lon: f64, alt: Option<f64>) {
        let (a_lat, a_lon, a_alt) = actual.unwrap();
        assert!((a_lat - lat).abs() < 1e-6, "lat: got {a_lat}, want {lat}");
        assert!((a_lon - lon).abs() < 1e-6, "lon: got {a_lon}, want {lon}");
        match (a_alt, alt) {
            (Some(a), Some(e)) => assert!((a - e).abs() < 1e-6, "alt: got {a}, want {e}"),
            (None, None) => {}
            (a, e) => panic!("alt: got {a:?}, want {e:?}"),
        }
    }

    #[test]
    fn parse_iso6709_lat_lon_only() {
        assert_close(
            parse_iso6709("+37.4220-122.0840/"),
            37.4220,
            -122.0840,
            None,
        );
    }

    #[test]
    fn parse_iso6709_with_altitude() {
        assert_close(
            parse_iso6709("+27.5916+086.5640+8850/"),
            27.5916,
            86.5640,
            Some(8850.0),
        );
    }

    #[test]
    fn parse_iso6709_negative_latitude() {
        assert_close(
            parse_iso6709("-33.8688+151.2093/"),
            -33.8688,
            151.2093,
            None,
        );
    }

    #[test]
    fn parse_iso6709_tolerates_missing_trailing_slash() {
        assert_close(parse_iso6709("+37.4220-122.0840"), 37.4220, -122.0840, None);
    }

    #[test]
    fn parse_iso6709_rejects_empty_string() {
        assert_eq!(parse_iso6709(""), None);
        assert_eq!(parse_iso6709("/"), None);
    }

    #[test]
    fn parse_3gpp_text_reads_declared_length() {
        // length(2)=5, language(2)=0, then "hello" plus trailing garbage past the
        // declared length that must be ignored.
        let mut data = vec![0, 5, 0, 0];
        data.extend_from_slice(b"helloXXXX");
        assert_eq!(parse_3gpp_text(&data), Some("hello".to_string()));
    }

    #[test]
    fn parse_3gpp_text_falls_back_when_length_invalid() {
        // declared length larger than the remaining buffer — falls back to
        // reading everything after the 4-byte header.
        let mut data = vec![255, 255, 0, 0];
        data.extend_from_slice(b"fallback");
        assert_eq!(parse_3gpp_text(&data), Some("fallback".to_string()));
    }

    #[test]
    fn utf8_nonempty_trims_and_rejects_blank() {
        assert_eq!(utf8_nonempty(b"  hello  "), Some("hello".to_string()));
        assert_eq!(utf8_nonempty(b""), None);
        assert_eq!(utf8_nonempty(b"   "), None);
    }
}
