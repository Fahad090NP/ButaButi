//! Tajima DST format reader
//!
//! DST is one of the most common industrial embroidery formats, using a 512-byte header
//! followed by 3-byte stitch records with bit-encoded coordinates and commands.
//!
//! ## Format Limitations
//!
//! - **Header size**: Fixed 512 bytes
//! - **Stitch range**: X/Y coordinates: -121 to +121 per stitch record (ternary encoding)
//! - **Maximum stitches**: 1,000,000 (enforced for safety)
//! - **Coordinate system**: 0.1mm units, Y-axis is negated
//! - **Color changes**: Supported via control bits
//!
//! ## Validation
//!
//! This reader validates:
//! - File must be at least 512 bytes (header size)
//! - Header contains DST markers (LA:, ST:, CO:) or valid ASCII text
//! - Stitch count does not exceed safety limit
//!
//! ## Example
//!
//! ```no_run
//! use butabuti::prelude::*;
//! use std::fs::File;
//!
//! let mut file = File::open("design.dst")?;
//! let mut pattern = EmbPattern::new();
//! butabuti::formats::io::readers::dst::read(&mut file, Some(Default::default()))?;
//! # Ok::<(), Box<dyn std::error::Error>>(())
//! ```

use crate::core::constants::*;
use crate::core::pattern::EmbPattern;
use crate::core::thread::EmbThread;
use crate::utils::error::{Error, Result};
use std::collections::HashMap;
use std::io::Read;

/// DST header size in bytes
const DST_HEADER_SIZE: usize = 512;

/// Maximum allowed stitches for safety
const MAX_STITCHES: usize = 1_000_000;

/// Get bit value at position
#[inline]
fn get_bit(b: u8, pos: u8) -> i32 {
    ((b >> pos) & 1) as i32
}

/// Decode X coordinate from 3 bytes
fn decode_dx(b0: u8, b1: u8, b2: u8) -> i32 {
    let mut x = 0;
    x += get_bit(b2, 2) * 81;
    x += get_bit(b2, 3) * -81;
    x += get_bit(b1, 2) * 27;
    x += get_bit(b1, 3) * -27;
    x += get_bit(b0, 2) * 9;
    x += get_bit(b0, 3) * -9;
    x += get_bit(b1, 0) * 3;
    x += get_bit(b1, 1) * -3;
    x += get_bit(b0, 0);
    x -= get_bit(b0, 1);
    x
}

/// Decode Y coordinate from 3 bytes
fn decode_dy(b0: u8, b1: u8, b2: u8) -> i32 {
    let mut y = 0;
    y += get_bit(b2, 5) * 81;
    y += get_bit(b2, 4) * -81;
    y += get_bit(b1, 5) * 27;
    y += get_bit(b1, 4) * -27;
    y += get_bit(b0, 5) * 9;
    y += get_bit(b0, 4) * -9;
    y += get_bit(b1, 7) * 3;
    y += get_bit(b1, 6) * -3;
    y += get_bit(b0, 7);
    y -= get_bit(b0, 6);
    -y
}

/// Process a header line
fn process_header_info(pattern: &mut EmbPattern, prefix: &str, value: &str) {
    match prefix {
        "LA" => {
            pattern.add_metadata("name", value);
        },
        "AU" => {
            pattern.add_metadata("author", value);
        },
        "CP" => {
            pattern.add_metadata("copyright", value);
        },
        "TC" => {
            // Thread color: hex, description, catalog
            let parts: Vec<&str> = value.split(',').map(|s| s.trim()).collect();
            if !parts.is_empty() {
                let mut thread = if let Ok(t) = EmbThread::from_string(parts[0]) {
                    t
                } else {
                    EmbThread::new(0x000000)
                };

                if parts.len() > 1 {
                    thread = thread.with_description(parts[1]);
                }
                if parts.len() > 2 {
                    thread = thread.with_catalog_number(parts[2]);
                }

                pattern.add_thread(thread);
            }
        },
        _ => {
            pattern.add_metadata(prefix, value);
        },
    }
}

/// Read DST header (512 bytes)
fn read_header<R: Read>(reader: &mut R, pattern: &mut EmbPattern) -> Result<()> {
    let mut header = vec![0u8; DST_HEADER_SIZE];
    reader.read_exact(&mut header).map_err(|e| {
        if e.kind() == std::io::ErrorKind::UnexpectedEof {
            Error::Parse(format!(
                "DST file too small: header must be {} bytes",
                DST_HEADER_SIZE
            ))
        } else {
            Error::from(e)
        }
    })?;

    // Validate DST header - should contain ASCII text with common DST markers
    // DST headers typically start with "LA:" (Label/Name) or at least contain readable ASCII
    let header_text = String::from_utf8_lossy(&header[0..32]);
    if !header_text.contains("LA:") && !header_text.contains("ST:") && !header_text.contains("CO:")
    {
        // Check if first 32 bytes contain mostly printable ASCII or are mostly zeros
        let printable_count = header[0..32]
            .iter()
            .filter(|&&b| (32..127).contains(&b) || b == 0 || b == 13 || b == 10)
            .count();
        if printable_count < 24 {
            return Err(Error::Parse(
                "Invalid DST header: expected DST text markers (LA:, ST:, CO:) or ASCII text"
                    .to_string(),
            ));
        }
    }

    let mut start = 0;
    for (i, &byte) in header.iter().enumerate() {
        if byte == 13 || byte == 10 {
            // '\r' or '\n'
            if i > start {
                let data = &header[start..i];
                if let Ok(line) = String::from_utf8(data.to_vec()) {
                    let line = line.trim();
                    if line.len() > 3 {
                        let prefix = &line[0..2].trim();
                        let value = &line[3..].trim();
                        process_header_info(pattern, prefix, value);
                    }
                }
            }
            start = i + 1;
        }
    }

    Ok(())
}

/// Read DST stitches
fn read_stitches<R: Read>(
    reader: &mut R,
    pattern: &mut EmbPattern,
    settings: &HashMap<String, String>,
) -> Result<()> {
    let mut sequin_mode = false;
    let mut buffer = [0u8; 3];
    let mut stitch_count = 0;

    loop {
        match reader.read_exact(&mut buffer) {
            Ok(_) => {},
            Err(e) if e.kind() == std::io::ErrorKind::UnexpectedEof => break,
            Err(e) => return Err(Error::from(e)),
        }

        // Check for excessive stitch count
        stitch_count += 1;
        if stitch_count > MAX_STITCHES {
            return Err(Error::Parse(format!(
                "DST file exceeds maximum stitch count of {}",
                MAX_STITCHES
            )));
        }

        let dx = decode_dx(buffer[0], buffer[1], buffer[2]) as f64;
        let dy = decode_dy(buffer[0], buffer[1], buffer[2]) as f64;

        // Check control bits
        if buffer[2] & 0b11110011 == 0b11110011 {
            // End pattern
            break;
        } else if buffer[2] & 0b11000011 == 0b11000011 {
            // Color change
            pattern.color_change(dx, dy);
        } else if buffer[2] & 0b01000011 == 0b01000011 {
            // Sequin mode toggle
            pattern.add_stitch_relative(dx, dy, SEQUIN_MODE);
            sequin_mode = !sequin_mode;
        } else if buffer[2] & 0b10000011 == 0b10000011 {
            // Move or sequin eject
            if sequin_mode {
                pattern.add_stitch_relative(dx, dy, SEQUIN_EJECT);
            } else {
                pattern.jump(dx, dy);
            }
        } else {
            // Normal stitch
            pattern.stitch(dx, dy);
        }
    }

    pattern.end();

    // Interpolate trims based on settings
    let trim_at = settings
        .get("trim_at")
        .and_then(|s| s.parse::<usize>().ok())
        .unwrap_or(3);

    let trim_distance = settings
        .get("trim_distance")
        .and_then(|s| s.parse::<f64>().ok())
        .map(|d| d * 10.0); // Convert mm to 1/10mm units

    let clipping = settings
        .get("clipping")
        .and_then(|s| s.parse::<bool>().ok())
        .unwrap_or(true);

    if let Some(distance) = trim_distance {
        pattern.interpolate_trims(trim_at, Some(distance), clipping);
    } else {
        pattern.interpolate_trims(trim_at, None, clipping);
    }

    Ok(())
}

/// Read a DST file
pub fn read<R: Read>(
    reader: &mut R,
    settings: Option<HashMap<String, String>>,
) -> Result<EmbPattern> {
    let mut pattern = EmbPattern::new();
    let settings = settings.unwrap_or_default();

    read_header(reader, &mut pattern)?;
    read_stitches(reader, &mut pattern, &settings)?;

    Ok(pattern)
}

/// Read a DST file from path
pub fn read_file(path: &str) -> Result<EmbPattern> {
    let file = std::fs::File::open(path)?;
    let mut reader = std::io::BufReader::new(file);
    read(&mut reader, None)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_decode_dx() {
        // Test zero
        assert_eq!(decode_dx(0, 0, 0), 0);

        // Test positive values
        assert_eq!(decode_dx(0b00000001, 0, 0), 1);
        assert_eq!(decode_dx(0, 0b00000001, 0), 3);
        assert_eq!(decode_dx(0b00000100, 0, 0), 9);
    }

    #[test]
    fn test_decode_dy() {
        // Test zero
        assert_eq!(decode_dy(0, 0, 0), 0);

        // Test values (y is negated)
        assert_eq!(decode_dy(0b10000000, 0, 0), -1);
    }

    #[test]
    fn test_get_bit() {
        assert_eq!(get_bit(0b00000001, 0), 1);
        assert_eq!(get_bit(0b00000001, 1), 0);
        assert_eq!(get_bit(0b00000010, 1), 1);
        assert_eq!(get_bit(0b10000000, 7), 1);
    }
}
