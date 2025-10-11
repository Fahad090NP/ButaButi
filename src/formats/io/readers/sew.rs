//! Janome SEW format reader
//!
//! SEW format uses a predefined 79-color thread palette for Janome sewing machines,
//! with binary-encoded stitches and thread color indices.

use crate::core::constants::*;
use crate::core::pattern::EmbPattern;
use crate::palettes::thread_sew;
use crate::utils::error::Result;
use std::io::{Read, Seek, SeekFrom};

/// Read SEW (Janome Sewing Machine) format
pub fn read(file: &mut (impl Read + Seek), pattern: &mut EmbPattern) -> Result<()> {
    // Read color count
    let colors = read_int_16le(file)?;

    // Read thread indices and add threads from SEW palette
    let threads = thread_sew::get_thread_set();
    for _ in 0..colors {
        let index = read_int_16le(file)? as usize;
        let thread_index = index % threads.len();
        pattern.add_thread(threads[thread_index].clone());
    }

    // Seek to stitch data at 0x1D78
    file.seek(SeekFrom::Start(0x1D78))?;

    read_sew_stitches(file, pattern)?;
    Ok(())
}

/// Read SEW stitch data
fn read_sew_stitches(file: &mut impl Read, pattern: &mut EmbPattern) -> Result<()> {
    loop {
        let mut buf = [0u8; 2];
        match file.read_exact(&mut buf) {
            Ok(_) => {}
            Err(e) if e.kind() == std::io::ErrorKind::UnexpectedEof => break,
            Err(e) => return Err(e.into()),
        }

        if buf[0] != 0x80 {
            // Regular stitch
            let x = buf[0] as i8 as f64;
            let y = -(buf[1] as i8 as f64);
            pattern.add_stitch_relative(x, y, STITCH);
            continue;
        }

        // Control sequence
        let control = buf[1];

        // Read next 2 bytes
        match file.read_exact(&mut buf) {
            Ok(_) => {}
            Err(e) if e.kind() == std::io::ErrorKind::UnexpectedEof => break,
            Err(e) => return Err(e.into()),
        }

        if control & 0x01 != 0 {
            // Color change
            pattern.add_command(COLOR_CHANGE, 0.0, 0.0);
            continue;
        }

        if control == 0x04 || control == 0x02 {
            // Move (translates to JUMP)
            let x = buf[0] as i8 as f64;
            let y = -(buf[1] as i8 as f64);
            pattern.add_stitch_relative(x, y, JUMP);
            continue;
        }

        if control == 0x10 {
            // Stitch (after control)
            let x = buf[0] as i8 as f64;
            let y = -(buf[1] as i8 as f64);
            pattern.add_stitch_relative(x, y, STITCH);
            continue;
        }

        // Unknown control - break
        break;
    }

    pattern.add_command(END, 0.0, 0.0);
    Ok(())
}

/// Read little-endian 16-bit integer
fn read_int_16le(file: &mut impl Read) -> Result<u16> {
    let mut buf = [0u8; 2];
    file.read_exact(&mut buf)?;
    Ok(u16::from_le_bytes(buf))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    #[test]
    fn test_read_sew_basic() {
        let mut data = vec![];

        // Color count (1 color)
        data.extend_from_slice(&[1, 0]);

        // Thread index (0)
        data.extend_from_slice(&[0, 0]);

        // Pad to 0x1D78
        data.resize(0x1D78, 0);

        // Simple stitch
        data.extend_from_slice(&[10, 10]);

        let mut cursor = Cursor::new(data);
        let mut pattern = EmbPattern::new();

        read(&mut cursor, &mut pattern).expect("Failed to read SEW");

        assert!(!pattern.stitches().is_empty());
        assert_eq!(pattern.threads().len(), 1);
    }

    #[test]
    fn test_sew_control_sequences() {
        let mut data = vec![];

        // Color count (2 colors)
        data.extend_from_slice(&[2, 0]);

        // Thread indices
        data.extend_from_slice(&[0, 0]);
        data.extend_from_slice(&[1, 0]);

        // Pad to 0x1D78
        data.resize(0x1D78, 0);

        // Stitch
        data.extend_from_slice(&[5, 5]);

        // Color change (0x80, control with bit 0x01)
        data.extend_from_slice(&[0x80, 0x01]);
        data.extend_from_slice(&[0, 0]); // Dummy bytes

        // Stitch
        data.extend_from_slice(&[10, 10]);

        let mut cursor = Cursor::new(data);
        let mut pattern = EmbPattern::new();

        read(&mut cursor, &mut pattern).expect("Failed to read SEW");

        assert!(!pattern.stitches().is_empty());
        assert_eq!(pattern.threads().len(), 2);
    }

    #[test]
    fn test_sew_move_command() {
        let mut data = vec![];

        // Color count (1 color)
        data.extend_from_slice(&[1, 0]);
        data.extend_from_slice(&[0, 0]);

        // Pad to 0x1D78
        data.resize(0x1D78, 0);

        // Move command (0x80, 0x04)
        data.extend_from_slice(&[0x80, 0x04]);
        data.extend_from_slice(&[20, 20]);

        // Stitch
        data.extend_from_slice(&[5, 5]);

        let mut cursor = Cursor::new(data);
        let mut pattern = EmbPattern::new();

        read(&mut cursor, &mut pattern).expect("Failed to read SEW");

        assert!(!pattern.stitches().is_empty());
    }
}
