//! Eltac EXY format reader
//!
//! EXY is DST with a 256-byte header. Skips the header and uses DST reader
//! for the stitch data, supporting all DST commands and encoding.

use crate::core::pattern::EmbPattern;
use crate::formats::io::readers::dst;
use anyhow::Result;
use std::io::{Read, Seek, SeekFrom};

/// Read EXY (Eltac) format
///
/// EXY format is DST format with a 256-byte header.
pub fn read(file: &mut (impl Read + Seek), pattern: &mut EmbPattern) -> Result<()> {
    // Skip 0x100 (256) byte header
    file.seek(SeekFrom::Start(0x100))?;

    // Use DST reader for the rest
    let dst_pattern = dst::read(file, None)?;

    // Copy data to target pattern
    for thread in dst_pattern.threads() {
        pattern.add_thread(thread.clone());
    }

    for stitch in dst_pattern.stitches() {
        pattern.add_stitch_absolute(stitch.command, stitch.x, stitch.y);
    }

    for (key, value) in dst_pattern.metadata() {
        pattern.add_metadata(key, value);
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    #[test]
    fn test_read_exy_basic() {
        // Create 256-byte header
        let mut exy_data = vec![0u8; 0x100];

        // Add DST header (512 bytes total for a valid DST file)
        exy_data.extend_from_slice(&vec![0u8; 512]);

        // Add DST-encoded stitch
        exy_data.extend_from_slice(&[0x01, 0x40, 0x80]);

        // DST end sequence
        exy_data.extend_from_slice(&[0x00, 0x00, 0xF3]);

        let mut cursor = Cursor::new(exy_data);
        let mut pattern = EmbPattern::new();

        read(&mut cursor, &mut pattern).expect("Failed to read EXY");

        assert!(!pattern.stitches().is_empty());
    }
}
