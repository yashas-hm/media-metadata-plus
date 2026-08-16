//! Generic ISOBMFF (ISO/IEC 14496-12) box-tree primitives.
//!
//! MP4, MOV/QuickTime, and HEIF/HEIC all share this same container shape: a
//! sequence of boxes ("atoms" in QuickTime terminology), each a 4-byte size +
//! 4-byte type tag optionally followed by nested child boxes. These helpers
//! know nothing about what any particular box *means* — that's left to the
//! format-specific readers in `crate::readers`.

use std::io::{Read, Seek, SeekFrom};

/// Scan a file sequentially from the start, seeking past non-target top-level boxes.
/// Returns the content bytes of the named box (excluding its 8-byte header).
pub fn read_top_level_box(f: &mut std::fs::File, name: &[u8; 4]) -> Option<Vec<u8>> {
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
pub fn find_child_box<'a>(data: &'a [u8], name: &[u8; 4]) -> Option<&'a [u8]> {
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

/// Like [`find_child_box`], but collects every matching child instead of only the first.
pub fn find_all_child_boxes<'a>(data: &'a [u8], name: &[u8; 4]) -> Vec<&'a [u8]> {
    let mut out = Vec::new();
    let mut i = 0usize;
    while i + 8 <= data.len() {
        let size = u32::from_be_bytes([data[i], data[i + 1], data[i + 2], data[i + 3]]) as usize;
        if size < 8 || i + size > data.len() {
            break;
        }
        if data[i + 4..i + 8] == *name {
            out.push(&data[i + 8..i + size]);
        }
        i += size;
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_box(name: &[u8; 4], content: &[u8]) -> Vec<u8> {
        let size = (8 + content.len()) as u32;
        let mut out = size.to_be_bytes().to_vec();
        out.extend_from_slice(name);
        out.extend_from_slice(content);
        out
    }

    #[test]
    fn find_child_box_locates_named_box() {
        let mut data = make_box(b"free", &[0, 0]);
        data.extend(make_box(b"mvhd", &[1, 2, 3, 4]));
        data.extend(make_box(b"trak", &[9]));

        assert_eq!(find_child_box(&data, b"mvhd"), Some(&[1, 2, 3, 4][..]));
        assert_eq!(find_child_box(&data, b"trak"), Some(&[9][..]));
        assert_eq!(find_child_box(&data, b"nope"), None);
    }

    #[test]
    fn find_all_child_boxes_collects_every_match() {
        let mut data = make_box(b"trak", &[1]);
        data.extend(make_box(b"free", &[0]));
        data.extend(make_box(b"trak", &[2]));

        let traks = find_all_child_boxes(&data, b"trak");
        assert_eq!(traks, vec![&[1u8][..], &[2u8][..]]);
    }

    #[test]
    fn find_child_box_rejects_truncated_or_oversized_size() {
        // Declared size larger than the remaining buffer must not panic or match.
        let mut data = 100u32.to_be_bytes().to_vec();
        data.extend_from_slice(b"mvhd");
        data.extend_from_slice(&[0, 0]);
        assert_eq!(find_child_box(&data, b"mvhd"), None);
    }
}
