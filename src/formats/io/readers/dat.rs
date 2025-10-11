//! Barudan DAT format reader
//!
//! DAT is Barudan's format with an extended command set including speed controls,
//! needle changes, and industrial embroidery machine features.

use crate::core::constants::*;
use crate::core::pattern::EmbPattern;
use crate::utils::error::Result;
use crate::utils::functions::encode_thread_change;
use std::io::{Read, Seek, SeekFrom};

/// Read DAT (Barudan/Sunstar) format
///
/// DAT format has two variants: Barudan and Sunstar.
/// This reader tries Barudan first, then falls back to Sunstar.
pub fn read(file: &mut (impl Read + Seek), pattern: &mut EmbPattern) -> Result<()> {
    // Try Barudan format first
    let start_pos = file.stream_position()?;
    let mut temp_pattern = EmbPattern::new();

    if read_barudan_dat(file, &mut temp_pattern)? {
        // Copy to output pattern
        for thread in temp_pattern.threads() {
            pattern.add_thread(thread.clone());
        }
        for stitch in temp_pattern.stitches() {
            pattern.add_stitch_absolute(stitch.command, stitch.x, stitch.y);
        }
        for (key, value) in temp_pattern.metadata() {
            pattern.add_metadata(key, value);
        }
        return Ok(());
    }

    // Reset and try Sunstar format
    file.seek(SeekFrom::Start(start_pos))?;
    read_sunstar_dat(file, pattern)?;

    Ok(())
}

/// Read Barudan DAT format
fn read_barudan_dat(file: &mut impl Read, pattern: &mut EmbPattern) -> Result<bool> {
    let mut buffer = [0u8; 3];

    loop {
        // Read 3 bytes
        match file.read_exact(&mut buffer) {
            Ok(_) => {}
            Err(e) if e.kind() == std::io::ErrorKind::UnexpectedEof => break,
            Err(e) => return Err(e.into()),
        }

        let ctrl = buffer[0];
        let mut dy = -(buffer[1] as i8 as i32);
        let mut dx = buffer[2] as i8 as i32;

        // Check if this is valid Barudan data
        if ctrl & 0x80 == 0 {
            // This bit should always be set in Barudan format
            return Ok(false);
        }

        // Check sign bits
        if ctrl & 0x20 != 0 {
            dx = -dx;
        }
        if ctrl & 0x40 != 0 {
            dy = -dy;
        }

        let command = ctrl & 0b11111;

        match command {
            0x00 => {
                // Stitch
                pattern.add_stitch_relative(dx as f64, dy as f64, STITCH);
            }
            0x01 => {
                // Jump
                pattern.add_stitch_relative(dx as f64, dy as f64, JUMP);
            }
            0x02 => {
                // Fast
                // Note: FAST constant not in our constants, treating as stitch
                if dx != 0 || dy != 0 {
                    pattern.add_stitch_relative(dx as f64, dy as f64, STITCH);
                }
            }
            0x03 => {
                // Fast + Jump
                if dx != 0 || dy != 0 {
                    pattern.add_stitch_relative(dx as f64, dy as f64, JUMP);
                }
            }
            0x04 => {
                // Slow
                if dx != 0 || dy != 0 {
                    pattern.add_stitch_relative(dx as f64, dy as f64, STITCH);
                }
            }
            0x05 => {
                // Slow + Jump
                if dx != 0 || dy != 0 {
                    pattern.add_stitch_relative(dx as f64, dy as f64, JUMP);
                }
            }
            0x06 | 0x07 => {
                // T1/T2 Trim
                pattern.add_command(TRIM, 0.0, 0.0);
                if dx != 0 || dy != 0 {
                    pattern.add_stitch_relative(dx as f64, dy as f64, JUMP);
                }
            }
            0x08 => {
                // Stop
                pattern.add_command(STOP, 0.0, 0.0);
                if dx != 0 || dy != 0 {
                    pattern.add_stitch_relative(dx as f64, dy as f64, JUMP);
                }
            }
            0x09..=0x17 => {
                // Needle change (C01-C14)
                let needle = command - 0x08;
                let cmd = encode_thread_change(NEEDLE_SET, None, Some(needle), None);
                pattern.add_command(cmd, 0.0, 0.0);
                if dx != 0 || dy != 0 {
                    pattern.add_stitch_relative(dx as f64, dy as f64, JUMP);
                }
            }
            0x18 => {
                // End
                break;
            }
            _ => {
                if ctrl == 0x2B {
                    // Rare postfix data - stop reading
                    break;
                }
                // Uncaught command
                break;
            }
        }
    }

    pattern.add_command(END, 0.0, 0.0);
    Ok(true)
}

/// Read Sunstar DAT format
fn read_sunstar_dat(file: &mut (impl Read + Seek), pattern: &mut EmbPattern) -> Result<()> {
    // Seek to stitch data at 0x100
    file.seek(SeekFrom::Start(0x100))?;

    read_sunstar_dat_stitches(file, pattern)?;

    Ok(())
}

/// Read Sunstar DAT stitches
fn read_sunstar_dat_stitches(file: &mut impl Read, pattern: &mut EmbPattern) -> Result<()> {
    let mut buffer = [0u8; 3];

    loop {
        // Read 3 bytes
        match file.read_exact(&mut buffer) {
            Ok(_) => {}
            Err(e) if e.kind() == std::io::ErrorKind::UnexpectedEof => break,
            Err(e) => return Err(e.into()),
        }

        let mut x = (buffer[0] & 0x7F) as i32;
        let mut y = (buffer[1] & 0x7F) as i32;

        if buffer[0] & 0x80 != 0 {
            x = -x;
        }
        if buffer[1] & 0x80 != 0 {
            y = -y;
        }

        y = -y;

        let ctrl = buffer[2];

        match ctrl {
            0x07 => {
                // Stitch
                pattern.add_stitch_relative(x as f64, y as f64, STITCH);
            }
            0x04 => {
                // Move/Jump
                pattern.add_stitch_relative(x as f64, y as f64, JUMP);
            }
            0x80 => {
                // Trim
                pattern.add_command(TRIM, 0.0, 0.0);
                if x != 0 || y != 0 {
                    pattern.add_stitch_relative(x as f64, y as f64, JUMP);
                }
            }
            0x87 => {
                // Color change
                pattern.add_command(COLOR_CHANGE, 0.0, 0.0);
                if x != 0 || y != 0 {
                    pattern.add_stitch_relative(x as f64, y as f64, JUMP);
                }
            }
            0x84 => {
                // Initialized info - treat as stitch
                pattern.add_stitch_relative(x as f64, y as f64, STITCH);
            }
            0x00 => {
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
    fn test_read_barudan_dat() {
        let mut dat_data = vec![];

        // Barudan stitch: ctrl has 0x80 bit set, dy=10, dx=10
        dat_data.extend_from_slice(&[0x80, 10, 10]); // ctrl=0x80 (0x80 | 0x00 command)

        // End: ctrl=0x80 | 0x18 = 0x98
        dat_data.extend_from_slice(&[0x98, 0, 0]);

        let mut cursor = Cursor::new(dat_data);
        let mut pattern = EmbPattern::new();

        read(&mut cursor, &mut pattern).expect("Failed to read DAT");

        assert!(!pattern.stitches().is_empty());
    }

    #[test]
    fn test_read_sunstar_dat() {
        let mut dat_data = vec![0u8; 0x100];

        // Sunstar stitch at offset 0x100: x=10, y=10, ctrl=0x07
        dat_data.extend_from_slice(&[10, 10, 0x07]);

        // End: ctrl=0x00
        dat_data.extend_from_slice(&[0, 0, 0x00]);

        let mut cursor = Cursor::new(dat_data);
        let mut pattern = EmbPattern::new();

        read(&mut cursor, &mut pattern).expect("Failed to read DAT");

        assert!(!pattern.stitches().is_empty());
    }

    #[test]
    fn test_barudan_needle_change() {
        let mut dat_data = vec![];

        // Stitch
        dat_data.extend_from_slice(&[0x80, 5, 5]);

        // Needle change to needle 1 (command=0x09)
        dat_data.extend_from_slice(&[0x80 | 0x09, 0, 0]);

        // End
        dat_data.extend_from_slice(&[0x98, 0, 0]);

        let mut cursor = Cursor::new(dat_data);
        let mut pattern = EmbPattern::new();

        read(&mut cursor, &mut pattern).expect("Failed to read DAT");

        assert!(!pattern.stitches().is_empty());
    }
}
