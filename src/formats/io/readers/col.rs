//! COL color list format reader
//!
//! COL is a simple text format storing thread colors only (no stitches). Format consists
//! of a count line followed by catalog_number,R,G,B entries for each thread.
//!
//! ## Format Limitations
//!
//! - **No stitches**: COL only stores thread colors, no stitch data
//! - **Max threads**: Limited to 10,000 threads (safety limit)
//! - **Text format**: Line-based parsing, no binary data
//! - **No metadata**: No pattern name, size, or other attributes

use crate::core::pattern::EmbPattern;
use crate::core::thread::EmbThread;
use crate::utils::error::{Error, Result};
use std::io::{BufRead, BufReader, Read};

// Format constants
const MAX_THREADS: usize = 10_000; // Safety limit for thread count

/// Read COL format file into a pattern
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
/// let mut file = File::open("colors.col").unwrap();
/// let mut pattern = EmbPattern::new();
/// butabuti::formats::io::readers::col::read(&mut file, &mut pattern).unwrap();
/// ```
pub fn read(file: &mut impl Read, pattern: &mut EmbPattern) -> Result<()> {
    let reader = BufReader::new(file);
    let mut lines = reader.lines();

    // Read first line: color count
    if let Some(Ok(first_line)) = lines.next() {
        let count: usize = first_line
            .trim()
            .parse()
            .map_err(|e| Error::Parse(format!("COL: Invalid thread count in header: {}", e)))?;

        // Validate thread count
        if count > MAX_THREADS {
            return Err(Error::Parse(format!(
                "COL: Thread count {} exceeds maximum of {}",
                count, MAX_THREADS
            )));
        }

        // Read each thread definition
        for line_num in 0..count {
            if let Some(Ok(line)) = lines.next() {
                let parts: Vec<&str> = line.split(',').collect();

                if parts.len() < 4 {
                    return Err(Error::Parse(format!(
                        "COL: Invalid thread line {} - expected 4 fields (catalog,R,G,B), got {}",
                        line_num + 2, // +2 because line 1 is count, threads start at line 2
                        parts.len()
                    )));
                }

                // Parse: catalog_number, R, G, B
                let catalog = parts[0].trim();

                let r = parts[1].trim().parse::<u8>().map_err(|e| {
                    Error::Parse(format!(
                        "COL: Invalid red value on line {}: {}",
                        line_num + 2,
                        e
                    ))
                })?;

                let g = parts[2].trim().parse::<u8>().map_err(|e| {
                    Error::Parse(format!(
                        "COL: Invalid green value on line {}: {}",
                        line_num + 2,
                        e
                    ))
                })?;

                let b = parts[3].trim().parse::<u8>().map_err(|e| {
                    Error::Parse(format!(
                        "COL: Invalid blue value on line {}: {}",
                        line_num + 2,
                        e
                    ))
                })?;

                let mut thread = EmbThread::from_rgb(r, g, b);
                if !catalog.is_empty() && catalog != "0" {
                    thread = thread.with_catalog_number(catalog);
                }
                pattern.add_thread(thread);
            } else {
                return Err(Error::Parse(format!(
                    "COL: Unexpected end of file - expected {} threads, got {}",
                    count, line_num
                )));
            }
        }
    } else {
        return Err(Error::Parse(
            "COL: Empty file - expected thread count header".to_string(),
        ));
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    #[test]
    fn test_read_col_basic() {
        let col_data = "3\r\n\
0,255,0,0\r\n\
1,0,255,0\r\n\
2,0,0,255\r\n";

        let mut cursor = Cursor::new(col_data.as_bytes());
        let mut pattern = EmbPattern::new();

        read(&mut cursor, &mut pattern).unwrap();

        assert_eq!(pattern.threads().len(), 3);
        assert_eq!(pattern.threads()[0].red(), 255);
        assert_eq!(pattern.threads()[1].green(), 255);
        assert_eq!(pattern.threads()[2].blue(), 255);
    }

    #[test]
    fn test_read_col_with_catalog() {
        let col_data = "2\r\n\
1001,255,0,0\r\n\
1002,0,0,255\r\n";

        let mut cursor = Cursor::new(col_data.as_bytes());
        let mut pattern = EmbPattern::new();

        read(&mut cursor, &mut pattern).unwrap();

        assert_eq!(pattern.threads().len(), 2);
        assert_eq!(pattern.threads()[0].catalog_number.as_deref(), Some("1001"));
        assert_eq!(pattern.threads()[1].catalog_number.as_deref(), Some("1002"));
    }

    #[test]
    fn test_read_col_empty() {
        let col_data = "0\r\n";

        let mut cursor = Cursor::new(col_data.as_bytes());
        let mut pattern = EmbPattern::new();

        read(&mut cursor, &mut pattern).unwrap();

        assert_eq!(pattern.threads().len(), 0);
    }
}
