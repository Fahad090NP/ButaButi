//! Pfaff VP3 format reader
//!
//! VP3 is Pfaff's proprietary format with compressed stitch data and extensive metadata
//! including hoop information, thread colors, and design properties.

use crate::core::constants::*;
use crate::core::pattern::EmbPattern;
use crate::formats::io::utils::ReadHelper;
use crate::utils::error::{Error, Result};
use std::io::Read;

/// VP3 file signature
const VP3_SIGNATURE: &[u8] = b"%vsm%";

/// Read a VP3 file from a reader
pub fn read<R: Read>(reader: &mut R) -> Result<EmbPattern> {
    let mut helper = ReadHelper::new(reader);
    let mut pattern = EmbPattern::new();

    // Read and verify signature
    let signature = helper.read_bytes(5)?;
    if signature != VP3_SIGNATURE {
        return Err(Error::Parse("Invalid VP3 signature".to_string()));
    }

    // Read file content until we find specific sections
    // VP3 format is quite complex with multiple sections
    read_vp3_sections(&mut helper, &mut pattern)?;

    Ok(pattern)
}

/// Read VP3 file sections
fn read_vp3_sections<R: Read>(helper: &mut ReadHelper<R>, pattern: &mut EmbPattern) -> Result<()> {
    // VP3 files contain various sections marked by specific strings
    // We need to find and parse:
    // - %nam% - design name
    // - %com% - comments
    // - %aut% - author
    // - %cop% - copyright
    // - %hst% - history
    // - %xxs% - stitch data section
    // - %fcn% - color data

    while let Ok(marker) = helper.read_bytes(5) {
        if marker.starts_with(b"%") && marker.ends_with(b"%") {
            let marker_str = String::from_utf8_lossy(&marker[1..4]);

            match marker_str.as_ref() {
                "nam" => read_string_section(helper, pattern, "name")?,
                "com" => read_string_section(helper, pattern, "comments")?,
                "aut" => read_string_section(helper, pattern, "author")?,
                "cop" => read_string_section(helper, pattern, "copyright")?,
                "xxs" => read_stitch_section(helper, pattern)?,
                _ => {
                    // Unknown section, skip it
                    skip_section(helper)?;
                }
            }
        }
    }

    Ok(())
}

/// Read a string metadata section
fn read_string_section<R: Read>(
    helper: &mut ReadHelper<R>,
    pattern: &mut EmbPattern,
    key: &str,
) -> Result<()> {
    // String sections typically have a length prefix
    let length = helper.read_u16_le()? as usize;
    if length > 0 && length < 10000 {
        let bytes = helper.read_bytes(length)?;
        if let Ok(text) = String::from_utf8(bytes) {
            let text = text.trim_end_matches('\0').trim();
            if !text.is_empty() {
                pattern.add_metadata(key, text);
            }
        }
    }
    Ok(())
}

/// Skip an unknown section
fn skip_section<R: Read>(helper: &mut ReadHelper<R>) -> Result<()> {
    // Try to read a length field and skip that many bytes
    if let Ok(length) = helper.read_u16_le() {
        let length = length as usize;
        if length > 0 && length < 100000 {
            let _ = helper.read_bytes(length);
        }
    }
    Ok(())
}

/// Read the stitch data section
fn read_stitch_section<R: Read>(
    helper: &mut ReadHelper<R>,
    pattern: &mut EmbPattern,
) -> Result<()> {
    // Read number of stitches or section size
    let section_size = helper.read_u32_le()? as usize;

    if section_size == 0 || section_size > 10_000_000 {
        return Ok(());
    }

    // VP3 stitch data is encoded in a proprietary format
    // Each stitch is typically 3 bytes: x (i8), y (i8), command (u8)
    let mut x = 0.0;
    let mut y = 0.0;

    let stitch_count = section_size / 3;

    for _ in 0..stitch_count {
        let dx = helper.read_i8()? as f64;
        let dy = helper.read_i8()? as f64;
        let flags = helper.read_u8()?;

        x += dx;
        y += dy;

        let command = decode_vp3_command(flags);

        if command != 0 {
            pattern.add_stitch_absolute(command, x, y);
        }
    }

    Ok(())
}

/// Decode VP3 command byte to embroidery command
fn decode_vp3_command(flags: u8) -> u32 {
    match flags {
        0x00 => STITCH,
        0x01 => JUMP,
        0x02 => COLOR_CHANGE,
        0x03 => TRIM,
        0x80 => END,
        _ => STITCH, // Default to stitch for unknown commands
    }
}

/// Read VP3 file from path
pub fn read_file(path: &str) -> Result<EmbPattern> {
    let file = std::fs::File::open(path)?;
    let mut reader = std::io::BufReader::new(file);
    read(&mut reader)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_vp3_signature() {
        assert_eq!(VP3_SIGNATURE, b"%vsm%");
    }

    #[test]
    fn test_decode_command() {
        assert_eq!(decode_vp3_command(0x00), STITCH);
        assert_eq!(decode_vp3_command(0x01), JUMP);
        assert_eq!(decode_vp3_command(0x02), COLOR_CHANGE);
        assert_eq!(decode_vp3_command(0x80), END);
    }

    #[test]
    fn test_invalid_signature() {
        let data = b"Invalid data";
        let mut cursor = std::io::Cursor::new(data);
        let result = read(&mut cursor);
        assert!(result.is_err());
    }
}
