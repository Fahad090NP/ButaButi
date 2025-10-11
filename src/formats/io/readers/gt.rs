//! Gold Thread GT format reader
//!
//! GT is DSZ with a 256-byte header. Skips the header and uses DSZ reader
//! for Z-stitch encoded data compatible with Gold Thread systems.

use crate::core::pattern::EmbPattern;
use crate::formats::io::readers::dsz;
use crate::utils::error::Result;
use std::io::{Read, Seek, SeekFrom};

/// Read GT (Gold Thread) format
///
/// GT format has a 512-byte header followed by Z-stitch encoded data
/// (same encoding as DSZ format).
pub fn read(file: &mut (impl Read + Seek), pattern: &mut EmbPattern) -> Result<()> {
    // Skip 512-byte header
    file.seek(SeekFrom::Start(0x200))?;

    // Read Z-stitch encoded data (same as DSZ)
    dsz::read_z_stitches(file, pattern)?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    #[test]
    fn test_read_gt_basic() {
        // Create 512-byte header + Z-stitch data
        let mut gt_data = vec![0u8; 512];

        // Add Z-stitch encoded stitches
        // Stitch at (10, -10)
        gt_data.extend_from_slice(&[10, 10, 0x00]);

        // Jump at (5, -5)
        gt_data.extend_from_slice(&[5, 5, 0x01]);

        // Trim
        gt_data.extend_from_slice(&[0, 0, 0x9B]);

        let mut cursor = Cursor::new(gt_data);
        let mut pattern = EmbPattern::new();

        read(&mut cursor, &mut pattern).expect("Failed to read GT");

        assert!(!pattern.stitches().is_empty());
    }

    #[test]
    fn test_gt_header_skip() {
        // Verify that GT properly skips the 512-byte header
        let mut gt_data = vec![0xFFu8; 512]; // Header with junk data

        // Add valid Z-stitch data after header
        gt_data.extend_from_slice(&[10, 10, 0x00]);

        let mut cursor = Cursor::new(gt_data);
        let mut pattern = EmbPattern::new();

        read(&mut cursor, &mut pattern).expect("Failed to read GT");

        assert!(!pattern.stitches().is_empty());
    }
}
