//! Pfaff PCS format reader
//!
//! PCS (Pfaff Small/Large hoop) format is identical to PCD format.
//! Re-exports the PCD reader for compatibility.

pub use super::pcd::read;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::pattern::EmbPattern;
    use std::io::Cursor;

    #[test]
    fn test_read_pcs_basic() {
        let mut pcs_data = vec![];

        // Version and hoop size (2=small, 3=large)
        pcs_data.extend_from_slice(&[0, 2]);

        // Color count: 1
        pcs_data.extend_from_slice(&[1, 0]);

        // Color: Green
        pcs_data.extend_from_slice(&[0, 0xFF, 0, 0]);

        // Stitch count: 1
        pcs_data.extend_from_slice(&[1, 0]);

        // Stitch
        pcs_data.extend_from_slice(&[0, 50, 0, 0, 0, 100, 0, 0, 0x00]);

        let mut cursor = Cursor::new(pcs_data);
        let mut pattern = EmbPattern::new();

        read(&mut cursor, &mut pattern).expect("Failed to read PCS");

        assert!(!pattern.stitches().is_empty());
        assert_eq!(pattern.threads().len(), 1);
    }
}
