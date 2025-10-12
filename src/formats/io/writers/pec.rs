//! Brother PEC format writer
//!
//! Writes PEC format with graphics section for LCD preview and thread colors
//! mapped to the 64-color PEC palette. Includes thumbnail generation.

use crate::core::constants::*;
use crate::core::pattern::EmbPattern;
use crate::core::thread::EmbThread;
use crate::formats::io::utils::WriteHelper;
use crate::palettes::thread_pec::PEC_THREADS;
use crate::utils::error::Result;
use std::io::{Seek, SeekFrom, Write};

const MASK_07_BIT: u8 = 0b01111111;
const JUMP_CODE: u8 = 0b00010000;
const TRIM_CODE: u8 = 0b00100000;
#[allow(dead_code)]
const FLAG_LONG: u8 = 0b10000000;
const PEC_ICON_WIDTH: usize = 48;
const PEC_ICON_HEIGHT: usize = 38;

/// PEC graphics blank template (48x38 bitmap with border)
const PEC_BLANK: [u8; 234] = [
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0xF0, 0xFF, 0xFF, 0xFF, 0xFF, 0x0F, 0x08, 0x00, 0x00, 0x00,
    0x00, 0x10, 0x04, 0x00, 0x00, 0x00, 0x00, 0x20, 0x02, 0x00, 0x00, 0x00, 0x00, 0x40, 0x02, 0x00,
    0x00, 0x00, 0x00, 0x40, 0x02, 0x00, 0x00, 0x00, 0x00, 0x40, 0x02, 0x00, 0x00, 0x00, 0x00, 0x40,
    0x02, 0x00, 0x00, 0x00, 0x00, 0x40, 0x02, 0x00, 0x00, 0x00, 0x00, 0x40, 0x02, 0x00, 0x00, 0x00,
    0x00, 0x40, 0x02, 0x00, 0x00, 0x00, 0x00, 0x40, 0x02, 0x00, 0x00, 0x00, 0x00, 0x40, 0x02, 0x00,
    0x00, 0x00, 0x00, 0x40, 0x02, 0x00, 0x00, 0x00, 0x00, 0x40, 0x02, 0x00, 0x00, 0x00, 0x00, 0x40,
    0x02, 0x00, 0x00, 0x00, 0x00, 0x40, 0x02, 0x00, 0x00, 0x00, 0x00, 0x40, 0x02, 0x00, 0x00, 0x00,
    0x00, 0x40, 0x02, 0x00, 0x00, 0x00, 0x00, 0x40, 0x02, 0x00, 0x00, 0x00, 0x00, 0x40, 0x02, 0x00,
    0x00, 0x00, 0x00, 0x40, 0x02, 0x00, 0x00, 0x00, 0x00, 0x40, 0x02, 0x00, 0x00, 0x00, 0x00, 0x40,
    0x02, 0x00, 0x00, 0x00, 0x00, 0x40, 0x02, 0x00, 0x00, 0x00, 0x00, 0x40, 0x02, 0x00, 0x00, 0x00,
    0x00, 0x40, 0x02, 0x00, 0x00, 0x00, 0x00, 0x40, 0x02, 0x00, 0x00, 0x00, 0x00, 0x40, 0x02, 0x00,
    0x00, 0x00, 0x00, 0x40, 0x02, 0x00, 0x00, 0x00, 0x00, 0x40, 0x02, 0x00, 0x00, 0x00, 0x00, 0x40,
    0x02, 0x00, 0x00, 0x00, 0x00, 0x40, 0x02, 0x00, 0x00, 0x00, 0x00, 0x40, 0x02, 0x00, 0x00, 0x00,
    0x00, 0x40, 0x02, 0x00, 0x00, 0x00, 0x00, 0x40, 0x04, 0x00, 0x00, 0x00, 0x00, 0x20, 0x08, 0x00,
    0x00, 0x00, 0x00, 0x10, 0xF0, 0xFF, 0xFF, 0xFF, 0xFF, 0x0F,
];

/// Build unique color palette for PEC
fn build_pec_palette(threads: &[EmbThread]) -> Vec<u8> {
    let mut palette = Vec::new();
    let mut used = vec![false; PEC_THREADS.len()];

    for thread in threads {
        let mut min_distance = f64::MAX;
        let mut best_index = 0;

        for (i, pec_thread) in PEC_THREADS.iter().enumerate() {
            if !used[i] {
                let distance = thread.color_distance(pec_thread.color);
                if distance < min_distance {
                    min_distance = distance;
                    best_index = i;
                }
            }
        }

        palette.push(best_index as u8);
        used[best_index] = true;
    }

    palette
}

/// Write PEC header
fn write_pec_header<W: Write>(
    helper: &mut WriteHelper<W>,
    pattern: &EmbPattern,
) -> Result<Vec<u8>> {
    // Get pattern name
    let name = pattern
        .get_metadata("name")
        .map(|s| s.as_str())
        .unwrap_or("Untitled");
    let truncated_name = if name.len() > 8 { &name[..8] } else { name };

    // Write label
    let label = format!("LA:{:16}\r", truncated_name);
    helper.write_bytes(label.as_bytes())?;

    // Write padding and icon header
    helper.write_bytes(&[0x20; 12])?;
    helper.write_u8(0xFF)?;
    helper.write_u8(0x00)?;
    helper.write_u8((PEC_ICON_WIDTH / 8) as u8)?; // byte stride
    helper.write_u8(PEC_ICON_HEIGHT as u8)?; // icon height

    // Build color palette
    let color_indices = build_pec_palette(pattern.threads());
    let thread_count = color_indices.len();

    if thread_count > 0 {
        // Write padding
        helper.write_bytes(&[0x20; 12])?;

        // Write thread count - 1 as first byte
        helper.write_u8((thread_count - 1) as u8)?;

        // Write color indices
        for &index in &color_indices {
            helper.write_u8(index)?;
        }

        // Pad to 463 bytes total
        for _ in (thread_count + 1)..463 {
            helper.write_u8(0x20)?;
        }
    } else {
        // Write default if no threads
        helper.write_bytes(&[0x20, 0x20, 0x20, 0x20, 0x64, 0x20, 0x00, 0x20])?;
        helper.write_bytes(&[0x00, 0x20, 0x20, 0x20, 0xFF])?;
        for _ in 13..463 {
            helper.write_u8(0x20)?;
        }
    }

    Ok(color_indices)
}

/// Write a coordinate value for PEC encoding
fn write_value<W: Write>(
    helper: &mut WriteHelper<W>,
    value: i32,
    long: bool,
    flag: u8,
) -> Result<()> {
    if !long && (-64..63).contains(&value) {
        helper.write_u8((value & MASK_07_BIT as i32) as u8)?;
    } else {
        let mut val = (value & 0b0000111111111111) as u16;
        val |= 0b1000000000000000;
        val |= (flag as u16) << 8;
        helper.write_u8((val >> 8) as u8)?;
        helper.write_u8((val & 0xFF) as u8)?;
    }
    Ok(())
}

/// Write trim+jump
fn write_trimjump<W: Write>(helper: &mut WriteHelper<W>, dx: i32, dy: i32) -> Result<()> {
    write_value(helper, dx, true, TRIM_CODE)?;
    write_value(helper, dy, true, TRIM_CODE)?;
    Ok(())
}

/// Write jump
fn write_jump<W: Write>(helper: &mut WriteHelper<W>, dx: i32, dy: i32) -> Result<()> {
    write_value(helper, dx, true, JUMP_CODE)?;
    write_value(helper, dy, true, JUMP_CODE)?;
    Ok(())
}

/// Write stitch
fn write_stitch<W: Write>(helper: &mut WriteHelper<W>, dx: i32, dy: i32) -> Result<()> {
    let long = false; // GROUP_LONG disabled for now
    write_value(helper, dx, long, 0)?;
    write_value(helper, dy, long, 0)?;
    Ok(())
}

/// Encode stitches in PEC format
fn pec_encode<W: Write>(helper: &mut WriteHelper<W>, pattern: &EmbPattern) -> Result<()> {
    let mut color_two = true;
    let mut jumping = true;
    let mut init = true;
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

        match data {
            STITCH => {
                if jumping {
                    if dx != 0 || dy != 0 {
                        write_stitch(helper, 0, 0)?;
                    }
                    jumping = false;
                }
                write_stitch(helper, dx, dy)?;
            },
            JUMP => {
                jumping = true;
                if init {
                    write_jump(helper, dx, dy)?;
                } else {
                    write_trimjump(helper, dx, dy)?;
                }
            },
            COLOR_CHANGE => {
                if jumping {
                    write_stitch(helper, 0, 0)?;
                    jumping = false;
                }
                helper.write_bytes(&[0xFE, 0xB0])?;
                if color_two {
                    helper.write_u8(0x02)?;
                } else {
                    helper.write_u8(0x01)?;
                }
                color_two = !color_two;
            },
            END => {
                helper.write_u8(0xFF)?;
                break;
            },
            _ => {}, // STOP, TRIM ignored
        }
        init = false;
    }

    Ok(())
}

/// Mark a bit in the graphics bitmap
fn graphic_mark_bit(graphic: &mut [u8], x: i32, y: i32, stride: usize) {
    if x >= 0 && y >= 0 {
        let x = x as usize;
        let y = y as usize;
        let byte_x = x / 8;
        let bit_x = x % 8;
        let index = (y * stride) + byte_x;

        if index < graphic.len() {
            graphic[index] |= 1 << bit_x;
        }
    }
}

/// Draw stitches scaled to fit in the graphics area
fn draw_scaled(
    bounds: (f64, f64, f64, f64),
    stitches: &[(f64, f64)],
    graphic: &mut [u8],
    stride: usize,
    buffer: i32,
) {
    let (left, top, right, bottom) = bounds;

    let diagram_width = right - left;
    let diagram_height = bottom - top;

    let graphic_width = (stride * 8) as f64;
    let graphic_height = (graphic.len() / stride) as f64;

    let diagram_width = if diagram_width == 0.0 {
        1.0
    } else {
        diagram_width
    };
    let diagram_height = if diagram_height == 0.0 {
        1.0
    } else {
        diagram_height
    };

    let scale_x = (graphic_width - buffer as f64) / diagram_width;
    let scale_y = (graphic_height - buffer as f64) / diagram_height;
    let scale = scale_x.min(scale_y);

    let cx = (right + left) / 2.0;
    let cy = (bottom + top) / 2.0;

    let translate_x = (-cx * scale) + (graphic_width / 2.0);
    let translate_y = (-cy * scale) + (graphic_height / 2.0);

    for &(x, y) in stitches {
        let px = ((x * scale) + translate_x).floor() as i32;
        let py = ((y * scale) + translate_y).floor() as i32;
        graphic_mark_bit(graphic, px, py, stride);
    }
}

/// Write PEC graphics section
fn write_pec_graphics<W: Write>(
    helper: &mut WriteHelper<W>,
    pattern: &EmbPattern,
    bounds: (f64, f64, f64, f64),
) -> Result<()> {
    // Write overall pattern graphic
    let mut blank = PEC_BLANK.to_vec();
    let all_stitches: Vec<(f64, f64)> = pattern
        .stitches()
        .iter()
        .filter(|s| (s.command & COMMAND_MASK) == STITCH)
        .map(|s| (s.x, s.y))
        .collect();
    draw_scaled(bounds, &all_stitches, &mut blank, 6, 4);
    helper.write_bytes(&blank)?;

    // Write per-color graphics
    let mut current_stitches = Vec::new();
    for stitch in pattern.stitches() {
        let cmd = stitch.command & COMMAND_MASK;
        if cmd == STITCH {
            current_stitches.push((stitch.x, stitch.y));
        } else if cmd == COLOR_CHANGE || cmd == END {
            if !current_stitches.is_empty() {
                let mut color_blank = PEC_BLANK.to_vec();
                draw_scaled(bounds, &current_stitches, &mut color_blank, 6, 5);
                helper.write_bytes(&color_blank)?;
                current_stitches.clear();
            }
            if cmd == END {
                break;
            }
        }
    }

    // Write final color if any
    if !current_stitches.is_empty() {
        let mut color_blank = PEC_BLANK.to_vec();
        draw_scaled(bounds, &current_stitches, &mut color_blank, 6, 5);
        helper.write_bytes(&color_blank)?;
    }

    Ok(())
}

/// Write PEC section (used by both standalone PEC and PES files)
pub fn write_pec_section<W: Write + Seek>(writer: &mut W, pattern: &EmbPattern) -> Result<()> {
    let mut helper = WriteHelper::new(writer);

    // Write header
    write_pec_header(&mut helper, pattern)?;

    // Get bounds
    let bounds = pattern.bounds();
    let width = (bounds.2 - bounds.0).round() as i32;
    let height = (bounds.3 - bounds.1).round() as i32;

    // Remember position for block length
    let stitch_block_start = helper.bytes_written();

    // Placeholder for block info
    helper.write_u16_le(0)?;
    helper.write_u8(0)?;
    helper.write_u8(0)?;
    helper.write_u8(0)?;

    // Write block header
    helper.write_bytes(&[0x31, 0xFF, 0xF0])?;
    helper.write_i16_le(width as i16)?;
    helper.write_i16_le(height as i16)?;
    helper.write_i16_le(0x1E0)?;
    helper.write_i16_le(0x1B0)?;

    // Encode stitches
    pec_encode(&mut helper, pattern)?;

    // Calculate block length and write it back
    let stitch_block_end = helper.bytes_written();
    let block_length = stitch_block_end - stitch_block_start;

    // Seek back and write block length
    helper.seek(SeekFrom::Start((stitch_block_start + 2) as u64))?;
    helper.write_u8((block_length & 0xFF) as u8)?;
    helper.write_u8(((block_length >> 8) & 0xFF) as u8)?;
    helper.write_u8(((block_length >> 16) & 0xFF) as u8)?;
    helper.seek(SeekFrom::Start(stitch_block_end as u64))?;

    // Write graphics
    write_pec_graphics(&mut helper, pattern, bounds)?;

    Ok(())
}

/// Write standalone PEC file
pub fn write<W: Write + Seek>(writer: &mut W, pattern: &EmbPattern) -> Result<()> {
    writer.write_all(b"#PEC0001")?;
    write_pec_section(writer, pattern)
}

/// Write PEC file to path
pub fn write_file(path: &str, pattern: &EmbPattern) -> Result<()> {
    let file = std::fs::File::create(path)?;
    let mut writer = std::io::BufWriter::new(file);
    write(&mut writer, pattern)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    #[test]
    fn test_pec_write_basic() {
        let mut pattern = EmbPattern::new();
        pattern.add_thread(crate::core::thread::EmbThread::new(0xFF0000));
        pattern.stitch(10.0, 20.0);
        pattern.stitch(5.0, 10.0);
        pattern.end();

        let mut buffer = Cursor::new(Vec::new());
        let result = write(&mut buffer, &pattern);
        assert!(result.is_ok());
        let data = buffer.into_inner();
        assert!(data.len() > 500); // PEC has header + graphics
        assert_eq!(&data[0..8], b"#PEC0001");
    }
}
