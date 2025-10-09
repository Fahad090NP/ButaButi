//! Inbro INB format reader
//!
//! INB is an Inbro embroidery format with basic stitch and color change commands,
//! using simple coordinate encoding for embroidery designs.

use crate::core::constants::*;
use crate::core::pattern::EmbPattern;
use anyhow::Result;
use std::io::{Read, Seek, SeekFrom};

/// Read INB (Inbro) format
///
/// INB format has an 8192-byte (0x2000) header followed by stitch data.
pub fn read(file: &mut (impl Read + Seek), pattern: &mut EmbPattern) -> Result<()> {
    // Skip 8192-byte header
    file.seek(SeekFrom::Start(0x2000))?;

    read_inb_stitches(file, pattern)?;

    Ok(())
}

/// Read INB stitch encoding
fn read_inb_stitches(file: &mut impl Read, pattern: &mut EmbPattern) -> Result<()> {
    let mut buffer = [0u8; 3];

    loop {
        // Try to read 3 bytes
        match file.read_exact(&mut buffer) {
            Ok(_) => {}
            Err(e) if e.kind() == std::io::ErrorKind::UnexpectedEof => break,
            Err(e) => return Err(e.into()),
        }

        let x_byte = buffer[0];
        let y_byte = buffer[1];
        let ctrl = buffer[2];

        let mut x = x_byte as i32;
        let mut y = -(y_byte as i8) as i32;

        // Apply sign bits
        if ctrl & 0x20 != 0 {
            y = -y;
        }
        if ctrl & 0x40 != 0 {
            x = -x;
        }

        let command_bits = ctrl & 0b1111;

        if command_bits == 0x00 {
            // Stitch
            pattern.add_stitch_relative(x as f64, y as f64, STITCH);
        } else if command_bits == 0x01 {
            // Color change
            pattern.add_stitch_relative(x as f64, y as f64, COLOR_CHANGE);
        } else if command_bits == 0x02 {
            // Move/Jump
            pattern.add_stitch_relative(x as f64, y as f64, JUMP);
        } else if ctrl == 0x04 {
            // End
            break;
        } else {
            // Unknown command
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
    fn test_read_inb_basic() {
        // Create 8192-byte header + stitch data
        let mut inb_data = vec![0u8; 0x2000];

        // Add INB-encoded stitches
        // Stitch at (10, -10): x=10, y=10 (negated), ctrl=0x00
        inb_data.extend_from_slice(&[10, 10, 0x00]);

        // Jump at (5, -5)
        inb_data.extend_from_slice(&[5, 5, 0x02]);

        // Color change
        inb_data.extend_from_slice(&[0, 0, 0x01]);

        // End
        inb_data.extend_from_slice(&[0, 0, 0x04]);

        let mut cursor = Cursor::new(inb_data);
        let mut pattern = EmbPattern::new();

        read(&mut cursor, &mut pattern).expect("Failed to read INB");

        assert!(!pattern.stitches().is_empty());
    }

    #[test]
    fn test_inb_negative_coordinates() {
        let mut inb_data = vec![0u8; 0x2000];

        // Stitch with x negative (bit 0x40 set)
        inb_data.extend_from_slice(&[10, 5, 0x40]);

        // Stitch with y negative (bit 0x20 set)
        inb_data.extend_from_slice(&[10, 5, 0x20]);

        // End
        inb_data.extend_from_slice(&[0, 0, 0x04]);

        let mut cursor = Cursor::new(inb_data);
        let mut pattern = EmbPattern::new();

        read(&mut cursor, &mut pattern).expect("Failed to read INB");

        assert!(!pattern.stitches().is_empty());
    }
}
