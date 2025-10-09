//! Tajima DST format writer
//!
//! Writes DST format with 512-byte header and 3-byte stitch records using bit-encoded
//! coordinates. Supports stitches, jumps, color changes, and trim commands.

use crate::core::constants::*;
use crate::core::pattern::EmbPattern;
use crate::formats::io::utils::WriteHelper;
use crate::utils::error::Result;
use std::io::Write;

const DST_HEADER_SIZE: usize = 512;

/// Set a bit at position
#[inline]
fn bit(b: u8) -> u8 {
    1 << b
}

/// Encode a DST record (3 bytes)
fn encode_record(x: i32, y: i32, flags: u32) -> Result<[u8; 3]> {
    let mut y = -y; // Flip Y coordinate
    let mut x = x;
    let mut b0 = 0u8;
    let mut b1 = 0u8;
    let mut b2 = 0u8;

    // Set jump bit if needed
    if flags == JUMP || flags == SEQUIN_EJECT {
        b2 |= 0b10000011;
    }

    // Encode movement or command flags
    if flags == STITCH || flags == JUMP || flags == SEQUIN_EJECT {
        b2 |= bit(0) | bit(1);

        // Encode X
        if x > 40 {
            b2 |= bit(2);
            x -= 81;
        }
        if x < -40 {
            b2 |= bit(3);
            x += 81;
        }
        if x > 13 {
            b1 |= bit(2);
            x -= 27;
        }
        if x < -13 {
            b1 |= bit(3);
            x += 27;
        }
        if x > 4 {
            b0 |= bit(2);
            x -= 9;
        }
        if x < -4 {
            b0 |= bit(3);
            x += 9;
        }
        if x > 1 {
            b1 |= bit(0);
            x -= 3;
        }
        if x < -1 {
            b1 |= bit(1);
            x += 3;
        }
        if x > 0 {
            b0 |= bit(0);
            x -= 1;
        }
        if x < 0 {
            b0 |= bit(1);
            x += 1;
        }
        if x != 0 {
            return Err(crate::utils::error::Error::Encoding(
                "X value exceeds maximum allowed for DST format".to_string(),
            ));
        }

        // Encode Y
        if y > 40 {
            b2 |= bit(5);
            y -= 81;
        }
        if y < -40 {
            b2 |= bit(4);
            y += 81;
        }
        if y > 13 {
            b1 |= bit(5);
            y -= 27;
        }
        if y < -13 {
            b1 |= bit(4);
            y += 27;
        }
        if y > 4 {
            b0 |= bit(5);
            y -= 9;
        }
        if y < -4 {
            b0 |= bit(4);
            y += 9;
        }
        if y > 1 {
            b1 |= bit(7);
            y -= 3;
        }
        if y < -1 {
            b1 |= bit(6);
            y += 3;
        }
        if y > 0 {
            b0 |= bit(7);
            y -= 1;
        }
        if y < 0 {
            b0 |= bit(6);
            y += 1;
        }
        if y != 0 {
            return Err(crate::utils::error::Error::Encoding(
                "Y value exceeds maximum allowed for DST format".to_string(),
            ));
        }
    } else if flags == COLOR_CHANGE || flags == STOP {
        b2 = 0b11000011;
    } else if flags == END {
        b2 = 0b11110011;
    } else if flags == SEQUIN_MODE {
        b2 = 0b01000011;
    }

    Ok([b0, b1, b2])
}

/// Write DST header
fn write_header<W: Write>(
    writer: &mut WriteHelper<W>,
    pattern: &EmbPattern,
    extended_header: bool,
) -> Result<()> {
    let name = pattern
        .get_metadata("name")
        .map(|s| s.as_str())
        .unwrap_or("Untitled");

    // Write basic header fields
    writer.write_string(&format!("LA:{:<16}\r", name))?;
    writer.write_string(&format!("ST:{:>7}\r", pattern.count_stitches()))?;
    writer.write_string(&format!("CO:{:>3}\r", pattern.count_color_changes()))?;

    let bounds = pattern.bounds();
    writer.write_string(&format!("+X:{:>5}\r", bounds.2.abs() as i32))?;
    writer.write_string(&format!("-X:{:>5}\r", bounds.0.abs() as i32))?;
    writer.write_string(&format!("+Y:{:>5}\r", bounds.3.abs() as i32))?;
    writer.write_string(&format!("-Y:{:>5}\r", bounds.1.abs() as i32))?;

    // Last stitch coordinates
    let (ax, ay) = if let Some(last) = pattern.stitches().last() {
        (last.x as i32, -(last.y as i32))
    } else {
        (0, 0)
    };

    if ax >= 0 {
        writer.write_string(&format!("AX:+{:>5}\r", ax))?;
    } else {
        writer.write_string(&format!("AX:-{:>5}\r", ax.abs()))?;
    }
    if ay >= 0 {
        writer.write_string(&format!("AY:+{:>5}\r", ay))?;
    } else {
        writer.write_string(&format!("AY:-{:>5}\r", ay.abs()))?;
    }

    writer.write_string(&format!("MX:+{:>5}\r", 0))?;
    writer.write_string(&format!("MY:+{:>5}\r", 0))?;
    writer.write_string(&format!("PD:{:>6}\r", "******"))?;

    // Extended header with metadata and threads
    if extended_header {
        if let Some(author) = pattern.get_metadata("author") {
            writer.write_string(&format!("AU:{}\r", author))?;
        }
        if let Some(copyright) = pattern.get_metadata("copyright") {
            writer.write_string(&format!("CP:{}\r", copyright))?;
        }
        for thread in pattern.threads() {
            let desc = thread.description.as_deref().unwrap_or("");
            let cat = thread.catalog_number.as_deref().unwrap_or("");
            writer.write_string(&format!("TC:{},{},{}\r", thread.hex_color(), desc, cat))?;
        }
    }

    // End of text marker
    writer.write_u8(0x1A)?;

    // Pad to 512 bytes with spaces
    let current_pos = writer.bytes_written();
    for _ in current_pos..DST_HEADER_SIZE {
        writer.write_u8(0x20)?; // Space
    }

    Ok(())
}

/// Write DST file
pub fn write<W: Write>(
    writer: &mut W,
    pattern: &EmbPattern,
    extended_header: bool,
    trim_at: usize,
) -> Result<()> {
    let mut helper = WriteHelper::new(writer);

    write_header(&mut helper, pattern, extended_header)?;

    let mut xx = 0.0;
    let mut yy = 0.0;

    for stitch in pattern.stitches() {
        let x = stitch.x;
        let y = stitch.y;
        let data = stitch.command & COMMAND_MASK;

        let dx = (x - xx).round() as i32;
        let dy = (y - yy).round() as i32;

        xx += dx as f64;
        yy += dy as f64;

        if data == TRIM {
            // Encode trim as a series of tiny jumps
            let delta = -4;
            helper.write_bytes(&encode_record(-delta / 2, -delta / 2, JUMP)?)?;
            let mut delta = delta;
            for _ in 1..(trim_at - 1) {
                helper.write_bytes(&encode_record(delta, delta, JUMP)?)?;
                delta = -delta;
            }
            helper.write_bytes(&encode_record(delta / 2, delta / 2, JUMP)?)?;
        } else {
            helper.write_bytes(&encode_record(dx, dy, data)?)?;
        }
    }

    Ok(())
}

/// Write DST file to path
pub fn write_file(path: &str, pattern: &EmbPattern, extended_header: bool) -> Result<()> {
    let file = std::fs::File::create(path)?;
    let mut writer = std::io::BufWriter::new(file);
    write(&mut writer, pattern, extended_header, 3)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bit() {
        assert_eq!(bit(0), 1);
        assert_eq!(bit(1), 2);
        assert_eq!(bit(2), 4);
        assert_eq!(bit(7), 128);
    }

    #[test]
    fn test_encode_end() {
        let result = encode_record(0, 0, END).unwrap();
        assert_eq!(result[2], 0b11110011);
    }

    #[test]
    fn test_encode_color_change() {
        let result = encode_record(0, 0, COLOR_CHANGE).unwrap();
        assert_eq!(result[2], 0b11000011);
    }

    #[test]
    fn test_encode_simple_stitch() {
        let result = encode_record(1, 1, STITCH);
        assert!(result.is_ok());
    }
}
