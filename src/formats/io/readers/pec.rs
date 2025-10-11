//! Brother PEC format reader
//!
//! PEC is the Brother Embroidery Card format, also used as an embedded section in PES files.
//! Contains a graphics section for LCD preview and uses the 64-color PEC thread palette.
//!
//! ## Format Limitations
//! - Uses 64-color PEC thread palette (indices 0-63)
//! - Maximum 1,000,000 stitches per file
//! - Stitch encoding: 7-bit or 12-bit signed deltas with control flags

/// Maximum allowed stitch count
const MAX_STITCHES: usize = 1_000_000;

use crate::core::pattern::EmbPattern;
use crate::core::thread::EmbThread;
use crate::formats::io::utils::ReadHelper;
use crate::palettes::thread_pec::PEC_THREADS;
use crate::utils::error::{Error, Result};
use std::io::{Read, Seek};

const JUMP_CODE: u8 = 0x10;
const TRIM_CODE: u8 = 0x20;
const FLAG_LONG: u8 = 0x80;

/// Convert 12-bit signed value
fn signed12(b: u16) -> i32 {
    let b = b & 0xFFF;
    if b > 0x7FF {
        (b as i32) - 0x1000
    } else {
        b as i32
    }
}

/// Convert 7-bit signed value
fn signed7(b: u8) -> i32 {
    if b > 63 {
        (b as i32) - 128
    } else {
        b as i32
    }
}

/// Process PEC color bytes using the PEC palette
fn process_pec_colors(color_bytes: &[u8], pattern: &mut EmbPattern) -> Vec<EmbThread> {
    let max_value = PEC_THREADS.len();
    let mut threads = Vec::new();

    for &byte in color_bytes {
        let idx = (byte as usize) % max_value;
        let thread = PEC_THREADS[idx].clone();
        pattern.add_thread(thread.clone());
        threads.push(thread);
    }

    threads
}

/// Process PEC color bytes with PES thread chart
fn process_pec_table(
    color_bytes: &[u8],
    pattern: &mut EmbPattern,
    chart: &mut Vec<EmbThread>,
) -> Vec<EmbThread> {
    let max_value = PEC_THREADS.len();
    let mut thread_map: std::collections::HashMap<usize, EmbThread> =
        std::collections::HashMap::new();
    let mut threads = Vec::new();

    for &byte in color_bytes {
        let color_index = (byte as usize) % max_value;

        let thread = if let Some(t) = thread_map.get(&color_index) {
            t.clone()
        } else {
            let t = if !chart.is_empty() {
                chart.remove(0)
            } else {
                PEC_THREADS[color_index].clone()
            };
            thread_map.insert(color_index, t.clone());
            t
        };

        pattern.add_thread(thread.clone());
        threads.push(thread);
    }

    threads
}

/// Map PEC colors to threads
fn map_pec_colors(
    color_bytes: &[u8],
    pattern: &mut EmbPattern,
    pes_chart: Option<&mut Vec<EmbThread>>,
) -> Vec<EmbThread> {
    if let Some(chart) = pes_chart {
        if chart.is_empty() {
            process_pec_colors(color_bytes, pattern)
        } else if chart.len() >= color_bytes.len() {
            // 1:1 mode
            let mut threads = Vec::new();
            for _ in 0..color_bytes.len() {
                if !chart.is_empty() {
                    let thread = chart.remove(0);
                    pattern.add_thread(thread.clone());
                    threads.push(thread);
                }
            }
            threads
        } else {
            process_pec_table(color_bytes, pattern, chart)
        }
    } else {
        process_pec_colors(color_bytes, pattern)
    }
}

/// Read PEC stitches
fn read_pec_stitches<R: Read>(reader: &mut ReadHelper<R>, pattern: &mut EmbPattern) -> Result<()> {
    let mut stitch_count = 0;
    loop {
        // Check for excessive stitch count
        stitch_count += 1;
        if stitch_count > MAX_STITCHES {
            return Err(Error::Parse(format!(
                "PEC file exceeds maximum stitch count of {}",
                MAX_STITCHES
            )));
        }

        let val1 = reader.read_u8()?;
        let val2 = match reader.read_u8() {
            Ok(v) => v,
            Err(_) => break,
        };

        if val2 == 0x00 {
            break;
        }

        if val1 == 0xFE && val2 == 0xB0 {
            reader.read_u8()?; // Skip 1 byte
            pattern.color_change(0.0, 0.0);
            continue;
        }

        let mut jump = false;
        let mut trim = false;
        let x: i32;

        if val1 & FLAG_LONG != 0 {
            if val1 & TRIM_CODE != 0 {
                trim = true;
            }
            if val1 & JUMP_CODE != 0 {
                jump = true;
            }
            let code = ((val1 as u16) << 8) | (val2 as u16);
            x = signed12(code);
            let val2_new = match reader.read_u8() {
                Ok(v) => v,
                Err(_) => break,
            };
            let val2 = val2_new;

            let y: i32;
            if val2 & FLAG_LONG != 0 {
                if val2 & TRIM_CODE != 0 {
                    trim = true;
                }
                if val2 & JUMP_CODE != 0 {
                    jump = true;
                }
                let val3 = match reader.read_u8() {
                    Ok(v) => v,
                    Err(_) => break,
                };
                let code = ((val2 as u16) << 8) | (val3 as u16);
                y = signed12(code);
            } else {
                y = signed7(val2);
            }

            if jump {
                pattern.jump(x as f64, y as f64);
            } else if trim {
                pattern.stitch(x as f64, y as f64);
                pattern.trim();
            } else {
                pattern.stitch(x as f64, y as f64);
            }
        } else {
            x = signed7(val1);
            let y: i32;

            if val2 & FLAG_LONG != 0 {
                if val2 & TRIM_CODE != 0 {
                    trim = true;
                }
                if val2 & JUMP_CODE != 0 {
                    jump = true;
                }
                let val3 = match reader.read_u8() {
                    Ok(v) => v,
                    Err(_) => break,
                };
                let code = ((val2 as u16) << 8) | (val3 as u16);
                y = signed12(code);
            } else {
                y = signed7(val2);
            }

            if jump {
                pattern.jump(x as f64, y as f64);
            } else if trim {
                pattern.stitch(x as f64, y as f64);
                pattern.trim();
            } else {
                pattern.stitch(x as f64, y as f64);
            }
        }
    }

    Ok(())
}

/// Read PEC graphics data
fn read_pec_graphics<R: Read>(
    reader: &mut ReadHelper<R>,
    pattern: &mut EmbPattern,
    size: usize,
    stride: usize,
    count: usize,
    _threads: &[EmbThread],
) -> Result<()> {
    for i in 0..count {
        let _graphic = reader.read_bytes(size)?;
        let name = format!("pec_graphic_{}", i);
        // Store as metadata - in a real implementation you'd decode the bitmap
        pattern.add_metadata(&name, format!("{}x{} bitmap", stride, size / stride));
    }
    Ok(())
}

/// Read PEC section (called from PES reader or standalone)
pub fn read_pec<R: Read + Seek>(
    reader: &mut R,
    pattern: &mut EmbPattern,
    pes_chart: Option<&mut Vec<EmbThread>>,
) -> Result<()> {
    let mut helper = ReadHelper::new(reader);

    // Skip 3 bytes (LA:)
    helper.read_bytes(3)?;

    // Read label (16 bytes)
    let label_bytes = helper.read_bytes(16)?;
    if let Ok(label) = String::from_utf8(label_bytes) {
        let label = label.trim_matches('\0').trim();
        if !label.is_empty() {
            pattern.add_metadata("Name", label);
        }
    }

    // Skip 15 bytes
    helper.read_bytes(15)?;

    let pec_graphic_byte_stride = helper.read_u8()? as usize;
    let pec_graphic_icon_height = helper.read_u8()? as usize;

    // Skip 12 bytes
    helper.read_bytes(12)?;

    let color_changes = helper.read_u8()?;
    let count_colors = (color_changes as usize) + 1;

    let color_bytes = helper.read_bytes(count_colors)?;
    let threads = map_pec_colors(&color_bytes, pattern, pes_chart);

    // Skip to stitch data
    helper.read_bytes(0x1D0 - color_changes as usize)?;

    // Read stitch block end position (not used in current implementation)
    let byte1 = helper.read_u8()? as u32;
    let byte2 = helper.read_u8()? as u32;
    let byte3 = helper.read_u8()? as u32;
    let _stitch_block_length = byte1 | (byte2 << 8) | (byte3 << 16);

    // Current position is already 3 bytes into calculation
    // Skip 8 more bytes (total 11 bytes: 3 already read + 8 more)
    helper.read_bytes(8)?;

    // Read stitches
    read_pec_stitches(&mut helper, pattern)?;

    // Read graphics if available
    let byte_size = pec_graphic_byte_stride * pec_graphic_icon_height;
    if byte_size > 0 {
        read_pec_graphics(
            &mut helper,
            pattern,
            byte_size,
            pec_graphic_byte_stride,
            count_colors + 1,
            &threads,
        )?;
    }

    Ok(())
}

/// Read a standalone PEC file
pub fn read<R: Read + Seek>(reader: &mut R) -> Result<EmbPattern> {
    let mut helper = ReadHelper::new(reader);

    // Read header
    let pec_string = helper.read_string(8)?;
    if pec_string != "#PEC0001" {
        return Err(Error::Parse(format!(
            "Invalid PEC header: expected '#PEC0001', got '{}'",
            pec_string
        )));
    }

    let mut pattern = EmbPattern::new();
    let mut reader = helper.into_inner();

    read_pec(&mut reader, &mut pattern, None)?;
    pattern.interpolate_duplicate_color_as_stop();

    Ok(pattern)
}

/// Read a PEC file from path
pub fn read_file(path: &str) -> Result<EmbPattern> {
    let file = std::fs::File::open(path)?;
    let mut reader = std::io::BufReader::new(file);
    read(&mut reader)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_signed12() {
        assert_eq!(signed12(0x000), 0);
        assert_eq!(signed12(0x7FF), 2047);
        assert_eq!(signed12(0x800), -2048);
        assert_eq!(signed12(0xFFF), -1);
    }

    #[test]
    fn test_signed7() {
        assert_eq!(signed7(0), 0);
        assert_eq!(signed7(63), 63);
        assert_eq!(signed7(64), -64);
        assert_eq!(signed7(127), -1);
    }

    #[test]
    fn test_pec_threads() {
        assert_eq!(PEC_THREADS.len(), 64);
    }
}
