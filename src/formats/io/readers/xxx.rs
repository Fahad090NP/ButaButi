//! Singer XXX format reader
//!
//! XXX uses variable-length encoding with 2-byte normal stitches, 5-byte long stitches,
//! and 4-byte special commands. Colors are stored at the end after all stitches.
//!
//! ## Format Limitations
//! - Minimum file size: 256 bytes (header size)
//! - Maximum 1,000 colors allowed
//! - Maximum 1,000,000 stitches per file
//! - Header at offset 0x00-0x100, stitches start at 0x100

/// Minimum valid file size in bytes
const MIN_FILE_SIZE: usize = 256;

/// Maximum allowed color count
const MAX_COLORS: u16 = 1000;

/// Maximum allowed stitch count
const MAX_STITCHES: usize = 1_000_000;

use crate::core::constants::*;
use crate::core::pattern::EmbPattern;
use crate::utils::error::Result;
use byteorder::{BigEndian, LittleEndian, ReadBytesExt};
use std::io::Read;

/// Read a signed 8-bit value as f64
fn read_signed_i8(value: u8) -> f64 {
    value as i8 as f64
}

/// Read a signed 16-bit value as f64
fn read_signed_i16(value: u16) -> f64 {
    value as i16 as f64
}

/// Read XXX format file into a pattern
///
/// # Arguments
///
/// * `file` - The input file/stream to read from
/// * `pattern` - The pattern to populate with data
///
/// # Example
///
/// ```no_run
/// use butabuti::prelude::*;
/// use std::fs::File;
///
/// let mut file = File::open("design.xxx").unwrap();
/// let mut pattern = EmbPattern::new();
/// butabuti::formats::io::readers::xxx::read(&mut file, &mut pattern).unwrap();
/// ```
pub fn read(file: &mut impl Read, pattern: &mut EmbPattern) -> Result<()> {
    // Skip to color count at offset 0x27
    let mut header = vec![0u8; 0x27];
    file.read_exact(&mut header).map_err(|e| {
        if e.kind() == std::io::ErrorKind::UnexpectedEof {
            crate::utils::error::Error::Parse(format!(
                "XXX file too small: minimum {} bytes required",
                MIN_FILE_SIZE
            ))
        } else {
            crate::utils::error::Error::from(e)
        }
    })?;

    let num_colors = file.read_u16::<LittleEndian>()?;

    // Validate color count
    if num_colors > MAX_COLORS {
        return Err(crate::utils::error::Error::Parse(format!(
            "XXX color count too large: {} (max {})",
            num_colors, MAX_COLORS
        )));
    }

    // Skip to stitch data at offset 0x100
    let mut skip_bytes = vec![0u8; 0x100 - 0x27 - 2];
    file.read_exact(&mut skip_bytes)?;

    // Read stitches
    let mut stitch_count = 0;

    loop {
        // Check for excessive stitch count
        stitch_count += 1;
        if stitch_count > MAX_STITCHES {
            return Err(crate::utils::error::Error::Parse(format!(
                "XXX file exceeds maximum stitch count of {}",
                MAX_STITCHES
            )));
        }

        let b1 = file.read_u8()?;

        // Long stitch/jump (0x7D or 0x7E)
        if b1 == 0x7D || b1 == 0x7E {
            let x = file.read_u16::<LittleEndian>()?;
            let y = file.read_u16::<LittleEndian>()?;
            pattern.add_stitch_relative(read_signed_i16(x), -read_signed_i16(y), JUMP);
            continue;
        }

        let b2 = file.read_u8()?;

        // Normal stitch (not starting with 0x7F)
        if b1 != 0x7F {
            pattern.add_stitch_relative(read_signed_i8(b1), -read_signed_i8(b2), STITCH);
            continue;
        }

        // Special command (starting with 0x7F)
        let b3 = file.read_u8()?;
        let b4 = file.read_u8()?;

        match b2 {
            // Move (0x7F 01 dx dy)
            0x01 => {
                pattern.add_stitch_relative(read_signed_i8(b3), -read_signed_i8(b4), JUMP);
            },
            // Trim (0x7F 03 dx dy)
            0x03 => {
                let x = read_signed_i8(b3);
                let y = -read_signed_i8(b4);
                pattern.add_stitch_relative(x, y, TRIM);
            },
            // Color change (0x7F 08 or 0x7F 0A-17)
            0x08 | 0x0A..=0x17 => {
                let x = read_signed_i8(b3);
                let y = -read_signed_i8(b4);
                pattern.add_stitch_relative(x, y, COLOR_CHANGE);
            },
            // End (0x7F 7F or 0x7F 18)
            0x7F | 0x18 => {
                break;
            },
            _ => {
                // Unknown command, skip
            },
        }
    }

    pattern.end();

    // Skip 2 bytes before color data
    let mut skip = [0u8; 2];
    file.read_exact(&mut skip)?;

    // Read thread colors
    for _ in 0..num_colors {
        let color = file.read_u32::<BigEndian>()?;
        let thread = crate::core::thread::EmbThread::from_rgb(
            ((color >> 16) & 0xFF) as u8,
            ((color >> 8) & 0xFF) as u8,
            (color & 0xFF) as u8,
        );
        pattern.add_thread(thread);
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    #[test]
    fn test_read_xxx_basic() {
        // Create minimal XXX file
        let mut data = vec![0u8; 0x100];

        // Set color count at offset 0x27
        data[0x27] = 1; // 1 color (little-endian low byte)
        data[0x28] = 0; // high byte

        // Add some stitches starting at 0x100
        data.extend_from_slice(&[
            10, 20, // Normal stitch (10, -20)
            0x7F, 0x7F, 0x02, 0x14, // End command
            0x00, 0x00, // 2-byte skip before colors
            0x00, 0xFF, 0x00, 0x00, // Color: red (0x00FF0000 big-endian)
        ]);

        let mut cursor = Cursor::new(data);
        let mut pattern = EmbPattern::new();

        read(&mut cursor, &mut pattern).unwrap();

        assert_eq!(pattern.stitches().len(), 2); // stitch + end
        assert_eq!(pattern.threads().len(), 1);
    }

    #[test]
    fn test_read_xxx_long_stitch() {
        let mut data = vec![0u8; 0x100];
        data[0x27] = 0; // 0 colors
        data[0x28] = 0;

        // Long stitch: 0x7D + i16le dx + i16le dy
        data.extend_from_slice(&[
            0x7D, // Long stitch marker
            200, 0, // dx = 200 (little-endian)
            100, 0, // dy = 100 (little-endian)
            0x7F, 0x7F, 0x02, 0x14, // End
            0x00, 0x00, // Skip before colors
        ]);

        let mut cursor = Cursor::new(data);
        let mut pattern = EmbPattern::new();

        read(&mut cursor, &mut pattern).unwrap();

        let stitches = pattern.stitches();
        assert!(!stitches.is_empty());
        // Y is flipped, so 100 becomes -100
        assert_eq!((stitches[0].command & COMMAND_MASK), JUMP);
    }

    #[test]
    fn test_read_xxx_color_change() {
        let mut data = vec![0u8; 0x100];
        data[0x27] = 2; // 2 colors
        data[0x28] = 0;

        data.extend_from_slice(&[
            5, 5, // Normal stitch
            0x7F, 0x08, 0, 0, // Color change
            10, 10, // Another stitch
            0x7F, 0x7F, 0x02, 0x14, // End
            0x00, 0x00, // Skip
            0x00, 0xFF, 0x00, 0x00, // Red
            0x00, 0x00, 0xFF, 0x00, // Blue
        ]);

        let mut cursor = Cursor::new(data);
        let mut pattern = EmbPattern::new();

        read(&mut cursor, &mut pattern).unwrap();

        assert_eq!(pattern.threads().len(), 2);
        let commands: Vec<u32> = pattern
            .stitches()
            .iter()
            .map(|s| s.command & COMMAND_MASK)
            .collect();
        assert!(commands.contains(&COLOR_CHANGE));
    }
}
