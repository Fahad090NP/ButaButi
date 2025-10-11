//! Data Stitch STC format reader
//!
//! STC is Data Stitch's embroidery format with simple stitch encoding and
//! color change commands for basic embroidery designs.

use crate::core::constants::*;
use crate::core::pattern::EmbPattern;
use crate::utils::error::Result;
use crate::utils::functions::encode_thread_change;
use std::io::{Read, Seek, SeekFrom};

/// Read STC (Gunold) format
///
/// STC format has a 40-byte (0x28) header followed by stitch data.
pub fn read(file: &mut (impl Read + Seek), pattern: &mut EmbPattern) -> Result<()> {
    // Skip 40-byte header
    file.seek(SeekFrom::Current(0x28))?;

    read_stc_stitches(file, pattern)?;

    Ok(())
}

/// Read STC stitch encoding
fn read_stc_stitches(file: &mut impl Read, pattern: &mut EmbPattern) -> Result<()> {
    let mut buffer = [0u8; 3];

    loop {
        // Try to read 3 bytes
        match file.read_exact(&mut buffer) {
            Ok(_) => {}
            Err(e) if e.kind() == std::io::ErrorKind::UnexpectedEof => break,
            Err(e) => return Err(e.into()),
        }

        let x = buffer[0] as i8 as i32;
        let y = -(buffer[1] as i8) as i32;
        let ctrl = buffer[2];

        if ctrl == 0x01 {
            // Stitch
            pattern.add_stitch_relative(x as f64, y as f64, STITCH);
        } else if ctrl == 0x00 {
            // Move/Jump
            pattern.add_stitch_relative(x as f64, y as f64, JUMP);
        } else if ctrl == 25 {
            // End
            break;
        } else {
            // Needle change (ctrl - 2 = needle number)
            let needle = ctrl.saturating_sub(2);
            let command = encode_thread_change(NEEDLE_SET, None, Some(needle), None);
            pattern.add_command(command, 0.0, 0.0);
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
    fn test_read_stc_basic() {
        // Create 40-byte header + stitch data
        let mut stc_data = vec![0u8; 0x28];

        // Add STC-encoded stitches
        // Stitch at (10, -10): x=10, y=10 (negated as i8), ctrl=0x01
        stc_data.extend_from_slice(&[10, 10, 0x01]);

        // Jump at (5, -5)
        stc_data.extend_from_slice(&[5, 5, 0x00]);

        // Stitch at (-3, 7)
        stc_data.extend_from_slice(&[(-3i8) as u8, (-7i8) as u8, 0x01]);

        // End
        stc_data.extend_from_slice(&[0, 0, 25]);

        let mut cursor = Cursor::new(stc_data);
        let mut pattern = EmbPattern::new();

        read(&mut cursor, &mut pattern).expect("Failed to read STC");

        assert!(!pattern.stitches().is_empty());
    }

    #[test]
    fn test_stc_needle_change() {
        let mut stc_data = vec![0u8; 0x28];

        // Needle change to needle 0 (ctrl=2)
        stc_data.extend_from_slice(&[0, 0, 2]);

        // Needle change to needle 5 (ctrl=7)
        stc_data.extend_from_slice(&[0, 0, 7]);

        // Stitch
        stc_data.extend_from_slice(&[10, 10, 0x01]);

        // End
        stc_data.extend_from_slice(&[0, 0, 25]);

        let mut cursor = Cursor::new(stc_data);
        let mut pattern = EmbPattern::new();

        read(&mut cursor, &mut pattern).expect("Failed to read STC");

        assert!(!pattern.stitches().is_empty());
    }
}
