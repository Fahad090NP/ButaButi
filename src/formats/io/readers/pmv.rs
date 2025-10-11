//! Brother PMV format reader
//!
//! PMV is a Brother PES variant format with similar structure and encoding,
//! supporting PES-compatible designs for Brother embroidery machines.

use crate::core::constants::*;
use crate::core::pattern::EmbPattern;
use crate::utils::error::Result;
use std::io::{Read, Seek, SeekFrom};

/// Read PMV (Brother PMV) format
///
/// PMV files are stitch files, not traditional embroidery patterns.
pub fn read(file: &mut (impl Read + Seek), pattern: &mut EmbPattern) -> Result<()> {
    // Seek to stitch data at offset 0x64
    file.seek(SeekFrom::Start(0x64))?;

    read_pmv_stitches(file, pattern)?;

    Ok(())
}

/// Read PMV stitches
fn read_pmv_stitches(file: &mut impl Read, pattern: &mut EmbPattern) -> Result<()> {
    let mut px = 0.0;

    #[allow(clippy::while_let_loop)]
    loop {
        let stitch_count = match read_u16_le(file) {
            Ok(v) => v,
            Err(_) => break,
        };

        let block_length = match read_u16_le(file) {
            Ok(v) => v,
            Err(_) => return Ok(()), // End of file
        };

        if block_length >= 256 {
            break;
        }

        if stitch_count == 0 {
            continue;
        }

        for _ in 0..stitch_count {
            let x_byte = match read_u8(file) {
                Ok(v) => v,
                Err(_) => break,
            };

            let y_byte = match read_u8(file) {
                Ok(v) => v,
                Err(_) => break,
            };

            // Decode 5-bit signed y
            let mut y = y_byte as i32;
            if y > 16 {
                y = -(32 - y);
            }

            // Decode 6-bit signed x
            let mut x = x_byte as i32;
            if x > 32 {
                x = -(64 - x);
            }

            // Scale coordinates
            let x_scaled = x as f64 * 2.5;
            let y_scaled = -(y as f64) * 2.5;

            // This is a hybrid relative-absolute format
            // X is accumulated, Y is absolute
            let dx = x_scaled;
            pattern.add_stitch_absolute(STITCH, px + x_scaled, y_scaled);
            px += dx;
        }
    }

    pattern.add_command(END, 0.0, 0.0);
    Ok(())
}

/// Read unsigned 8-bit integer
fn read_u8(file: &mut impl Read) -> Result<u8> {
    let mut buf = [0u8; 1];
    file.read_exact(&mut buf)?;
    Ok(buf[0])
}

/// Read unsigned 16-bit little-endian integer
fn read_u16_le(file: &mut impl Read) -> Result<u16> {
    let mut buf = [0u8; 2];
    file.read_exact(&mut buf)?;
    Ok(u16::from_le_bytes(buf))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    #[test]
    fn test_read_pmv_basic() {
        // Create header (0x64 bytes)
        let mut pmv_data = vec![0u8; 0x64];

        // Block: stitch_count=2, block_length=4
        pmv_data.extend_from_slice(&2u16.to_le_bytes());
        pmv_data.extend_from_slice(&4u16.to_le_bytes());

        // Stitch 1: x=10, y=10
        pmv_data.extend_from_slice(&[10, 10]);

        // Stitch 2: x=5, y=5
        pmv_data.extend_from_slice(&[5, 5]);

        // End block: stitch_count=0, block_length=256+
        pmv_data.extend_from_slice(&0u16.to_le_bytes());
        pmv_data.extend_from_slice(&256u16.to_le_bytes());

        let mut cursor = Cursor::new(pmv_data);
        let mut pattern = EmbPattern::new();

        read(&mut cursor, &mut pattern).expect("Failed to read PMV");

        assert!(!pattern.stitches().is_empty());
    }

    #[test]
    fn test_pmv_negative_coords() {
        let mut pmv_data = vec![0u8; 0x64];

        // Block with negative coordinates
        pmv_data.extend_from_slice(&1u16.to_le_bytes());
        pmv_data.extend_from_slice(&2u16.to_le_bytes());

        // x=50 (> 32, so -(64-50) = -14), y=20 (> 16, so -(32-20) = -12)
        pmv_data.extend_from_slice(&[50, 20]);

        // End
        pmv_data.extend_from_slice(&0u16.to_le_bytes());
        pmv_data.extend_from_slice(&256u16.to_le_bytes());

        let mut cursor = Cursor::new(pmv_data);
        let mut pattern = EmbPattern::new();

        read(&mut cursor, &mut pattern).expect("Failed to read PMV");

        assert!(!pattern.stitches().is_empty());
    }

    #[test]
    fn test_pmv_multiple_blocks() {
        let mut pmv_data = vec![0u8; 0x64];

        // Block 1
        pmv_data.extend_from_slice(&1u16.to_le_bytes());
        pmv_data.extend_from_slice(&2u16.to_le_bytes());
        pmv_data.extend_from_slice(&[10, 10]);

        // Block 2
        pmv_data.extend_from_slice(&1u16.to_le_bytes());
        pmv_data.extend_from_slice(&2u16.to_le_bytes());
        pmv_data.extend_from_slice(&[5, 5]);

        // End
        pmv_data.extend_from_slice(&0u16.to_le_bytes());
        pmv_data.extend_from_slice(&256u16.to_le_bytes());

        let mut cursor = Cursor::new(pmv_data);
        let mut pattern = EmbPattern::new();

        read(&mut cursor, &mut pattern).expect("Failed to read PMV");

        assert!(!pattern.stitches().is_empty());
    }
}
