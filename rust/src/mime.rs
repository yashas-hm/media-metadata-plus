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

#[cfg(test)]
mod tests {
    use super::*;

    fn ftyp_buf(brand: &[u8; 4]) -> [u8; 16] {
        let mut buf = [0u8; 16];
        buf[4..8].copy_from_slice(b"ftyp");
        buf[8..12].copy_from_slice(brand);
        buf
    }

    #[test]
    fn detect_ftyp_recognizes_heic_variants() {
        for brand in [b"heic", b"heix", b"heim", b"heis"] {
            assert_eq!(detect_ftyp(&ftyp_buf(brand)), "image/heic");
        }
    }

    #[test]
    fn detect_ftyp_recognizes_heif_variants() {
        for brand in [b"mif1", b"msf1"] {
            assert_eq!(detect_ftyp(&ftyp_buf(brand)), "image/heif");
        }
    }

    #[test]
    fn detect_ftyp_recognizes_mp4_variants() {
        for brand in [b"mp41", b"mp42", b"isom", b"M4V "] {
            assert_eq!(detect_ftyp(&ftyp_buf(brand)), "video/mp4");
        }
    }

    #[test]
    fn detect_ftyp_recognizes_quicktime() {
        assert_eq!(detect_ftyp(&ftyp_buf(b"qt  ")), "video/quicktime");
    }

    #[test]
    fn detect_ftyp_defaults_unknown_brand_to_mp4() {
        assert_eq!(detect_ftyp(&ftyp_buf(b"xyz1")), "video/mp4");
    }

    #[test]
    fn detect_tiff_le_recognizes_cr2() {
        let mut buf = [0u8; 16];
        buf[0..4].copy_from_slice(b"II\x2a\x00");
        buf[8..10].copy_from_slice(b"CR");
        assert_eq!(detect_tiff_le(&buf), "image/x-canon-cr2");
    }

    #[test]
    fn detect_tiff_le_defaults_to_generic_tiff() {
        // NEF/ARW/DNG/generic TIFF are indistinguishable at this level.
        let buf = [0u8; 16];
        assert_eq!(detect_tiff_le(&buf), "image/tiff");
    }

    fn detect_bytes(bytes: &[u8]) -> String {
        use std::sync::atomic::{AtomicU32, Ordering};
        static COUNTER: AtomicU32 = AtomicU32::new(0);
        let id = COUNTER.fetch_add(1, Ordering::Relaxed);
        let path = std::env::temp_dir().join(format!("mmp_mime_test_{id}"));
        std::fs::write(&path, bytes).unwrap();
        let result = detect(&path).unwrap();
        std::fs::remove_file(&path).ok();
        result
    }

    #[test]
    fn detect_recognizes_jpeg_png_webp_tiff_and_unknown() {
        assert_eq!(detect_bytes(&[0xFF, 0xD8, 0xFF, 0xE0]), "image/jpeg");
        assert_eq!(
            detect_bytes(b"\x89PNG\r\n\x1a\n\x00\x00\x00\x00"),
            "image/png"
        );
        let mut webp = b"RIFF".to_vec();
        webp.extend_from_slice(&[0, 0, 0, 0]);
        webp.extend_from_slice(b"WEBP");
        assert_eq!(detect_bytes(&webp), "image/webp");
        assert_eq!(detect_bytes(b"MM\x00\x2a\x00\x00\x00\x08"), "image/tiff");
        assert_eq!(
            detect_bytes(b"not a media file"),
            "application/octet-stream"
        );
    }
}
