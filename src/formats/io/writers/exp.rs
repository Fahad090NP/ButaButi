//! Melco EXP format writer
//!
//! Writes EXP format with 2-byte stitch records using bit-encoded coordinates.
//! Supports stitches, jumps, trims, and color changes for Melco machines.

use crate::core::constants::*;
use crate::core::pattern::EmbPattern;
use crate::formats::io::utils::WriteHelper;
use crate::utils::error::Result;
use std::io::Write;

/// Write EXP file
pub fn write<W: Write>(writer: &mut W, pattern: &EmbPattern) -> Result<()> {
    let mut helper = WriteHelper::new(writer);

    let mut xx = 0.0;
    let mut yy = 0.0;

    for stitch in pattern.stitches() {
        let x = stitch.x;
        let y = stitch.y;
        let data = stitch.command & COMMAND_MASK;

        let dx = (x - xx).round() as i32;
        let dy = (y - yy).round() as i32;

        xx += dx as f64;
        yy += dy as f64;

        match data {
            STITCH => {
                let delta_x = (dx & 0xFF) as u8;
                let delta_y = ((-dy) & 0xFF) as u8;
                helper.write_u8(delta_x)?;
                helper.write_u8(delta_y)?;
            },
            JUMP => {
                helper.write_u8(0x80)?;
                helper.write_u8(0x04)?;
                let delta_x = (dx & 0xFF) as u8;
                let delta_y = ((-dy) & 0xFF) as u8;
                helper.write_u8(delta_x)?;
                helper.write_u8(delta_y)?;
            },
            TRIM => {
                helper.write_bytes(&[0x80, 0x80, 0x07, 0x00])?;
            },
            COLOR_CHANGE | STOP => {
                helper.write_bytes(&[0x80, 0x01, 0x00, 0x00])?;
            },
            END => {
                // END doesn't write anything in EXP
            },
            _ => {
                // Other commands ignored
            },
        }
    }

    Ok(())
}

/// Write EXP file to path
pub fn write_file(path: &str, pattern: &EmbPattern) -> Result<()> {
    let file = std::fs::File::create(path)?;
    let mut writer = std::io::BufWriter::new(file);
    write(&mut writer, pattern)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_exp_write_basic() {
        let mut pattern = EmbPattern::new();
        pattern.stitch(10.0, 20.0);
        pattern.stitch(5.0, 10.0);
        pattern.end();

        let mut buffer = Vec::new();
        let result = write(&mut buffer, &pattern);
        assert!(result.is_ok());
        assert!(!buffer.is_empty());
    }

    #[test]
    fn test_exp_write_with_trim() {
        let mut pattern = EmbPattern::new();
        pattern.stitch(10.0, 20.0);
        pattern.trim();
        pattern.stitch(5.0, 10.0);
        pattern.end();

        let mut buffer = Vec::new();
        let result = write(&mut buffer, &pattern);
        assert!(result.is_ok());
        assert!(!buffer.is_empty());
    }

    #[test]
    fn test_exp_round_trip() {
        use crate::formats::io::readers::exp;
        use std::io::Cursor;

        // Create original pattern
        let mut original = EmbPattern::new();
        original.add_stitch_absolute(STITCH, 0.0, 0.0);
        original.add_stitch_absolute(STITCH, 100.0, 0.0);
        original.add_stitch_absolute(STITCH, 100.0, 100.0);
        original.add_stitch_absolute(STITCH, 0.0, 100.0);
        original.add_stitch_absolute(STITCH, 0.0, 0.0);
        original.end();

        let original_stitch_count = original.count_stitches();

        // Write to buffer
        let mut buffer = Cursor::new(Vec::new());
        write(&mut buffer, &original).unwrap();

        // Verify buffer has data
        assert!(!buffer.get_ref().is_empty());

        // Read back
        buffer.set_position(0);
        let read_back = exp::read(&mut buffer).unwrap();

        // Verify stitch count (should be close, allowing for encoding differences)
        let read_stitch_count = read_back.count_stitches();
        assert!(
            read_stitch_count >= original_stitch_count - 2
                && read_stitch_count <= original_stitch_count + 2,
            "Stitch count mismatch: original={}, read={}",
            original_stitch_count,
            read_stitch_count
        );

        // Verify we have stitches
        assert!(!read_back.stitches().is_empty());

        // Verify coordinate bounds are reasonable
        let (min_x, min_y, max_x, max_y) = read_back.bounds();
        assert!(max_x - min_x > 0.0, "Pattern has no width");
        assert!(max_y - min_y > 0.0, "Pattern has no height");
    }
}
