//! INF thread information format reader
//!
//! INF is a binary format storing detailed thread information (RGB colors, descriptions,
//! catalog numbers) with no stitch data. Uses variable-length string fields.
//!
//! ## Format Limitations
//!
//! - **No stitches**: INF only stores thread metadata, no stitch data
//! - **Max threads**: Limited to 10,000 threads (safety limit)
//! - **Max record size**: Each thread limited to 65,535 bytes (u16 length)
//! - **Binary format**: Big-endian encoding, not human-readable
//! - **Variable length**: String fields are variable-length with null terminators

use crate::core::pattern::EmbPattern;
use crate::core::thread::EmbThread;
use crate::utils::error::{Error, Result};
use byteorder::{BigEndian, ReadBytesExt};
use std::io::Read;

// Format constants
const MAX_INF_THREADS: usize = 10_000; // Safety limit for thread count
const MIN_INF_RECORD_SIZE: usize = 5; // Minimum: index(2) + RGB(3)

/// Read INF format file into a pattern
///
/// # Arguments
///
/// * `file` - The input file/stream to read from
/// * `pattern` - The pattern to populate with thread data
///
/// # Example
///
/// ```no_run
/// use butabuti::prelude::*;
/// use std::fs::File;
///
/// let mut file = File::open("threads.inf").unwrap();
/// let mut pattern = EmbPattern::new();
/// butabuti::formats::io::readers::inf::read(&mut file, &mut pattern).unwrap();
/// ```
pub fn read(file: &mut impl Read, pattern: &mut EmbPattern) -> Result<()> {
    // Read header
    let _u0 = file.read_u32::<BigEndian>()?;
    let _u1 = file.read_u32::<BigEndian>()?;
    let _u2 = file.read_u32::<BigEndian>()?;
    let number_of_colors = file.read_u32::<BigEndian>()?;

    // Validate thread count
    if number_of_colors > MAX_INF_THREADS as u32 {
        return Err(Error::Parse(format!(
            "INF: Thread count {} exceeds maximum of {}",
            number_of_colors, MAX_INF_THREADS
        )));
    }

    // Read each thread record
    for thread_idx in 0..number_of_colors {
        let length = file.read_u16::<BigEndian>()? as usize;
        if length < 2 {
            return Err(Error::Parse(format!(
                "INF: Thread {} has invalid record length {}",
                thread_idx, length
            )));
        }

        let data_length = length - 2; // Subtract the 2 bytes of length itself

        if data_length < MIN_INF_RECORD_SIZE {
            return Err(Error::Parse(format!(
                "INF: Thread {} record too small ({} bytes, min {})",
                thread_idx, data_length, MIN_INF_RECORD_SIZE
            )));
        }

        let mut byte_data = vec![0u8; data_length];

        if file.read_exact(&mut byte_data).is_err() {
            return Err(Error::Parse(format!(
                "INF: Unexpected EOF reading thread {} data",
                thread_idx
            )));
        }

        if byte_data.len() < MIN_INF_RECORD_SIZE {
            return Err(Error::Parse(format!(
                "INF: Thread {} data truncated",
                thread_idx
            )));
        }

        // Parse RGB at positions 2, 3, 4
        let red = byte_data[2];
        let green = byte_data[3];
        let blue = byte_data[4];

        let mut thread = EmbThread::from_rgb(red, green, blue);

        // Skip to position 7 for description
        if byte_data.len() > 7 {
            let mut data = &byte_data[7..];

            // Read description (null-terminated string)
            if let Some(null_pos) = data.iter().position(|&b| b == 0) {
                if null_pos > 0 {
                    if let Ok(desc) = std::str::from_utf8(&data[..null_pos]) {
                        thread = thread.with_description(desc);
                    }
                }

                // Move past description and null terminator
                if data.len() > null_pos + 1 {
                    data = &data[null_pos + 1..];

                    // Read chart (null-terminated string)
                    if let Some(chart_null) = data.iter().position(|&b| b == 0) {
                        if chart_null > 0 {
                            if let Ok(chart) = std::str::from_utf8(&data[..chart_null]) {
                                thread = thread.with_chart(chart);
                            }
                        }
                    }
                }
            }
        }

        pattern.add_thread(thread);
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    #[test]
    fn test_read_inf_basic() {
        // Create minimal INF data
        let mut data = Vec::new();

        // Header: 3 u32 values + color count
        data.extend_from_slice(&1u32.to_be_bytes());
        data.extend_from_slice(&8u32.to_be_bytes());
        data.extend_from_slice(&0u32.to_be_bytes());
        data.extend_from_slice(&1u32.to_be_bytes()); // 1 color

        // Thread record
        let desc = b"Red Thread";
        let chart = b"Chart1";
        let record_length = (11 + desc.len() + chart.len()) as u16;

        data.extend_from_slice(&record_length.to_be_bytes());
        data.extend_from_slice(&0u16.to_be_bytes()); // index
        data.push(255); // red
        data.push(0); // green
        data.push(0); // blue
        data.extend_from_slice(&1u16.to_be_bytes()); // needle number
        data.extend_from_slice(desc);
        data.push(0); // null terminator
        data.extend_from_slice(chart);
        data.push(0); // null terminator

        let mut cursor = Cursor::new(data);
        let mut pattern = EmbPattern::new();

        read(&mut cursor, &mut pattern).unwrap();

        assert_eq!(pattern.threads().len(), 1);
        assert_eq!(pattern.threads()[0].red(), 255);
        assert_eq!(
            pattern.threads()[0].description.as_deref(),
            Some("Red Thread")
        );
        assert_eq!(pattern.threads()[0].chart.as_deref(), Some("Chart1"));
    }

    #[test]
    fn test_read_inf_multiple_threads() {
        let mut data = Vec::new();

        // Header
        data.extend_from_slice(&1u32.to_be_bytes());
        data.extend_from_slice(&8u32.to_be_bytes());
        data.extend_from_slice(&0u32.to_be_bytes());
        data.extend_from_slice(&2u32.to_be_bytes()); // 2 colors

        // Thread 1
        let desc1 = b"Red";
        let chart1 = b"C1";
        let len1 = (11 + desc1.len() + chart1.len()) as u16;
        data.extend_from_slice(&len1.to_be_bytes());
        data.extend_from_slice(&0u16.to_be_bytes());
        data.push(255);
        data.push(0);
        data.push(0);
        data.extend_from_slice(&1u16.to_be_bytes());
        data.extend_from_slice(desc1);
        data.push(0);
        data.extend_from_slice(chart1);
        data.push(0);

        // Thread 2
        let desc2 = b"Blue";
        let chart2 = b"C2";
        let len2 = (11 + desc2.len() + chart2.len()) as u16;
        data.extend_from_slice(&len2.to_be_bytes());
        data.extend_from_slice(&1u16.to_be_bytes());
        data.push(0);
        data.push(0);
        data.push(255);
        data.extend_from_slice(&2u16.to_be_bytes());
        data.extend_from_slice(desc2);
        data.push(0);
        data.extend_from_slice(chart2);
        data.push(0);

        let mut cursor = Cursor::new(data);
        let mut pattern = EmbPattern::new();

        read(&mut cursor, &mut pattern).unwrap();

        assert_eq!(pattern.threads().len(), 2);
        assert_eq!(pattern.threads()[0].red(), 255);
        assert_eq!(pattern.threads()[1].blue(), 255);
    }
}
