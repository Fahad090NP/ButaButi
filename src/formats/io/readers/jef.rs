//! Janome JEF format reader
//!
//! JEF is the Janome Embroidery Format with a binary header containing design bounds,
//! hoop information, and thread colors from the predefined JEF palette.
//!
//! ## Format Limitations
//! - Stitch offset must be within 0-100MB range (100,000,000 bytes)
//! - Maximum 1,000 colors allowed
//! - Maximum 1,000,000 stitches per file

/// Maximum allowed stitch data offset in bytes (100MB)
const MAX_STITCH_OFFSET: i32 = 100_000_000;

/// Maximum allowed color count
const MAX_COLORS: usize = 1000;

/// Maximum allowed stitch count
const MAX_STITCHES: usize = 1_000_000;

use crate::core::pattern::EmbPattern;
use crate::formats::io::utils::ReadHelper;
use crate::palettes::thread_jef::JEF_THREADS;
use crate::utils::error::{Error, Result};
use std::collections::HashMap;
use std::io::{Read, Seek, SeekFrom};

/// Read JEF stitches
fn read_stitches<R: Read>(
    reader: &mut R,
    pattern: &mut EmbPattern,
    _color_count: usize,
    settings: &HashMap<String, String>,
) -> Result<()> {
    let mut color_index = 1;
    let mut buffer = [0u8; 2];
    let mut stitch_count = 0;

    loop {
        match reader.read_exact(&mut buffer) {
            Ok(_) => {}
            Err(e) if e.kind() == std::io::ErrorKind::UnexpectedEof => break,
            Err(e) => return Err(Error::from(e)),
        }

        // Check for excessive stitch count
        stitch_count += 1;
        if stitch_count > MAX_STITCHES {
            return Err(Error::Parse(format!(
                "JEF file exceeds maximum stitch count of {}",
                MAX_STITCHES
            )));
        }

        if buffer[0] != 0x80 {
            // Normal stitch
            let x = buffer[0] as i8 as f64;
            let y = -(buffer[1] as i8 as f64);
            pattern.stitch(x, y);
            continue;
        }

        let ctrl = buffer[1];

        // Read next 2 bytes for coordinates
        match reader.read_exact(&mut buffer) {
            Ok(_) => {}
            Err(e) if e.kind() == std::io::ErrorKind::UnexpectedEof => break,
            Err(e) => return Err(Error::from(e)),
        }

        let x = buffer[0] as i8 as f64;
        let y = -(buffer[1] as i8 as f64);

        match ctrl {
            0x02 => {
                // Jump/Move
                pattern.jump(x, y);
            }
            0x01 => {
                // Color change
                // Check if this is a stop (color index 0 means None)
                if color_index < pattern.threads().len() {
                    pattern.color_change(0.0, 0.0);
                    color_index += 1;
                } else {
                    pattern.stop();
                }
            }
            0x10 => {
                // End pattern
                break;
            }
            _ => {
                // Uncaught control - break
                break;
            }
        }
    }

    pattern.end();

    // Interpolate trims based on settings
    let trims = settings
        .get("trims")
        .and_then(|s| s.parse::<bool>().ok())
        .unwrap_or(false);

    let mut trim_at = settings
        .get("trim_at")
        .and_then(|s| s.parse::<usize>().ok());

    let trim_distance = settings
        .get("trim_distance")
        .and_then(|s| s.parse::<f64>().ok())
        .unwrap_or(3.0)
        * 10.0; // Convert mm to 1/10mm

    let clipping = settings
        .get("clipping")
        .and_then(|s| s.parse::<bool>().ok())
        .unwrap_or(true);

    if trims && trim_at.is_none() {
        trim_at = Some(3);
    }

    if let Some(count) = trim_at {
        pattern.interpolate_trims(count, Some(trim_distance), clipping);
    }

    Ok(())
}

/// Read a JEF file
pub fn read<R: Read + Seek>(
    reader: &mut R,
    settings: Option<HashMap<String, String>>,
) -> Result<EmbPattern> {
    let mut pattern = EmbPattern::new();
    let settings = settings.unwrap_or_default();

    let mut helper = ReadHelper::new(reader);

    // Read stitch offset
    let stitch_offset = helper.read_i32_le()?;

    // Validate stitch offset is reasonable
    if !(0..=MAX_STITCH_OFFSET).contains(&stitch_offset) {
        return Err(Error::Parse(format!(
            "Invalid JEF stitch offset: {} (must be between 0 and {})",
            stitch_offset, MAX_STITCH_OFFSET
        )));
    }

    // Skip 20 bytes
    helper.read_bytes(20)?;

    // Read color count
    let count_colors = helper.read_i32_le()? as usize;

    // Validate color count is reasonable
    if count_colors > MAX_COLORS {
        return Err(Error::Parse(format!(
            "Invalid JEF color count: {} (must be <= {})",
            count_colors, MAX_COLORS
        )));
    }

    // Skip 88 bytes
    helper.read_bytes(88)?;

    // Read thread indices
    for _ in 0..count_colors {
        let index = helper.read_i32_le()?.unsigned_abs() as usize;

        if index == 0 {
            // Color 0 is a placeholder/stop - skip adding thread
            // but we need to track it for color changes
        } else {
            let thread_idx = index % JEF_THREADS.len();
            if let Some(thread_ref) = &JEF_THREADS[thread_idx] {
                pattern.add_thread(thread_ref.clone());
            }
        }
    }

    // Seek to stitch data
    let mut reader = helper.into_inner();
    reader.seek(SeekFrom::Start(stitch_offset as u64))?;

    read_stitches(&mut reader, &mut pattern, count_colors, &settings)?;

    Ok(pattern)
}

/// Read a JEF file from path
pub fn read_file(path: &str) -> Result<EmbPattern> {
    let file = std::fs::File::open(path)?;
    let mut reader = std::io::BufReader::new(file);
    read(&mut reader, None)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_jef_threads() {
        // Verify that the JEF thread palette is available
        assert!(JEF_THREADS.len() > 70);
        assert!(JEF_THREADS[0].is_none()); // Placeholder
        assert!(JEF_THREADS[1].is_some()); // Black
    }
}
