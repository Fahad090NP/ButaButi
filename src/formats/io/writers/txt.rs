// TXT writer - Human-readable text format for inspection and debugging

//! TXT human-readable text format writer for embroidery patterns
//!
//! Writes plain text format for inspection and debugging with stitch coordinates,
//! command names, and thread information in readable format.

use crate::core::constants::*;
use crate::core::pattern::EmbPattern;
use crate::utils::error::Result;
use std::io::Write;

/// Settings for TXT writer
#[derive(Default)]
pub struct TxtSettings {
    /// Use embroidermodder-compatible format
    pub mimic: bool,
}

/// Write TXT embroidery format
///
/// TXT is a human-readable text format showing stitches and commands.
/// Two variants:
/// - Default: Shows all stitch data with color and command names
/// - Mimic (embroidermodder): Simplified numeric format
pub fn write(pattern: &EmbPattern, file: &mut impl Write) -> Result<()> {
    write_with_settings(pattern, file, TxtSettings::default())
}

/// Write TXT with custom settings
pub fn write_with_settings(
    pattern: &EmbPattern,
    file: &mut impl Write,
    settings: TxtSettings,
) -> Result<()> {
    if settings.mimic {
        write_mimic(pattern, file)
    } else {
        write_normal(pattern, file)
    }
}

/// Write in embroidermodder-compatible format
fn write_mimic(pattern: &EmbPattern, file: &mut impl Write) -> Result<()> {
    let mut color = 0;

    for stitch in pattern.stitches() {
        let x = stitch.x / 10.0; // Convert to mm
        let y = stitch.y / 10.0;
        let command = stitch.command & COMMAND_MASK;

        if command == COLOR_CHANGE {
            color += 1;
        }

        // Map command to embroidermodder flags
        let flags = match command {
            STITCH => 0,
            JUMP => 1,
            TRIM => 2,
            STOP | COLOR_CHANGE | NEEDLE_SET => 4,
            SEQUIN_MODE | SEQUIN_EJECT => 8,
            END => 16,
            _ => 0,
        };

        writeln!(file, "{:.1},{:.1} color:{} flags:{}", x, y, color, flags)?;
    }

    Ok(())
}

/// Write in normal detailed format
fn write_normal(pattern: &EmbPattern, file: &mut impl Write) -> Result<()> {
    let mut color_index = 0;
    let mut color = if pattern.threads().is_empty() {
        0
    } else {
        pattern.threads()[0].color
    };

    for stitch in pattern.stitches() {
        let x = stitch.x;
        let y = stitch.y;
        let command = stitch.command & COMMAND_MASK;

        if command == COLOR_CHANGE {
            color_index += 1;
            if color_index < pattern.threads().len() {
                color = pattern.threads()[color_index].color;
            }
        }

        let command_name = command_name(command);

        writeln!(
            file,
            "{:.1},{:.1} color:{} command:{} flags:{}",
            x, y, color, command_name, command
        )?;
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::thread::EmbThread;

    #[test]
    fn test_write_normal_txt() {
        let mut pattern = EmbPattern::new();
        pattern.add_thread(EmbThread::from_rgb(255, 0, 0));
        pattern.add_thread(EmbThread::from_rgb(0, 255, 0));
        pattern.add_stitch_absolute(STITCH, 100.0, 200.0);
        pattern.add_stitch_absolute(STITCH, 110.0, 210.0);
        pattern.add_command(COLOR_CHANGE, 0.0, 0.0);
        pattern.add_stitch_absolute(STITCH, 120.0, 220.0);
        pattern.add_command(END, 0.0, 0.0);

        let mut output = Vec::new();
        write(&pattern, &mut output).expect("Failed to write TXT");

        let text = String::from_utf8(output).expect("Invalid UTF-8");
        assert!(text.contains("STITCH"));
        assert!(text.contains("COLOR_CHANGE"));
        assert!(text.contains("END"));
    }

    #[test]
    fn test_write_mimic_txt() {
        let mut pattern = EmbPattern::new();
        pattern.add_thread(EmbThread::from_rgb(255, 0, 0));
        pattern.add_stitch_absolute(STITCH, 100.0, 200.0);
        pattern.add_stitch_absolute(JUMP, 150.0, 250.0);
        pattern.add_command(COLOR_CHANGE, 0.0, 0.0);
        pattern.add_stitch_absolute(STITCH, 120.0, 220.0);

        let mut output = Vec::new();
        write_with_settings(&pattern, &mut output, TxtSettings { mimic: true })
            .expect("Failed to write TXT");

        let text = String::from_utf8(output).expect("Invalid UTF-8");
        // In mimic mode, coordinates are in mm (divided by 10)
        assert!(text.contains("10.0,20.0")); // 100.0/10, 200.0/10
        assert!(text.contains("color:"));
        assert!(text.contains("flags:"));
    }

    #[test]
    fn test_write_empty_txt() {
        let pattern = EmbPattern::new();

        let mut output = Vec::new();
        write(&pattern, &mut output).expect("Failed to write empty TXT");

        assert_eq!(output.len(), 0);
    }
}
