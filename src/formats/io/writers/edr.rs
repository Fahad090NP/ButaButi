//! Embird EDR color format writer
//!
//! Writes Embird's color format with 4-byte RGB records [R, G, B, 0x00],
//! storing thread color information in a simple binary structure.

use crate::core::pattern::EmbPattern;
use anyhow::Result;
use std::io::Write;

/// Write EDR (Embird Color) format
///
/// EDR is a simple color list format with RGB values.
/// Each thread is stored as 4 bytes: [RED, GREEN, BLUE, 0x00]
pub fn write(pattern: &EmbPattern, file: &mut impl Write) -> Result<()> {
    // Write all threads
    for thread in pattern.threads() {
        file.write_all(&[thread.red(), thread.green(), thread.blue(), 0])?;
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::thread::EmbThread;
    use crate::formats::io::readers::edr as edr_reader;
    use std::io::Cursor;

    #[test]
    fn test_write_basic_edr() {
        let mut pattern = EmbPattern::new();
        pattern.add_thread(EmbThread::from_rgb(255, 0, 0)); // Red
        pattern.add_thread(EmbThread::from_rgb(0, 255, 0)); // Green
        pattern.add_thread(EmbThread::from_rgb(0, 0, 255)); // Blue

        let mut output = Vec::new();
        write(&pattern, &mut output).expect("Failed to write EDR");

        let expected = vec![
            255, 0, 0, 0, // Red
            0, 255, 0, 0, // Green
            0, 0, 255, 0, // Blue
        ];

        assert_eq!(output, expected);
    }

    #[test]
    fn test_write_empty_edr() {
        let pattern = EmbPattern::new();

        let mut output = Vec::new();
        write(&pattern, &mut output).expect("Failed to write empty EDR");

        assert_eq!(output.len(), 0);
    }

    #[test]
    fn test_edr_round_trip() {
        let mut original = EmbPattern::new();
        original.add_thread(EmbThread::from_rgb(128, 64, 192));
        original.add_thread(EmbThread::from_rgb(255, 128, 0));

        // Write
        let mut buffer = Vec::new();
        write(&original, &mut buffer).expect("Failed to write EDR");

        // Read back
        let mut cursor = Cursor::new(buffer);
        let mut read_back = EmbPattern::new();
        edr_reader::read(&mut cursor, &mut read_back).expect("Failed to read EDR");

        // Verify
        assert_eq!(read_back.threads().len(), original.threads().len());
        for (i, thread) in original.threads().iter().enumerate() {
            assert_eq!(read_back.threads()[i].red(), thread.red());
            assert_eq!(read_back.threads()[i].green(), thread.green());
            assert_eq!(read_back.threads()[i].blue(), thread.blue());
        }
    }
}
