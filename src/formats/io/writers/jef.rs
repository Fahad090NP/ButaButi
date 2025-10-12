//! Janome JEF format writer
//!
//! Writes JEF format with binary header containing design bounds, hoop size,
//! and thread colors mapped to the predefined 79-color JEF palette.

use crate::core::constants::*;
use crate::core::pattern::EmbPattern;
use crate::formats::io::utils::WriteHelper;
use crate::palettes::thread_jef::JEF_THREADS;
use crate::utils::error::Result;
use std::io::Write;

// Hoop size constants (in mm, embroidery units are 1/10 mm)
const HOOP_110X110: i32 = 0;
const HOOP_50X50: i32 = 1;
const HOOP_140X200: i32 = 2;
const HOOP_126X110: i32 = 3;
const HOOP_200X200: i32 = 4;

/// Get JEF hoop size based on design dimensions
fn get_jef_hoop_size(width: i32, height: i32) -> i32 {
    // Select smallest hoop that fits the design
    if width < 500 && height < 500 {
        return HOOP_50X50;
    }
    if width < 1260 && height < 1100 {
        return HOOP_126X110;
    }
    if width < 1400 && height < 2000 {
        return HOOP_140X200;
    }
    if width < 2000 && height < 2000 {
        return HOOP_200X200;
    }
    HOOP_110X110
}

/// Write hoop edge distances
fn write_hoop_edge_distance<W: Write>(
    helper: &mut WriteHelper<W>,
    x_hoop_edge: i32,
    y_hoop_edge: i32,
) -> Result<()> {
    if x_hoop_edge.min(y_hoop_edge) >= 0 {
        helper.write_i32_le(x_hoop_edge)?; // left
        helper.write_i32_le(y_hoop_edge)?; // top
        helper.write_i32_le(x_hoop_edge)?; // right
        helper.write_i32_le(y_hoop_edge)?; // bottom
    } else {
        helper.write_i32_le(-1)?;
        helper.write_i32_le(-1)?;
        helper.write_i32_le(-1)?;
        helper.write_i32_le(-1)?;
    }
    Ok(())
}

/// Build thread palette, avoiding duplicates
fn build_palette(pattern: &EmbPattern) -> Vec<i32> {
    let mut palette = Vec::new();
    let mut last_index: Option<usize> = None;
    let mut last_thread: Option<&crate::core::thread::EmbThread> = None;
    let mut color_toggled = false;
    let mut index_in_threadlist = 0;
    let mut jef_available = vec![true; JEF_THREADS.len()];

    for stitch in pattern.stitches() {
        let flags = stitch.command & COMMAND_MASK;

        if (flags == COLOR_CHANGE || index_in_threadlist == 0)
            && index_in_threadlist < pattern.threads().len()
        {
            let thread = &pattern.threads()[index_in_threadlist];
            index_in_threadlist += 1;

            // Find nearest JEF color
            let mut min_distance = f64::MAX;
            let mut index_of_jefthread = 0;

            for (i, jef_thread_opt) in JEF_THREADS.iter().enumerate() {
                if let Some(jef_thread) = jef_thread_opt {
                    if jef_available[i] {
                        let distance = thread.color_distance(jef_thread.color);
                        if distance < min_distance {
                            min_distance = distance;
                            index_of_jefthread = i;
                        }
                    }
                }
            }

            // If this is the same index as last, mark it unavailable and find next best
            if let Some(last_idx) = last_index {
                if last_idx == index_of_jefthread && last_thread != Some(thread) {
                    jef_available[index_of_jefthread] = false;

                    min_distance = f64::MAX;
                    for (i, jef_thread_opt) in JEF_THREADS.iter().enumerate() {
                        if let Some(jef_thread) = jef_thread_opt {
                            if jef_available[i] {
                                let distance = thread.color_distance(jef_thread.color);
                                if distance < min_distance {
                                    min_distance = distance;
                                    index_of_jefthread = i;
                                }
                            }
                        }
                    }

                    jef_available[last_idx] = true; // Restore availability
                }
            }

            palette.push(index_of_jefthread as i32);
            last_index = Some(index_of_jefthread);
            last_thread = Some(thread);
            color_toggled = false;
        }

        if flags == STOP {
            color_toggled = !color_toggled;
            if color_toggled {
                palette.push(0);
            } else if let Some(last_idx) = last_index {
                palette.push(last_idx as i32);
            }
        }
    }

    palette
}

/// Count points needed for JEF file
fn count_points(pattern: &EmbPattern, trims: bool, trim_at: usize) -> i32 {
    let mut point_count = 1; // 1 for END statement

    for stitch in pattern.stitches() {
        let data = stitch.command & COMMAND_MASK;
        match data {
            STITCH => point_count += 1,
            JUMP => point_count += 2,
            TRIM if trims => point_count += 2 * trim_at as i32,
            COLOR_CHANGE | STOP => point_count += 2,
            END => break,
            _ => {},
        }
    }

    point_count
}

/// Write JEF file
pub fn write<W: Write>(
    writer: &mut W,
    pattern: &EmbPattern,
    trims: bool,
    trim_at: usize,
    date_string: &str,
) -> Result<()> {
    let mut helper = WriteHelper::new(writer);

    // Build palette
    let palette = build_palette(pattern);
    let color_count = palette.len() as i32;

    // Calculate offsets
    let offsets = 0x74 + (color_count * 8);
    helper.write_i32_le(offsets)?;
    helper.write_i32_le(0x14)?;

    // Write date string (14 bytes)
    let date_bytes = date_string.as_bytes();
    let len = date_bytes.len().min(14);
    helper.write_bytes(&date_bytes[..len])?;
    for _ in len..14 {
        helper.write_u8(0)?;
    }
    helper.write_u8(0)?;
    helper.write_u8(0)?;

    helper.write_i32_le(color_count)?;

    let point_count = count_points(pattern, trims, trim_at);
    helper.write_i32_le(point_count)?;

    // Calculate bounds
    let bounds = pattern.bounds();
    let design_width = (bounds.2 - bounds.0).round() as i32;
    let design_height = (bounds.3 - bounds.1).round() as i32;

    helper.write_i32_le(get_jef_hoop_size(design_width, design_height))?;

    let half_width = design_width / 2;
    let half_height = design_height / 2;

    // Distance from center of hoop
    helper.write_i32_le(half_width)?;
    helper.write_i32_le(half_height)?;
    helper.write_i32_le(half_width)?;
    helper.write_i32_le(half_height)?;

    // Distance from default 110 x 110 hoop
    write_hoop_edge_distance(&mut helper, 550 - half_width, 550 - half_height)?;

    // Distance from default 50 x 50 hoop
    write_hoop_edge_distance(&mut helper, 250 - half_width, 250 - half_height)?;

    // Distance from default 140 x 200 hoop
    write_hoop_edge_distance(&mut helper, 700 - half_width, 1000 - half_height)?;

    // Distance from custom hoop
    write_hoop_edge_distance(&mut helper, 700 - half_width, 1000 - half_height)?;

    // Write palette
    for &t in &palette {
        helper.write_i32_le(t)?;
    }

    // Write thread types (0x0D for each)
    for _ in 0..color_count {
        helper.write_i32_le(0x0D)?;
    }

    // Write stitches
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
                helper.write_i8(dx as i8)?;
                helper.write_i8((-dy) as i8)?;
            },
            COLOR_CHANGE | STOP => {
                helper.write_bytes(&[0x80, 0x01])?;
                helper.write_i8(dx as i8)?;
                helper.write_i8((-dy) as i8)?;
            },
            TRIM if trims => {
                for _ in 0..trim_at {
                    helper.write_bytes(&[0x80, 0x02, 0x00, 0x00])?;
                }
            },
            JUMP => {
                helper.write_bytes(&[0x80, 0x02])?;
                helper.write_i8(dx as i8)?;
                helper.write_i8((-dy) as i8)?;
            },
            END => break,
            _ => {},
        }
    }

    // Write end marker
    helper.write_bytes(&[0x80, 0x10])?;

    Ok(())
}

/// Write JEF file to path
pub fn write_file(path: &str, pattern: &EmbPattern) -> Result<()> {
    let file = std::fs::File::create(path)?;
    let mut writer = std::io::BufWriter::new(file);

    // Get current date in JEF format (YYYYMMDDHHmmss)
    let date_string = chrono::Local::now().format("%Y%m%d%H%M%S").to_string();

    write(&mut writer, pattern, false, 3, &date_string)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_jef_hoop_sizes() {
        assert_eq!(get_jef_hoop_size(400, 400), HOOP_50X50);
        assert_eq!(get_jef_hoop_size(1000, 1000), HOOP_126X110);
        assert_eq!(get_jef_hoop_size(1300, 1900), HOOP_140X200);
        assert_eq!(get_jef_hoop_size(1900, 1900), HOOP_200X200);
        assert_eq!(get_jef_hoop_size(2500, 2500), HOOP_110X110);
    }

    #[test]
    fn test_jef_write_basic() {
        let mut pattern = EmbPattern::new();
        pattern.add_thread(crate::core::thread::EmbThread::new(0xFF0000));
        pattern.stitch(10.0, 20.0);
        pattern.stitch(5.0, 10.0);
        pattern.end();

        let mut buffer = Vec::new();
        let result = write(&mut buffer, &pattern, false, 3, "20251008120000");
        assert!(result.is_ok());
        assert!(buffer.len() > 100); // JEF has a header
    }

    #[test]
    fn test_jef_round_trip() {
        use crate::formats::io::readers::jef;
        use std::io::Cursor;

        // Create original pattern
        let mut original = EmbPattern::new();
        original.add_thread(crate::core::thread::EmbThread::from_rgb(255, 0, 0));
        original.add_thread(crate::core::thread::EmbThread::from_rgb(0, 0, 255));
        original.add_stitch_absolute(STITCH, 0.0, 0.0);
        original.add_stitch_absolute(STITCH, 100.0, 0.0);
        original.add_stitch_absolute(STITCH, 100.0, 100.0);
        original.add_stitch_relative(0.0, 0.0, COLOR_CHANGE);
        original.add_stitch_absolute(STITCH, 0.0, 100.0);
        original.add_stitch_absolute(STITCH, 0.0, 0.0);
        original.end();

        let original_stitch_count = original.count_stitches();
        let original_thread_count = original.threads().len();

        // Write to buffer (trims=false, trim_at=127, date)
        let mut buffer = Cursor::new(Vec::new());
        write(&mut buffer, &original, false, 127, "20251011120000").unwrap();

        // Verify buffer has data
        assert!(!buffer.get_ref().is_empty());

        // Read back (needs Seek)
        buffer.set_position(0);
        let read_back = jef::read(&mut buffer, None).unwrap();

        // Verify thread count
        assert_eq!(read_back.threads().len(), original_thread_count);

        // Verify stitch count (should be close, allowing for encoding differences)
        let read_stitch_count = read_back.count_stitches();
        assert!(
            read_stitch_count >= original_stitch_count - 2
                && read_stitch_count <= original_stitch_count + 2,
            "Stitch count mismatch: original={}, read={}",
            original_stitch_count,
            read_stitch_count
        );

        // Verify we have stitches
        assert!(!read_back.stitches().is_empty());

        // Verify coordinate bounds are reasonable
        let (min_x, min_y, max_x, max_y) = read_back.bounds();
        assert!(max_x - min_x > 0.0, "Pattern has no width");
        assert!(max_y - min_y > 0.0, "Pattern has no height");
    }
}
