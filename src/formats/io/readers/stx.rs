//! Data Stitch STX format reader
//!
//! STX is an EXP variant format from Data Stitch. Delegates to EXP reader
//! for parsing 2-byte stitch records and embroidery commands.

use crate::core::pattern::EmbPattern;
use crate::formats::io::readers::exp;
use anyhow::Result;
use std::io::{Read, Seek, SeekFrom};

/// Read STX (Data Stitch) format
///
/// STX format uses EXP stitch encoding with a custom header.
pub fn read(file: &mut (impl Read + Seek), pattern: &mut EmbPattern) -> Result<()> {
    // Skip 0x0C bytes from start
    file.seek(SeekFrom::Current(0x0C))?;

    let _color_start_position = read_u32_le(file)?;
    let _dunno_block_start_position = read_u32_le(file)?;
    let stitch_start_position = read_u32_le(file)?;

    // Seek to stitch data
    file.seek(SeekFrom::Start(stitch_start_position as u64))?;

    // Use EXP reader for stitches
    let exp_pattern = exp::read(file)?;

    // Copy data to target pattern
    for thread in exp_pattern.threads() {
        pattern.add_thread(thread.clone());
    }

    for stitch in exp_pattern.stitches() {
        pattern.add_stitch_absolute(stitch.command, stitch.x, stitch.y);
    }

    for (key, value) in exp_pattern.metadata() {
        pattern.add_metadata(key, value);
    }

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
    fn test_read_stx_basic() {
        let mut stx_data = vec![];

        // Skip first 0x0C bytes
        stx_data.extend_from_slice(&vec![0u8; 0x0C]);

        // Positions
        let color_pos = 100u32;
        let dunno_pos = 200u32;
        let stitch_pos = 300u32;

        stx_data.extend_from_slice(&color_pos.to_le_bytes());
        stx_data.extend_from_slice(&dunno_pos.to_le_bytes());
        stx_data.extend_from_slice(&stitch_pos.to_le_bytes());

        // Pad to stitch position
        while stx_data.len() < stitch_pos as usize {
            stx_data.push(0);
        }

        // Add minimal EXP data (just end)
        stx_data.extend_from_slice(&[0x80, 0x00, 0x00, 0x00]);

        let mut cursor = Cursor::new(stx_data);
        let mut pattern = EmbPattern::new();

        read(&mut cursor, &mut pattern).expect("Failed to read STX");

        assert!(!pattern.stitches().is_empty());
    }
}
