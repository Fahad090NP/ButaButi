//! Zeng Hsing ZHS format reader
//!
//! ZHS is Zeng Hsing's embroidery format with coordinate stitches and thread colors,
//! used by Zeng Hsing embroidery machines and software.

use crate::core::constants::*;
use crate::core::pattern::EmbPattern;
use crate::core::thread::EmbThread;
use anyhow::Result;
use std::io::{Read, Seek, SeekFrom};

/// Read ZHS (Zeng Hsing) format
pub fn read(file: &mut (impl Read + Seek), pattern: &mut EmbPattern) -> Result<()> {
    // Seek to offset 0x0F
    file.seek(SeekFrom::Start(0x0F))?;

    let stitch_start_position = read_u32_le(file)?;
    let header_start_position = read_u32_le(file)?;

    // Read header
    file.seek(SeekFrom::Start(header_start_position as u64))?;
    read_zhs_header(file, pattern)?;

    // Read stitches
    file.seek(SeekFrom::Start(stitch_start_position as u64))?;
    read_zhs_stitches(file, pattern)?;

    Ok(())
}

/// Read ZHS header with thread information
fn read_zhs_header(file: &mut impl Read, pattern: &mut EmbPattern) -> Result<()> {
    let color_count = read_u8(file)?;

    // Read thread colors
    for _ in 0..color_count {
        let rgb = read_u24_be(file)?;
        let thread = EmbThread::from_rgb(
            ((rgb >> 16) & 0xFF) as u8,
            ((rgb >> 8) & 0xFF) as u8,
            (rgb & 0xFF) as u8,
        );
        pattern.add_thread(thread);
    }

    // Read thread metadata string
    let length = read_u16_le(file)?;
    let mut string_buf = vec![0u8; length as usize];
    file.read_exact(&mut string_buf)?;

    // Try to parse thread metadata (best effort)
    if let Ok(thread_data) = String::from_utf8(string_buf) {
        let threads_parts: Vec<&str> = thread_data.split("&$").collect();

        for (i, data) in threads_parts.iter().skip(1).enumerate() {
            if i >= pattern.threads().len() {
                break;
            }

            let parts: Vec<&str> = data.split("&#").collect();
            let mut thread_desc = String::new();

            if parts.len() > 1 && !parts[1].is_empty() {
                thread_desc = parts[1].to_string();
            }

            if !thread_desc.is_empty() {
                // Update thread description
                // Note: This would require mutable access to threads, which we'll skip for now
            }
        }
    }

    Ok(())
}

/// Read ZHS stitches
fn read_zhs_stitches(file: &mut impl Read, pattern: &mut EmbPattern) -> Result<()> {
    let mut buffer = [0u8; 3];
    let mut xx = 0i32;
    let mut yy = 0i32;

    loop {
        // Read 3 bytes
        match file.read_exact(&mut buffer) {
            Ok(_) => {}
            Err(e) if e.kind() == std::io::ErrorKind::UnexpectedEof => break,
            Err(e) => return Err(e.into()),
        }

        let ctrl = buffer[0];

        if ctrl == 0x10 {
            // Checksum - skip
            continue;
        }

        // Decode X coordinate
        let mut x = 0u8;
        x += buffer[1] & 0b00000001;
        x += buffer[2] & 0b00000010;
        x += buffer[1] & 0b00000100;
        x += buffer[2] & 0b00001000;
        x += buffer[1] & 0b00010000;
        x += buffer[2] & 0b00100000;
        x += buffer[1] & 0b01000000;
        x += buffer[2] & 0b10000000;

        let mut x_signed = x as i8 as i32;
        if x_signed >= 63 {
            x_signed += 1;
        }
        if x_signed <= -63 {
            x_signed -= 1;
        }

        // Decode Y coordinate
        let mut y = 0u8;
        y += buffer[2] & 0b00000001;
        y += buffer[1] & 0b00000010;
        y += buffer[2] & 0b00000100;
        y += buffer[1] & 0b00001000;
        y += buffer[2] & 0b00010000;
        y += buffer[1] & 0b00100000;
        y += buffer[2] & 0b01000000;
        y += buffer[1] & 0b10000000;

        let mut y_signed = y as i8 as i32;
        if y_signed >= 63 {
            y_signed += 1;
        }
        if y_signed <= -63 {
            y_signed -= 1;
        }

        xx += x_signed;
        yy += y_signed;

        match ctrl {
            0x41 => {
                // Unmapped control - ignore
            }
            0x02 => {
                // Stitch
                pattern.add_stitch_relative(xx as f64, -yy as f64, STITCH);
                xx = 0;
                yy = 0;
            }
            0x01 => {
                // Move/Jump
                pattern.add_stitch_relative(xx as f64, -yy as f64, JUMP);
                xx = 0;
                yy = 0;
            }
            0x04 => {
                // Color change
                xx = 0;
                yy = 0;
                pattern.add_command(COLOR_CHANGE, 0.0, 0.0);
            }
            0x80 => {
                // End
                break;
            }
            _ => {}
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

/// Read unsigned 16-bit little-endian integer
fn read_u16_le(file: &mut impl Read) -> Result<u16> {
    let mut buf = [0u8; 2];
    file.read_exact(&mut buf)?;
    Ok(u16::from_le_bytes(buf))
}

/// Read unsigned 24-bit big-endian integer
fn read_u24_be(file: &mut impl Read) -> Result<u32> {
    let mut buf = [0u8; 3];
    file.read_exact(&mut buf)?;
    Ok(u32::from_be_bytes([0, buf[0], buf[1], buf[2]]))
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
    fn test_read_zhs_basic() {
        let mut zhs_data = vec![0u8; 0x0F];

        // At 0x0F: stitch_start_position
        let header_pos = 100u32;
        let stitch_pos = 200u32;
        zhs_data.extend_from_slice(&stitch_pos.to_le_bytes());
        zhs_data.extend_from_slice(&header_pos.to_le_bytes());

        // Pad to header position (100)
        while zhs_data.len() < header_pos as usize {
            zhs_data.push(0);
        }

        // Header: color count = 1
        zhs_data.push(1);
        // Color: Red
        zhs_data.extend_from_slice(&[0xFF, 0, 0]);
        // String length = 0
        zhs_data.extend_from_slice(&[0, 0]);

        // Pad to stitch position (200)
        while zhs_data.len() < stitch_pos as usize {
            zhs_data.push(0);
        }

        // Stitch: ctrl=0x02, simple encoding
        zhs_data.extend_from_slice(&[0x02, 10, 10]);

        // End: ctrl=0x80
        zhs_data.extend_from_slice(&[0x80, 0, 0]);

        let mut cursor = Cursor::new(zhs_data);
        let mut pattern = EmbPattern::new();

        read(&mut cursor, &mut pattern).expect("Failed to read ZHS");

        assert!(!pattern.stitches().is_empty());
    }
}
