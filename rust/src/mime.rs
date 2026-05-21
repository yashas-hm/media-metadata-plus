use std::path::Path;

pub fn detect(path: &Path) -> anyhow::Result<String> {
    let mut buf = [0u8; 16];
    let mut f = std::fs::File::open(path)?;
    std::io::Read::read(&mut f, &mut buf)?;

    let mime = match &buf {
        b if b.starts_with(b"\xff\xd8\xff") => "image/jpeg",
        b if b[4..8] == *b"ftyp" => detect_ftyp(&buf),
        b if b.starts_with(b"\x89PNG\r\n\x1a\n") => "image/png",
        b if b.starts_with(b"RIFF") && b[8..12] == *b"WEBP" => "image/webp",
        // TIFF family: little-endian (II) or big-endian (MM)
        b if b.starts_with(b"II\x2a\x00") => detect_tiff_le(&buf),
        b if b.starts_with(b"MM\x00\x2a") => "image/tiff",
        _ => "application/octet-stream",
    };

    Ok(mime.to_string())
}

/// Distinguish TIFF little-endian variants by additional identifier bytes.
fn detect_tiff_le(buf: &[u8]) -> &'static str {
    // CR2 (Canon): "CR" magic at bytes 8-9
    if buf.get(8..10) == Some(b"CR") {
        return "image/x-canon-cr2";
    }
    // NEF (Nikon), ARW (Sony), DNG (Adobe), and generic TIFF all share the
    // same magic bytes and require deeper IFD inspection to distinguish.
    // kamadak-exif reads standard EXIF tags from all of them correctly.
    "image/tiff"
}

fn detect_ftyp(buf: &[u8]) -> &'static str {
    match &buf[8..12] {
        b"heic" | b"heix" | b"heim" | b"heis" => "image/heic",
        b"mif1" | b"msf1" => "image/heif",
        b"mp41" | b"mp42" | b"isom" | b"M4V " => "video/mp4",
        b"qt  " => "video/quicktime",
        _ => "video/mp4",
    }
}
