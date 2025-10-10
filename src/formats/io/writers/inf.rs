//! INF thread information format writer
//!
//! Writes binary format storing detailed thread information including RGB colors,
//! descriptions, and chart references. Thread-only format with no stitch data.

use crate::core::pattern::EmbPattern;
use crate::utils::error::Result;
use byteorder::{BigEndian, WriteBytesExt};
use std::io::{Seek, SeekFrom, Write};

/// Write INF format file from a pattern
///
/// # Arguments
///
/// * `pattern` - The pattern to write (only thread data is used)
/// * `file` - The output file/stream to write to
///
/// # Example
///
/// ```no_run
/// use butabuti::prelude::*;
/// use std::fs::File;
///
/// let mut pattern = EmbPattern::new();
/// pattern.add_thread(
///     EmbThread::from_rgb(255, 0, 0)
///         .with_description("Red Thread")
///         .with_chart("Chart1")
/// );
///
/// let mut file = File::create("threads.inf").unwrap();
/// butabuti::io::writers::inf::write(&pattern, &mut file).unwrap();
/// ```
pub fn write(pattern: &EmbPattern, file: &mut (impl Write + Seek)) -> Result<()> {
    let threads = pattern.threads();

    // Write header
    file.write_u32::<BigEndian>(1)?;
    file.write_u32::<BigEndian>(8)?;

    // Placeholder for offset (will be patched later)
    let placeholder_pos = file.stream_position()?;
    file.write_u32::<BigEndian>(0)?;

    file.write_u32::<BigEndian>(threads.len() as u32)?;

    // Write each thread record
    for (index, thread) in threads.iter().enumerate() {
        let description = thread.description.as_deref().unwrap_or("Unknown");
        let chart = thread.chart.as_deref().unwrap_or("Unknown");

        // Calculate record length: 2 + 2 + 1 + 1 + 1 + 2 + desc + 1 + chart + 1 = 11 + desc.len() + chart.len()
        let record_length = 11 + description.len() + chart.len();

        file.write_u16::<BigEndian>(record_length as u16)?;
        file.write_u16::<BigEndian>(index as u16)?; // record index
        file.write_u8(thread.red())?;
        file.write_u8(thread.green())?;
        file.write_u8(thread.blue())?;
        file.write_u16::<BigEndian>((index + 1) as u16)?; // needle number (1-indexed)
        file.write_all(description.as_bytes())?;
        file.write_u8(0)?; // null terminator
        file.write_all(chart.as_bytes())?;
        file.write_u8(0)?; // null terminator
    }

    // Patch the placeholder with the offset
    let current_pos = file.stream_position()?;
    file.seek(SeekFrom::Start(placeholder_pos))?;
    let offset = (current_pos - placeholder_pos - 4) as u32;
    file.write_u32::<BigEndian>(offset)?;
    file.seek(SeekFrom::Start(current_pos))?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    #[test]
    fn test_write_inf_basic() {
        let mut pattern = EmbPattern::new();
        pattern.add_thread(
            crate::core::thread::EmbThread::from_rgb(255, 0, 0)
                .with_description("Red Thread")
                .with_chart("Chart1"),
        );

        let mut output = Cursor::new(Vec::new());
        write(&pattern, &mut output).unwrap();

        let data = output.into_inner();

        // Check header
        assert!(data.len() > 16); // At least header + 1 thread

        // First u32 should be 1
        assert_eq!(u32::from_be_bytes([data[0], data[1], data[2], data[3]]), 1);
    }

    #[test]
    fn test_write_inf_multiple_threads() {
        let mut pattern = EmbPattern::new();
        pattern.add_thread(
            crate::core::thread::EmbThread::from_rgb(255, 0, 0)
                .with_description("Red")
                .with_chart("C1"),
        );
        pattern.add_thread(
            crate::core::thread::EmbThread::from_rgb(0, 255, 0)
                .with_description("Green")
                .with_chart("C2"),
        );

        let mut output = Cursor::new(Vec::new());
        write(&pattern, &mut output).unwrap();

        let data = output.into_inner();

        // Should have data for 2 threads
        assert!(data.len() > 16);
    }

    #[test]
    fn test_inf_round_trip() {
        use crate::formats::io::readers::inf;

        // Create original pattern
        let mut original = EmbPattern::new();
        original.add_thread(
            crate::core::thread::EmbThread::from_rgb(255, 0, 0)
                .with_description("Red Thread")
                .with_chart("Chart1"),
        );
        original.add_thread(
            crate::core::thread::EmbThread::from_rgb(0, 0, 255)
                .with_description("Blue Thread")
                .with_chart("Chart2"),
        );

        // Write to buffer
        let mut buffer = Cursor::new(Vec::new());
        write(&original, &mut buffer).unwrap();

        // Read back
        buffer.set_position(0);
        let mut read_back = EmbPattern::new();
        inf::read(&mut buffer, &mut read_back).unwrap();

        // Verify thread count
        assert_eq!(read_back.threads().len(), original.threads().len());

        // Verify colors and metadata
        for (i, thread) in read_back.threads().iter().enumerate() {
            assert_eq!(thread.red(), original.threads()[i].red());
            assert_eq!(thread.green(), original.threads()[i].green());
            assert_eq!(thread.blue(), original.threads()[i].blue());
            assert_eq!(thread.description, original.threads()[i].description);
            assert_eq!(thread.chart, original.threads()[i].chart);
        }
    }
}
