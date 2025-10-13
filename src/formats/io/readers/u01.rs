//! Barudan U01 format reader
//!
//! U01 format supports FAST/SLOW speed commands and explicit needle changes,
//! used by industrial Barudan embroidery machines with byte-encoded coordinates.
//!
//! ## Format Limitations
//! - Fixed header size: 256 bytes (0x100)
//! - Maximum 1,000,000 stitches per file
//! - 3-byte stitch encoding: control, dy, dx

/// U01 header size in bytes
const HEADER_SIZE: usize = 0x100;

/// Maximum allowed stitch count
const MAX_STITCHES: usize = 1_000_000;

use crate::core::constants::*;
use crate::core::pattern::EmbPattern;
use crate::utils::error::Result;
use std::io::Read;

/// Read U01 format embroidery file
pub fn read(file: &mut impl Read, pattern: &mut EmbPattern) -> Result<()> {
    // Skip the first 256 bytes (0x100) header
    let mut header = vec![0u8; HEADER_SIZE];
    file.read_exact(&mut header).map_err(|e| {
        if e.kind() == std::io::ErrorKind::UnexpectedEof {
            crate::utils::error::Error::Parse(format!(
                "U01 file too small: header must be {} bytes",
                HEADER_SIZE
            ))
        } else {
            crate::utils::error::Error::from(e)
        }
    })?;

    read_stitches(file, pattern)?;

    Ok(())
}

fn read_stitches(file: &mut impl Read, pattern: &mut EmbPattern) -> Result<()> {
    let mut stitch_count = 0;

    loop {
        let mut buf = [0u8; 3];
        if file.read_exact(&mut buf).is_err() {
            break; // End of file
        }

        // Check for excessive stitch count
        stitch_count += 1;
        if stitch_count > MAX_STITCHES {
            return Err(crate::utils::error::Error::Parse(format!(
                "U01 file exceeds maximum stitch count of {}",
                MAX_STITCHES
            )));
        }

        let ctrl = buf[0];
        let dy = -((buf[1] as i8) as f64); // Negative Y (cast to f64 first to avoid overflow)
        let mut dx = buf[2] as f64;

        // Check direction flags
        if (ctrl & 0x20) != 0 {
            dx = -dx;
        }
        let dy = if (ctrl & 0x40) != 0 { -dy } else { dy };

        let command = ctrl & 0b11111;

        match command {
            0x00 => {
                // Stitch
                pattern.add_stitch_relative(dx, dy, STITCH);
            }
            0x01 => {
                // Jump
                pattern.add_stitch_relative(dx, dy, JUMP);
            }
            0x02 => {
                // Fast
                pattern.add_stitch_relative(0.0, 0.0, FAST);
                if dx != 0.0 || dy != 0.0 {
                    pattern.add_stitch_relative(dx, dy, STITCH);
                }
            }
            0x03 => {
                // Fast, Jump
                pattern.add_stitch_relative(0.0, 0.0, FAST);
                if dx != 0.0 || dy != 0.0 {
                    pattern.add_stitch_relative(dx, dy, JUMP);
                }
            }
            0x04 => {
                // Slow
                pattern.add_stitch_relative(0.0, 0.0, SLOW);
                if dx != 0.0 || dy != 0.0 {
                    pattern.add_stitch_relative(dx, dy, STITCH);
                }
            }
            0x05 => {
                // Slow, Jump
                pattern.add_stitch_relative(0.0, 0.0, SLOW);
                if dx != 0.0 || dy != 0.0 {
                    pattern.add_stitch_relative(dx, dy, JUMP);
                }
            }
            0x06 => {
                // T1 Top Thread Trimming
                pattern.add_stitch_relative(0.0, 0.0, TRIM);
                if dx != 0.0 || dy != 0.0 {
                    pattern.add_stitch_relative(dx, dy, JUMP);
                }
            }
            0x07 => {
                // T2 Bobbin Threading
                pattern.add_stitch_relative(0.0, 0.0, TRIM);
                if dx != 0.0 || dy != 0.0 {
                    pattern.add_stitch_relative(dx, dy, JUMP);
                }
            }
            0x08 => {
                // C00 Stop
                pattern.add_stitch_relative(0.0, 0.0, STOP);
                if dx != 0.0 || dy != 0.0 {
                    pattern.add_stitch_relative(dx, dy, JUMP);
                }
            }
            0x09..=0x17 => {
                // C01 - C15 (Needle changes)
                let needle = command - 0x08;
                pattern.add_stitch_relative(
                    0.0,
                    0.0,
                    crate::utils::functions::encode_thread_change(
                        NEEDLE_SET,
                        None,
                        Some(needle),
                        None,
                    ),
                );
                if dx != 0.0 || dy != 0.0 {
                    pattern.add_stitch_relative(dx, dy, JUMP);
                }
            }
            0x18 => {
                // End command
                break;
            }
            _ if ctrl == 0x2B => {
                // Rare postfix data from machine
                break;
            }
            _ => {
                // Unknown command, stop reading
                break;
            }
        }
    }

    pattern.add_stitch_relative(0.0, 0.0, END);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    #[test]
    fn test_read_u01_basic() {
        let mut data = vec![0u8; 0x100]; // Header

        // Add some test stitches
        // Stitch (0, 0)
        data.extend_from_slice(&[0x80, 0, 0]);
        // Stitch (10, 10)
        data.extend_from_slice(&[0x80, 10, 10]);
        // End
        data.extend_from_slice(&[0x98, 0, 0]);

        let mut cursor = Cursor::new(data);
        let mut pattern = EmbPattern::new();

        read(&mut cursor, &mut pattern).unwrap();

        assert!(!pattern.stitches().is_empty());
    }

    #[test]
    fn test_read_u01_with_trim() {
        let mut data = vec![0u8; 0x100]; // Header

        // Stitch
        data.extend_from_slice(&[0x80, 0, 10]);
        // Trim
        data.extend_from_slice(&[0x86, 0, 0]);
        // End
        data.extend_from_slice(&[0x98, 0, 0]);

        let mut cursor = Cursor::new(data);
        let mut pattern = EmbPattern::new();

        read(&mut cursor, &mut pattern).unwrap();

        // Should have stitches including trim command
        assert!(pattern.stitches().len() > 1);
    }

    #[test]
    fn test_read_u01_needle_change() {
        let mut data = vec![0u8; 0x100]; // Header

        // Stitch
        data.extend_from_slice(&[0x80, 0, 10]);
        // Needle change to needle 2 (C01 = 0x09)
        data.extend_from_slice(&[0x89, 0, 0]);
        // Stitch
        data.extend_from_slice(&[0x80, 0, 10]);
        // End
        data.extend_from_slice(&[0x98, 0, 0]);

        let mut cursor = Cursor::new(data);
        let mut pattern = EmbPattern::new();

        read(&mut cursor, &mut pattern).unwrap();

        assert!(pattern.stitches().len() > 2);
    }
}
