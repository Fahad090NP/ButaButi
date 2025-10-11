//! Brother PHC format reader
//!
//! PHC is a Brother PEC variant format. Delegates to PEC reader for parsing
//! embroidery card data with graphics and thread information.

use crate::core::pattern::EmbPattern;
use crate::formats::io::readers::pec;
use crate::palettes::thread_pec::PEC_THREADS;
use crate::utils::error::Result;
use std::io::{Read, Seek, SeekFrom};

/// Read PHC (Brother PHC) format
///
/// PHC format uses PEC stitch encoding with a custom header and graphics.
pub fn read(file: &mut (impl Read + Seek), pattern: &mut EmbPattern) -> Result<()> {
    // Read graphics metadata at offset 0x4A
    file.seek(SeekFrom::Start(0x4A))?;
    let _pec_graphic_icon_height = read_u8(file)?;
    file.seek(SeekFrom::Current(1))?;
    let _pec_graphic_byte_stride = read_u8(file)?;

    let color_count = read_u16_le(file)?;

    // Read thread indices
    for _ in 0..color_count {
        let color_index = match read_u8(file) {
            Ok(idx) => idx,
            Err(_) => return Ok(()), // File terminated
        };
        let thread_index = (color_index as usize) % PEC_THREADS.len();
        pattern.add_thread(PEC_THREADS[thread_index].clone());
    }

    // Skip graphics data
    // byte_size = pec_graphic_byte_stride * pec_graphic_icon_height
    // We'll skip this for simplicity

    // Navigate to stitch data
    file.seek(SeekFrom::Start(0x2B))?;
    let pec_add = read_u8(file)?;
    file.seek(SeekFrom::Current(4))?; // Skip 0x30
    let pec_offset = read_u16_le(file)?;

    file.seek(SeekFrom::Start((pec_offset + pec_add as u16) as u64))?;
    let bytes_in_section = read_u16_le(file)?;
    file.seek(SeekFrom::Current(bytes_in_section as i64))?;

    let bytes_in_section2 = read_u32_le(file)?;
    file.seek(SeekFrom::Current(bytes_in_section2 as i64 + 10))?;

    let color_count2 = read_u8(file)?;
    file.seek(SeekFrom::Current(color_count2 as i64 + 0x1D))?;

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
    fn test_read_phc_basic() {
        let mut phc_data = vec![0u8; 0x4A];

        // At 0x4A: graphics metadata
        phc_data.push(10); // icon height
        phc_data.push(0); // padding
        phc_data.push(8); // byte stride

        // Color count
        phc_data.extend_from_slice(&1u16.to_le_bytes());

        // Color index 0
        phc_data.push(0);

        // Pad to ensure we have enough data
        while phc_data.len() < 0x100 {
            phc_data.push(0);
        }

        let mut cursor = Cursor::new(phc_data);
        let mut pattern = EmbPattern::new();

        // This will fail to read PEC data, but should at least parse the header
        let _ = read(&mut cursor, &mut pattern);

        // Check that thread was added
        assert_eq!(pattern.threads().len(), 1);
    }
}
