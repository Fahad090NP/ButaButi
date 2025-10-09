//! Toyota 100 format reader
//!
//! Toyota 100 format uses coordinate-based stitches and color changes,
//! designed for Toyota industrial embroidery machines and software.

use crate::core::constants::*;
use crate::core::pattern::EmbPattern;
use anyhow::Result;
use std::io::Read;

/// Read 100 (Toyota) format
pub fn read(file: &mut impl Read, pattern: &mut EmbPattern) -> Result<()> {
    read_100_stitches(file, pattern)?;
    Ok(())
}

/// Read 100 stitches
fn read_100_stitches(file: &mut impl Read, pattern: &mut EmbPattern) -> Result<()> {
    let mut buffer = [0u8; 4];

    loop {
        // Read 4 bytes
        match file.read_exact(&mut buffer) {
            Ok(_) => {}
            Err(e) if e.kind() == std::io::ErrorKind::UnexpectedEof => break,
            Err(e) => return Err(e.into()),
        }

        let ctrl = buffer[0];
        let mut x = buffer[2] as i32;
        let mut y = buffer[3] as i32;

        // Decode coordinates (non-standard 2's complement)
        if x > 0x80 {
            x -= 0x80;
            x = -x;
        }
        if y > 0x80 {
            y -= 0x80;
            y = -y;
        }

        // Y is negated
        let y = -y;

        if ctrl == 0x61 {
            // Stitch
            pattern.add_stitch_relative(x as f64, y as f64, STITCH);
        } else if ctrl & 0x01 != 0 {
            // Move/Jump
            pattern.add_stitch_relative(x as f64, y as f64, JUMP);
        } else {
            // Color change (default case)
            pattern.add_command(COLOR_CHANGE, 0.0, 0.0);
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
    fn test_read_100_basic() {
        let mut data = vec![];

        // Stitch: ctrl=0x61, padding, x=10, y=10
        data.extend_from_slice(&[0x61, 0, 10, 10]);

        let mut cursor = Cursor::new(data);
        let mut pattern = EmbPattern::new();

        read(&mut cursor, &mut pattern).expect("Failed to read 100");

        assert!(!pattern.stitches().is_empty());
    }

    #[test]
    fn test_100_move() {
        let mut data = vec![];

        // Move: ctrl with bit 0x01 set
        data.extend_from_slice(&[0x03, 0, 20, 20]);

        // Stitch
        data.extend_from_slice(&[0x61, 0, 5, 5]);

        let mut cursor = Cursor::new(data);
        let mut pattern = EmbPattern::new();

        read(&mut cursor, &mut pattern).expect("Failed to read 100");

        assert!(!pattern.stitches().is_empty());
    }

    #[test]
    fn test_100_negative_coords() {
        let mut data = vec![];

        // Stitch with negative coordinates
        // x=-10 -> 0x80 + 10 = 0x8A
        // y=-20 -> 0x80 + 20 = 0x94
        data.extend_from_slice(&[0x61, 0, 0x8A, 0x94]);

        let mut cursor = Cursor::new(data);
        let mut pattern = EmbPattern::new();

        read(&mut cursor, &mut pattern).expect("Failed to read 100");

        assert!(!pattern.stitches().is_empty());
    }

    #[test]
    fn test_100_color_change() {
        let mut data = vec![];

        // Stitch
        data.extend_from_slice(&[0x61, 0, 5, 5]);

        // Color change: ctrl without 0x61 or 0x01
        data.extend_from_slice(&[0x00, 0, 0, 0]);

        // Stitch
        data.extend_from_slice(&[0x61, 0, 10, 10]);

        let mut cursor = Cursor::new(data);
        let mut pattern = EmbPattern::new();

        read(&mut cursor, &mut pattern).expect("Failed to read 100");

        assert!(!pattern.stitches().is_empty());
    }
}
