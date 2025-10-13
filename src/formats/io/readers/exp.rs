//! Melco EXP format reader
//!
//! EXP is a widely-used format with 2-byte stitch records using bit-encoded coordinates.
//! Supports stitches, jumps, color changes, and standard embroidery commands.
//!
//! ## Format Limitations
//! - Maximum 1,000,000 stitches per file
//! - 2-byte stitch encoding with control byte (0x80) for commands
//! - Coordinate range: -128 to +127 per stitch

/// Maximum allowed stitch count
const MAX_STITCHES: usize = 1_000_000;

use crate::core::pattern::EmbPattern;
use crate::utils::error::{Error, Result};
use std::io::Read;

/// Read EXP stitches
fn read_stitches<R: Read>(reader: &mut R, pattern: &mut EmbPattern) -> Result<()> {
    let mut buffer = [0u8; 2];
    let mut stitch_count = 0;

    loop {
        match reader.read_exact(&mut buffer) {
            Ok(_) => {}
            Err(e) if e.kind() == std::io::ErrorKind::UnexpectedEof => break,
            Err(e) => return Err(Error::from(e)),
        }

        // Check for excessive stitch count
        stitch_count += 1;
        if stitch_count > MAX_STITCHES {
            return Err(Error::Parse(format!(
                "EXP file exceeds maximum stitch count of {}",
                MAX_STITCHES
            )));
        }

        if buffer[0] != 0x80 {
            // Normal stitch
            let x = buffer[0] as i8 as f64;
            let y = -(buffer[1] as i8 as f64);
            pattern.stitch(x, y);
            continue;
        }

        let control = buffer[1];

        // Read next 2 bytes for coordinates
        match reader.read_exact(&mut buffer) {
            Ok(_) => {}
            Err(e) if e.kind() == std::io::ErrorKind::UnexpectedEof => break,
            Err(e) => return Err(Error::from(e)),
        }

        let x = buffer[0] as i8 as f64;
        let y = -(buffer[1] as i8 as f64);

        match control {
            0x80 => {
                // Trim
                pattern.trim();
            }
            0x02 => {
                // Stitch (shouldn't exist but handle it)
                pattern.stitch(x, y);
            }
            0x04 => {
                // Jump
                pattern.jump(x, y);
            }
            0x01 => {
                // Color change
                pattern.color_change(0.0, 0.0);
                if x != 0.0 || y != 0.0 {
                    pattern.jump(x, y);
                }
            }
            _ => {
                // Uncaught control - break
                break;
            }
        }
    }

    pattern.end();
    Ok(())
}

/// Read an EXP file
pub fn read<R: Read>(reader: &mut R) -> Result<EmbPattern> {
    let mut pattern = EmbPattern::new();
    read_stitches(reader, &mut pattern)?;
    Ok(pattern)
}

/// Read an EXP file from path
pub fn read_file(path: &str) -> Result<EmbPattern> {
    let file = std::fs::File::open(path)?;
    let mut reader = std::io::BufReader::new(file);
    read(&mut reader)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_exp_basic() {
        // Test reading simple EXP data
        let data = [
            0x10, 0x20, // Stitch (16, -32)
            0x05, 0x0A, // Stitch (5, -10)
            0x80, 0x80, // Trim command
            0x00, 0x00, // Coordinates
        ];

        let result = read(&mut &data[..]);
        assert!(result.is_ok());
        let pattern = result.unwrap();
        assert!(!pattern.stitches().is_empty());
    }
}
