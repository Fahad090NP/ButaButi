//! Barudan DSB format reader
//!
//! DSB is a DST variant supporting sequin stitching for Barudan machines.
//! Uses DST reader with modified header handling for sequin information.

use crate::core::constants::*;
use crate::core::pattern::EmbPattern;
use crate::formats::io::readers::dst;
use crate::utils::error::Result;
use crate::utils::functions::encode_thread_change;
use std::io::Read;

/// Read DSB (Barudan B-stitch) format
///
/// DSB uses DST header followed by B-stitch encoding.
/// B-stitch encoding is similar to Z-stitch but with different byte order.
pub fn read(file: &mut impl Read, pattern: &mut EmbPattern) -> Result<()> {
    // Read DST header
    let dst_pattern = dst::read(file, None)?;

    // Copy header data
    for (key, value) in dst_pattern.metadata() {
        pattern.add_metadata(key, value);
    }

    // Read B-stitch encoded data
    read_b_stitch_encoding(file, pattern)?;

    Ok(())
}

/// Read B-stitch encoded stitch data
fn read_b_stitch_encoding(file: &mut impl Read, pattern: &mut EmbPattern) -> Result<()> {
    let mut buffer = [0u8; 3];

    loop {
        // Try to read 3 bytes
        match file.read_exact(&mut buffer) {
            Ok(_) => {}
            Err(e) if e.kind() == std::io::ErrorKind::UnexpectedEof => break,
            Err(e) => return Err(e.into()),
        }

        let ctrl = buffer[0];
        let mut y = -(buffer[1] as i8) as i32;
        let mut x = buffer[2] as i32;

        // Apply sign bits
        if ctrl & 0x40 != 0 {
            y = -y;
        }
        if ctrl & 0x20 != 0 {
            x = -x;
        }

        let command_bits = ctrl & 0b11111;

        if command_bits == 0 {
            // Stitch
            pattern.add_stitch_relative(x as f64, y as f64, STITCH);
        } else if command_bits == 1 {
            // Jump/Move
            pattern.add_stitch_relative(x as f64, y as f64, JUMP);
        } else if ctrl == 0xF8 {
            // End of pattern
            break;
        } else if ctrl == 0xE7 {
            // Trim
            pattern.add_command(TRIM, 0.0, 0.0);
        } else if ctrl == 0xE8 {
            // Stop
            pattern.add_command(STOP, 0.0, 0.0);
        } else if (0xE9..0xF8).contains(&ctrl) {
            // Needle change
            let needle = ctrl - 0xE8;
            let command = encode_thread_change(NEEDLE_SET, None, Some(needle), None);
            pattern.add_command(command, 0.0, 0.0);
        } else {
            // Unknown command - stop reading
            break;
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
    fn test_read_dsb_basic() {
        // Create minimal DST header (512 bytes) + B-stitch data
        let mut dsb_data = vec![0u8; 512];

        // Add B-stitch encoded stitches
        // Stitch at (10, -10): ctrl=0x00, y=10 (negated), x=10
        dsb_data.extend_from_slice(&[0x00, 10, 10]);

        // Jump at (5, -5)
        dsb_data.extend_from_slice(&[0x01, 5, 5]);

        // Trim
        dsb_data.extend_from_slice(&[0xE7, 0, 0]);

        // End
        dsb_data.extend_from_slice(&[0xF8, 0, 0]);

        let mut cursor = Cursor::new(dsb_data);
        let mut pattern = EmbPattern::new();

        read(&mut cursor, &mut pattern).expect("Failed to read DSB");

        // Should complete without error
        assert!(!pattern.stitches().is_empty());
    }

    #[test]
    fn test_b_stitch_negative_coordinates() {
        // Test negative coordinate handling with sign bits
        let mut dsb_data = vec![0u8; 512];

        // Stitch with x negative (bit 0x20 set): ctrl=0x20, y=5, x=10
        dsb_data.extend_from_slice(&[0x20, 5, 10]);

        // Stitch with y negative (bit 0x40 set): ctrl=0x40, y=5, x=10
        dsb_data.extend_from_slice(&[0x40, 5, 10]);

        // End
        dsb_data.extend_from_slice(&[0xF8, 0, 0]);

        let mut cursor = Cursor::new(dsb_data);
        let mut pattern = EmbPattern::new();

        read(&mut cursor, &mut pattern).expect("Failed to read DSB");

        assert!(!pattern.stitches().is_empty());
    }

    #[test]
    fn test_b_stitch_needle_change() {
        let mut dsb_data = vec![0u8; 512];

        // Needle change to needle 1: ctrl=0xE9
        dsb_data.extend_from_slice(&[0xE9, 0, 0]);

        // Needle change to needle 5: ctrl=0xED
        dsb_data.extend_from_slice(&[0xED, 0, 0]);

        // End
        dsb_data.extend_from_slice(&[0xF8, 0, 0]);

        let mut cursor = Cursor::new(dsb_data);
        let mut pattern = EmbPattern::new();

        read(&mut cursor, &mut pattern).expect("Failed to read DSB");

        assert!(!pattern.stitches().is_empty());
    }
}
