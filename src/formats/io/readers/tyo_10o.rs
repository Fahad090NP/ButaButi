//! Toyota 10o format reader
//!
//! Toyota 10o format uses coordinate-based stitches and color changes,
//! designed for Toyota embroidery machines with 10-needle capability.

use crate::core::constants::*;
use crate::core::pattern::EmbPattern;
use crate::utils::error::Result;
use std::io::Read;

/// Read 10O (Toyota) format
pub fn read(file: &mut impl Read, pattern: &mut EmbPattern) -> Result<()> {
    read_10o_stitches(file, pattern)?;
    Ok(())
}

/// Read 10O stitches
fn read_10o_stitches(file: &mut impl Read, pattern: &mut EmbPattern) -> Result<()> {
    let mut buffer = [0u8; 3];

    loop {
        // Read 3 bytes
        match file.read_exact(&mut buffer) {
            Ok(_) => {}
            Err(e) if e.kind() == std::io::ErrorKind::UnexpectedEof => break,
            Err(e) => return Err(e.into()),
        }

        let ctrl = buffer[0];
        let mut y = -(buffer[1] as i8 as i32);
        let mut x = buffer[2] as i8 as i32;

        // Check sign bits
        if ctrl & 0x20 != 0 {
            x = -x;
        }
        if ctrl & 0x40 != 0 {
            y = -y;
        }

        let command = ctrl & 0b11111;

        match command {
            0x00 => {
                // Stitch
                pattern.add_stitch_relative(x as f64, y as f64, STITCH);
            }
            0x10 => {
                // Move/Jump
                pattern.add_stitch_relative(x as f64, y as f64, JUMP);
            }
            _ => {
                // Check specific control codes
                match ctrl {
                    0x8A => {
                        // Start - ignore
                        continue;
                    }
                    0x85 => {
                        // Color change
                        pattern.add_command(COLOR_CHANGE, 0.0, 0.0);
                    }
                    0x82 => {
                        // Stop
                        pattern.add_command(STOP, 0.0, 0.0);
                    }
                    0x81 => {
                        // Trim
                        pattern.add_command(TRIM, 0.0, 0.0);
                    }
                    0x87 => {
                        // End
                        break;
                    }
                    _ => {
                        // Unknown control
                        break;
                    }
                }
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
    fn test_read_10o_basic() {
        let mut data = vec![];

        // Stitch: ctrl=0x00, y=10, x=10
        data.extend_from_slice(&[0x00, 10, 10]);

        // End: ctrl=0x87
        data.extend_from_slice(&[0x87, 0, 0]);

        let mut cursor = Cursor::new(data);
        let mut pattern = EmbPattern::new();

        read(&mut cursor, &mut pattern).expect("Failed to read 10O");

        assert!(!pattern.stitches().is_empty());
    }

    #[test]
    fn test_10o_color_change() {
        let mut data = vec![];

        // Stitch
        data.extend_from_slice(&[0x00, 5, 5]);

        // Color change: ctrl=0x85
        data.extend_from_slice(&[0x85, 0, 0]);

        // Stitch
        data.extend_from_slice(&[0x00, 10, 10]);

        // End
        data.extend_from_slice(&[0x87, 0, 0]);

        let mut cursor = Cursor::new(data);
        let mut pattern = EmbPattern::new();

        read(&mut cursor, &mut pattern).expect("Failed to read 10O");

        assert!(!pattern.stitches().is_empty());
    }

    #[test]
    fn test_10o_negative_coords() {
        let mut data = vec![];

        // Stitch with sign bits: ctrl=0x60 (0x20 | 0x40)
        data.extend_from_slice(&[0x60, 10, 10]);

        // End
        data.extend_from_slice(&[0x87, 0, 0]);

        let mut cursor = Cursor::new(data);
        let mut pattern = EmbPattern::new();

        read(&mut cursor, &mut pattern).expect("Failed to read 10O");

        assert!(!pattern.stitches().is_empty());
    }
}
