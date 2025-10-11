//! Mitsubishi MIT format reader
//!
//! MIT is Mitsubishi's embroidery format with byte-encoded stitches and
//! color changes for Mitsubishi industrial embroidery machines.

use crate::core::constants::*;
use crate::core::pattern::EmbPattern;
use crate::utils::error::Result;
use std::io::Read;

const MIT_SIZE_CONVERSION_RATIO: f64 = 2.0;

/// Read MIT (Mitsubishi) format
///
/// MIT format has 2-byte stitch records with packed coordinates and control bits.
pub fn read(file: &mut impl Read, pattern: &mut EmbPattern) -> Result<()> {
    let mut previous_ctrl = -1i32;
    let mut buffer = [0u8; 2];

    loop {
        // Try to read 2 bytes
        match file.read_exact(&mut buffer) {
            Ok(_) => {}
            Err(e) if e.kind() == std::io::ErrorKind::UnexpectedEof => break,
            Err(e) => return Err(e.into()),
        }

        // Extract 5-bit coordinates
        let x = (buffer[0] & 0x1F) as i32;
        let y = -((buffer[1] & 0x1F) as i32);

        // Apply size conversion
        let mut x_f = x as f64 * MIT_SIZE_CONVERSION_RATIO;
        let mut y_f = y as f64 * MIT_SIZE_CONVERSION_RATIO;

        // Check sign bits
        if buffer[0] & 0b10000000 != 0 {
            x_f = -x_f;
        }
        if buffer[1] & 0b10000000 != 0 {
            y_f = -y_f;
        }

        // Extract control bits
        let ctrl = ((buffer[0] & 0x60) >> 3) | ((buffer[1] & 0x60) >> 5);

        match ctrl {
            0b0111 => {
                // Stitch
                pattern.add_stitch_relative(x_f, y_f, STITCH);
            }
            0b1100 => {
                // Move/Jump
                pattern.add_stitch_relative(x_f, y_f, JUMP);
            }
            0b0100 | 0b0101 => {
                // Stitch
                pattern.add_stitch_relative(x_f, y_f, STITCH);
            }
            0b1000 => {
                if previous_ctrl == 0b0111 {
                    pattern.add_command(COLOR_CHANGE, 0.0, 0.0);
                }
            }
            0b0000 => {
                // End
                break;
            }
            _ => {
                // Default to stitch
                pattern.add_stitch_relative(x_f, y_f, STITCH);
            }
        }

        previous_ctrl = ctrl as i32;
    }

    pattern.add_command(END, 0.0, 0.0);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    #[test]
    fn test_read_mit_basic() {
        // Stitch at (10, -10): x=10/2=5 (0x05), y=10/2=5, ctrl=0b0111
        // byte[0] = (5 & 0x1F) | ((0b0111 << 3) & 0x60) = 5 | 0x38 = 0x3D
        // byte[1] = (5 & 0x1F) | ((0b0111 << 5) & 0x60) = 5 | 0x60 = 0x65
        let mit_data = vec![0x3D, 0x65];

        let mut cursor = Cursor::new(mit_data);
        let mut pattern = EmbPattern::new();

        read(&mut cursor, &mut pattern).expect("Failed to read MIT");

        assert!(!pattern.stitches().is_empty());
    }

    #[test]
    fn test_mit_end() {
        // End marker: ctrl=0b0000
        let mit_data = vec![0x00, 0x00];

        let mut cursor = Cursor::new(mit_data);
        let mut pattern = EmbPattern::new();

        read(&mut cursor, &mut pattern).expect("Failed to read MIT");

        // Should have END command
        assert!(!pattern.stitches().is_empty());
    }

    #[test]
    fn test_mit_color_change() {
        // First stitch with ctrl=0b0111
        let mut mit_data = vec![0x3D, 0x65];

        // Color change with ctrl=0b1000
        // byte[0] = (0 & 0x1F) | ((0b1000 << 3) & 0x60) = 0 | 0x40 = 0x40
        // byte[1] = (0 & 0x1F) | ((0b1000 << 5) & 0x60) = 0 | 0x00 = 0x00
        mit_data.extend_from_slice(&[0x40, 0x00]);

        // End
        mit_data.extend_from_slice(&[0x00, 0x00]);

        let mut cursor = Cursor::new(mit_data);
        let mut pattern = EmbPattern::new();

        read(&mut cursor, &mut pattern).expect("Failed to read MIT");

        assert!(!pattern.stitches().is_empty());
    }
}
