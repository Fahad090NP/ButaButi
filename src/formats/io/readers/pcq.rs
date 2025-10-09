//! Pfaff PCQ format reader
//!
//! PCQ (Pfaff MAXI) format is identical to PCD format.
//! Re-exports the PCD reader for compatibility.

pub use super::pcd::read;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::pattern::EmbPattern;
    use std::io::Cursor;

    #[test]
    fn test_read_pcq_basic() {
        let mut pcq_data = vec![];

        // Version and hoop size (1 for PCQ MAXI)
        pcq_data.extend_from_slice(&[0, 1]);

        // Color count: 1
        pcq_data.extend_from_slice(&[1, 0]);

        // Color: Blue
        pcq_data.extend_from_slice(&[0, 0, 0xFF, 0]);

        // Stitch count: 1
        pcq_data.extend_from_slice(&[1, 0]);

        // Stitch
        pcq_data.extend_from_slice(&[0, 100, 0, 0, 0, 200, 0, 0, 0x00]);

        let mut cursor = Cursor::new(pcq_data);
        let mut pattern = EmbPattern::new();

        read(&mut cursor, &mut pattern).expect("Failed to read PCQ");

        assert!(!pattern.stitches().is_empty());
        assert_eq!(pattern.threads().len(), 1);
    }
}
