// U01 writer - Barudan format with FAST/SLOW speed commands

//! U01 (Barudan) embroidery format writer
//!
//! The U01 format is used by Barudan embroidery machines.
//! It supports FAST/SLOW commands and uses explicit needle changes.

//! Barudan U01 format writer
//!
//! Writes U01 format with FAST/SLOW speed commands and byte-encoded coordinates
//! for industrial Barudan embroidery machines.

use crate::core::constants::*;
use crate::core::encoder::{EncoderSettings, Transcoder};
use crate::core::pattern::EmbPattern;
use crate::utils::error::Result;
use crate::utils::functions::decode_embroidery_command;
use std::io::Write;

/// Default encoder settings for U01 format
pub fn default_settings() -> EncoderSettings {
    EncoderSettings {
        max_stitch: 127.0,
        max_jump: 127.0,
        full_jump: false,
        thread_change_command: NEEDLE_SET,
        sequin_contingency: CONTINGENCY_SEQUIN_JUMP,
        writes_speeds: true,
        ..Default::default()
    }
}

/// Write U01 format embroidery file
pub fn write(pattern: &EmbPattern, file: &mut impl Write) -> Result<()> {
    write_with_settings(pattern, file, default_settings())
}

/// Write U01 format with custom settings
pub fn write_with_settings(
    pattern: &EmbPattern,
    file: &mut impl Write,
    settings: EncoderSettings,
) -> Result<()> {
    // Encode the pattern
    let mut encoded = EmbPattern::new();
    let mut transcoder = Transcoder::with_settings(settings);
    transcoder.transcode(pattern, &mut encoded)?;

    // Write first 128 bytes of padding
    for _ in 0..0x80 {
        file.write_all(b"0")?;
    }

    let stitches = encoded.stitches();
    if stitches.is_empty() {
        return Ok(());
    }

    // Calculate bounds
    let (min_x, min_y, max_x, max_y) = encoded.bounds();

    // Write header information (128 bytes more)
    write_i16_le(file, min_x as i16)?;
    write_i16_le(file, -(max_y as i16))?; // Flip Y
    write_i16_le(file, max_x as i16)?;
    write_i16_le(file, -(min_y as i16))?; // Flip Y
    write_i32_le(file, 0)?; // Unknown

    // Write stitch count
    write_i32_le(file, (stitches.len() + 1) as i32)?;

    // Write last stitch position
    let last_stitch = &stitches[stitches.len() - 1];
    write_i16_le(file, last_stitch.x as i16)?;
    write_i16_le(file, -(last_stitch.y as i16))?; // Flip Y

    // Pad to 0x100
    let current_pos = 0x80 + 20; // 128 + header data
    for _ in current_pos..0x100 {
        file.write_all(b"\x00")?;
    }

    // Write stitches
    let mut xx = 0.0;
    let mut yy = 0.0;
    let mut trigger_fast = false;
    let mut trigger_slow = false;

    for stitch in stitches {
        let x = stitch.x;
        let y = stitch.y;
        let data = stitch.command & COMMAND_MASK;

        let dx = (x - xx).round() as i32;
        let dy = (y - yy).round() as i32;
        xx += dx as f64;
        yy += dy as f64;

        // Handle FAST/SLOW triggers
        if data == SLOW {
            trigger_slow = true;
            continue;
        }
        if data == FAST {
            trigger_fast = true;
            continue;
        }

        let mut cmd: u8 = 0x80;

        // Set direction flags
        if dy >= 0 {
            cmd |= 0x40;
        }
        if dx <= 0 {
            cmd |= 0x20;
        }

        let delta_x = dx.unsigned_abs() as u8;
        let delta_y = dy.unsigned_abs() as u8;

        match data {
            STITCH => {
                if trigger_fast {
                    trigger_fast = false;
                    cmd |= 0x02;
                }
                if trigger_slow {
                    trigger_slow = false;
                    cmd |= 0x04;
                }
                file.write_all(&[cmd, delta_y, delta_x])?;
            }
            JUMP => {
                if trigger_fast {
                    trigger_fast = false;
                    cmd |= 0x02;
                }
                if trigger_slow {
                    trigger_slow = false;
                    cmd |= 0x04;
                }
                cmd |= 0x01;
                file.write_all(&[cmd, delta_y, delta_x])?;
            }
            STOP => {
                cmd |= 0x08;
                file.write_all(&[cmd, delta_y, delta_x])?;
            }
            TRIM => {
                cmd |= 0x07;
                file.write_all(&[cmd, delta_y, delta_x])?;
            }
            NEEDLE_SET => {
                let decoded = decode_embroidery_command(stitch.command);
                let mut needle = decoded.2.unwrap_or(1);
                if needle >= 15 {
                    needle = (needle % 15) + 1;
                }
                cmd |= 0x08;
                cmd += needle as u8;
                file.write_all(&[cmd, delta_y, delta_x])?;
            }
            END => {
                break;
            }
            _ => {}
        }
    }

    // Write end marker
    file.write_all(&[0xF8, 0x00, 0x00])?;

    Ok(())
}

fn write_i16_le(file: &mut impl Write, value: i16) -> Result<()> {
    file.write_all(&value.to_le_bytes())?;
    Ok(())
}

fn write_i32_le(file: &mut impl Write, value: i32) -> Result<()> {
    file.write_all(&value.to_le_bytes())?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::thread::EmbThread;
    use crate::formats::io::readers::u01 as u01_reader;
    use std::io::Cursor;

    #[test]
    fn test_write_u01_basic() {
        let mut pattern = EmbPattern::new();
        pattern.add_thread(EmbThread::from_rgb(255, 0, 0));
        pattern.add_stitch_absolute(STITCH, 0.0, 0.0);
        pattern.add_stitch_absolute(STITCH, 100.0, 100.0);
        pattern.add_stitch_absolute(END, 100.0, 100.0);

        let mut buffer = Vec::new();
        write(&pattern, &mut buffer).unwrap();

        // Should have written data
        assert!(buffer.len() > 0x100);
    }

    #[test]
    fn test_u01_round_trip() {
        let mut pattern = EmbPattern::new();
        pattern.add_thread(EmbThread::from_rgb(255, 0, 0));
        pattern.add_stitch_absolute(STITCH, 0.0, 0.0);
        pattern.add_stitch_absolute(STITCH, 100.0, 0.0);
        pattern.add_stitch_absolute(STITCH, 100.0, 100.0);
        pattern.add_stitch_absolute(STITCH, 0.0, 100.0);
        pattern.add_stitch_absolute(TRIM, 0.0, 100.0);
        pattern.add_stitch_absolute(END, 0.0, 100.0);

        // Write to buffer
        let mut buffer = Vec::new();
        write(&pattern, &mut buffer).unwrap();

        // Read back
        let mut cursor = Cursor::new(buffer);
        let mut pattern2 = EmbPattern::new();
        u01_reader::read(&mut cursor, &mut pattern2).unwrap();

        // Should have similar number of stitches (encoding may add/modify slightly)
        assert!(!pattern2.stitches().is_empty());
    }

    #[test]
    fn test_u01_with_needle_change() {
        let mut pattern = EmbPattern::new();
        pattern.add_thread(EmbThread::from_rgb(255, 0, 0));
        pattern.add_thread(EmbThread::from_rgb(0, 255, 0));

        pattern.add_stitch_absolute(STITCH, 0.0, 0.0);
        pattern.add_stitch_absolute(STITCH, 100.0, 0.0);
        pattern.add_stitch_absolute(
            crate::utils::functions::encode_thread_change(NEEDLE_SET, None, Some(2), None),
            100.0,
            0.0,
        );
        pattern.add_stitch_absolute(STITCH, 100.0, 100.0);
        pattern.add_stitch_absolute(END, 100.0, 100.0);

        let mut buffer = Vec::new();
        write(&pattern, &mut buffer).unwrap();

        assert!(buffer.len() > 0x100);
    }
}
