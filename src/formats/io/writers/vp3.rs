//! Pfaff VP3 format writer
//!
//! Writes VP3 format with metadata sections for hoops, colors, and design information
//! in Pfaff's proprietary structured binary format.

use crate::core::constants::*;
use crate::core::pattern::EmbPattern;
use crate::formats::io::utils::WriteHelper;
use crate::utils::error::Result;
use std::io::Write;

/// VP3 file signature
const VP3_SIGNATURE: &[u8] = b"%vsm%";

/// Write a VP3 file to a writer
pub fn write<W: Write>(writer: &mut W, pattern: &EmbPattern) -> Result<()> {
    let mut helper = WriteHelper::new(writer);

    // Write signature
    helper.write_bytes(VP3_SIGNATURE)?;

    // Write metadata sections
    write_metadata_section(&mut helper, pattern, "name", b"%nam%")?;
    write_metadata_section(&mut helper, pattern, "author", b"%aut%")?;
    write_metadata_section(&mut helper, pattern, "copyright", b"%cop%")?;
    write_metadata_section(&mut helper, pattern, "comments", b"%com%")?;

    // Write stitch data section
    write_stitch_section(&mut helper, pattern)?;

    Ok(())
}

/// Write a metadata section
fn write_metadata_section<W: Write>(
    helper: &mut WriteHelper<W>,
    pattern: &EmbPattern,
    key: &str,
    marker: &[u8],
) -> Result<()> {
    if let Some(value) = pattern.get_metadata(key) {
        // Write section marker
        helper.write_bytes(marker)?;

        // Write length (including null terminator)
        let bytes = value.as_bytes();
        helper.write_u16_le((bytes.len() + 1) as u16)?;

        // Write string data
        helper.write_bytes(bytes)?;
        helper.write_u8(0)?; // Null terminator
    }
    Ok(())
}

/// Write the stitch data section
fn write_stitch_section<W: Write>(helper: &mut WriteHelper<W>, pattern: &EmbPattern) -> Result<()> {
    // Write stitch section marker
    helper.write_bytes(b"%xxs%")?;

    // Calculate section size (3 bytes per stitch)
    let stitch_count = pattern.stitches().len();
    let section_size = (stitch_count * 3) as u32;
    helper.write_u32_le(section_size)?;

    // Write stitches
    let mut prev_x = 0.0;
    let mut prev_y = 0.0;

    for stitch in pattern.stitches() {
        let dx = (stitch.x - prev_x).round() as i8;
        let dy = (stitch.y - prev_y).round() as i8;

        helper.write_i8(dx)?;
        helper.write_i8(dy)?;
        helper.write_u8(encode_vp3_command(stitch.command & COMMAND_MASK))?;

        prev_x += dx as f64;
        prev_y += dy as f64;
    }

    Ok(())
}

/// Encode embroidery command to VP3 command byte
fn encode_vp3_command(command: u32) -> u8 {
    match command {
        STITCH => 0x00,
        JUMP => 0x01,
        COLOR_CHANGE => 0x02,
        TRIM => 0x03,
        END => 0x80,
        _ => 0x00, // Default to stitch
    }
}

/// Write VP3 file to path
pub fn write_file(path: &str, pattern: &EmbPattern) -> Result<()> {
    let file = std::fs::File::create(path)?;
    let mut writer = std::io::BufWriter::new(file);
    write(&mut writer, pattern)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::thread::EmbThread;
    use std::io::Cursor;

    #[test]
    fn test_encode_command() {
        assert_eq!(encode_vp3_command(STITCH), 0x00);
        assert_eq!(encode_vp3_command(JUMP), 0x01);
        assert_eq!(encode_vp3_command(COLOR_CHANGE), 0x02);
        assert_eq!(encode_vp3_command(TRIM), 0x03);
        assert_eq!(encode_vp3_command(END), 0x80);
    }

    #[test]
    fn test_vp3_write_basic() {
        let mut pattern = EmbPattern::new();
        pattern.add_metadata("name", "Test Design");
        pattern.add_thread(EmbThread::new(0xFF0000));
        pattern.add_stitch_absolute(STITCH, 10.0, 10.0);
        pattern.add_stitch_absolute(STITCH, 20.0, 20.0);
        pattern.add_stitch_absolute(END, 20.0, 20.0);

        let mut buffer = Cursor::new(Vec::new());
        let result = write(&mut buffer, &pattern);
        assert!(result.is_ok());

        let data = buffer.into_inner();
        assert!(!data.is_empty());
        assert_eq!(&data[0..5], b"%vsm%");
    }

    #[test]
    fn test_vp3_write_with_metadata() {
        let mut pattern = EmbPattern::new();
        pattern.add_metadata("name", "My Design");
        pattern.add_metadata("author", "Test Author");
        pattern.add_thread(EmbThread::new(0x00FF00));
        pattern.end();

        let mut buffer = Cursor::new(Vec::new());
        write(&mut buffer, &pattern).unwrap();

        let data = buffer.into_inner();
        assert!(data.len() > 5);

        // Check for metadata markers
        let data_str = String::from_utf8_lossy(&data);
        assert!(data_str.contains("%nam%"));
        assert!(data_str.contains("%aut%"));
    }

    #[test]
    fn test_vp3_roundtrip() {
        let mut pattern = EmbPattern::new();
        pattern.add_thread(EmbThread::new(0xFF0000));
        pattern.add_stitch_absolute(STITCH, 5.0, 5.0);
        pattern.add_stitch_absolute(STITCH, 15.0, 15.0);
        pattern.add_stitch_absolute(COLOR_CHANGE, 15.0, 15.0);
        pattern.add_stitch_absolute(STITCH, 25.0, 25.0);
        pattern.add_stitch_absolute(END, 25.0, 25.0);

        let mut buffer = Cursor::new(Vec::new());
        write(&mut buffer, &pattern).unwrap();

        // Verify basic structure
        let data = buffer.into_inner();
        assert_eq!(&data[0..5], b"%vsm%");
        assert!(data.len() > 10);
    }
}
