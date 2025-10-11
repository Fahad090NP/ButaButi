//! Husqvarna Viking HUS format reader
//!
//! HUS format uses Huffman compression for stitch data and includes a predefined
//! 29-color thread palette specific to Husqvarna Viking machines.

use crate::core::constants::*;
use crate::core::pattern::EmbPattern;
use crate::palettes::thread_hus;
use crate::utils::compress;
use crate::utils::error::Result;
use std::io::{Read, Seek, SeekFrom};

/// Read HUS (Husqvarna Viking) format
pub fn read(file: &mut (impl Read + Seek), pattern: &mut EmbPattern) -> Result<()> {
    // Read header
    let _magic_code = read_int_32le(file)?;
    let number_of_stitches = read_int_32le(file)?;
    let number_of_colors = read_int_32le(file)?;

    let _extend_pos_x = read_int_16le(file)? as i16;
    let _extend_pos_y = read_int_16le(file)? as i16;
    let _extend_neg_x = read_int_16le(file)? as i16;
    let _extend_neg_y = read_int_16le(file)? as i16;

    let command_offset = read_int_32le(file)?;
    let x_offset = read_int_32le(file)?;
    let y_offset = read_int_32le(file)?;

    // Read 8-byte string
    let mut _string_buf = [0u8; 8];
    file.read_exact(&mut _string_buf)?;

    let _unknown_16_bit = read_int_16le(file)?;

    // Read thread palette
    let hus_thread_set = thread_hus::get_thread_set();
    for _ in 0..number_of_colors {
        let index = read_int_16le(file)? as usize;
        if index < hus_thread_set.len() {
            pattern.add_thread(hus_thread_set[index].clone());
        }
    }

    // Read compressed data
    file.seek(SeekFrom::Start(command_offset as u64))?;
    let command_size = (x_offset - command_offset) as usize;
    let mut command_compressed = vec![0u8; command_size];
    file.read_exact(&mut command_compressed)?;

    file.seek(SeekFrom::Start(x_offset as u64))?;
    let x_size = (y_offset - x_offset) as usize;
    let mut x_compressed = vec![0u8; x_size];
    file.read_exact(&mut x_compressed)?;

    file.seek(SeekFrom::Start(y_offset as u64))?;
    let mut y_compressed = Vec::new();
    file.read_to_end(&mut y_compressed)?;

    // Decompress data
    let command_decompressed =
        compress::expand(&command_compressed, Some(number_of_stitches as usize))?;
    let x_decompressed = compress::expand(&x_compressed, Some(number_of_stitches as usize))?;
    let y_decompressed = compress::expand(&y_compressed, Some(number_of_stitches as usize))?;

    // Process stitches
    let stitch_count = command_decompressed
        .len()
        .min(x_decompressed.len())
        .min(y_decompressed.len());

    for i in 0..stitch_count {
        let cmd = command_decompressed[i];
        let x = x_decompressed[i] as i8 as f64;
        let y = -(y_decompressed[i] as i8 as f64);

        match cmd {
            0x80 => {
                // STITCH
                pattern.add_stitch_relative(x, y, STITCH);
            }
            0x81 => {
                // JUMP (move)
                pattern.add_stitch_relative(x, y, JUMP);
            }
            0x84 => {
                // COLOR_CHANGE
                if x != 0.0 || y != 0.0 {
                    pattern.add_stitch_relative(x, y, STITCH);
                }
                pattern.add_command(COLOR_CHANGE, 0.0, 0.0);
            }
            0x88 => {
                // TRIM
                if x != 0.0 || y != 0.0 {
                    pattern.add_stitch_relative(x, y, JUMP);
                }
                pattern.add_command(TRIM, 0.0, 0.0);
            }
            0x90 => {
                // END
                break;
            }
            _ => {
                // Unknown command - stop processing
                break;
            }
        }
    }

    pattern.add_command(END, 0.0, 0.0);
    Ok(())
}

/// Read little-endian 16-bit integer
fn read_int_16le(file: &mut impl Read) -> Result<u16> {
    let mut buf = [0u8; 2];
    file.read_exact(&mut buf)?;
    Ok(u16::from_le_bytes(buf))
}

/// Read little-endian 32-bit integer
fn read_int_32le(file: &mut impl Read) -> Result<u32> {
    let mut buf = [0u8; 4];
    file.read_exact(&mut buf)?;
    Ok(u32::from_le_bytes(buf))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    #[test]
    fn test_read_hus_header() {
        let mut data = vec![];

        // Magic code
        data.extend_from_slice(&[0x00, 0x00, 0x00, 0x00]);

        // Number of stitches
        data.extend_from_slice(&[10, 0, 0, 0]);

        // Number of colors
        data.extend_from_slice(&[1, 0, 0, 0]);

        // Extents (8 bytes)
        data.extend_from_slice(&[0; 8]);

        // Offsets (command, x, y)
        let header_size = 42u32; // Header ends at byte 42
        data.extend_from_slice(&header_size.to_le_bytes());
        data.extend_from_slice(&(header_size + 10).to_le_bytes());
        data.extend_from_slice(&(header_size + 20).to_le_bytes());

        // String (8 bytes)
        data.extend_from_slice(&[0; 8]);

        // Unknown 16-bit
        data.extend_from_slice(&[0, 0]);

        // Thread index
        data.extend_from_slice(&[0, 0]);

        // Add some dummy compressed data
        data.extend_from_slice(&[0; 30]);

        let mut cursor = Cursor::new(data);
        let mut pattern = EmbPattern::new();

        // This will fail decompression but should read header correctly
        let _result = read(&mut cursor, &mut pattern);

        // Header reading should work, decompression might fail
        // Check that we got the thread
        assert_eq!(pattern.threads().len(), 1);
    }

    #[test]
    fn test_hus_basic_structure() {
        // Test the basic structure parsing
        let mut data = vec![
            // Magic, stitches, colors
            0, 0, 0, 0, 5, 0, 0, 0, 2, 0, 0, 0, // Extents (8 bytes)
            0, 0, 0, 0, 0, 0, 0, 0, // Offsets
            50, 0, 0, 0, 60, 0, 0, 0, 70, 0, 0, 0, // String
            0, 0, 0, 0, 0, 0, 0, 0, // Unknown
            0, 0, // Thread indices
            0, 0, 1, 0,
        ];

        // Pad to offset
        data.resize(80, 0);

        let mut cursor = Cursor::new(data);
        let mut pattern = EmbPattern::new();

        let _result = read(&mut cursor, &mut pattern);

        // Should have 2 threads
        assert_eq!(pattern.threads().len(), 2);
    }
}
