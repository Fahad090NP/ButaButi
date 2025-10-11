//! Happy TAP format reader
//!
//! TAP is a DST variant format from Happy embroidery machines. Delegates to DST reader
//! for parsing 3-byte stitch records with standard DST encoding.

use crate::core::pattern::EmbPattern;
use crate::formats::io::readers::dst;
use crate::utils::error::Result;
use std::io::Read;

/// Read TAP (Happy Embroidery Tap) format
///
/// TAP format is essentially DST format with a different extension.
/// It uses the same stitch encoding as DST.
pub fn read(file: &mut impl Read, pattern: &mut EmbPattern) -> Result<()> {
    // TAP uses DST stitch encoding
    let dst_pattern = dst::read(file, None)?;

    // Copy stitches and threads to the target pattern
    for thread in dst_pattern.threads() {
        pattern.add_thread(thread.clone());
    }

    for stitch in dst_pattern.stitches() {
        pattern.add_stitch_absolute(stitch.command, stitch.x, stitch.y);
    }

    // Copy metadata
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
    fn test_read_tap() {
        // TAP uses DST encoding, so we can use DST-formatted data
        // Create minimal DST header (512 bytes) + stitches
        let mut tap_data = vec![0u8; 512];

        // Add some DST-encoded stitches
        // Stitch at (10, 10)
        tap_data.extend_from_slice(&[0x03, 0x03, 0x03]); // Small move
                                                         // End
        tap_data.extend_from_slice(&[0xF3, 0x00, 0x00]);

        let mut cursor = Cursor::new(tap_data);
        let mut pattern = EmbPattern::new();

        read(&mut cursor, &mut pattern).expect("Failed to read TAP");

        // Should have at least end command
        assert!(!pattern.stitches().is_empty());
    }

    #[test]
    fn test_tap_compatibility_with_dst() {
        // TAP should read exactly like DST
        let dst_data = vec![0u8; 512]; // Minimal DST header

        let pattern1 =
            dst::read(&mut Cursor::new(dst_data.clone()), None).expect("DST read failed");

        let mut pattern2 = EmbPattern::new();
        read(&mut Cursor::new(dst_data), &mut pattern2).expect("TAP read failed");

        // Both should produce identical results
        assert_eq!(pattern1.stitches().len(), pattern2.stitches().len());
    }
}
