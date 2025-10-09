//! Janome JPX format reader
//!
//! JPX is a Janome format variant of JEF with similar structure but different
//! header signatures and threading information encoding.

use crate::core::constants::*;
use crate::core::pattern::EmbPattern;
use crate::core::thread::EmbThread;
use anyhow::Result;
use std::io::{Read, Seek, SeekFrom};

/// Read JPX (Janome) format
pub fn read(file: &mut (impl Read + Seek), pattern: &mut EmbPattern) -> Result<()> {
    let stitch_start_position = read_u32_le(file)?;

    // Skip 0x1C bytes
    file.seek(SeekFrom::Current(0x1C))?;

    let colors = read_u32_le(file)?;

    // Skip 0x18 bytes
    file.seek(SeekFrom::Current(0x18))?;

    // Read thread color indices
    for _ in 0..colors {
        match read_u32_le(file) {
            Ok(color_index) => {
                // Use color index to generate a deterministic but varied color
                let color = 0x100000 * (color_index % 256);
                let thread =
                    EmbThread::new(color).with_description(format!("JPX index {}", color_index));
                pattern.add_thread(thread);
            }
            Err(_) => break,
        }
    }

    // Seek to stitch data
    file.seek(SeekFrom::Start(stitch_start_position as u64))?;

    read_jpx_stitches(file, pattern)?;

    Ok(())
}

/// Read JPX stitches
fn read_jpx_stitches(file: &mut impl Read, pattern: &mut EmbPattern) -> Result<()> {
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
        let ctrl = buffer[1];

        // Read next 2 bytes for coordinates
        match file.read_exact(&mut buffer) {
            Ok(_) => {}
            Err(_) => break,
        }

        let x = buffer[0] as i8 as f64;
        let y = -(buffer[1] as i8 as f64);

        if ctrl == 0x02 {
            // Move/Jump
            pattern.add_stitch_relative(x, y, JUMP);
        } else if ctrl == 0x01 {
            // Color change
            pattern.add_command(COLOR_CHANGE, 0.0, 0.0);
            if x != 0.0 || y != 0.0 {
                pattern.add_stitch_relative(x, y, JUMP);
            }
        } else if ctrl == 0x10 {
            // End
            break;
        } else {
            // Uncaught control - break
            break;
        }
    }

    pattern.add_command(END, 0.0, 0.0);
    Ok(())
}

/// Read unsigned 32-bit little-endian integer
fn read_u32_le(file: &mut impl Read) -> Result<u32> {
    let mut buf = [0u8; 4];
    file.read_exact(&mut buf)?;
    Ok(u32::from_le_bytes(buf))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    #[test]
    fn test_read_jpx_basic() {
        let mut jpx_data = vec![];

        // Stitch start position (skip header for now, position 100)
        let stitch_pos = 100u32;
        jpx_data.extend_from_slice(&stitch_pos.to_le_bytes());

        // Skip 0x1C bytes
        jpx_data.extend_from_slice(&vec![0u8; 0x1C]);

        // Colors: 1
        jpx_data.extend_from_slice(&[1, 0, 0, 0]);

        // Skip 0x18 bytes
        jpx_data.extend_from_slice(&vec![0u8; 0x18]);

        // Color index 0
        jpx_data.extend_from_slice(&[0, 0, 0, 0]);

        // Pad to stitch position
        while jpx_data.len() < stitch_pos as usize {
            jpx_data.push(0);
        }

        // Regular stitch: x=10, y=10 (will be negated to -10)
        jpx_data.extend_from_slice(&[10, 10]);

        // End: 0x80, 0x10
        jpx_data.extend_from_slice(&[0x80, 0x10]);

        let mut cursor = Cursor::new(jpx_data);
        let mut pattern = EmbPattern::new();

        read(&mut cursor, &mut pattern).expect("Failed to read JPX");

        assert!(!pattern.stitches().is_empty());
        assert_eq!(pattern.threads().len(), 1);
    }

    #[test]
    fn test_jpx_color_change() {
        let mut jpx_data = vec![];

        let stitch_pos = 100u32;
        jpx_data.extend_from_slice(&stitch_pos.to_le_bytes());
        jpx_data.extend_from_slice(&vec![0u8; 0x1C]);
        jpx_data.extend_from_slice(&[2, 0, 0, 0]); // 2 colors
        jpx_data.extend_from_slice(&vec![0u8; 0x18]);
        jpx_data.extend_from_slice(&[0, 0, 0, 0]); // Color 0
        jpx_data.extend_from_slice(&[1, 0, 0, 0]); // Color 1

        while jpx_data.len() < stitch_pos as usize {
            jpx_data.push(0);
        }

        // Color change: 0x80, 0x01, x=0, y=0
        jpx_data.extend_from_slice(&[0x80, 0x01, 0, 0]);

        // Stitch
        jpx_data.extend_from_slice(&[5, 5]);

        // End
        jpx_data.extend_from_slice(&[0x80, 0x10]);

        let mut cursor = Cursor::new(jpx_data);
        let mut pattern = EmbPattern::new();

        read(&mut cursor, &mut pattern).expect("Failed to read JPX");

        assert_eq!(pattern.threads().len(), 2);
    }
}
