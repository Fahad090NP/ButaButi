//! Tajima TBF format reader
//!
//! TBF is a Tajima binary format with ASCII header (0x00-0x5FF), thread definitions,
//! and 3-byte stitch encoding. Supports explicit trim commands for industrial machines.
//!
//! ## Format Limitations
//! - Fixed header structure: name at 0x83 (16 bytes), thread order at 0x10A (256 bytes)
//! - Thread definitions start at 0x20E (marker 0x45 + RGB + 0x20 per thread)
//! - Stitch data starts at 0x600 (1536 bytes offset)
//! - Maximum 256 needles supported (thread order array size)
//! - Maximum 1,000,000 stitches per file
//! - 3-byte stitch encoding: x, y, control byte

/// TBF stitch data offset
const STITCH_DATA_OFFSET: u64 = 0x600;

/// TBF name offset
const NAME_OFFSET: u64 = 0x83;

/// TBF thread order offset
const THREAD_ORDER_OFFSET: u64 = 0x10A;

/// TBF thread definition offset
const THREAD_DEF_OFFSET: u64 = 0x20E;

/// Maximum allowed stitch count
const MAX_STITCHES: usize = 1_000_000;

/// Maximum allowed thread/needle count
const MAX_THREADS: usize = 256;

use crate::core::constants::*;
use crate::core::pattern::EmbPattern;
use crate::utils::error::Result;
use crate::utils::functions::encode_thread_change;
use byteorder::ReadBytesExt;
use std::io::{Read, Seek, SeekFrom};

/// Read a signed 8-bit value as f64
fn read_signed_i8(value: u8) -> f64 {
    value as i8 as f64
}

/// Read TBF format file into a pattern
///
/// # Arguments
///
/// * `file` - The input file/stream to read from
/// * `pattern` - The pattern to populate with data
///
/// # Example
///
/// ```no_run
/// use butabuti::prelude::*;
/// use std::fs::File;
///
/// let mut file = File::open("design.tbf").unwrap();
/// let mut pattern = EmbPattern::new();
/// butabuti::formats::io::readers::tbf::read(&mut file, &mut pattern).unwrap();
/// ```
pub fn read(file: &mut (impl Read + Seek), pattern: &mut EmbPattern) -> Result<()> {
    // Read name at offset 0x83 (16 bytes)
    file.seek(SeekFrom::Start(NAME_OFFSET))?;
    let mut name_bytes = [0u8; 16];
    file.read_exact(&mut name_bytes)?;
    if let Ok(name) = std::str::from_utf8(&name_bytes) {
        let trimmed = name.trim_end_matches('\0').trim();
        if !trimmed.is_empty() {
            pattern.add_metadata("name", trimmed);
        }
    }

    // Read thread order at offset 0x10A (256 bytes)
    file.seek(SeekFrom::Start(THREAD_ORDER_OFFSET))?;
    let mut thread_order = [0u8; 256];
    file.read_exact(&mut thread_order)?;

    // Read thread definitions at offset 0x20E
    file.seek(SeekFrom::Start(THREAD_DEF_OFFSET))?;
    let mut thread_count = 0;
    loop {
        let marker = file.read_u8()?;
        if marker == 0x45 {
            // Validate thread count
            thread_count += 1;
            if thread_count > MAX_THREADS {
                return Err(crate::utils::error::Error::Parse(format!(
                    "TBF file exceeds maximum thread count of {}",
                    MAX_THREADS
                )));
            }

            // Thread definition: 0x45 + R + G + B + 0x20
            let r = file.read_u8()?;
            let g = file.read_u8()?;
            let b = file.read_u8()?;
            let _space = file.read_u8()?; // Should be 0x20

            let thread = crate::core::thread::EmbThread::from_rgb(r, g, b);
            pattern.add_thread(thread);
        } else {
            break;
        }
    }

    // Read stitch data starting at 0x600
    file.seek(SeekFrom::Start(STITCH_DATA_OFFSET))?;

    let mut needle = 0;
    let mut stitch_count = 0;
    loop {
        let mut byte = [0u8; 3];
        if file.read_exact(&mut byte).is_err() {
            break;
        }

        // Check for excessive stitch count
        stitch_count += 1;
        if stitch_count > MAX_STITCHES {
            return Err(crate::utils::error::Error::Parse(format!(
                "TBF file exceeds maximum stitch count of {}",
                MAX_STITCHES
            )));
        }

        let x = byte[0];
        let y = byte[1];
        let ctrl = byte[2];

        match ctrl {
            // Regular stitch
            0x80 => {
                pattern.add_stitch_relative(read_signed_i8(x), -read_signed_i8(y), STITCH);
            }
            // Needle change
            0x81 => {
                let needle_value = thread_order[needle];
                needle += 1;

                if needle_value == 0 {
                    // Needle value 0 is treated as STOP
                    pattern.add_stitch_relative(read_signed_i8(x), -read_signed_i8(y), STOP);
                } else {
                    // Use NEEDLE_SET with explicit needle number
                    let cmd = encode_thread_change(NEEDLE_SET, None, Some(needle_value), None);
                    pattern.add_stitch_relative(read_signed_i8(x), -read_signed_i8(y), cmd);
                }
            }
            // Move/Jump
            0x90 => {
                if x == 0 && y == 0 {
                    pattern.add_stitch_relative(0.0, 0.0, TRIM);
                } else {
                    pattern.add_stitch_relative(read_signed_i8(x), -read_signed_i8(y), JUMP);
                }
            }
            // Stop
            0x40 => {
                pattern.add_stitch_relative(read_signed_i8(x), -read_signed_i8(y), STOP);
            }
            // Trim
            0x86 => {
                pattern.add_stitch_relative(read_signed_i8(x), -read_signed_i8(y), TRIM);
            }
            // End
            0x8F => {
                break;
            }
            _ => {
                // Unknown control byte, stop reading
                break;
            }
        }
    }

    pattern.end();
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    #[test]
    fn test_read_tbf_basic() {
        // Create minimal TBF file
        let mut data = vec![0u8; 0x600];

        // Set name at offset 0x83
        let name = b"TestPattern     ";
        data[0x83..0x93].copy_from_slice(name);

        // Thread order at 0x10A (set first needle to 1)
        data[0x10A] = 1;

        // Thread definition at 0x20E
        let thread_def_pos = 0x20E;
        data[thread_def_pos] = 0x45; // Marker
        data[thread_def_pos + 1] = 0xFF; // Red
        data[thread_def_pos + 2] = 0x00; // Green
        data[thread_def_pos + 3] = 0x00; // Blue
        data[thread_def_pos + 4] = 0x20; // Space
        data[thread_def_pos + 5] = 0x00; // End of threads (not 0x45)

        // Add some stitches starting at 0x600
        data.extend_from_slice(&[
            10, 20, 0x80, // Stitch (10, -20)
            0, 0, 0x8F, // End
        ]);

        let mut cursor = Cursor::new(data);
        let mut pattern = EmbPattern::new();

        read(&mut cursor, &mut pattern).unwrap();

        assert_eq!(pattern.threads().len(), 1);
        assert!(!pattern.stitches().is_empty());

        // Check metadata
        if let Some(name) = pattern.get_metadata("name") {
            assert_eq!(name, "TestPattern");
        }
    }

    #[test]
    fn test_read_tbf_needle_change() {
        let mut data = vec![0u8; 0x600];

        // Thread order: needle 1, then needle 2
        data[0x10A] = 1;
        data[0x10B] = 2;

        // Two thread definitions
        let mut pos = 0x20E;
        // Thread 1: Red
        data[pos] = 0x45;
        data[pos + 1] = 0xFF;
        data[pos + 2] = 0x00;
        data[pos + 3] = 0x00;
        data[pos + 4] = 0x20;
        pos += 5;

        // Thread 2: Blue
        data[pos] = 0x45;
        data[pos + 1] = 0x00;
        data[pos + 2] = 0x00;
        data[pos + 3] = 0xFF;
        data[pos + 4] = 0x20;
        pos += 5;

        // End marker
        data[pos] = 0x00;

        // Stitches
        data.extend_from_slice(&[
            5, 5, 0x80, // Stitch
            0, 0, 0x81, // Needle change
            10, 10, 0x80, // Stitch with new needle
            0, 0, 0x8F, // End
        ]);

        let mut cursor = Cursor::new(data);
        let mut pattern = EmbPattern::new();

        read(&mut cursor, &mut pattern).unwrap();

        assert_eq!(pattern.threads().len(), 2);
    }

    #[test]
    fn test_read_tbf_trim_and_jump() {
        let mut data = vec![0u8; 0x600];

        // One thread
        data[0x10A] = 1;
        data[0x20E] = 0x45;
        data[0x20F] = 0xFF;
        data[0x210] = 0xFF;
        data[0x211] = 0xFF;
        data[0x212] = 0x20;
        data[0x213] = 0x00;

        // Stitches with various commands
        data.extend_from_slice(&[
            5, 5, 0x80, // Stitch
            0, 0, 0x90, // Trim (jump with x=0, y=0)
            10, 10, 0x90, // Jump
            5, 5, 0x86, // Trim
            0, 0, 0x8F, // End
        ]);

        let mut cursor = Cursor::new(data);
        let mut pattern = EmbPattern::new();

        read(&mut cursor, &mut pattern).unwrap();

        let commands: Vec<u32> = pattern
            .stitches()
            .iter()
            .map(|s| s.command & COMMAND_MASK)
            .collect();

        assert!(commands.contains(&TRIM));
        assert!(commands.contains(&JUMP));
    }
}
