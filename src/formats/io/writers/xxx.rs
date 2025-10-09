//! Singer XXX format writer
//!
//! Writes XXX format with variable-length encoding (2 or 5 bytes per stitch),
//! maximum stitch distance of ±124 units, and colors stored at end after stitches.

use crate::core::constants::*;
use crate::core::encoder::{EncoderSettings, Transcoder};
use crate::core::pattern::EmbPattern;
use anyhow::Result;
use byteorder::{LittleEndian, WriteBytesExt};
use std::io::{Seek, Write};

/// Get default encoder settings for XXX format
pub fn default_settings() -> EncoderSettings {
    EncoderSettings {
        max_stitch: 124.0,
        max_jump: 124.0,
        full_jump: false,
        round: true,
        writes_speeds: false,
        sequin_contingency: CONTINGENCY_SEQUIN_JUMP,
        long_stitch_contingency: CONTINGENCY_LONG_STITCH_SEW_TO,
        ..Default::default()
    }
}

/// Write XXX format file from a pattern
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
/// let mut file = File::create("output.xxx").unwrap();
/// butabuti::io::writers::xxx::write(&pattern, &mut file).unwrap();
/// ```
pub fn write(pattern: &EmbPattern, file: &mut (impl Write + Seek)) -> Result<()> {
    // Encode pattern with XXX settings
    let mut transcoder = Transcoder::new();
    *transcoder.settings_mut() = default_settings();

    let mut encoded = EmbPattern::new();
    transcoder.transcode(pattern, &mut encoded)?;

    // Write header (0x100 bytes)
    write_header(&encoded, file)?;

    // Remember position for end-of-stitches pointer
    let placeholder_pos = file.stream_position()?;
    file.write_u32::<LittleEndian>(0)?; // Placeholder

    // Write stitches
    write_stitches(&encoded, file)?;

    // Write end marker
    let end_pos = file.stream_position()?;
    file.write_u8(0x7F)?;
    file.write_u8(0x7F)?;
    file.write_u8(0x02)?;
    file.write_u8(0x14)?;

    // Go back and fill in end-of-stitches pointer
    file.seek(std::io::SeekFrom::Start(placeholder_pos))?;
    file.write_u32::<LittleEndian>(end_pos as u32)?;
    file.seek(std::io::SeekFrom::Start(end_pos + 4))?; // After end marker

    // Write colors
    write_colors(&encoded, file)?;

    Ok(())
}

/// Write the XXX file header
fn write_header(pattern: &EmbPattern, file: &mut impl Write) -> Result<()> {
    let stitches = pattern.stitches();

    // Write zeros up to offset 0x17
    for _ in 0..0x17 {
        file.write_u8(0)?;
    }

    // Stitch count (excluding END)
    let stitch_count = stitches
        .iter()
        .filter(|s| (s.command & COMMAND_MASK) != END)
        .count();
    file.write_u32::<LittleEndian>(stitch_count as u32)?;

    // Write zeros for 0x0C bytes
    for _ in 0..0x0C {
        file.write_u8(0)?;
    }

    // Thread count
    file.write_u32::<LittleEndian>(pattern.threads().len() as u32)?;
    file.write_u16::<LittleEndian>(0)?;

    // Get pattern bounds
    let bounds = pattern.bounds();
    let width = (bounds.2 - bounds.0) as i16;
    let height = (bounds.3 - bounds.1) as i16;

    file.write_u16::<LittleEndian>(width as u16)?;
    file.write_u16::<LittleEndian>(height as u16)?;

    // Last stitch position
    if let Some(last) = stitches.last() {
        file.write_u16::<LittleEndian>(last.x as i16 as u16)?;
        file.write_u16::<LittleEndian>((-last.y) as i16 as u16)?;
    } else {
        file.write_u16::<LittleEndian>(0)?;
        file.write_u16::<LittleEndian>(0)?;
    }

    // Min X and max Y
    file.write_u16::<LittleEndian>((-bounds.0) as i16 as u16)?;
    file.write_u16::<LittleEndian>(bounds.3 as i16 as u16)?;

    // Fill rest of header with zeros up to 0x100
    let bytes_written = 0x17 + 4 + 0x0C + 4 + 2 + 2 + 2 + 2 + 2 + 2 + 2;
    for _ in bytes_written..0x100 {
        file.write_u8(0)?;
    }

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

        match command {
            COLOR_CHANGE | STOP => {
                file.write_u8(0x7F)?;
                file.write_u8(0x08)?;
                file.write_u8(dx as i8 as u8)?;
                file.write_u8((-dy) as i8 as u8)?;
            }
            END => {
                break;
            }
            STITCH => {
                // Check if it fits in short encoding
                if (-124..124).contains(&dx) && (-124..124).contains(&dy) {
                    file.write_u8(dx as i8 as u8)?;
                    file.write_u8((-dy) as i8 as u8)?;
                } else {
                    // Long stitch encoding
                    file.write_u8(0x7D)?;
                    file.write_u16::<LittleEndian>(dx as i16 as u16)?;
                    file.write_u16::<LittleEndian>((-dy) as i16 as u16)?;
                }
            }
            TRIM => {
                file.write_u8(0x7F)?;
                file.write_u8(0x03)?;
                file.write_u8(dx as i8 as u8)?;
                file.write_u8((-dy) as i8 as u8)?;
            }
            JUMP => {
                file.write_u8(0x7F)?;
                file.write_u8(0x01)?;
                file.write_u8(dx as i8 as u8)?;
                file.write_u8((-dy) as i8 as u8)?;
            }
            _ => {
                // Unknown command, skip
            }
        }
    }

    Ok(())
}

/// Write color data
fn write_colors(pattern: &EmbPattern, file: &mut impl Write) -> Result<()> {
    file.write_u8(0)?;
    file.write_u8(0)?;

    let threads = pattern.threads();
    let mut count = 0;

    // Write actual thread colors
    for thread in threads {
        file.write_u8(0)?; // Alpha channel
        file.write_u8(thread.red())?;
        file.write_u8(thread.green())?;
        file.write_u8(thread.blue())?;
        count += 1;
    }

    // Fill remaining slots (up to 21 total)
    for _ in count..21 {
        file.write_u32::<LittleEndian>(0)?;
    }

    // Terminator
    file.write_u32::<LittleEndian>(0xFFFFFF00)?;
    file.write_u8(0)?;
    file.write_u8(1)?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    #[test]
    fn test_write_xxx_basic() {
        let mut pattern = EmbPattern::new();
        pattern.add_thread(crate::core::thread::EmbThread::from_rgb(255, 0, 0));
        pattern.add_stitch_absolute(STITCH, 0.0, 0.0);
        pattern.add_stitch_absolute(STITCH, 10.0, 10.0);
        pattern.end();

        let mut output = Cursor::new(Vec::new());
        write(&pattern, &mut output).unwrap();

        let data = output.into_inner();
        assert!(data.len() > 0x100); // Has header + data

        // Check color count in header (at offset 0x27)
        assert_eq!(data[0x27], 1); // 1 thread
    }

    #[test]
    fn test_write_xxx_multiple_colors() {
        let mut pattern = EmbPattern::new();
        pattern.add_thread(crate::core::thread::EmbThread::from_rgb(255, 0, 0));
        pattern.add_thread(crate::core::thread::EmbThread::from_rgb(0, 255, 0));
        pattern.add_stitch_absolute(STITCH, 0.0, 0.0);
        pattern.add_stitch_relative(0.0, 0.0, COLOR_CHANGE);
        pattern.add_stitch_absolute(STITCH, 20.0, 20.0);
        pattern.end();

        let mut output = Cursor::new(Vec::new());
        write(&pattern, &mut output).unwrap();

        let data = output.into_inner();
        assert_eq!(data[0x27], 2); // 2 threads
    }

    #[test]
    fn test_xxx_round_trip() {
        use crate::formats::io::readers::xxx;

        // Create original pattern
        let mut original = EmbPattern::new();
        original.add_thread(crate::core::thread::EmbThread::from_rgb(255, 0, 0));
        original.add_thread(crate::core::thread::EmbThread::from_rgb(0, 0, 255));
        original.add_stitch_absolute(STITCH, 0.0, 0.0);
        original.add_stitch_absolute(STITCH, 50.0, 50.0);
        original.add_stitch_relative(0.0, 0.0, COLOR_CHANGE);
        original.add_stitch_absolute(STITCH, 100.0, 100.0);
        original.end();

        // Write to buffer
        let mut buffer = Cursor::new(Vec::new());
        write(&original, &mut buffer).unwrap();

        // Read back
        buffer.set_position(0);
        let mut read_back = EmbPattern::new();
        xxx::read(&mut buffer, &mut read_back).unwrap();

        // Verify thread count
        assert_eq!(read_back.threads().len(), original.threads().len());

        // Verify we have stitches
        assert!(!read_back.stitches().is_empty());
    }
}
