//! COL color list format reader
//!
//! COL is a simple text format storing thread colors only (no stitches). Format consists
//! of a count line followed by catalog_number,R,G,B entries for each thread.

use crate::core::pattern::EmbPattern;
use crate::core::thread::EmbThread;
use anyhow::Result;
use std::io::{BufRead, BufReader, Read};

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
/// butabuti::io::readers::col::read(&mut file, &mut pattern).unwrap();
/// ```
pub fn read(file: &mut impl Read, pattern: &mut EmbPattern) -> Result<()> {
    let reader = BufReader::new(file);
    let mut lines = reader.lines();

    // Read first line: color count
    if let Some(Ok(first_line)) = lines.next() {
        let count: usize = first_line.trim().parse().unwrap_or(0);

        // Read each thread definition
        for _ in 0..count {
            if let Some(Ok(line)) = lines.next() {
                let parts: Vec<&str> = line.split(',').collect();

                if parts.len() >= 4 {
                    // Parse: catalog_number, R, G, B
                    let catalog = parts[0].trim();

                    if let (Ok(r), Ok(g), Ok(b)) = (
                        parts[1].trim().parse::<u8>(),
                        parts[2].trim().parse::<u8>(),
                        parts[3].trim().parse::<u8>(),
                    ) {
                        let mut thread = EmbThread::from_rgb(r, g, b);
                        if !catalog.is_empty() && catalog != "0" {
                            thread = thread.with_catalog_number(catalog);
                        }
                        pattern.add_thread(thread);
                    }
                }
            }
        }
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
