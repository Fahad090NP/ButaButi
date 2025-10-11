//! ZSK DSZ format reader
//!
//! DSZ is a ZSK variant of DST format with header differences and Z-stitch encoding.
//! Compatible with DST reader after skipping the ZSK-specific header.

use crate::core::constants::*;
use crate::core::pattern::EmbPattern;
use crate::formats::io::readers::dst;
use crate::utils::error::Result;
use crate::utils::functions::encode_thread_change;
use std::io::Read;

/// Read DSZ (ZSK USA Design) format
///
/// DSZ format uses a DST header followed by Z-stitch encoding.
pub fn read(file: &mut impl Read, pattern: &mut EmbPattern) -> Result<()> {
    // Read DST header (512 bytes)
    let dst_pattern = dst::read(file, None)?;

    // Copy header metadata
    for (key, value) in dst_pattern.metadata() {
        pattern.add_metadata(key, value);
    }

    // Read Z-stitch encoded stitches
    read_z_stitches(file, pattern)?;

    Ok(())
}

/// Read Z-stitch encoding (used by DSZ, GT formats)
pub fn read_z_stitches(file: &mut impl Read, pattern: &mut EmbPattern) -> Result<()> {
    let mut buffer = [0u8; 3];

    loop {
        // Try to read 3 bytes
        match file.read_exact(&mut buffer) {
            Ok(_) => {}
            Err(e) if e.kind() == std::io::ErrorKind::UnexpectedEof => break,
            Err(e) => return Err(e.into()),
        }

        let y_byte = buffer[0];
        let x_byte = buffer[1];
        let ctrl = buffer[2];

        // Decode coordinates
        let mut x = x_byte as i32;
        let mut y = -(y_byte as i32);

        if ctrl & 0x40 != 0 {
            x = -x;
        }
        if ctrl & 0x20 != 0 {
            y = -y;
        }

        // Process command
        let command_bits = ctrl & 0b11111;

        if command_bits == 0 {
            // Stitch
            pattern.add_stitch_relative(x as f64, y as f64, STITCH);
        } else if command_bits == 1 {
            // Jump
            pattern.add_stitch_relative(x as f64, y as f64, JUMP);
        } else if ctrl == 0x82 {
            // Stop
            pattern.add_command(STOP, 0.0, 0.0);
        } else if ctrl == 0x9B {
            // Trim
            pattern.add_command(TRIM, 0.0, 0.0);
        } else if (0x83..=0x9A).contains(&ctrl) {
            // Needle change
            let needle = (ctrl - 0x83) >> 1;
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
    fn test_read_dsz_basic() {
        // Create minimal DST header (512 bytes) + Z-stitch data
        let mut dsz_data = vec![0u8; 512];

        // Add Z-stitch encoded stitches
        // Stitch at (10, -10)
        dsz_data.extend_from_slice(&[10, 10, 0x00]); // y=-10, x=10, stitch

        // Jump at (5, -5)
        dsz_data.extend_from_slice(&[5, 5, 0x01]); // y=-5, x=5, jump

        // Trim
        dsz_data.extend_from_slice(&[0, 0, 0x9B]);

        let mut cursor = Cursor::new(dsz_data);
        let mut pattern = EmbPattern::new();

        read(&mut cursor, &mut pattern).expect("Failed to read DSZ");

        // Should complete without error
        assert!(!pattern.stitches().is_empty());
    }

    #[test]
    fn test_z_stitch_negative_coordinates() {
        // Test negative x and y flags
        let z_data = vec![
            10, 10, 0x60, // y=-10 (negated), x=-10 (0x40), stitch
        ];

        let mut cursor = Cursor::new(z_data);
        let mut pattern = EmbPattern::new();

        read_z_stitches(&mut cursor, &mut pattern).expect("Failed to read Z-stitches");

        // Should process the stitch
        assert!(!pattern.stitches().is_empty());
    }

    #[test]
    fn test_z_stitch_needle_change() {
        // Test needle change command
        let z_data = vec![
            0, 0, 0x83, // Needle 0
            0, 0, 0x85, // Needle 1
        ];

        let mut cursor = Cursor::new(z_data);
        let mut pattern = EmbPattern::new();

        read_z_stitches(&mut cursor, &mut pattern).expect("Failed to read Z-stitches");

        // Should have needle change commands
        assert!(pattern.stitches().len() >= 2);
    }
}
