//! Compucon SPX format reader
//!
//! SPX is Compucon's embroidery format with coordinate-based stitches and
//! color changes for industrial embroidery applications.

use crate::core::constants::*;
use crate::core::pattern::EmbPattern;
use crate::utils::error::Result;
use std::io::{Read, Seek, SeekFrom};

/// Read SPX (Sperry) format
///
/// SPX format has a 0x11E (286) byte header followed by stitch data.
pub fn read(file: &mut (impl Read + Seek), pattern: &mut EmbPattern) -> Result<()> {
    // Skip 0x11E bytes of header
    file.seek(SeekFrom::Current(0x11E))?;

    let mut buffer = [0u8; 2];

    loop {
        // Read 2-byte marker
        match file.read_exact(&mut buffer) {
            Ok(_) => {}
            Err(e) if e.kind() == std::io::ErrorKind::UnexpectedEof => break,
            Err(e) => return Err(e.into()),
        }

        // Read dy (16-bit BE, signed)
        let dy = match read_i16_be(file) {
            Ok(v) => -v,
            Err(_) => break,
        };

        // Read dx (16-bit BE, signed)
        let dx = match read_i16_be(file) {
            Ok(v) => v,
            Err(_) => break,
        };

        // Read c (8-bit signed)
        let c = match read_i8(file) {
            Ok(v) => v,
            Err(_) => break,
        };

        // Adjust dy
        let dy = dy - c as i32;

        // Read another 2-byte marker
        match file.read_exact(&mut buffer) {
            Ok(_) => {}
            Err(_) => break,
        }

        // Add stitch
        pattern.add_stitch_relative(dx as f64, dy as f64, STITCH);
    }

    pattern.add_command(END, 0.0, 0.0);
    Ok(())
}

/// Read signed 8-bit integer
fn read_i8(file: &mut impl Read) -> Result<i8> {
    let mut buf = [0u8; 1];
    file.read_exact(&mut buf)?;
    Ok(buf[0] as i8)
}

/// Read signed 16-bit big-endian integer
fn read_i16_be(file: &mut impl Read) -> Result<i32> {
    let mut buf = [0u8; 2];
    file.read_exact(&mut buf)?;
    Ok(i16::from_be_bytes(buf) as i32)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    #[test]
    fn test_read_spx_basic() {
        // Create header
        let mut spx_data = vec![0u8; 0x11E];

        // Stitch record:
        // - 2-byte marker
        // - dy (16-bit BE): 10 -> negated to -10
        // - dx (16-bit BE): 20
        // - c (8-bit signed): 2 -> dy becomes -10 - 2 = -12
        // - 2-byte marker
        spx_data.extend_from_slice(&[0, 0]); // marker
        spx_data.extend_from_slice(&[0, 10]); // dy = 10
        spx_data.extend_from_slice(&[0, 20]); // dx = 20
        spx_data.extend_from_slice(&[2]); // c = 2
        spx_data.extend_from_slice(&[0, 0]); // marker

        let mut cursor = Cursor::new(spx_data);
        let mut pattern = EmbPattern::new();

        read(&mut cursor, &mut pattern).expect("Failed to read SPX");

        assert!(!pattern.stitches().is_empty());
    }

    #[test]
    fn test_spx_negative_coords() {
        let mut spx_data = vec![0u8; 0x11E];

        // Stitch with negative values
        // dy = -20 -> negated to 20, c=-3 -> final = 20 - (-3) = 23
        // dx = -10
        spx_data.extend_from_slice(&[0, 0]); // marker
        let dy_bytes = (-20i16).to_be_bytes();
        spx_data.extend_from_slice(&dy_bytes);
        let dx_bytes = (-10i16).to_be_bytes();
        spx_data.extend_from_slice(&dx_bytes);
        spx_data.extend_from_slice(&[(-3i8) as u8]); // c
        spx_data.extend_from_slice(&[0, 0]); // marker

        let mut cursor = Cursor::new(spx_data);
        let mut pattern = EmbPattern::new();

        read(&mut cursor, &mut pattern).expect("Failed to read SPX");

        assert!(!pattern.stitches().is_empty());
    }
}
