//! Ameco NEW format reader
//!
//! NEW is Ameco's embroidery format with coordinate-based stitches and
//! basic command encoding for Ameco embroidery systems.

use crate::core::constants::*;
use crate::core::pattern::EmbPattern;
use anyhow::Result;
use std::io::{Read, Seek, SeekFrom};

/// Read NEW (Ameco) format
///
/// NEW format has a 2-byte stitch count header followed by stitch data.
pub fn read(file: &mut (impl Read + Seek), pattern: &mut EmbPattern) -> Result<()> {
    // Skip 2-byte stitch count
    file.seek(SeekFrom::Current(2))?;

    read_new_stitches(file, pattern)?;

    Ok(())
}

/// Read NEW stitch encoding
fn read_new_stitches(file: &mut impl Read, pattern: &mut EmbPattern) -> Result<()> {
    let mut buffer = [0u8; 3];

    loop {
        // Try to read 3 bytes
        match file.read_exact(&mut buffer) {
            Ok(_) => {}
            Err(e) if e.kind() == std::io::ErrorKind::UnexpectedEof => break,
            Err(e) => return Err(e.into()),
        }

        let mut x = buffer[0] as i32;
        let mut y = -(buffer[1] as i32);
        let mut ctrl = buffer[2];

        // Check sign bits
        if ctrl & 0b01000000 != 0 {
            x = -x;
        }
        if ctrl & 0b00100000 != 0 {
            y = -y;
        }

        // Clear upper bits
        ctrl &= !0b11100000;

        if ctrl == 0 {
            // Stitch
            pattern.add_stitch_relative(x as f64, y as f64, STITCH);
        } else if ctrl == 0b00010001 {
            // End
            break;
        } else if ctrl & 0b00000010 != 0 {
            // Color change
            pattern.add_command(COLOR_CHANGE, 0.0, 0.0);
        } else if ctrl & 0b00000001 != 0 {
            // Move/Jump
            pattern.add_stitch_relative(x as f64, y as f64, JUMP);
        }
    }

    pattern.add_command(END, 0.0, 0.0);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    #[test]
    fn test_read_new_basic() {
        // Create 2-byte header (stitch count)
        let mut new_data = vec![0u8, 3u8];

        // Stitch at (10, -10): x=10, y=10 (negated), ctrl=0
        new_data.extend_from_slice(&[10, 10, 0]);

        // Jump at (5, -5): ctrl=0b00000001
        new_data.extend_from_slice(&[5, 5, 0b00000001]);

        // Stitch at (-3, 7): x=3, ctrl with sign bit 0b01000000, y=-7 (input 7, negated to -7, sign bit 0b00100000)
        new_data.extend_from_slice(&[3, 7, 0b01100000]);

        // End
        new_data.extend_from_slice(&[0, 0, 0b00010001]);

        let mut cursor = Cursor::new(new_data);
        let mut pattern = EmbPattern::new();

        read(&mut cursor, &mut pattern).expect("Failed to read NEW");

        assert!(!pattern.stitches().is_empty());
    }

    #[test]
    fn test_new_color_change() {
        let mut new_data = vec![0u8, 1u8];

        // Color change: ctrl=0b00000010
        new_data.extend_from_slice(&[0, 0, 0b00000010]);

        // Stitch
        new_data.extend_from_slice(&[10, 10, 0]);

        // End
        new_data.extend_from_slice(&[0, 0, 0b00010001]);

        let mut cursor = Cursor::new(new_data);
        let mut pattern = EmbPattern::new();

        read(&mut cursor, &mut pattern).expect("Failed to read NEW");

        assert!(!pattern.stitches().is_empty());
    }
}
