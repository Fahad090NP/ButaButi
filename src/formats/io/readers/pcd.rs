//! Pfaff PCD format reader
//!
//! PCD is a Pfaff format variant with compressed stitch data and metadata,
//! used by Pfaff Creative Design software and machines.

use crate::core::constants::*;
use crate::core::pattern::EmbPattern;
use crate::core::thread::EmbThread;
use crate::utils::error::Result;
use std::io::{Read, Seek, SeekFrom};

const PC_SIZE_CONVERSION_RATIO: f64 = 5.0 / 3.0;

/// Read PCD (Pfaff) format
///
/// PCD format structure:
/// - Version (1 byte)
/// - Hoop size (1 byte): 0=PCD, 1=PCQ(MAXI), 2=PCS small, 3=PCS large
/// - Color count (2 bytes LE)
/// - Thread colors (4 bytes each: 3-byte RGB BE + 1 padding)
/// - Stitch count (2 bytes LE)
/// - Stitches (8 bytes each: c0, x(24-bit LE), c1, y(24-bit LE), ctrl)
pub fn read(file: &mut (impl Read + Seek), pattern: &mut EmbPattern) -> Result<()> {
    // Read version and hoop size
    let _version = read_u8(file)?;
    let _hoop_size = read_u8(file)?;

    // Read color count
    let color_count = read_u16_le(file)?;

    // Read thread palette
    for _ in 0..color_count {
        let rgb = read_u24_be(file)?;
        file.seek(SeekFrom::Current(1))?; // Skip padding byte

        let thread = EmbThread::from_rgb(
            ((rgb >> 16) & 0xFF) as u8,
            ((rgb >> 8) & 0xFF) as u8,
            (rgb & 0xFF) as u8,
        );
        pattern.add_thread(thread);
    }

    // Read stitch count (but don't use it)
    let _stitch_count = read_u16_le(file)?;

    // Read stitches
    #[allow(clippy::while_let_loop)]
    loop {
        // Try to read c0
        let _c0 = match read_u8(file) {
            Ok(v) => v,
            Err(_) => break,
        };

        let x = match read_i24_le(file) {
            Ok(v) => v,
            Err(_) => break,
        };

        let _c1 = match read_u8(file) {
            Ok(v) => v,
            Err(_) => break,
        };

        let y = match read_i24_le(file) {
            Ok(v) => v,
            Err(_) => break,
        };

        let ctrl = match read_u8(file) {
            Ok(c) => c,
            Err(_) => break,
        };

        let x = x as f64 * PC_SIZE_CONVERSION_RATIO;
        let y = -(y as f64) * PC_SIZE_CONVERSION_RATIO;

        if ctrl == 0x00 {
            // Stitch
            pattern.add_stitch_absolute(STITCH, x, y);
        } else if ctrl & 0x01 != 0 {
            // Color change
            pattern.add_command(COLOR_CHANGE, 0.0, 0.0);
        } else if ctrl & 0x04 != 0 {
            // Move/Jump
            pattern.add_stitch_absolute(JUMP, x, y);
        } else {
            // Uncaught control - break
            break;
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

/// Read unsigned 24-bit big-endian integer
fn read_u24_be(file: &mut impl Read) -> Result<u32> {
    let mut buf = [0u8; 3];
    file.read_exact(&mut buf)?;
    Ok(u32::from_be_bytes([0, buf[0], buf[1], buf[2]]))
}

/// Read signed 24-bit little-endian integer
fn read_i24_le(file: &mut impl Read) -> Result<i32> {
    let mut buf = [0u8; 3];
    file.read_exact(&mut buf)?;

    // Build 32-bit value from little-endian bytes
    let value = u32::from_le_bytes([buf[0], buf[1], buf[2], 0]);

    // If sign bit (bit 23) is set, extend to negative
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
    fn test_read_pcd_basic() {
        let mut pcd_data = vec![];

        // Version and hoop size
        pcd_data.extend_from_slice(&[0, 0]);

        // Color count: 1
        pcd_data.extend_from_slice(&[1, 0]); // LE

        // Color: RGB (255, 0, 0) = red, BE + padding
        pcd_data.extend_from_slice(&[0xFF, 0, 0, 0]);

        // Stitch count: 1
        pcd_data.extend_from_slice(&[1, 0]); // LE

        // Stitch: c0=0, x=100(LE), c1=0, y=200(LE), ctrl=0x00
        pcd_data.extend_from_slice(&[0]); // c0
        pcd_data.extend_from_slice(&[100, 0, 0]); // x (24-bit LE)
        pcd_data.extend_from_slice(&[0]); // c1
        pcd_data.extend_from_slice(&[200, 0, 0]); // y (24-bit LE)
        pcd_data.extend_from_slice(&[0x00]); // ctrl

        let mut cursor = Cursor::new(pcd_data);
        let mut pattern = EmbPattern::new();

        read(&mut cursor, &mut pattern).expect("Failed to read PCD");

        assert!(!pattern.stitches().is_empty());
        assert_eq!(pattern.threads().len(), 1);
    }

    #[test]
    fn test_pcd_multiple_colors() {
        let mut pcd_data = vec![];

        // Version and hoop size
        pcd_data.extend_from_slice(&[0, 0]);

        // Color count: 2
        pcd_data.extend_from_slice(&[2, 0]);

        // Colors
        pcd_data.extend_from_slice(&[0xFF, 0, 0, 0]); // Red
        pcd_data.extend_from_slice(&[0, 0xFF, 0, 0]); // Green

        // Stitch count: 1
        pcd_data.extend_from_slice(&[1, 0]);

        // Stitch with color change
        pcd_data.extend_from_slice(&[0, 50, 0, 0, 0, 50, 0, 0, 0x01]);

        let mut cursor = Cursor::new(pcd_data);
        let mut pattern = EmbPattern::new();

        read(&mut cursor, &mut pattern).expect("Failed to read PCD");

        assert_eq!(pattern.threads().len(), 2);
    }

    #[test]
    fn test_i24_le_signed() {
        // Test positive value
        let data = vec![0x34, 0x12, 0x00];
        let mut cursor = Cursor::new(data);
        let value = read_i24_le(&mut cursor).expect("Failed to read");
        assert_eq!(value, 0x1234);

        // Test negative value (-1)
        let data = vec![0xFF, 0xFF, 0xFF];
        let mut cursor = Cursor::new(data);
        let value = read_i24_le(&mut cursor).expect("Failed to read");
        assert_eq!(value, -1);
    }
}
