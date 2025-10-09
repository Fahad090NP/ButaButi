//! G-code embroidery format writer
//!
//! Writes G-code CNC machine language adapted for embroidery with configurable Z-axis
//! travel, G00/G01 for movement/stitches, M00/M01 for color changes, and thread metadata comments.

use crate::core::constants::*;
use crate::core::encoder::EncoderSettings;
use crate::core::pattern::EmbPattern;
use crate::utils::functions::decode_embroidery_command;
use anyhow::Result;
use std::io::Write;

/// Get default encoder settings for G-code format
pub fn default_settings() -> EncoderSettings {
    EncoderSettings {
        sequin_contingency: CONTINGENCY_SEQUIN_STITCH,
        ..Default::default()
    }
}

/// Write G-code format file from a pattern
///
/// # Arguments
///
/// * `pattern` - The pattern to write
/// * `file` - The output file/stream to write to
///
/// # Example
///
/// ```no_run
/// use butabuti::prelude::*;
/// use std::fs::File;
///
/// let mut pattern = EmbPattern::new();
/// pattern.add_thread(EmbThread::from_hex("#FF0000"));
/// pattern.add_stitch_absolute(STITCH, 0.0, 0.0);
/// pattern.add_stitch_absolute(STITCH, 10.0, 10.0);
/// pattern.end();
///
/// let mut file = File::create("output.gcode").unwrap();
/// butabuti::io::writers::gcode::write(&pattern, &mut file).unwrap();
/// ```
pub fn write(pattern: &EmbPattern, file: &mut impl Write) -> Result<()> {
    write_with_settings(pattern, file, 10.0)
}

/// Write G-code with custom Z-axis travel increment
///
/// # Arguments
///
/// * `pattern` - The pattern to write
/// * `file` - The output file/stream to write to
/// * `stitch_z_travel` - Z-axis increment per stitch (default 10.0)
pub fn write_with_settings(
    pattern: &EmbPattern,
    file: &mut impl Write,
    stitch_z_travel: f64,
) -> Result<()> {
    // Write header comments with pattern data
    write_header(pattern, file)?;

    // Write metadata comments
    write_metadata(pattern, file)?;

    // Write thread information
    write_threads(pattern, file)?;

    // Write stitch data as G-code commands
    write_stitches(pattern, file, stitch_z_travel)?;

    Ok(())
}

/// Write header with pattern statistics
fn write_header(pattern: &EmbPattern, file: &mut impl Write) -> Result<()> {
    // Convert bounds to millimeters (divide by 10)
    let bounds = pattern.bounds();
    let bounds_mm = (
        bounds.0 / 10.0,
        bounds.1 / 10.0,
        bounds.2 / 10.0,
        bounds.3 / 10.0,
    );

    let width = bounds_mm.2 - bounds_mm.0;
    let height = bounds_mm.3 - bounds_mm.1;

    // Count stitches and color changes
    let stitch_count = pattern
        .stitches()
        .iter()
        .filter(|s| (s.command & COMMAND_MASK) == STITCH)
        .count();

    let color_count = pattern
        .stitches()
        .iter()
        .filter(|s| (s.command & COMMAND_MASK) == COLOR_CHANGE)
        .count();

    writeln!(file, "(STITCH_COUNT: {})", stitch_count)?;
    writeln!(file, "(THREAD_COUNT: {})", color_count)?;
    writeln!(file, "(EXTENTS_LEFT: {:.3})", bounds_mm.0)?;
    writeln!(file, "(EXTENTS_TOP: {:.3})", bounds_mm.1)?;
    writeln!(file, "(EXTENTS_RIGHT: {:.3})", bounds_mm.2)?;
    writeln!(file, "(EXTENTS_BOTTOM: {:.3})", bounds_mm.3)?;
    writeln!(file, "(EXTENTS_WIDTH: {:.3})", width)?;
    writeln!(file, "(EXTENTS_HEIGHT: {:.3})", height)?;

    // Write command statistics
    let mut command_counts = std::collections::HashMap::new();
    for stitch in pattern.stitches() {
        let cmd = stitch.command & COMMAND_MASK;
        *command_counts.entry(cmd).or_insert(0) += 1;
    }

    for (cmd, count) in command_counts.iter() {
        let name = command_name(*cmd);
        writeln!(file, "(COMMAND_{}: {})", name, count)?;
    }

    Ok(())
}

/// Get human-readable name for command
fn command_name(cmd: u32) -> &'static str {
    match cmd {
        STITCH => "STITCH",
        JUMP => "JUMP",
        TRIM => "TRIM",
        COLOR_CHANGE => "COLOR_CHANGE",
        NEEDLE_SET => "NEEDLE_SET",
        STOP => "STOP",
        END => "END",
        SEQUIN_MODE => "SEQUIN_MODE",
        SEQUIN_EJECT => "SEQUIN_EJECT",
        _ => "UNKNOWN",
    }
}

/// Write metadata as comments
fn write_metadata(pattern: &EmbPattern, file: &mut impl Write) -> Result<()> {
    for (key, value) in pattern.extras() {
        writeln!(file, "({}: {})", key, value)?;
    }
    Ok(())
}

/// Write thread information as comments
fn write_threads(pattern: &EmbPattern, file: &mut impl Write) -> Result<()> {
    for (i, thread) in pattern.threads().iter().enumerate() {
        writeln!(
            file,
            "(Thread{}: #{:02X}{:02X}{:02X} {} {} {})",
            i,
            thread.red(),
            thread.green(),
            thread.blue(),
            thread.description.as_deref().unwrap_or(""),
            thread.brand.as_deref().unwrap_or(""),
            thread.catalog_number.as_deref().unwrap_or(""),
        )?;
    }
    Ok(())
}

/// Write stitch data as G-code commands
fn write_stitches(pattern: &EmbPattern, file: &mut impl Write, stitch_z_travel: f64) -> Result<()> {
    let mut z = 0.0;

    for stitch in pattern.stitches() {
        // Convert to millimeters
        let x = stitch.x / 10.0;
        let y = stitch.y / 10.0;

        let (command, _, _, _) = decode_embroidery_command(stitch.command);

        match command {
            STITCH => {
                // G00 rapid move to position, then increment Z
                writeln!(file, "G00 X{:.3} Y{:.3}", x, y)?;
                writeln!(file, "G00 Z{:.1}", z)?;
                z += stitch_z_travel;
            }
            JUMP => {
                // Jumps are just skipped in G-code
                continue;
            }
            TRIM => {
                // Trims are skipped
                continue;
            }
            COLOR_CHANGE | STOP => {
                // M00 - Program stop (for color change)
                writeln!(file, "M00")?;
            }
            END => {
                // M30 - Program end
                writeln!(file, "M30")?;
                break;
            }
            _ => {
                // Unknown commands are skipped
                continue;
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
    fn test_write_gcode_basic() {
        let mut pattern = EmbPattern::new();
        pattern.add_thread(crate::core::thread::EmbThread::from_rgb(255, 0, 0));
        pattern.add_stitch_absolute(STITCH, 0.0, 0.0);
        pattern.add_stitch_absolute(STITCH, 100.0, 100.0);
        pattern.end();

        let mut output = Cursor::new(Vec::new());
        write(&pattern, &mut output).unwrap();

        let result = String::from_utf8(output.into_inner()).unwrap();

        // Check for expected content
        assert!(result.contains("STITCH_COUNT"));
        assert!(result.contains("G00"));
        assert!(result.contains("M30"));
    }

    #[test]
    fn test_write_gcode_with_metadata() {
        let mut pattern = EmbPattern::new();
        pattern.add_metadata("author", "TestUser");
        pattern.add_metadata("title", "TestPattern");
        pattern.add_thread(crate::core::thread::EmbThread::from_rgb(0, 255, 0));
        pattern.add_stitch_absolute(STITCH, 0.0, 0.0);
        pattern.end();

        let mut output = Cursor::new(Vec::new());
        write(&pattern, &mut output).unwrap();

        let result = String::from_utf8(output.into_inner()).unwrap();

        assert!(result.contains("author"));
        assert!(result.contains("TestUser"));
        assert!(result.contains("title"));
        assert!(result.contains("TestPattern"));
    }

    #[test]
    fn test_write_gcode_color_change() {
        let mut pattern = EmbPattern::new();
        pattern.add_thread(crate::core::thread::EmbThread::from_rgb(255, 0, 0));
        pattern.add_thread(crate::core::thread::EmbThread::from_rgb(0, 0, 255));
        pattern.add_stitch_absolute(STITCH, 0.0, 0.0);
        pattern.add_stitch_relative(0.0, 0.0, COLOR_CHANGE);
        pattern.add_stitch_absolute(STITCH, 50.0, 50.0);
        pattern.end();

        let mut output = Cursor::new(Vec::new());
        write(&pattern, &mut output).unwrap();

        let result = String::from_utf8(output.into_inner()).unwrap();

        assert!(result.contains("M00")); // Color change command
        assert!(result.contains("Thread0"));
        assert!(result.contains("Thread1"));
    }

    #[test]
    fn test_gcode_round_trip() {
        use crate::formats::io::readers::gcode;

        // Create original pattern
        let mut original = EmbPattern::new();
        original.add_thread(crate::core::thread::EmbThread::from_rgb(255, 0, 0));
        original.add_stitch_absolute(STITCH, 0.0, 0.0);
        original.add_stitch_absolute(STITCH, 100.0, 100.0);
        original.add_stitch_relative(0.0, 0.0, COLOR_CHANGE);
        original.add_stitch_absolute(STITCH, 200.0, 200.0);
        original.end();

        // Write to buffer
        let mut buffer = Cursor::new(Vec::new());
        write(&original, &mut buffer).unwrap();

        // Read back
        buffer.set_position(0);
        let mut read_back = EmbPattern::new();
        gcode::read(&mut buffer, &mut read_back).unwrap();

        // Verify we have stitches
        assert!(read_back.stitches().len() > 0);

        // Verify color change is preserved
        let commands: Vec<u32> = read_back
            .stitches()
            .iter()
            .map(|s| s.command & COMMAND_MASK)
            .collect();
        assert!(commands.contains(&COLOR_CHANGE));
    }
}
