//! CSV format writer for embroidery patterns
//!
//! Writes human-readable comma-separated values for debugging and analysis.
//! Each line represents a stitch with X, Y, command name, and thread index.

use crate::core::constants::*;
use crate::core::pattern::EmbPattern;
use crate::utils::error::Result;
use std::io::Write;

/// CSV output version
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum CsvVersion {
    /// Default version: command, x, y
    Default,
    /// Delta version: command, dx, dy (relative coordinates)
    Delta,
    /// Full version: includes all stitch data and command details
    Full,
}

/// Write pattern to CSV format
pub fn write<W: Write>(writer: &mut W, pattern: &EmbPattern, version: CsvVersion) -> Result<()> {
    match version {
        CsvVersion::Default => write_default(writer, pattern),
        CsvVersion::Delta => write_delta(writer, pattern),
        CsvVersion::Full => write_full(writer, pattern),
    }
}

/// Write default CSV format: command, x, y
fn write_default<W: Write>(writer: &mut W, pattern: &EmbPattern) -> Result<()> {
    // Write header
    writeln!(writer, "command,x,y")?;

    // Write stitches
    for stitch in pattern.stitches() {
        let command_name = command_name(stitch.command & COMMAND_MASK);
        writeln!(writer, "{},{},{}", command_name, stitch.x, stitch.y)?;
    }

    Ok(())
}

/// Write delta CSV format: command, dx, dy
fn write_delta<W: Write>(writer: &mut W, pattern: &EmbPattern) -> Result<()> {
    // Write header
    writeln!(writer, "command,dx,dy")?;

    let mut prev_x = 0.0;
    let mut prev_y = 0.0;

    // Write stitches
    for stitch in pattern.stitches() {
        let command_name = command_name(stitch.command & COMMAND_MASK);
        let dx = stitch.x - prev_x;
        let dy = stitch.y - prev_y;

        writeln!(writer, "{},{},{}", command_name, dx, dy)?;

        prev_x = stitch.x;
        prev_y = stitch.y;
    }

    Ok(())
}

/// Write full CSV format: includes all data
fn write_full<W: Write>(writer: &mut W, pattern: &EmbPattern) -> Result<()> {
    // Write metadata
    writeln!(writer, "# Metadata")?;
    for (key, value) in pattern.extras() {
        writeln!(writer, "# {}: {}", key, value)?;
    }
    writeln!(writer)?;

    // Write threads
    writeln!(writer, "# Threads")?;
    for (i, thread) in pattern.threads().iter().enumerate() {
        writeln!(writer, "# Thread {}: #{:06X}", i, thread.color & 0xFFFFFF)?;
    }
    writeln!(writer)?;

    // Write header
    writeln!(writer, "index,command,x,y,dx,dy,color_index")?;

    let mut prev_x = 0.0;
    let mut prev_y = 0.0;
    let mut color_index = 0;

    // Write stitches
    for (i, stitch) in pattern.stitches().iter().enumerate() {
        let command = stitch.command & COMMAND_MASK;
        let command_name = command_name(command);
        let dx = stitch.x - prev_x;
        let dy = stitch.y - prev_y;

        writeln!(
            writer,
            "{},{},{},{},{},{},{}",
            i, command_name, stitch.x, stitch.y, dx, dy, color_index
        )?;

        // Track color changes
        if command == COLOR_CHANGE {
            color_index += 1;
        }

        prev_x = stitch.x;
        prev_y = stitch.y;
    }

    Ok(())
}

/// Write CSV file to path
pub fn write_file(path: &str, pattern: &EmbPattern, version: CsvVersion) -> Result<()> {
    let file = std::fs::File::create(path)?;
    let mut writer = std::io::BufWriter::new(file);
    write(&mut writer, pattern, version)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::thread::EmbThread;
    use std::io::Cursor;

    #[test]
    fn test_csv_default() {
        let mut pattern = EmbPattern::new();
        pattern.add_thread(EmbThread::new(0xFF0000));
        pattern.add_stitch_absolute(STITCH, 10.0, 20.0);
        pattern.add_stitch_absolute(STITCH, 30.0, 40.0);
        pattern.add_stitch_absolute(END, 30.0, 40.0);

        let mut buffer = Cursor::new(Vec::new());
        write(&mut buffer, &pattern, CsvVersion::Default).unwrap();

        let output = String::from_utf8(buffer.into_inner()).unwrap();
        assert!(output.contains("command,x,y"));
        assert!(output.contains("STITCH,10,20"));
        assert!(output.contains("STITCH,30,40"));
        assert!(output.contains("END,30,40"));
    }

    #[test]
    fn test_csv_delta() {
        let mut pattern = EmbPattern::new();
        pattern.add_thread(EmbThread::new(0x00FF00));
        pattern.add_stitch_absolute(STITCH, 10.0, 10.0);
        pattern.add_stitch_absolute(STITCH, 20.0, 30.0);
        pattern.add_stitch_absolute(END, 20.0, 30.0);

        let mut buffer = Cursor::new(Vec::new());
        write(&mut buffer, &pattern, CsvVersion::Delta).unwrap();

        let output = String::from_utf8(buffer.into_inner()).unwrap();
        assert!(output.contains("command,dx,dy"));
        assert!(output.contains("STITCH,10,10"));
        assert!(output.contains("STITCH,10,20"));
    }

    #[test]
    fn test_csv_full() {
        let mut pattern = EmbPattern::new();
        pattern.add_metadata("name", "Test Pattern");
        pattern.add_metadata("author", "Test Author");
        pattern.add_thread(EmbThread::new(0xFF0000));
        pattern.add_thread(EmbThread::new(0x00FF00));
        pattern.add_stitch_absolute(STITCH, 5.0, 5.0);
        pattern.add_stitch_absolute(COLOR_CHANGE, 5.0, 5.0);
        pattern.add_stitch_absolute(STITCH, 15.0, 15.0);
        pattern.add_stitch_absolute(END, 15.0, 15.0);

        let mut buffer = Cursor::new(Vec::new());
        write(&mut buffer, &pattern, CsvVersion::Full).unwrap();

        let output = String::from_utf8(buffer.into_inner()).unwrap();
        assert!(output.contains("# Metadata"));
        assert!(output.contains("# name: Test Pattern"));
        assert!(output.contains("# author: Test Author"));
        assert!(output.contains("# Threads"));
        assert!(output.contains("# Thread 0: #FF0000"));
        assert!(output.contains("# Thread 1: #00FF00"));
        assert!(output.contains("index,command,x,y,dx,dy,color_index"));
        assert!(output.contains("COLOR_CHANGE"));
    }

    #[test]
    fn test_csv_empty_pattern() {
        let pattern = EmbPattern::new();
        let mut buffer = Cursor::new(Vec::new());
        write(&mut buffer, &pattern, CsvVersion::Default).unwrap();

        let output = String::from_utf8(buffer.into_inner()).unwrap();
        assert!(output.contains("command,x,y"));
    }
}
