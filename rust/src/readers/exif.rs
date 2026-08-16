use std::path::Path;

use crate::api::MediaMeta;

pub fn read(path: &Path, mime: &str) -> anyhow::Result<MediaMeta> {
    let file = std::fs::File::open(path)?;
    let mut bufreader = std::io::BufReader::new(file);

    match exif::Reader::new().read_from_container(&mut bufreader) {
        Ok(exif) => {
            let mut width = read_u32(&exif, exif::Tag::ImageWidth);
            let mut height = read_u32(&exif, exif::Tag::ImageLength);

            // Orientation 5-8 means the image is rotated 90° or 270°, so
            // the stored pixel dimensions are transposed relative to display.
            let orientation = read_u32(&exif, exif::Tag::Orientation).unwrap_or(1);
            if matches!(orientation, 5 | 6 | 7 | 8) {
                std::mem::swap(&mut width, &mut height);
            }

            // WebP: EXIF dimension tags are optional — fall back to bitstream
            if mime == "image/webp" && (width.is_none() || height.is_none()) {
                if let Some((w, h)) = webp_dimensions(path) {
                    width = width.or(Some(w));
                    height = height.or(Some(h));
                }
            }

            Ok(MediaMeta {
                mime_type: mime.to_string(),
                width,
                height,
                captured_at_ms: read_date(&exif),
                modified_at_ms: read_str_date(&exif, exif::Tag::DateTime),
                camera_make: read_str(&exif, exif::Tag::Make),
                camera_model: read_str(&exif, exif::Tag::Model),
                latitude: read_gps_lat(&exif),
                longitude: read_gps_lon(&exif),
                altitude: read_gps_alt(&exif),
                duration_ms: None,
            })
        }

        // WebP files without EXIF — return dimensions from bitstream
        Err(_) if mime == "image/webp" => {
            let (width, height) = webp_dimensions(path)
                .map(|(w, h)| (Some(w), Some(h)))
                .unwrap_or((None, None));
            Ok(MediaMeta {
                mime_type: mime.to_string(),
                width,
                height,
                captured_at_ms: None,
                modified_at_ms: None,
                camera_make: None,
                camera_model: None,
                latitude: None,
                longitude: None,
                altitude: None,
                duration_ms: None,
            })
        }

        Err(e) => Err(e.into()),
    }
}

/// Read image dimensions directly from the WebP bitstream (VP8X / VP8L / VP8).
/// Used when EXIF dimension tags are absent.
fn webp_dimensions(path: &Path) -> Option<(u32, u32)> {
    use std::io::Read;
    let mut f = std::fs::File::open(path).ok()?;
    // RIFF(4) + file_size(4) + WEBP(4) + chunk_type(4) + chunk_size(4) + chunk_data(up to 10)
    let mut buf = [0u8; 30];
    f.read_exact(&mut buf).ok()?;
    if &buf[..4] != b"RIFF" || &buf[8..12] != b"WEBP" {
        return None;
    }
    match &buf[12..16] {
        b"VP8X" => {
            // flags(4) + canvas_width_minus_1(3 LE) + canvas_height_minus_1(3 LE)
            let w = u32::from_le_bytes([buf[24], buf[25], buf[26], 0]) + 1;
            let h = u32::from_le_bytes([buf[27], buf[28], buf[29], 0]) + 1;
            Some((w, h))
        }
        b"VP8L" => {
            // signature byte (0x2F), then bits[0:13]=width-1, bits[14:27]=height-1
            if buf[20] != 0x2F {
                return None;
            }
            let bits = u32::from_le_bytes([buf[21], buf[22], buf[23], buf[24]]);
            let w = (bits & 0x3FFF) + 1;
            let h = ((bits >> 14) & 0x3FFF) + 1;
            Some((w, h))
        }
        b"VP8 " => {
            // 3-byte frame tag, then start code 0x9d 0x01 0x2a, then width/height (14-bit LE)
            if buf[23] == 0x9d && buf[24] == 0x01 && buf[25] == 0x2a {
                let w = (u16::from_le_bytes([buf[26], buf[27]]) & 0x3FFF) as u32;
                let h = (u16::from_le_bytes([buf[28], buf[29]]) & 0x3FFF) as u32;
                Some((w, h))
            } else {
                None
            }
        }
        _ => None,
    }
}

fn read_u32(exif: &exif::Exif, tag: exif::Tag) -> Option<u32> {
    match &exif.get_field(tag, exif::In::PRIMARY)?.value {
        exif::Value::Long(v) => v.first().copied(),
        exif::Value::Short(v) => v.first().map(|&x| x as u32),
        _ => None,
    }
}

fn read_str(exif: &exif::Exif, tag: exif::Tag) -> Option<String> {
    if let exif::Value::Ascii(ref v) = exif.get_field(tag, exif::In::PRIMARY)?.value {
        let s = std::str::from_utf8(v.first()?).ok()?.trim().to_string();
        if s.is_empty() { None } else { Some(s) }
    } else {
        None
    }
}

fn read_date(exif: &exif::Exif) -> Option<i64> {
    let tags = [
        exif::Tag::DateTimeOriginal,
        exif::Tag::DateTimeDigitized,
        exif::Tag::DateTime,
    ];
    for tag in tags {
        if let Some(field) = exif.get_field(tag, exif::In::PRIMARY) {
            if let exif::Value::Ascii(ref v) = field.value {
                if let Some(s) = v.first() {
                    if let Ok(ms) = parse_exif_datetime(s) {
                        return Some(ms);
                    }
                }
            }
        }
    }
    None
}

fn read_str_date(exif: &exif::Exif, tag: exif::Tag) -> Option<i64> {
    if let Some(field) = exif.get_field(tag, exif::In::PRIMARY) {
        if let exif::Value::Ascii(ref v) = field.value {
            if let Some(s) = v.first() {
                return parse_exif_datetime(s).ok();
            }
        }
    }
    None
}

fn parse_exif_datetime(s: &[u8]) -> anyhow::Result<i64> {
    let s = std::str::from_utf8(s)?;
    // EXIF format: "2021:01:15 12:30:00" — replace first two colons with dashes
    let normalized = s.replacen(':', "-", 2);
    let dt = chrono::NaiveDateTime::parse_from_str(&normalized, "%Y-%m-%d %H:%M:%S")?;
    Ok(dt.and_utc().timestamp_millis())
}

fn read_gps_lat(exif: &exif::Exif) -> Option<f64> {
    let lat = exif.get_field(exif::Tag::GPSLatitude, exif::In::PRIMARY)?;
    let ref_ = exif.get_field(exif::Tag::GPSLatitudeRef, exif::In::PRIMARY)?;
    dms_to_decimal(&lat.value, &ref_.value)
}

fn read_gps_lon(exif: &exif::Exif) -> Option<f64> {
    let lon = exif.get_field(exif::Tag::GPSLongitude, exif::In::PRIMARY)?;
    let ref_ = exif.get_field(exif::Tag::GPSLongitudeRef, exif::In::PRIMARY)?;
    dms_to_decimal(&lon.value, &ref_.value)
}

fn read_gps_alt(exif: &exif::Exif) -> Option<f64> {
    let field = exif.get_field(exif::Tag::GPSAltitude, exif::In::PRIMARY)?;
    if let exif::Value::Rational(ref v) = field.value {
        let alt = v.first()?.to_f64();
        let below_sea = exif
            .get_field(exif::Tag::GPSAltitudeRef, exif::In::PRIMARY)
            .and_then(|f| {
                if let exif::Value::Byte(ref b) = f.value {
                    b.first().copied()
                } else {
                    None
                }
            })
            .unwrap_or(0);
        Some(if below_sea == 1 { -alt } else { alt })
    } else {
        None
    }
}

fn dms_to_decimal(dms: &exif::Value, ref_: &exif::Value) -> Option<f64> {
    if let (exif::Value::Rational(dms), exif::Value::Ascii(ref_)) = (dms, ref_) {
        let d = dms.first()?.to_f64();
        let m = dms.get(1)?.to_f64();
        let s = dms.get(2)?.to_f64();
        let deg = d + m / 60.0 + s / 3600.0;
        let sign = if ref_.first()?.starts_with(b"S") || ref_.first()?.starts_with(b"W") {
            -1.0
        } else {
            1.0
        };
        Some(sign * deg)
    } else {
        None
    }
}
