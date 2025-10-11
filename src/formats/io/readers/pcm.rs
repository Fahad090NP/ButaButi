//! Pfaff PCM format reader
//!
//! PCM is a Pfaff embroidery format supporting compressed stitches and colors,
//! used by Pfaff PC-Stitcher and compatible embroidery software.

use crate::core::constants::*;
use crate::core::pattern::EmbPattern;
use crate::core::thread::EmbThread;
use crate::utils::error::Result;
use std::io::{Read, Seek, SeekFrom};

const PC_SIZE_CONVERSION_RATIO: f64 = 5.0 / 3.0;

/// PCM thread palette (16 colors)
const PCM_THREADS: [(u32, &str); 16] = [
    (0x000000, "PCM Color 1"),
    (0x000080, "PCM Color 2"),
    (0x0000FF, "PCM Color 3"),
    (0x008080, "PCM Color 4"),
    (0x00FFFF, "PCM Color 5"),
    (0x800080, "PCM Color 6"),
    (0xFF00FF, "PCM Color 7"),
    (0x800000, "PCM Color 8"),
    (0xFF0000, "PCM Color 9"),
    (0x008000, "PCM Color 10"),
    (0x00FF00, "PCM Color 11"),
    (0x808000, "PCM Color 12"),
    (0xFFFF00, "PCM Color 13"),
    (0x808080, "PCM Color 14"),
    (0xC0C0C0, "PCM Color 15"),
    (0xFFFFFF, "PCM Color 16"),
];

/// Read PCM (Pfaff) format
pub fn read(file: &mut (impl Read + Seek), pattern: &mut EmbPattern) -> Result<()> {
    // Skip to offset 2
    file.seek(SeekFrom::Start(2))?;

    // Read color count
    let colors = read_u16_be(file)?;

    // Read thread palette
    for _ in 0..colors {
        let color_index = read_u16_be(file)? as usize;
        if color_index < PCM_THREADS.len() {
            let (rgb, desc) = PCM_THREADS[color_index];
            let thread = EmbThread::from_rgb(
                ((rgb >> 16) & 0xFF) as u8,
                ((rgb >> 8) & 0xFF) as u8,
                (rgb & 0xFF) as u8,
            )
            .with_description(desc);
            pattern.add_thread(thread);
        }
    }

    // Read stitch count (but don't use it)
    let _stitch_count = read_u16_be(file)?;

    // Read stitches
    #[allow(clippy::while_let_loop)]
    loop {
        // Try to read x coordinate
        let x = match read_i24_be(file) {
            Ok(v) => v,
            Err(_) => break,
        };

        let _c0 = match read_u8(file) {
            Ok(v) => v,
            Err(_) => break,
        };

        let y = match read_i24_be(file) {
            Ok(v) => v,
            Err(_) => break,
        };

        let _c1 = match read_u8(file) {
            Ok(v) => v,
            Err(_) => break,
        };

        let ctrl = match read_u8(file) {
            Ok(c) => c,
            Err(_) => break,
        };

        let x = x as f64 * PC_SIZE_CONVERSION_RATIO;
        let y = -(y as f64) * PC_SIZE_CONVERSION_RATIO;

        if ctrl == 0x00 {
            // Stitch
            pattern.add_stitch_absolute(STITCH, x, y);
        } else if ctrl & 0x01 != 0 {
            // Color change
            pattern.add_command(COLOR_CHANGE, 0.0, 0.0);
        } else if ctrl & 0x04 != 0 {
            // Move/Jump
            pattern.add_stitch_absolute(JUMP, x, y);
        } else {
            // Uncaught control - break
            break;
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

/// Read unsigned 16-bit big-endian integer
fn read_u16_be(file: &mut impl Read) -> Result<u16> {
    let mut buf = [0u8; 2];
    file.read_exact(&mut buf)?;
    Ok(u16::from_be_bytes(buf))
}

/// Read signed 24-bit big-endian integer
fn read_i24_be(file: &mut impl Read) -> Result<i32> {
    let mut buf = [0u8; 3];
    file.read_exact(&mut buf)?;

    // Build 32-bit value from big-endian bytes
    let value = u32::from_be_bytes([0, buf[0], buf[1], buf[2]]);

    // If sign bit (bit 23) is set, extend to negative
    if value & 0x00800000 != 0 {
        Ok((value | 0xFF000000) as i32)
    } else {
        Ok(value as i32)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    #[test]
    fn test_read_pcm_basic() {
        let mut pcm_data = vec![0u8, 0]; // Padding to offset 2

        // Color count: 1
        pcm_data.extend_from_slice(&[0, 1]);

        // Color index 0 (black)
        pcm_data.extend_from_slice(&[0, 0]);

        // Stitch count: 1
        pcm_data.extend_from_slice(&[0, 1]);

        // Stitch: x=100, c0=0, y=200, c1=0, ctrl=0x00
        pcm_data.extend_from_slice(&[0, 0, 100]); // x (24-bit BE)
        pcm_data.extend_from_slice(&[0]); // c0
        pcm_data.extend_from_slice(&[0, 0, 200]); // y (24-bit BE)
        pcm_data.extend_from_slice(&[0]); // c1
        pcm_data.extend_from_slice(&[0x00]); // ctrl

        // EOF - no more data will trigger break

        let mut cursor = Cursor::new(pcm_data);
        let mut pattern = EmbPattern::new();

        read(&mut cursor, &mut pattern).expect("Failed to read PCM");

        assert!(!pattern.stitches().is_empty());
        assert_eq!(pattern.threads().len(), 1);
    }

    #[test]
    fn test_pcm_color_change() {
        let mut pcm_data = vec![0u8, 0];

        // Color count: 2
        pcm_data.extend_from_slice(&[0, 2]);

        // Colors
        pcm_data.extend_from_slice(&[0, 0]); // Black
        pcm_data.extend_from_slice(&[0, 8]); // Red

        // Stitch count: 1
        pcm_data.extend_from_slice(&[0, 1]);

        // Stitch with color change
        pcm_data.extend_from_slice(&[0, 0, 50, 0, 0, 0, 50, 0, 0x01]);

        let mut cursor = Cursor::new(pcm_data);
        let mut pattern = EmbPattern::new();

        read(&mut cursor, &mut pattern).expect("Failed to read PCM");

        assert_eq!(pattern.threads().len(), 2);
    }

    #[test]
    fn test_i24_be_signed() {
        // Test positive value
        let data = vec![0x00, 0x12, 0x34];
        let mut cursor = Cursor::new(data);
        let value = read_i24_be(&mut cursor).expect("Failed to read");
        assert_eq!(value, 0x1234);

        // Test negative value (-1)
        let data = vec![0xFF, 0xFF, 0xFF];
        let mut cursor = Cursor::new(data);
        let value = read_i24_be(&mut cursor).expect("Failed to read");
        assert_eq!(value, -1);
    }
}
