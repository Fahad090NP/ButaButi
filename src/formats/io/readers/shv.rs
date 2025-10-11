//! Husqvarna Viking SHV format reader
//!
//! SHV format uses a predefined 43-color thread palette specific to Husqvarna Viking
//! machines, with binary stitch encoding and color indices.

use crate::core::constants::*;
use crate::core::pattern::EmbPattern;
use crate::palettes::thread_shv;
use crate::utils::error::Result;
use std::io::{Read, Seek, SeekFrom};

/// Read SHV (Husqvarna Viking SHV) format
pub fn read(file: &mut (impl Read + Seek), pattern: &mut EmbPattern) -> Result<()> {
    // Skip header text (0x56 bytes)
    file.seek(SeekFrom::Current(0x56))?;

    // Read design name
    let name_length = read_int_8(file)?;
    let mut name_buf = vec![0u8; name_length as usize];
    file.read_exact(&mut name_buf)?;
    if let Ok(name) = String::from_utf8(name_buf) {
        pattern.add_metadata("name", &name);
    }

    // Read design dimensions
    let design_width = read_int_8(file)?;
    let design_height = read_int_8(file)?;

    // Skip design bitmap (height/2 rounded up * width)
    let skip = ((design_height as f64 / 2.0).ceil() as usize) * design_width as usize;
    file.seek(SeekFrom::Current((4 + skip) as i64))?;

    // Read color count
    let color_count = read_int_8(file)?;

    // Skip 18 bytes
    file.seek(SeekFrom::Current(18))?;

    // Read thread information
    let threads = thread_shv::get_thread_set();
    let mut stitch_per_color = Vec::new();

    for _ in 0..color_count {
        let stitch_count = read_int_32be(file)?;
        let color_code = read_int_8(file)?;

        let thread_index = (color_code as usize) % threads.len();
        pattern.add_thread(threads[thread_index].clone());

        stitch_per_color.push(stitch_count);

        // Skip 9 bytes
        file.seek(SeekFrom::Current(9))?;
    }

    // Seek back 2 bytes
    file.seek(SeekFrom::Current(-2))?;

    // Read stitches
    read_shv_stitches(file, pattern, &stitch_per_color)?;

    Ok(())
}

/// Read SHV stitch data
fn read_shv_stitches(
    file: &mut impl Read,
    pattern: &mut EmbPattern,
    stitch_per_color: &[u32],
) -> Result<()> {
    let mut in_jump = false;
    let mut stitches_since_stop = 0u32;
    let mut current_color_index = 0usize;
    let mut max_stitches = stitch_per_color.first().copied().unwrap_or(0);

    #[allow(clippy::while_let_loop)]
    loop {
        let b0 = match read_int_8(file) {
            Ok(v) => v,
            Err(_) => break,
        };

        let b1 = match read_int_8(file) {
            Ok(v) => v,
            Err(_) => break,
        };

        // Check if we need to change color
        if stitches_since_stop >= max_stitches {
            pattern.add_command(COLOR_CHANGE, 0.0, 0.0);
            stitches_since_stop = 0;
            current_color_index += 1;
            max_stitches = stitch_per_color
                .get(current_color_index)
                .copied()
                .unwrap_or(0xFFFFFFFF);
        }

        // Check for special commands
        if b0 == 0x80 {
            stitches_since_stop += 1;

            if b1 == 3 {
                // Continue without action
                continue;
            } else if b1 == 2 {
                // End of jump
                in_jump = false;
                continue;
            } else if b1 == 1 {
                // Long jump (move)
                stitches_since_stop += 2;
                let sx = read_int_16be(file)? as i16 as f64;
                let sy = read_int_16be(file)? as i16 as f64;
                in_jump = true;
                pattern.add_stitch_relative(sx, sy, JUMP);
                continue;
            }
        }

        // Regular stitch or jump
        let dx = b0 as i8 as f64;
        let dy = b1 as i8 as f64;
        stitches_since_stop += 1;

        let flags = if in_jump { JUMP } else { STITCH };
        pattern.add_stitch_relative(dx, dy, flags);
    }

    pattern.add_command(END, 0.0, 0.0);
    Ok(())
}

/// Read 8-bit integer
fn read_int_8(file: &mut impl Read) -> Result<u8> {
    let mut buf = [0u8; 1];
    file.read_exact(&mut buf)?;
    Ok(buf[0])
}

/// Read big-endian 16-bit integer
fn read_int_16be(file: &mut impl Read) -> Result<u16> {
    let mut buf = [0u8; 2];
    file.read_exact(&mut buf)?;
    Ok(u16::from_be_bytes(buf))
}

/// Read big-endian 32-bit integer
fn read_int_32be(file: &mut impl Read) -> Result<u32> {
    let mut buf = [0u8; 4];
    file.read_exact(&mut buf)?;
    Ok(u32::from_be_bytes(buf))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    #[test]
    fn test_read_shv_basic() {
        let mut data = vec![0u8; 0x56]; // Header

        // Name length and name
        data.push(4);
        data.extend_from_slice(b"Test");

        // Design dimensions
        data.push(10); // width
        data.push(10); // height

        // Design bitmap (5 * 10 + 4 bytes)
        data.extend_from_slice(&[0u8; 54]);

        // Color count
        data.push(1);

        // 18 bytes padding
        data.extend_from_slice(&[0u8; 18]);

        // Thread info (14 bytes per color)
        data.extend_from_slice(&[0, 0, 0, 10]); // Stitch count (10)
        data.push(0); // Color code
        data.extend_from_slice(&[0u8; 9]); // Padding

        // Seek back 2 (already at correct position in our test)

        // Simple stitch
        data.push(5);
        data.push(5);

        let mut cursor = Cursor::new(data);
        let mut pattern = EmbPattern::new();

        read(&mut cursor, &mut pattern).expect("Failed to read SHV");

        assert!(!pattern.stitches().is_empty());
        assert_eq!(pattern.threads().len(), 1);
    }

    #[test]
    fn test_shv_long_jump() {
        let mut data = vec![0u8; 0x56]; // Header

        // Name
        data.push(0); // Empty name

        // Dimensions
        data.push(10);
        data.push(10);

        // Bitmap + padding
        data.extend_from_slice(&[0u8; 54]);

        // Color count
        data.push(1);

        // Padding
        data.extend_from_slice(&[0u8; 18]);

        // Thread info
        data.extend_from_slice(&[0, 0, 0, 20]); // Stitch count
        data.push(0); // Color
        data.extend_from_slice(&[0u8; 9]);

        // Long jump (0x80, 0x01, then 16-bit BE coords)
        data.push(0x80);
        data.push(0x01);
        data.extend_from_slice(&[0, 100]); // sx = 100
        data.extend_from_slice(&[0, 50]); // sy = 50

        // Regular stitch
        data.push(5);
        data.push(5);

        let mut cursor = Cursor::new(data);
        let mut pattern = EmbPattern::new();

        read(&mut cursor, &mut pattern).expect("Failed to read SHV");

        assert!(!pattern.stitches().is_empty());
    }

    #[test]
    fn test_shv_color_change() {
        let mut data = vec![0u8; 0x56]; // Header

        // Name
        data.push(0);

        // Dimensions
        data.push(10);
        data.push(10);

        // Bitmap + padding
        data.extend_from_slice(&[0u8; 54]);

        // Color count (2)
        data.push(2);

        // Padding
        data.extend_from_slice(&[0u8; 18]);

        // Thread 1
        data.extend_from_slice(&[0, 0, 0, 2]); // 2 stitches
        data.push(0);
        data.extend_from_slice(&[0u8; 9]);

        // Thread 2
        data.extend_from_slice(&[0, 0, 0, 2]); // 2 stitches
        data.push(1);
        data.extend_from_slice(&[0u8; 9]);

        // Stitches for color 1
        data.push(5);
        data.push(5);
        data.push(5);
        data.push(5);

        // Stitches for color 2 (automatic color change)
        data.push(10);
        data.push(10);

        let mut cursor = Cursor::new(data);
        let mut pattern = EmbPattern::new();

        read(&mut cursor, &mut pattern).expect("Failed to read SHV");

        assert!(!pattern.stitches().is_empty());
        assert_eq!(pattern.threads().len(), 2);
    }
}
