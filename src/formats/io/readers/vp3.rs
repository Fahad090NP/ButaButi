//! Pfaff VP3 format reader
//!
//! VP3 is Pfaff's proprietary format with compressed stitch data and extensive metadata
//! including hoop information, thread colors, and design properties.
//!
//! ## Format Limitations
//! - String sections (metadata) limited to 10KB each
//! - Stitch data sections limited to 30MB
//! - Maximum 1,000,000 stitches per file
//! - Unknown sections limited to 100KB for safety

/// Maximum allowed string section size (10KB)
const MAX_STRING_SIZE: usize = 10_000;

/// Maximum allowed section skip size (100KB)
const MAX_SKIP_SIZE: usize = 100_000;

/// Maximum allowed stitch section size (30MB)
const MAX_STITCH_SECTION: usize = 30_000_000;

/// Maximum allowed stitch count
const MAX_STITCHES: usize = 1_000_000;

use crate::core::constants::*;
use crate::core::pattern::EmbPattern;
use crate::formats::io::utils::ReadHelper;
use crate::utils::error::{Error, Result};
use std::io::Read;

/// VP3 file signature
const VP3_SIGNATURE: &[u8] = b"%vsm%";

/// Read VP3 (Pfaff VP3) format
///
/// Reads a VP3 embroidery file into the provided pattern.
///
/// # Arguments
///
/// * `file` - The input stream to read from
/// * `pattern` - The pattern to populate with stitches and metadata
///
/// # Example
///
/// ```no_run
/// use butabuti::core::pattern::EmbPattern;
/// use butabuti::formats::io::readers::vp3;
/// use std::fs::File;
///
/// let mut file = File::open("design.vp3")?;
/// let mut pattern = EmbPattern::new();
/// vp3::read(&mut file, &mut pattern)?;
/// # Ok::<(), butabuti::utils::error::Error>(())
/// ```
pub fn read(file: &mut impl Read, pattern: &mut EmbPattern) -> Result<()> {
    let mut helper = ReadHelper::new(file);

    // Read and verify signature
    let signature = helper.read_bytes(5)?;
    if signature != VP3_SIGNATURE {
        let sig_str = String::from_utf8_lossy(&signature);
        return Err(Error::Parse(format!(
            "Invalid VP3 signature: expected '{}', got '{}' ({:02X?})",
            String::from_utf8_lossy(VP3_SIGNATURE),
            sig_str,
            signature
        )));
    }

    // Read file content until we find specific sections
    // VP3 format is quite complex with multiple sections
    read_vp3_sections(&mut helper, pattern)?;

    Ok(())
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
                },
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

    // Validate string length is reasonable (max 10KB)
    if length > MAX_STRING_SIZE {
        return Err(Error::Parse(format!(
            "VP3 string section too large: {} bytes (max {})",
            length, MAX_STRING_SIZE
        )));
    }

    if length > 0 {
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
        if length > 0 && length < MAX_SKIP_SIZE {
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

    // Validate section size is reasonable
    if section_size == 0 {
        return Ok(());
    }

    if section_size > MAX_STITCH_SECTION {
        return Err(Error::Parse(format!(
            "VP3 stitch section too large: {} bytes (max {})",
            section_size, MAX_STITCH_SECTION
        )));
    }

    // VP3 stitch data is encoded in a proprietary format
    // Each stitch is typically 3 bytes: x (i8), y (i8), command (u8)
    let mut x = 0.0;
    let mut y = 0.0;

    let stitch_count = section_size / 3;

    if stitch_count > MAX_STITCHES {
        return Err(Error::Parse(format!(
            "VP3 file exceeds maximum stitch count: {} (max {})",
            stitch_count, MAX_STITCHES
        )));
    }

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
///
/// Convenience function to read a VP3 file directly from a file path.
///
/// # Example
///
/// ```no_run
/// use butabuti::formats::io::readers::vp3;
///
/// let pattern = vp3::read_file("design.vp3")?;
/// # Ok::<(), butabuti::utils::error::Error>(())
/// ```
pub fn read_file(path: &str) -> Result<EmbPattern> {
    let file = std::fs::File::open(path)?;
    let mut reader = std::io::BufReader::new(file);
    let mut pattern = EmbPattern::new();
    read(&mut reader, &mut pattern)?;
    Ok(pattern)
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
        let mut pattern = EmbPattern::new();
        let result = read(&mut cursor, &mut pattern);
        assert!(result.is_err());
    }
}
