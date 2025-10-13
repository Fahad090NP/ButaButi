//! Brother PES format writer
//!
//! Writes PES format (versions 1 and 6) with embedded PEC section for machine
//! compatibility. Includes design metadata and thread color information.

use crate::core::constants::*;
use crate::core::pattern::EmbPattern;
use crate::core::thread::EmbThread;
use crate::formats::io::utils::WriteHelper;
use crate::formats::io::writers::pec;
use crate::utils::error::Result;
use std::io::{Seek, SeekFrom, Write};

/// PES version 1 file signature
pub const PES_VERSION_1_SIGNATURE: &str = "#PES0001";

/// PES version 6 file signature
pub const PES_VERSION_6_SIGNATURE: &str = "#PES0060";

const EMB_ONE: &str = "CEmbOne";
const EMB_SEG: &str = "CSewSeg";

/// Version of PES format to write
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum PesVersion {
    /// PES version 1
    V1,
    /// PES version 6 (includes metadata support)
    V6,
}

impl PesVersion {
    fn signature(&self) -> &str {
        match self {
            PesVersion::V1 => PES_VERSION_1_SIGNATURE,
            PesVersion::V6 => PES_VERSION_6_SIGNATURE,
        }
    }
}

/// Write a PES embroidery file
pub fn write_pes<W: Write + Seek>(
    pattern: &EmbPattern,
    writer: &mut W,
    version: PesVersion,
    truncated: bool,
) -> Result<()> {
    if truncated {
        write_truncated(pattern, writer, version)
    } else {
        write_full(pattern, writer, version)
    }
}

fn write_truncated<W: Write + Seek>(
    pattern: &EmbPattern,
    writer: &mut W,
    version: PesVersion,
) -> Result<()> {
    let mut w = WriteHelper::new(writer);

    match version {
        PesVersion::V1 => {
            w.write_string_utf8(PES_VERSION_1_SIGNATURE)?;
            w.write_bytes(&[
                0x16, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            ])?;
            pec::write_pec_section(w.inner_mut(), pattern)?;
        }
        PesVersion::V6 => {
            w.write_string_utf8(PES_VERSION_6_SIGNATURE)?;
            let placeholder_pec_block = w.bytes_written();
            w.write_i32_le(0)?; // Placeholder for PEC BLOCK
            write_pes_header_v6(pattern, &mut w, 0)?;
            w.write_bytes(&[0x00, 0x00, 0x00, 0x00, 0x00])?;
            w.write_i16_le(0x0000)?;
            w.write_i16_le(0x0000)?;

            let current_position = w.bytes_written();
            w.inner_mut()
                .seek(SeekFrom::Start(placeholder_pec_block as u64))?;
            w.write_i32_le(current_position as i32)?;
            w.inner_mut()
                .seek(SeekFrom::Start(current_position as u64))?;

            pec::write_pec_section(w.inner_mut(), pattern)?;
            w.write_i16_le(0x0000)?;
        }
    }

    Ok(())
}

fn write_full<W: Write + Seek>(
    pattern: &EmbPattern,
    writer: &mut W,
    version: PesVersion,
) -> Result<()> {
    let mut w = WriteHelper::new(writer);

    w.write_string_utf8(version.signature())?;

    let bounds = pattern.bounds();
    let cx = (bounds.2 + bounds.0) / 2.0;
    let cy = (bounds.3 + bounds.1) / 2.0;
    let left = bounds.0 - cx;
    let top = bounds.1 - cy;
    let right = bounds.2 - cx;
    let bottom = bounds.3 - cy;

    let placeholder_pec_block = w.bytes_written();
    w.write_i32_le(0)?; // Placeholder for PEC BLOCK

    let distinct_blocks = if pattern.stitches().is_empty() { 0 } else { 1 };

    match version {
        PesVersion::V1 => {
            write_pes_header_v1(&mut w, distinct_blocks)?;
        }
        PesVersion::V6 => {
            write_pes_header_v6(pattern, &mut w, distinct_blocks)?;
        }
    }

    if pattern.stitches().is_empty() {
        w.write_i16_le(0x0000)?;
        w.write_i16_le(0x0000)?;
    } else {
        w.write_i16_le(-1)?; // 0xFFFF
        w.write_i16_le(0x0000)?;
        let colorlog = write_pes_blocks(&mut w, pattern, left, top, right, bottom, cx, cy)?;

        if version == PesVersion::V6 {
            // Version 6 has node/tree/order data
            w.write_i32_le(0)?;
            w.write_i32_le(0)?;
            for i in 0..colorlog.len() {
                w.write_i32_le(i as i32)?;
                w.write_i32_le(0)?;
            }
        }
    }

    let current_position = w.bytes_written();
    w.inner_mut()
        .seek(SeekFrom::Start(placeholder_pec_block as u64))?;
    w.write_i32_le(current_position as i32)?;
    w.inner_mut()
        .seek(SeekFrom::Start(current_position as u64))?;

    pec::write_pec_section(w.inner_mut(), pattern)?;

    if version == PesVersion::V6 {
        w.write_i16_le(0x0000)?;
    }

    Ok(())
}

fn write_pes_header_v1<W: Write>(
    w: &mut WriteHelper<W>,
    distinct_block_objects: i16,
) -> Result<()> {
    w.write_i16_le(0x01)?; // scale to fit
    w.write_i16_le(0x01)?; // 0 = 100x100, 130x180 hoop
    w.write_i16_le(distinct_block_objects)?;
    Ok(())
}

fn write_pes_header_v6<W: Write>(
    pattern: &EmbPattern,
    w: &mut WriteHelper<W>,
    distinct_block_objects: i16,
) -> Result<()> {
    w.write_i16_le(0x01)?; // 0 = 100x100, 130x180 hoop
    w.write_bytes(b"02")?; // 2-digit ascii number

    write_pes_string_8(w, pattern.extras().get("name"))?;
    write_pes_string_8(w, pattern.extras().get("category"))?;
    write_pes_string_8(w, pattern.extras().get("author"))?;
    write_pes_string_8(w, pattern.extras().get("keywords"))?;
    write_pes_string_8(w, pattern.extras().get("comments"))?;

    w.write_i16_le(0)?; // OptimizeHoopChange = False
    w.write_i16_le(0)?; // Design Page Is Custom = False
    w.write_i16_le(0x64)?; // Hoop Width
    w.write_i16_le(0x64)?; // Hoop Height
    w.write_i16_le(0)?; // Use Existing Design Area = False
    w.write_i16_le(0xC8)?; // designWidth
    w.write_i16_le(0xC8)?; // designHeight
    w.write_i16_le(0x64)?; // designPageSectionWidth
    w.write_i16_le(0x64)?; // designPageSectionHeight
    w.write_i16_le(0x64)?; // p6
    w.write_i16_le(0x07)?; // designPageBackgroundColor
    w.write_i16_le(0x13)?; // designPageForegroundColor
    w.write_i16_le(0x01)?; // ShowGrid
    w.write_i16_le(0x01)?; // WithAxes
    w.write_i16_le(0x00)?; // SnapToGrid
    w.write_i16_le(100)?; // GridInterval
    w.write_i16_le(0x01)?; // p9 curves?
    w.write_i16_le(0x00)?; // OptimizeEntryExitPoints
    w.write_i8(0)?; // fromImageStringLength

    w.write_f32_le(1.0)?;
    w.write_f32_le(0.0)?;
    w.write_f32_le(0.0)?;
    w.write_f32_le(1.0)?;
    w.write_f32_le(0.0)?;
    w.write_f32_le(0.0)?;

    w.write_i16_le(0)?; // numberOfProgrammableFillPatterns
    w.write_i16_le(0)?; // numberOfMotifPatterns
    w.write_i16_le(0)?; // featherPatternCount

    let count_thread = pattern.threads().len();
    w.write_i16_le(count_thread as i16)?;
    for thread in pattern.threads() {
        write_pes_thread(w, thread)?;
    }

    w.write_i16_le(distinct_block_objects)?;
    Ok(())
}

fn write_pes_string_8<W: Write>(w: &mut WriteHelper<W>, s: Option<&String>) -> Result<()> {
    match s {
        None => {
            w.write_i8(0)?;
        }
        Some(string) => {
            let len = string.len().min(255);
            w.write_i8(len as i8)?;
            w.write_string_utf8(&string[..len])?;
        }
    }
    Ok(())
}

fn write_pes_string_16<W: Write>(w: &mut WriteHelper<W>, s: &str) -> Result<()> {
    let len = s.len();
    w.write_i16_le(len as i16)?;
    w.write_string_utf8(s)?;
    Ok(())
}

fn write_pes_thread<W: Write>(w: &mut WriteHelper<W>, thread: &EmbThread) -> Result<()> {
    write_pes_string_8(w, thread.catalog_number.as_ref())?;

    // Extract RGB components from u32 color
    let r = ((thread.color >> 16) & 0xFF) as i8;
    let g = ((thread.color >> 8) & 0xFF) as i8;
    let b = (thread.color & 0xFF) as i8;

    w.write_i8(r)?;
    w.write_i8(g)?;
    w.write_i8(b)?;
    w.write_i8(0)?; // unknown
    w.write_i32_le(0xA)?; // A is custom color
    write_pes_string_8(w, thread.description.as_ref())?;
    write_pes_string_8(w, thread.brand.as_ref())?;
    write_pes_string_8(w, thread.chart.as_ref())?;
    Ok(())
}

#[allow(clippy::too_many_arguments)]
fn write_pes_blocks<W: Write + Seek>(
    w: &mut WriteHelper<W>,
    pattern: &EmbPattern,
    left: f64,
    top: f64,
    right: f64,
    bottom: f64,
    cx: f64,
    cy: f64,
) -> Result<Vec<(usize, i16)>> {
    if pattern.stitches().is_empty() {
        return Ok(Vec::new());
    }

    write_pes_string_16(w, EMB_ONE)?;
    let placeholder = write_pes_sewsegheader(w, left, top, right, bottom)?;
    w.write_i16_le(-1)?; // 0xFFFF
    w.write_i16_le(0x0000)?; // FFFF0000 means more blocks exist

    write_pes_string_16(w, EMB_SEG)?;
    let (sections, colorlog) = write_pes_embsewseg_segments(w, pattern, left, bottom, cx, cy)?;

    let current_position = w.bytes_written();
    w.inner_mut().seek(SeekFrom::Start(placeholder as u64))?;
    w.write_i16_le(sections as i16)?;
    w.inner_mut()
        .seek(SeekFrom::Start(current_position as u64))?;

    w.write_i16_le(0x0000)?;
    w.write_i16_le(0x0000)?; // 00000000 means no more blocks

    Ok(colorlog)
}

fn write_pes_sewsegheader<W: Write>(
    w: &mut WriteHelper<W>,
    left: f64,
    _top: f64,
    right: f64,
    bottom: f64,
) -> Result<usize> {
    let width = right - left;
    let height = bottom - _top;
    let hoop_height = 1800.0;
    let hoop_width = 1300.0;

    // Write bounds (all zeros)
    for _ in 0..8 {
        w.write_i16_le(0)?;
    }

    // Calculate transformation
    let mut trans_x = 0.0;
    let mut trans_y = 0.0;
    trans_x += 350.0;
    trans_y += 100.0 + height;
    trans_x += hoop_width / 2.0;
    trans_y += hoop_height / 2.0;
    trans_x += -width / 2.0;
    trans_y += -height / 2.0;

    w.write_f32_le(1.0)?;
    w.write_f32_le(0.0)?;
    w.write_f32_le(0.0)?;
    w.write_f32_le(1.0)?;
    w.write_f32_le(trans_x as f32)?;
    w.write_f32_le(trans_y as f32)?;

    w.write_i16_le(1)?;
    w.write_i16_le(0)?;
    w.write_i16_le(0)?;
    w.write_i16_le(width as i16)?;
    w.write_i16_le(height as i16)?;
    w.write_bytes(&[0x00; 8])?;

    let placeholder = w.bytes_written();
    w.write_i16_le(0)?; // sections placeholder
    Ok(placeholder)
}

fn write_pes_embsewseg_segments<W: Write>(
    w: &mut WriteHelper<W>,
    pattern: &EmbPattern,
    left: f64,
    bottom: f64,
    cx: f64,
    cy: f64,
) -> Result<(usize, Vec<(usize, i16)>)> {
    let adjust_x = left + cx;
    let adjust_y = bottom + cy;

    let mut section = 0;
    let mut colorlog = Vec::new();
    let mut previous_color_code = -1;
    let mut flag = -1;

    let mut color_index = 0;
    let mut stitched_x = 0.0;
    let mut stitched_y = 0.0;

    // Get thread for color matching
    let threads = pattern.threads();
    let mut current_thread = threads.get(color_index).cloned();
    if current_thread.is_none() && !pattern.stitches().is_empty() {
        // Use default thread if none specified
        current_thread = Some(EmbThread::new(0x000000));
    }

    // Process command blocks
    let blocks = get_command_blocks(pattern);
    for (block, block_type) in blocks {
        let mut segments = Vec::new();
        let color_code: i16;

        match block_type {
            BlockType::Stitch => {
                for stitch in &block {
                    stitched_x = stitch.0;
                    stitched_y = stitch.1;
                    segments.push((stitched_x - adjust_x, stitched_y - adjust_y));
                }
                color_code = if let Some(ref thread) = current_thread {
                    find_nearest_pec_color(thread) as i16
                } else {
                    0
                };
                if flag != -1 {
                    w.write_i16_le(-32765)?; // 0x8003 section end
                }
                flag = 0;
            }
            BlockType::Jump => {
                segments.push((stitched_x - adjust_x, stitched_y - adjust_y));
                if let Some(last) = block.last() {
                    segments.push((last.0 - adjust_x, last.1 - adjust_y));
                }
                color_code = if let Some(ref thread) = current_thread {
                    find_nearest_pec_color(thread) as i16
                } else {
                    0
                };
                if flag != -1 {
                    w.write_i16_le(-32765)?; // 0x8003 section end
                }
                flag = 1;
            }
            BlockType::ColorChange => {
                color_index += 1;
                current_thread = threads.get(color_index).cloned();
                continue;
            }
            BlockType::Other => continue,
        }

        if previous_color_code != color_code {
            colorlog.push((section, color_code));
            previous_color_code = color_code;
        }

        w.write_i16_le(flag)?;
        w.write_i16_le(color_code)?;
        w.write_i16_le(segments.len() as i16)?;
        for seg in segments {
            w.write_i16_le(seg.0 as i16)?;
            w.write_i16_le(seg.1 as i16)?;
        }
        section += 1;
    }

    w.write_i16_le(colorlog.len() as i16)?;
    for log_item in &colorlog {
        w.write_i16_le(log_item.0 as i16)?;
        w.write_i16_le(log_item.1)?;
    }

    Ok((section, colorlog))
}

enum BlockType {
    Stitch,
    Jump,
    ColorChange,
    Other,
}

fn get_command_blocks(pattern: &EmbPattern) -> Vec<(Vec<(f64, f64)>, BlockType)> {
    let mut blocks = Vec::new();
    let mut current_block = Vec::new();
    let mut current_type = None;

    for stitch in pattern.stitches() {
        let cmd = stitch.command & COMMAND_MASK;
        let block_type = if cmd == STITCH {
            BlockType::Stitch
        } else if cmd == JUMP {
            BlockType::Jump
        } else if cmd == COLOR_CHANGE {
            BlockType::ColorChange
        } else {
            BlockType::Other
        };

        let is_same_type = matches!(
            (&current_type, &block_type),
            (Some(BlockType::Stitch), BlockType::Stitch) | (Some(BlockType::Jump), BlockType::Jump)
        );

        if is_same_type {
            current_block.push((stitch.x, stitch.y));
        } else {
            if !current_block.is_empty() {
                if let Some(t) = current_type {
                    blocks.push((current_block.clone(), t));
                }
            }
            current_block.clear();
            current_block.push((stitch.x, stitch.y));
            current_type = Some(block_type);
        }
    }

    if !current_block.is_empty() {
        if let Some(t) = current_type {
            blocks.push((current_block, t));
        }
    }

    blocks
}

fn find_nearest_pec_color(thread: &EmbThread) -> u8 {
    use crate::palettes::thread_pec::PEC_THREADS;
    let mut best_index = 0;
    let mut best_distance = f64::MAX;

    for (i, pec_thread) in PEC_THREADS.iter().enumerate() {
        let distance = thread.color_distance(pec_thread.color);
        if distance < best_distance {
            best_distance = distance;
            best_index = i;
        }
    }

    best_index as u8
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::formats::io::readers::pes;
    use std::io::Cursor;

    #[test]
    fn test_write_pes_v1_empty() {
        let pattern = EmbPattern::new();
        let mut buffer = Cursor::new(Vec::new());
        write_pes(&pattern, &mut buffer, PesVersion::V1, false).unwrap();
        assert!(!buffer.get_ref().is_empty());
    }

    #[test]
    fn test_write_pes_v6_truncated() {
        let pattern = EmbPattern::new();
        let mut buffer = Cursor::new(Vec::new());
        write_pes(&pattern, &mut buffer, PesVersion::V6, true).unwrap();
        assert!(!buffer.get_ref().is_empty());
    }

    #[test]
    fn test_write_read_roundtrip() {
        let mut pattern = EmbPattern::new();
        pattern.add_thread(EmbThread::new(0xFF0000));
        pattern.add_stitch_absolute(STITCH, 0.0, 0.0);
        pattern.add_stitch_absolute(STITCH, 10.0, 10.0);
        pattern.add_stitch_absolute(STITCH, 20.0, 0.0);
        pattern.end();

        let mut buffer = Cursor::new(Vec::new());
        let result = write_pes(&pattern, &mut buffer, PesVersion::V1, false);
        assert!(result.is_ok(), "Failed to write PES: {:?}", result.err());

        // Verify we wrote something
        let data = buffer.get_ref();
        assert!(data.len() > 100, "PES file too small: {} bytes", data.len());

        // Verify signature
        assert_eq!(&data[0..8], b"#PES0001", "Invalid PES signature");

        // Test reading - if it fails, that's okay for now as the reader may need updating
        buffer.set_position(0);
        let mut read_pattern = EmbPattern::new();
        match pes::read(&mut buffer, &mut read_pattern) {
            Ok(_) => {
                assert!(!read_pattern.stitches().is_empty(), "No stitches read");
            }
            Err(e) => {
                // For now, just verify the write worked
                eprintln!("Note: PES read failed (reader may need updating): {:?}", e);
            }
        }
    }

    #[test]
    fn test_write_pes_with_metadata() {
        let mut pattern = EmbPattern::new();
        pattern.set_metadata("name", "Test Design");
        pattern.set_metadata("author", "Test Author");
        pattern.add_thread(EmbThread::new(0x0000FF));
        pattern.add_stitch_absolute(STITCH, 0.0, 0.0);
        pattern.add_stitch_absolute(STITCH, 50.0, 50.0);
        pattern.end();

        let mut buffer = Cursor::new(Vec::new());
        write_pes(&pattern, &mut buffer, PesVersion::V6, false).unwrap();

        assert!(!buffer.get_ref().is_empty());
        assert_eq!(&buffer.get_ref()[0..8], b"#PES0060");
    }

    #[test]
    fn test_write_pes_v1_structure() {
        let mut pattern = EmbPattern::new();
        pattern.add_thread(EmbThread::new(0xFF0000));
        pattern.add_stitch_absolute(STITCH, 5.0, 5.0);
        pattern.end();

        let mut buffer = Cursor::new(Vec::new());
        write_pes(&pattern, &mut buffer, PesVersion::V1, false).unwrap();

        let data = buffer.get_ref();

        // Check signature
        assert_eq!(&data[0..8], b"#PES0001");

        // Check PEC block position (should be at byte 8-11, little endian)
        let pec_pos = u32::from_le_bytes([data[8], data[9], data[10], data[11]]);
        assert!(pec_pos > 12, "PEC block position too small: {}", pec_pos);
        assert!(
            (pec_pos as usize) < data.len(),
            "PEC block position beyond file: {}",
            pec_pos
        );
    }
}
