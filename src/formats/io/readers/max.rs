//! Pfaff MAX format reader
//!
//! MAX is a Pfaff embroidery format with simple binary stitch encoding,
//! used by older Pfaff embroidery software and machines.

use crate::core::constants::*;
use crate::core::pattern::EmbPattern;
use anyhow::Result;
use std::io::{Read, Seek, SeekFrom};

const MAX_SIZE_CONVERSION_RATIO: f64 = 1.235;

/// Read MAX (Pfaff) format
///
/// MAX format has a header with stitch count at offset 0xD5 (213).
/// Each stitch is 8 bytes: x (3 bytes), c0 (1 byte), y (3 bytes), c1 (1 byte).
pub fn read(file: &mut (impl Read + Seek), pattern: &mut EmbPattern) -> Result<()> {
    // Seek to stitch count at offset 0xD5
    file.seek(SeekFrom::Start(0xD5))?;

    let stitch_count = read_u32_le(file)?;

    for _ in 0..stitch_count {
        let x = read_i24_le(file)?;
        let _c0 = read_u8(file)?;
        let y = read_i24_le(file)?;

        // Try to read c1
        match read_u8(file) {
            Ok(_c1) => {
                let x = x as f64 * MAX_SIZE_CONVERSION_RATIO;
                let y = y as f64 * MAX_SIZE_CONVERSION_RATIO;
                pattern.add_stitch_absolute(STITCH, x, y);
            }
            Err(_) => break,
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

/// Read unsigned 32-bit little-endian integer
fn read_u32_le(file: &mut impl Read) -> Result<u32> {
    let mut buf = [0u8; 4];
    file.read_exact(&mut buf)?;
    Ok(u32::from_le_bytes(buf))
}

/// Read signed 24-bit little-endian integer
fn read_i24_le(file: &mut impl Read) -> Result<i32> {
    let mut buf = [0u8; 3];
    file.read_exact(&mut buf)?;

    // Extend to 32-bit signed value
    let value = u32::from_le_bytes([buf[0], buf[1], buf[2], 0]);

    // If sign bit is set, extend to negative
    if value & 0x00800000 != 0 {
        Ok((value | 0xFF000000) as i32)
    } else {
        Ok(value as i32)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    #[test]
    fn test_read_max_basic() {
        // Create header with stitch count at 0xD5
        let mut max_data = vec![0u8; 0xD5];

        // Stitch count: 2
        max_data.extend_from_slice(&[2, 0, 0, 0]);

        // Stitch 1: x=100, c0=0, y=200, c1=0
        max_data.extend_from_slice(&[100, 0, 0]); // x (24-bit LE)
        max_data.extend_from_slice(&[0]); // c0
        max_data.extend_from_slice(&[200, 0, 0]); // y (24-bit LE)
        max_data.extend_from_slice(&[0]); // c1

        // Stitch 2: x=-50, c0=0, y=-100, c1=0
        let x_bytes = (-50i32 as u32).to_le_bytes();
        max_data.extend_from_slice(&[x_bytes[0], x_bytes[1], x_bytes[2]]);
        max_data.extend_from_slice(&[0]);
        let y_bytes = (-100i32 as u32).to_le_bytes();
        max_data.extend_from_slice(&[y_bytes[0], y_bytes[1], y_bytes[2]]);
        max_data.extend_from_slice(&[0]);

        let mut cursor = Cursor::new(max_data);
        let mut pattern = EmbPattern::new();

        read(&mut cursor, &mut pattern).expect("Failed to read MAX");

        assert!(!pattern.stitches().is_empty());
    }

    #[test]
    fn test_i24_le_positive() {
        let data = vec![0x34, 0x12, 0x00]; // 0x001234 = 4660
        let mut cursor = Cursor::new(data);
        let value = read_i24_le(&mut cursor).expect("Failed to read i24");
        assert_eq!(value, 0x1234);
    }

    #[test]
    fn test_i24_le_negative() {
        let data = vec![0xFF, 0xFF, 0xFF]; // -1 in 24-bit
        let mut cursor = Cursor::new(data);
        let value = read_i24_le(&mut cursor).expect("Failed to read i24");
        assert_eq!(value, -1);
    }
}
