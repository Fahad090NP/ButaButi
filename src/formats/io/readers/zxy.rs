//! ZSK USA ZXY format reader
//!
//! ZXY is ZSK USA's embroidery format with coordinate-based stitches and colors,
//! designed for ZSK industrial embroidery equipment.

use crate::core::constants::*;
use crate::core::pattern::EmbPattern;
use crate::utils::error::Result;
use crate::utils::functions::encode_thread_change;
use std::io::{Read, Seek, SeekFrom};

/// Read ZXY (ZSK TC) format
pub fn read(file: &mut (impl Read + Seek), pattern: &mut EmbPattern) -> Result<()> {
    // Seek to offset 0x01
    file.seek(SeekFrom::Start(0x01))?;

    let stitch_start_distance = read_u16_be(file)?;

    // Skip to stitch data
    file.seek(SeekFrom::Current(stitch_start_distance as i64))?;

    read_zxy_stitches(file, pattern)?;

    Ok(())
}

/// Read ZXY stitches
fn read_zxy_stitches(file: &mut impl Read, pattern: &mut EmbPattern) -> Result<()> {
    let mut buffer = [0u8; 3];

    loop {
        // Read 3 bytes
        match file.read_exact(&mut buffer) {
            Ok(_) => {}
            Err(e) if e.kind() == std::io::ErrorKind::UnexpectedEof => break,
            Err(e) => return Err(e.into()),
        }

        let mut ctrl = buffer[0];
        let mut x = buffer[1] as i8 as i32;
        let mut y = -(buffer[2] as i8 as i32);

        // Check sign bits
        if ctrl & 0x08 != 0 {
            x = -x;
        }
        if ctrl & 0x04 != 0 {
            y = -y;
        }

        // Clear sign bits
        ctrl &= !0x0C;

        if ctrl == 0 {
            // Stitch
            pattern.add_stitch_relative(x as f64, y as f64, STITCH);
        } else if ctrl & 0x02 != 0 {
            // Move/Jump
            pattern.add_stitch_relative(x as f64, y as f64, JUMP);
        } else if ctrl & 0x20 != 0 {
            // Needle change or end
            if buffer[1] == 0xFF {
                // End marker
                break;
            }
            let needle = buffer[2];
            let command = encode_thread_change(NEEDLE_SET, None, Some(needle), None);
            pattern.add_command(command, 0.0, 0.0);
        }
    }

    pattern.add_command(END, 0.0, 0.0);
    Ok(())
}

/// Read unsigned 16-bit big-endian integer
fn read_u16_be(file: &mut impl Read) -> Result<u16> {
    let mut buf = [0u8; 2];
    file.read_exact(&mut buf)?;
    Ok(u16::from_be_bytes(buf))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    #[test]
    fn test_read_zxy_basic() {
        let mut zxy_data = vec![0u8]; // Padding to offset 1

        // At offset 1: stitch start distance (0 = immediate)
        zxy_data.extend_from_slice(&[0, 0]);

        // Stitch: ctrl=0, x=10, y=10 (will be negated to -10)
        zxy_data.extend_from_slice(&[0, 10, 10]);

        // End: ctrl with 0x20 bit set, x=0xFF
        zxy_data.extend_from_slice(&[0x20, 0xFF, 0]);

        let mut cursor = Cursor::new(zxy_data);
        let mut pattern = EmbPattern::new();

        read(&mut cursor, &mut pattern).expect("Failed to read ZXY");

        assert!(!pattern.stitches().is_empty());
    }

    #[test]
    fn test_zxy_needle_change() {
        let mut zxy_data = vec![0u8, 0, 0];

        // Needle change to needle 3
        zxy_data.extend_from_slice(&[0x20, 0, 3]);

        // Stitch
        zxy_data.extend_from_slice(&[0, 5, 5]);

        // End
        zxy_data.extend_from_slice(&[0x20, 0xFF, 0]);

        let mut cursor = Cursor::new(zxy_data);
        let mut pattern = EmbPattern::new();

        read(&mut cursor, &mut pattern).expect("Failed to read ZXY");

        assert!(!pattern.stitches().is_empty());
    }

    #[test]
    fn test_zxy_negative_coords() {
        let mut zxy_data = vec![0u8, 0, 0];

        // Stitch with sign bits set: ctrl=0x0C (both x and y negative)
        // x=10, y=10, both will be negated
        zxy_data.extend_from_slice(&[0x0C, 10, 10]);

        // End
        zxy_data.extend_from_slice(&[0x20, 0xFF, 0]);

        let mut cursor = Cursor::new(zxy_data);
        let mut pattern = EmbPattern::new();

        read(&mut cursor, &mut pattern).expect("Failed to read ZXY");

        assert!(!pattern.stitches().is_empty());
    }
}
