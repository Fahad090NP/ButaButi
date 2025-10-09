//! Bits & Volts BRO format reader
//!
//! BRO is a simple embroidery format with basic stitch encoding and color changes,
//! used by Bits & Volts embroidery software.

use crate::core::constants::*;
use crate::core::pattern::EmbPattern;
use crate::utils::functions::encode_thread_change;
use anyhow::Result;
use std::io::{Read, Seek, SeekFrom};

/// Read BRO (Bits and Volts) format
pub fn read(file: &mut (impl Read + Seek), pattern: &mut EmbPattern) -> Result<()> {
    // Seek to stitch data at offset 0x100
    file.seek(SeekFrom::Start(0x100))?;

    read_bro_stitches(file, pattern)?;

    Ok(())
}

/// Read BRO stitches
fn read_bro_stitches(file: &mut impl Read, pattern: &mut EmbPattern) -> Result<()> {
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

        // Read control byte
        let control = match read_u8(file) {
            Ok(c) => c,
            Err(_) => break,
        };

        match control {
            0x00 => {
                // Continue (no-op)
                continue;
            }
            0x02 | 0xE0 => {
                // End
                break;
            }
            0x7E | 0x03 => {
                // Move/Jump with 16-bit coordinates
                let x = match read_i16_le(file) {
                    Ok(v) => v,
                    Err(_) => break,
                };
                let y = match read_i16_le(file) {
                    Ok(v) => v,
                    Err(_) => break,
                };
                pattern.add_stitch_relative(x as f64, -y as f64, JUMP);
            }
            0xE1..=0xEF => {
                // Needle change
                let needle = control - 0xE0;
                let cmd = encode_thread_change(NEEDLE_SET, None, Some(needle), None);
                pattern.add_command(cmd, 0.0, 0.0);

                // Read move coordinates
                let x = match read_i16_le(file) {
                    Ok(v) => v,
                    Err(_) => break,
                };
                let y = match read_i16_le(file) {
                    Ok(v) => v,
                    Err(_) => break,
                };
                pattern.add_stitch_relative(x as f64, -y as f64, JUMP);
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

/// Read unsigned 8-bit integer
fn read_u8(file: &mut impl Read) -> Result<u8> {
    let mut buf = [0u8; 1];
    file.read_exact(&mut buf)?;
    Ok(buf[0])
}

/// Read signed 16-bit little-endian integer
fn read_i16_le(file: &mut impl Read) -> Result<i16> {
    let mut buf = [0u8; 2];
    file.read_exact(&mut buf)?;
    Ok(i16::from_le_bytes(buf))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    #[test]
    fn test_read_bro_basic() {
        // Create header (0x100 bytes)
        let mut bro_data = vec![0u8; 0x100];

        // Regular stitch: x=10, y=10 (will be negated to -10)
        bro_data.extend_from_slice(&[10, 10]);

        // End: 0x80, 0x02
        bro_data.extend_from_slice(&[0x80, 0x02]);

        let mut cursor = Cursor::new(bro_data);
        let mut pattern = EmbPattern::new();

        read(&mut cursor, &mut pattern).expect("Failed to read BRO");

        assert!(!pattern.stitches().is_empty());
    }

    #[test]
    fn test_bro_move() {
        let mut bro_data = vec![0u8; 0x100];

        // Move: 0x80, 0x03, x (16-bit LE), y (16-bit LE)
        bro_data.extend_from_slice(&[0x80, 0x03]);
        bro_data.extend_from_slice(&100i16.to_le_bytes());
        bro_data.extend_from_slice(&200i16.to_le_bytes());

        // Stitch
        bro_data.extend_from_slice(&[5, 5]);

        // End
        bro_data.extend_from_slice(&[0x80, 0x02]);

        let mut cursor = Cursor::new(bro_data);
        let mut pattern = EmbPattern::new();

        read(&mut cursor, &mut pattern).expect("Failed to read BRO");

        assert!(!pattern.stitches().is_empty());
    }

    #[test]
    fn test_bro_needle_change() {
        let mut bro_data = vec![0u8; 0x100];

        // Stitch
        bro_data.extend_from_slice(&[5, 5]);

        // Needle change to needle 1: 0x80, 0xE1, x, y
        bro_data.extend_from_slice(&[0x80, 0xE1]);
        bro_data.extend_from_slice(&10i16.to_le_bytes());
        bro_data.extend_from_slice(&10i16.to_le_bytes());

        // Stitch
        bro_data.extend_from_slice(&[10, 10]);

        // End
        bro_data.extend_from_slice(&[0x80, 0x02]);

        let mut cursor = Cursor::new(bro_data);
        let mut pattern = EmbPattern::new();

        read(&mut cursor, &mut pattern).expect("Failed to read BRO");

        assert!(!pattern.stitches().is_empty());
    }
}
