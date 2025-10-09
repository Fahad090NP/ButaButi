//! Fortron FXY format reader
//!
//! FXY is DSZ with a 256-byte header. Skips the header and uses DSZ Z-stitch reader
//! for the stitch data with relative coordinate encoding.

use crate::core::pattern::EmbPattern;
use crate::formats::io::readers::dsz;
use anyhow::Result;
use std::io::{Read, Seek, SeekFrom};

/// Read FXY (Fortron) format
///
/// FXY format is DSZ format (Z-stitch encoding) with a 256-byte header.
pub fn read(file: &mut (impl Read + Seek), pattern: &mut EmbPattern) -> Result<()> {
    // Skip 0x100 (256) byte header
    file.seek(SeekFrom::Start(0x100))?;

    // Use DSZ Z-stitch reader for the rest
    dsz::read_z_stitches(file, pattern)?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    #[test]
    fn test_read_fxy_basic() {
        // Create 256-byte header
        let mut fxy_data = vec![0u8; 0x100];

        // Add Z-stitch encoded data (similar to DSZ)
        // Simple stitch at (10, -10)
        fxy_data.extend_from_slice(&[0x00, 10, 10]);

        // End sequence
        fxy_data.extend_from_slice(&[0xF3, 0x00, 0x00]);

        let mut cursor = Cursor::new(fxy_data);
        let mut pattern = EmbPattern::new();

        read(&mut cursor, &mut pattern).expect("Failed to read FXY");

        assert!(!pattern.stitches().is_empty());
    }
}
