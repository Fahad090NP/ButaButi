//! COL color list format writer
//!
//! Writes simple text format with thread colors only (no stitches). Format consists
//! of a count line followed by index,R,G,B entries for each thread.

use crate::core::pattern::EmbPattern;
use crate::utils::error::Result;
use std::io::Write;

/// Write COL format file from a pattern
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
/// pattern.add_thread(EmbThread::from_rgb(255, 0, 0));
/// pattern.add_thread(EmbThread::from_rgb(0, 255, 0));
///
/// let mut file = File::create("colors.col").unwrap();
/// butabuti::io::writers::col::write(&pattern, &mut file).unwrap();
/// ```
pub fn write(pattern: &EmbPattern, file: &mut impl Write) -> Result<()> {
    let threads = pattern.threads();

    // Write thread count
    writeln!(file, "{}", threads.len())?;

    // Write each thread
    for (index, thread) in threads.iter().enumerate() {
        writeln!(
            file,
            "{},{},{},{}",
            index,
            thread.red(),
            thread.green(),
            thread.blue()
        )?;
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    #[test]
    fn test_write_col_basic() {
        let mut pattern = EmbPattern::new();
        pattern.add_thread(crate::core::thread::EmbThread::from_rgb(255, 0, 0));
        pattern.add_thread(crate::core::thread::EmbThread::from_rgb(0, 255, 0));
        pattern.add_thread(crate::core::thread::EmbThread::from_rgb(0, 0, 255));

        let mut output = Cursor::new(Vec::new());
        write(&pattern, &mut output).unwrap();

        let result = String::from_utf8(output.into_inner()).unwrap();

        assert!(result.starts_with("3\n"));
        assert!(result.contains("0,255,0,0"));
        assert!(result.contains("1,0,255,0"));
        assert!(result.contains("2,0,0,255"));
    }

    #[test]
    fn test_write_col_empty() {
        let pattern = EmbPattern::new();

        let mut output = Cursor::new(Vec::new());
        write(&pattern, &mut output).unwrap();

        let result = String::from_utf8(output.into_inner()).unwrap();

        assert_eq!(result.trim(), "0");
    }

    #[test]
    fn test_col_round_trip() {
        use crate::formats::io::readers::col;

        // Create original pattern
        let mut original = EmbPattern::new();
        original.add_thread(crate::core::thread::EmbThread::from_rgb(255, 0, 0));
        original.add_thread(crate::core::thread::EmbThread::from_rgb(0, 255, 0));
        original.add_thread(crate::core::thread::EmbThread::from_rgb(0, 0, 255));

        // Write to buffer
        let mut buffer = Cursor::new(Vec::new());
        write(&original, &mut buffer).unwrap();

        // Read back
        buffer.set_position(0);
        let mut read_back = EmbPattern::new();
        col::read(&mut buffer, &mut read_back).unwrap();

        // Verify thread count
        assert_eq!(read_back.threads().len(), original.threads().len());

        // Verify colors match
        for (i, thread) in read_back.threads().iter().enumerate() {
            assert_eq!(thread.red(), original.threads()[i].red());
            assert_eq!(thread.green(), original.threads()[i].green());
            assert_eq!(thread.blue(), original.threads()[i].blue());
        }
    }
}
