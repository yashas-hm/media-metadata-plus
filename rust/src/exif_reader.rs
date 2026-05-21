use std::path::Path;

use crate::api::MediaMeta;

pub fn read(path: &Path, mime: &str) -> anyhow::Result<MediaMeta> {
    let file = std::fs::File::open(path)?;
    let mut bufreader = std::io::BufReader::new(file);
    let exif = exif::Reader::new().read_from_container(&mut bufreader)?;

    Ok(MediaMeta {
        mime_type: mime.to_string(),
        width: read_u32(&exif, exif::Tag::ImageWidth),
        height: read_u32(&exif, exif::Tag::ImageLength),
        captured_at_ms: read_date(&exif),
        camera_make: read_str(&exif, exif::Tag::Make),
        camera_model: read_str(&exif, exif::Tag::Model),
        latitude: read_gps_lat(&exif),
        longitude: read_gps_lon(&exif),
        altitude: read_gps_alt(&exif),
        duration_ms: None,
    })
}

fn read_u32(exif: &exif::Exif, tag: exif::Tag) -> Option<u32> {
    match &exif.get_field(tag, exif::In::PRIMARY)?.value {
        exif::Value::Long(v) => v.first().copied(),
        exif::Value::Short(v) => v.first().map(|&x| x as u32),
        _ => None,
    }
}

fn read_str(exif: &exif::Exif, tag: exif::Tag) -> Option<String> {
    if let exif::Value::Ascii(ref v) =
        exif.get_field(tag, exif::In::PRIMARY)?.value
    {
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
