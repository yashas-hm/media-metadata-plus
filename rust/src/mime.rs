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
        _ => "application/octet-stream",
    };

    Ok(mime.to_string())
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
