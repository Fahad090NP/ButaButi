//! Pfaff KSM format reader
//!
//! KSM is a Pfaff embroidery format with coordinate-based stitch encoding
//! and color change commands for Pfaff sewing machines.

use crate::core::constants::*;
use crate::core::pattern::EmbPattern;
use crate::utils::error::Result;
use crate::utils::functions::encode_thread_change;
use std::io::{Read, Seek, SeekFrom};

/// Read KSM (Pfaff) format
pub fn read(file: &mut (impl Read + Seek), pattern: &mut EmbPattern) -> Result<()> {
    // Seek to offset 0x200 (512 bytes)
    file.seek(SeekFrom::Start(0x200))?;

    read_ksm_stitches(file, pattern)?;

    Ok(())
}

/// Read KSM stitches
fn read_ksm_stitches(file: &mut impl Read, pattern: &mut EmbPattern) -> Result<()> {
    let mut buffer = [0u8; 3];
    let mut trimmed = false;
    let mut stitched_yet = false;

    loop {
        // Read 3 bytes
        match file.read_exact(&mut buffer) {
            Ok(_) => {}
            Err(e) if e.kind() == std::io::ErrorKind::UnexpectedEof => break,
            Err(e) => return Err(e.into()),
        }

        let mut y = -(buffer[0] as i8 as i32);
        let mut x = buffer[1] as i8 as i32;
        let mut ctrl = buffer[2];

        // Check sign bits
        if ctrl & 0x40 != 0 {
            x = -x;
        }
        if ctrl & 0x20 != 0 {
            y = -y;
        }

        // Clear sign bits
        ctrl &= 0b11111;

        // Process movement
        if x != 0 || y != 0 {
            if trimmed {
                pattern.add_stitch_relative(x as f64, y as f64, JUMP);
            } else {
                pattern.add_stitch_relative(x as f64, y as f64, STITCH);
                stitched_yet = true;
            }
        }

        if ctrl == 0x00 {
            continue;
        }

        // Process control commands
        if ctrl == 0x07 || ctrl == 0x13 || ctrl == 0x1D {
            if stitched_yet {
                pattern.add_command(TRIM, 0.0, 0.0);
            }
            trimmed = true;
        } else if (0x17..=0x19).contains(&ctrl) {
            // Start sewing again
            trimmed = false;
        } else if (0x0B..=0x12).contains(&ctrl) {
            // Needle change
            let needle = ctrl - 0x0A;
            let command = encode_thread_change(NEEDLE_SET, None, Some(needle), None);
            pattern.add_command(command, 0.0, 0.0);
            trimmed = true;
        } else if ctrl == 0x05 {
            // Stop
            pattern.add_command(STOP, 0.0, 0.0);
        } else if ctrl == 0x1B {
            // Called before end command
            trimmed = false;
        } else if ctrl == 0x08 {
            // End command
            break;
        } else {
            // Uncaught control - break
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
    fn test_read_ksm_basic() {
        // Create 512-byte header
        let mut ksm_data = vec![0u8; 0x200];

        // Regular stitch: y=10 (negated to -10), x=10, ctrl=0
        ksm_data.extend_from_slice(&[10, 10, 0]);

        // End: ctrl=0x08
        ksm_data.extend_from_slice(&[0, 0, 0x08]);

        let mut cursor = Cursor::new(ksm_data);
        let mut pattern = EmbPattern::new();

        read(&mut cursor, &mut pattern).expect("Failed to read KSM");

        assert!(!pattern.stitches().is_empty());
    }

    #[test]
    fn test_ksm_needle_change() {
        let mut ksm_data = vec![0u8; 0x200];

        // Stitch
        ksm_data.extend_from_slice(&[5, 5, 0]);

        // Needle change to needle 1 (ctrl = 0x0B = 0x0A + 1)
        ksm_data.extend_from_slice(&[0, 0, 0x0B]);

        // Another stitch
        ksm_data.extend_from_slice(&[10, 10, 0]);

        // End
        ksm_data.extend_from_slice(&[0, 0, 0x08]);

        let mut cursor = Cursor::new(ksm_data);
        let mut pattern = EmbPattern::new();

        read(&mut cursor, &mut pattern).expect("Failed to read KSM");

        assert!(!pattern.stitches().is_empty());
    }

    #[test]
    fn test_ksm_trim() {
        let mut ksm_data = vec![0u8; 0x200];

        // Stitch
        ksm_data.extend_from_slice(&[5, 5, 0]);

        // Trim (ctrl=0x07)
        ksm_data.extend_from_slice(&[0, 0, 0x07]);

        // Move after trim (should be JUMP)
        ksm_data.extend_from_slice(&[10, 10, 0]);

        // Start sewing again (ctrl=0x17)
        ksm_data.extend_from_slice(&[0, 0, 0x17]);

        // Stitch
        ksm_data.extend_from_slice(&[5, 5, 0]);

        // End
        ksm_data.extend_from_slice(&[0, 0, 0x08]);

        let mut cursor = Cursor::new(ksm_data);
        let mut pattern = EmbPattern::new();

        read(&mut cursor, &mut pattern).expect("Failed to read KSM");

        assert!(!pattern.stitches().is_empty());
    }
}
