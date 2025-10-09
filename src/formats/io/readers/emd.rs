//! Elna EMD format reader
//!
//! EMD is Elna's embroidery design format with basic stitch encoding and
//! color information for Elna/Janome-compatible machines.

use crate::core::constants::*;
use crate::core::pattern::EmbPattern;
use anyhow::Result;
use std::io::{Read, Seek, SeekFrom};

/// Read EMD (Elna) format
pub fn read(file: &mut (impl Read + Seek), pattern: &mut EmbPattern) -> Result<()> {
    // Seek to stitch data at offset 0x30
    file.seek(SeekFrom::Start(0x30))?;

    read_emd_stitches(file, pattern)?;

    Ok(())
}

/// Read EMD stitches
fn read_emd_stitches(file: &mut impl Read, pattern: &mut EmbPattern) -> Result<()> {
    let mut buffer = [0u8; 2];

    loop {
        // Read 2 bytes
        match file.read_exact(&mut buffer) {
            Ok(_) => {}
            Err(e) if e.kind() == std::io::ErrorKind::UnexpectedEof => break,
            Err(e) => return Err(e.into()),
        }

        if buffer[0] != 0x80 {
            // Regular stitch
            let x = buffer[0] as i8 as f64;
            let y = -(buffer[1] as i8 as f64);
            pattern.add_stitch_relative(x, y, STITCH);
            continue;
        }

        // Control byte
        let control = buffer[1];

        match control {
            0x80 => {
                // Move/Jump
                match file.read_exact(&mut buffer) {
                    Ok(_) => {}
                    Err(_) => break,
                }
                let x = buffer[0] as i8 as f64;
                let y = -(buffer[1] as i8 as f64);
                pattern.add_stitch_relative(x, y, JUMP);
            }
            0x2A => {
                // Color change
                pattern.add_command(COLOR_CHANGE, 0.0, 0.0);
            }
            0x7D => {
                // Unknown - occurs at position 0, ignore
                continue;
            }
            0xAD => {
                // Trim
                pattern.add_command(TRIM, 0.0, 0.0);
            }
            0x90 => {
                // Trim - final command before returning to start
                pattern.add_command(TRIM, 0.0, 0.0);
            }
            0xFD => {
                // End
                break;
            }
            _ => {
                // Uncaught control
                break;
            }
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
    fn test_read_emd_basic() {
        // Create header (0x30 bytes)
        let mut emd_data = vec![0u8; 0x30];

        // Regular stitch: x=10, y=10 (will be negated to -10)
        emd_data.extend_from_slice(&[10, 10]);

        // End: 0x80, 0xFD
        emd_data.extend_from_slice(&[0x80, 0xFD]);

        let mut cursor = Cursor::new(emd_data);
        let mut pattern = EmbPattern::new();

        read(&mut cursor, &mut pattern).expect("Failed to read EMD");

        assert!(!pattern.stitches().is_empty());
    }

    #[test]
    fn test_emd_move() {
        let mut emd_data = vec![0u8; 0x30];

        // Move: 0x80, 0x80, x, y
        emd_data.extend_from_slice(&[0x80, 0x80, 5, 5]);

        // Stitch
        emd_data.extend_from_slice(&[10, 10]);

        // End
        emd_data.extend_from_slice(&[0x80, 0xFD]);

        let mut cursor = Cursor::new(emd_data);
        let mut pattern = EmbPattern::new();

        read(&mut cursor, &mut pattern).expect("Failed to read EMD");

        assert!(!pattern.stitches().is_empty());
    }

    #[test]
    fn test_emd_color_change() {
        let mut emd_data = vec![0u8; 0x30];

        // Stitch
        emd_data.extend_from_slice(&[5, 5]);

        // Color change: 0x80, 0x2A
        emd_data.extend_from_slice(&[0x80, 0x2A]);

        // Stitch
        emd_data.extend_from_slice(&[10, 10]);

        // End
        emd_data.extend_from_slice(&[0x80, 0xFD]);

        let mut cursor = Cursor::new(emd_data);
        let mut pattern = EmbPattern::new();

        read(&mut cursor, &mut pattern).expect("Failed to read EMD");

        assert!(!pattern.stitches().is_empty());
    }
}
