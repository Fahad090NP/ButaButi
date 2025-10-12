//! Tajima TBF format writer
//!
//! Writes TBF format with ASCII header, thread definitions, and 3-byte stitch encoding.
//! Supports explicit TRIM commands and NEEDLE_SET for thread changes on industrial machines.

use crate::core::constants::*;
use crate::core::encoder::{EncoderSettings, Transcoder};
use crate::core::pattern::EmbPattern;
use crate::utils::error::Result;
use crate::utils::functions::decode_embroidery_command;
use std::io::{Seek, Write};

/// Get default encoder settings for TBF format
pub fn default_settings() -> EncoderSettings {
    EncoderSettings {
        max_stitch: 127.0,
        max_jump: 127.0,
        full_jump: false,
        round: true,
        writes_speeds: false,
        thread_change_command: NEEDLE_SET,
        explicit_trim: true,
        sequin_contingency: CONTINGENCY_SEQUIN_JUMP,
        long_stitch_contingency: CONTINGENCY_LONG_STITCH_SEW_TO,
        ..Default::default()
    }
}

/// Write TBF format file from a pattern
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
/// let mut file = File::create("output.tbf").unwrap();
/// butabuti::formats::io::writers::tbf::write(&pattern, &mut file).unwrap();
/// ```
pub fn write(pattern: &EmbPattern, file: &mut (impl Write + Seek)) -> Result<()> {
    // Encode pattern with TBF settings
    let mut transcoder = Transcoder::new();
    *transcoder.settings_mut() = default_settings();

    let mut encoded = EmbPattern::new();
    transcoder.transcode(pattern, &mut encoded)?;

    // Write header
    write_header(&encoded, file)?;

    // Write stitch data
    write_stitches(&encoded, file)?;

    // Terminal character
    file.write_all(b"\x1a")?;

    Ok(())
}

/// Write the TBF file header (0x600 bytes)
fn write_header(pattern: &EmbPattern, file: &mut (impl Write + Seek)) -> Result<()> {
    // Version string
    file.write_all(b"3.00")?;

    // Pad to 0x80 with spaces
    pad_to_position(file, 0x80)?;

    // Get metadata
    let name = pattern
        .get_metadata("name")
        .map_or("Untitled", |s| s.as_str());
    let bounds = pattern.bounds();

    // LA: Label/Name (%-16s means left-aligned, 16 chars)
    write_ascii_field(file, &format!("LA:{:<16}\r", name))?;

    // ST: Stitch count
    let stitch_count = pattern
        .stitches()
        .iter()
        .filter(|s| (s.command & COMMAND_MASK) == STITCH)
        .count();
    write_ascii_field(file, &format!("ST:{:>7}\r", stitch_count))?;

    // CO: Color (needle set) count
    let needle_count = count_needle_sets(pattern);
    write_ascii_field(file, &format!("CO:{:>3}\r", needle_count))?;

    // Bounds
    write_ascii_field(file, &format!("+X:{:>5}\r", bounds.2.abs() as i32))?;
    write_ascii_field(file, &format!("-X:{:>5}\r", bounds.0.abs() as i32))?;
    write_ascii_field(file, &format!("+Y:{:>5}\r", bounds.3.abs() as i32))?;
    write_ascii_field(file, &format!("-Y:{:>5}\r", bounds.1.abs() as i32))?;

    // AX, AY: Last stitch position
    let (ax, ay) = if let Some(last) = pattern.stitches().last() {
        (last.x as i32, -last.y as i32)
    } else {
        (0, 0)
    };

    if ax >= 0 {
        write_ascii_field(file, &format!("AX:+{:>5}\r", ax))?;
    } else {
        write_ascii_field(file, &format!("AX:-{:>5}\r", ax.abs()))?;
    }

    if ay >= 0 {
        write_ascii_field(file, &format!("AY:+{:>5}\r", ay))?;
    } else {
        write_ascii_field(file, &format!("AY:-{:>5}\r", ay.abs()))?;
    }

    // TP: Unknown field (default to "EG/")
    let tp = pattern.get_metadata("tp").map_or("EG/", |s| s.as_str());
    write_ascii_field(file, &format!("TP:{:<32}\r", tp))?;

    // JC: Unknown field (default to "3")
    write_ascii_field(file, "JC:3\r")?;

    // DO: Thread order (256 bytes)
    write_ascii_field(file, "DO:")?;
    let thread_order = build_thread_order(pattern);
    file.write_all(&thread_order)?;
    file.write_all(b"\r")?;

    // DA: Thread list
    write_ascii_field(file, "DA:")?;
    for thread in pattern.threads() {
        file.write_all(&[0x45])?; // Marker
        file.write_all(&[thread.red()])?;
        file.write_all(&[thread.green()])?;
        file.write_all(&[thread.blue()])?;
        file.write_all(&[0x20])?; // Space
    }

    // Pad to 0x376
    pad_to_position(file, 0x376)?;

    // Some files have this
    file.write_all(b"\x0d\x1a")?;

    // Pad to 0x600 (start of stitch data)
    pad_to_position(file, 0x600)?;

    Ok(())
}

/// Write stitch data
fn write_stitches(pattern: &EmbPattern, file: &mut impl Write) -> Result<()> {
    let mut xx = 0.0;
    let mut yy = 0.0;

    for stitch in pattern.stitches() {
        let x = stitch.x;
        let y = stitch.y;
        let command = stitch.command & COMMAND_MASK;

        let dx = (x - xx).round() as i32;
        let dy = (y - yy).round() as i32;
        xx += dx as f64;
        yy += dy as f64;

        let cmd = match command {
            STITCH => 0x80,
            JUMP => 0x90,
            STOP => 0x40,
            TRIM => 0x86,
            NEEDLE_SET => 0x81,
            END => {
                // Write END and break
                file.write_all(&[dx as i8 as u8, (-dy) as i8 as u8, 0x8F])?;
                break;
            },
            _ => continue, // Skip unknown commands
        };

        file.write_all(&[dx as i8 as u8, (-dy) as i8 as u8, cmd])?;
    }

    Ok(())
}

/// Count the number of NEEDLE_SET commands in the pattern
fn count_needle_sets(pattern: &EmbPattern) -> usize {
    pattern
        .stitches()
        .iter()
        .filter(|s| (s.command & COMMAND_MASK) == NEEDLE_SET)
        .count()
}

/// Build the thread order array (256 bytes)
fn build_thread_order(pattern: &EmbPattern) -> [u8; 256] {
    let mut thread_order = [0u8; 256];
    let mut index = 0;

    for stitch in pattern.stitches() {
        if (stitch.command & COMMAND_MASK) == NEEDLE_SET {
            let (_, _, needle, _) = decode_embroidery_command(stitch.command);
            if let Some(needle_num) = needle {
                if index < 256 {
                    thread_order[index] = needle_num;
                    index += 1;
                }
            }
        }
    }

    thread_order
}

/// Write an ASCII field to the file
fn write_ascii_field(file: &mut impl Write, text: &str) -> Result<()> {
    file.write_all(text.as_bytes())?;
    Ok(())
}

/// Pad file with spaces to reach a specific position
fn pad_to_position(file: &mut (impl Write + Seek), position: u64) -> Result<()> {
    let current = file.stream_position()?;
    if current < position {
        let padding = vec![0x20u8; (position - current) as usize];
        file.write_all(&padding)?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::utils::functions::encode_thread_change;
    use std::io::Cursor;

    #[test]
    fn test_write_tbf_basic() {
        let mut pattern = EmbPattern::new();
        pattern.add_thread(crate::core::thread::EmbThread::from_rgb(255, 0, 0));
        pattern.add_stitch_absolute(STITCH, 0.0, 0.0);
        pattern.add_stitch_absolute(STITCH, 10.0, 10.0);
        pattern.end();

        let mut output = Cursor::new(Vec::new());
        write(&pattern, &mut output).unwrap();

        let data = output.into_inner();
        assert!(data.len() >= 0x600); // Has header + data

        // Check version string
        assert_eq!(&data[0..4], b"3.00");
    }

    #[test]
    fn test_write_tbf_with_name() {
        let mut pattern = EmbPattern::new();
        pattern.add_metadata("name", "TestPattern");
        pattern.add_thread(crate::core::thread::EmbThread::from_rgb(0, 255, 0));
        pattern.add_stitch_absolute(STITCH, 0.0, 0.0);
        pattern.end();

        let mut output = Cursor::new(Vec::new());
        write(&pattern, &mut output).unwrap();

        let data = output.into_inner();

        // Check that name appears in header (search for "LA:" pattern in first 0x200 bytes)
        let header_start = &data[0..0x200];
        let header_str = String::from_utf8_lossy(header_start);
        assert!(header_str.contains("LA:TestPattern"));
    }

    #[test]
    fn test_tbf_round_trip() {
        use crate::formats::io::readers::tbf;

        // Create original pattern with needle changes
        let mut original = EmbPattern::new();
        original.add_thread(crate::core::thread::EmbThread::from_rgb(255, 0, 0));
        original.add_thread(crate::core::thread::EmbThread::from_rgb(0, 0, 255));
        original.add_metadata("name", "RoundTrip");

        original.add_stitch_absolute(STITCH, 0.0, 0.0);
        original.add_stitch_absolute(STITCH, 50.0, 50.0);

        // Add needle change
        let needle_cmd = encode_thread_change(NEEDLE_SET, None, Some(2), None);
        original.add_stitch_relative(0.0, 0.0, needle_cmd);

        original.add_stitch_absolute(STITCH, 100.0, 100.0);
        original.end();

        // Write to buffer
        let mut buffer = Cursor::new(Vec::new());
        write(&original, &mut buffer).unwrap();

        // Read back
        buffer.set_position(0);
        let mut read_back = EmbPattern::new();
        tbf::read(&mut buffer, &mut read_back).unwrap();

        // Verify thread count
        assert_eq!(read_back.threads().len(), original.threads().len());

        // Verify name metadata
        if let Some(name) = read_back.get_metadata("name") {
            assert_eq!(name, "RoundTrip");
        }

        // Verify we have stitches
        assert!(!read_back.stitches().is_empty());
    }
}
