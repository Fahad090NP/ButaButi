//! Embird EDR color format reader
//!
//! EDR is Embird's color format with 4-byte RGB records, storing thread color
//! information for embroidery designs in a simple binary structure.
//!
//! ## Format Limitations
//!
//! - **No stitches**: EDR only stores thread colors, no stitch data
//! - **Max threads**: Limited to 10,000 threads (safety limit)
//! - **Fixed record size**: Each thread is exactly 4 bytes (RGB + padding)
//! - **No metadata**: No pattern name, size, or other attributes

use crate::core::pattern::EmbPattern;
use crate::core::thread::EmbThread;
use crate::utils::error::{Error, Result};
use std::io::Read;

// Format constants
const EDR_RECORD_SIZE: usize = 4; // Each thread: R, G, B, padding
const MAX_EDR_THREADS: usize = 10_000; // Safety limit for thread count

/// Read EDR (Embird Color) format
///
/// EDR is a simple color list format with RGB values.
/// Each thread is stored as 4 bytes: [RED, GREEN, BLUE, PADDING]
pub fn read(file: &mut impl Read, pattern: &mut EmbPattern) -> Result<()> {
    let mut buffer = [0u8; EDR_RECORD_SIZE];
    let mut thread_count = 0;

    loop {
        // Validate thread count before reading
        if thread_count >= MAX_EDR_THREADS {
            return Err(Error::Parse(format!(
                "EDR: Thread count exceeds maximum of {}",
                MAX_EDR_THREADS
            )));
        }

        // Try to read 4 bytes (R, G, B, padding)
        match file.read_exact(&mut buffer) {
            Ok(_) => {
                let red = buffer[0];
                let green = buffer[1];
                let blue = buffer[2];
                // buffer[3] is padding, ignored

                let thread = EmbThread::from_rgb(red, green, blue);
                pattern.add_thread(thread);
                thread_count += 1;
            },
            Err(e) if e.kind() == std::io::ErrorKind::UnexpectedEof => {
                // End of file reached
                break;
            },
            Err(e) => {
                return Err(Error::Io(e));
            },
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    #[test]
    fn test_read_basic_edr() {
        // 3 threads: Red, Green, Blue
        let edr_data = vec![
            255, 0, 0, 0, // Red thread
            0, 255, 0, 0, // Green thread
            0, 0, 255, 0, // Blue thread
        ];

        let mut cursor = Cursor::new(edr_data);
        let mut pattern = EmbPattern::new();

        read(&mut cursor, &mut pattern).expect("Failed to read EDR");

        assert_eq!(pattern.threads().len(), 3);
        assert_eq!(pattern.threads()[0].red(), 255);
        assert_eq!(pattern.threads()[0].green(), 0);
        assert_eq!(pattern.threads()[0].blue(), 0);

        assert_eq!(pattern.threads()[1].red(), 0);
        assert_eq!(pattern.threads()[1].green(), 255);
        assert_eq!(pattern.threads()[1].blue(), 0);

        assert_eq!(pattern.threads()[2].red(), 0);
        assert_eq!(pattern.threads()[2].green(), 0);
        assert_eq!(pattern.threads()[2].blue(), 255);
    }

    #[test]
    fn test_read_empty_edr() {
        let edr_data = vec![];
        let mut cursor = Cursor::new(edr_data);
        let mut pattern = EmbPattern::new();

        read(&mut cursor, &mut pattern).expect("Failed to read empty EDR");

        assert_eq!(pattern.threads().len(), 0);
    }

    #[test]
    fn test_read_partial_edr() {
        // Incomplete thread (only 2 bytes instead of 4) should stop gracefully
        let edr_data = vec![
            255, 0, 0, 0, // Complete thread
            128, 64, // Incomplete thread
        ];

        let mut cursor = Cursor::new(edr_data);
        let mut pattern = EmbPattern::new();

        read(&mut cursor, &mut pattern).expect("Failed to read partial EDR");

        // Should only read the complete thread
        assert_eq!(pattern.threads().len(), 1);
        assert_eq!(pattern.threads()[0].red(), 255);
    }
}
