//! Brother PHB format reader
//!
//! PHB is a Brother PEC variant format. Delegates to PEC reader for parsing
//! stitch data and thread colors using the PEC palette.

use crate::core::pattern::EmbPattern;
use crate::formats::io::readers::pec;
use crate::palettes::thread_pec::PEC_THREADS;
use crate::utils::error::Result;
use std::io::{Read, Seek, SeekFrom};

/// Read PHB (Brother PHB) format
///
/// PHB format uses PEC stitch encoding with a custom header.
pub fn read(file: &mut (impl Read + Seek), pattern: &mut EmbPattern) -> Result<()> {
    // Read color count at offset 0x71
    file.seek(SeekFrom::Start(0x71))?;
    let color_count = read_u16_le(file)?;

    // Read thread indices
    for _ in 0..color_count {
        let color_index = read_u8(file)?;
        let thread_index = (color_index as usize) % PEC_THREADS.len();
        pattern.add_thread(PEC_THREADS[thread_index].clone());
    }

    // Calculate file offset for stitch data
    let mut file_offset = 0x52u64;

    file.seek(SeekFrom::Start(0x54))?;
    file_offset += read_u32_le(file)? as u64;

    file.seek(SeekFrom::Start(file_offset))?;
    file_offset += read_u32_le(file)? as u64 + 2;

    file.seek(SeekFrom::Start(file_offset))?;
    file_offset += read_u32_le(file)? as u64;

    file.seek(SeekFrom::Start(file_offset + 14))?;

    let color_count2 = read_u8(file)?;
    file.seek(SeekFrom::Current(color_count2 as i64 + 0x15))?;

    // Read PEC stitches
    let pec_pattern = pec::read(file)?;

    // Copy stitches
    for stitch in pec_pattern.stitches() {
        pattern.add_stitch_absolute(stitch.command, stitch.x, stitch.y);
    }

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
    fn test_read_phb_basic() {
        let mut phb_data = vec![0u8; 0x71];

        // At 0x71: color count
        phb_data.extend_from_slice(&1u16.to_le_bytes());

        // Color index 0
        phb_data.push(0);

        // Pad to ensure we have enough data for offset calculations
        while phb_data.len() < 0x100 {
            phb_data.push(0);
        }

        let mut cursor = Cursor::new(phb_data);
        let mut pattern = EmbPattern::new();

        // This will fail to read PEC data, but should at least parse the header
        let _ = read(&mut cursor, &mut pattern);

        // Check that thread was added
        assert_eq!(pattern.threads().len(), 1);
    }
}
