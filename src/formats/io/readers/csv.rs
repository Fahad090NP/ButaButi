//! CSV format reader for embroidery patterns
//!
//! Human-readable comma-separated values format for debugging and analysis.
//! Each line represents a stitch with coordinates, command, and thread information.

use crate::core::constants::*;
use crate::core::pattern::EmbPattern;
use crate::core::thread::EmbThread;
use anyhow::{Context, Result};
use std::io::{BufRead, BufReader, Read};

/// Read CSV embroidery format
///
/// CSV format supports lossless encoding with metadata, threads, and stitches
/// Format lines:
/// - `*,index,command_name [modifiers],x,y` - Stitch or command
/// - `#,...` - Comment (ignored)
/// - `@,key,value` - Metadata
/// - `$,index,color[,description,brand,catalog,details,weight]` - Thread
pub fn read(file: &mut impl Read, pattern: &mut EmbPattern) -> Result<()> {
    let reader = BufReader::new(file);

    for line in reader.lines() {
        let line = line.context("Failed to read CSV line")?;
        let trimmed = line.trim();

        // Skip empty lines
        if trimmed.is_empty() {
            continue;
        }

        let parts: Vec<&str> = trimmed.split(',').map(|s| s.trim()).collect();
        if parts.is_empty() {
            continue;
        }

        match parts[0] {
            // Comment line
            s if s.starts_with('#') => continue,

            // Stitch or command line: *,index,command_name [modifiers],x,y
            s if s.starts_with('*') => {
                if parts.len() < 3 {
                    continue;
                }

                let command_str = parts[2];
                let command = parse_command(command_str)?;

                if parts.len() >= 5 {
                    // Stitch with coordinates
                    let x = parts[3]
                        .parse::<f64>()
                        .context("Failed to parse X coordinate")?;
                    let y = parts[4]
                        .parse::<f64>()
                        .context("Failed to parse Y coordinate")?;
                    pattern.add_stitch_absolute(command, x, y);
                } else {
                    // Command without stitching (0, 0 position)
                    pattern.add_command(command, 0.0, 0.0);
                }
            }

            // Metadata line: @,key,value
            s if s.starts_with('@') => {
                if parts.len() >= 3 {
                    pattern.add_metadata(parts[1], parts[2]);
                }
            }

            // Thread line: $,index,color[,description,brand,catalog,details,weight]
            s if s.starts_with('$') => {
                if parts.len() < 3 {
                    continue;
                }

                let mut thread = EmbThread::from_rgb(0, 0, 0);

                // Parse color (can be hex #RRGGBB or RGB components)
                let color_str = parts[2];
                if !color_str.is_empty() {
                    // Check if this is embroidermodder format with separate RGB components
                    if parts.len() >= 5
                        && parts[2].len() <= 3
                        && parts[3].len() <= 3
                        && parts[4].len() <= 3
                    {
                        // Embroidermodder format: [index], [RED], [GREEN], [BLUE], [DESCRIPTION], [CATALOG]
                        if let (Ok(r), Ok(g), Ok(b)) = (
                            parts[2].parse::<u8>(),
                            parts[3].parse::<u8>(),
                            parts[4].parse::<u8>(),
                        ) {
                            thread = EmbThread::from_rgb(r, g, b);
                            if parts.len() > 5 && !parts[5].is_empty() {
                                thread.description = Some(parts[5].to_string());
                            }
                            if parts.len() > 6 && !parts[6].is_empty() {
                                thread.catalog_number = Some(parts[6].to_string());
                            }
                        }
                    } else {
                        // Standard format with color string
                        thread = EmbThread::from_string(color_str)?;

                        // Optional thread properties
                        if parts.len() > 3 && !parts[3].is_empty() {
                            thread.description = Some(parts[3].to_string());
                        }
                        if parts.len() > 4 && !parts[4].is_empty() {
                            thread.brand = Some(parts[4].to_string());
                        }
                        if parts.len() > 5 && !parts[5].is_empty() {
                            thread.catalog_number = Some(parts[5].to_string());
                        }
                        if parts.len() > 6 && !parts[6].is_empty() {
                            thread.chart = Some(parts[6].to_string());
                        }
                        if parts.len() > 7 && !parts[7].is_empty() {
                            thread.chart = Some(parts[7].to_string());
                        }
                    }
                }

                pattern.add_thread(thread);
            }

            _ => continue,
        }
    }

    Ok(())
}

/// Parse command string like "STITCH n1 t2" into command code
fn parse_command(cmd_str: &str) -> Result<u32> {
    let parts: Vec<&str> = cmd_str.split_whitespace().collect();
    if parts.is_empty() {
        return Ok(STITCH);
    }

    // Get base command
    let mut command = match parts[0] {
        "NO_COMMAND" => NO_COMMAND,
        "STITCH" => STITCH,
        "JUMP" => JUMP,
        "TRIM" => TRIM,
        "STOP" => STOP,
        "END" => END,
        "COLOR_CHANGE" => COLOR_CHANGE,
        "SEQUIN_MODE" => SEQUIN_MODE,
        "SEQUIN_EJECT" => SEQUIN_EJECT,
        "NEEDLE_SET" => NEEDLE_SET,
        "SLOW" => SLOW,
        "FAST" => FAST,
        "SET_CHANGE_SEQUENCE" => SET_CHANGE_SEQUENCE,
        "SEW_TO" => SEW_TO,
        "NEEDLE_AT" => NEEDLE_AT,
        "STITCH_BREAK" => STITCH_BREAK,
        "SEQUENCE_BREAK" => SEQUENCE_BREAK,
        "COLOR_BREAK" => COLOR_BREAK,
        "TIE_ON" => TIE_ON,
        "TIE_OFF" => TIE_OFF,
        "FRAME_EJECT" => FRAME_EJECT,
        "MATRIX_TRANSLATE" => MATRIX_TRANSLATE,
        "MATRIX_SCALE_ORIGIN" => MATRIX_SCALE_ORIGIN,
        "MATRIX_ROTATE_ORIGIN" => MATRIX_ROTATE_ORIGIN,
        "MATRIX_RESET" => MATRIX_RESET,
        "MATRIX_SCALE" => MATRIX_SCALE,
        "MATRIX_ROTATE" => MATRIX_ROTATE,
        _ => STITCH,
    };

    // Parse modifiers (n=needle, t=thread, o=order)
    for part in &parts[1..] {
        if let Some(stripped) = part.strip_prefix('n') {
            // Needle modifier: n1 = needle 1
            if let Ok(needle) = stripped.parse::<u32>() {
                command |= (needle + 1) << 16;
            }
        } else if let Some(stripped) = part.strip_prefix('t') {
            // Thread modifier: t2 = thread 2
            if let Ok(thread) = stripped.parse::<u32>() {
                command |= (thread + 1) << 8;
            }
        } else if let Some(stripped) = part.strip_prefix('o') {
            // Order modifier: o3 = order 3
            if let Ok(order) = stripped.parse::<u32>() {
                command |= (order + 1) << 24;
            }
        }
    }

    Ok(command)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    #[test]
    fn test_read_basic_csv() {
        let csv_data = "\
# Comment line
$,0,#FF0000,Red Thread,BrandA,CAT001
$,1,#00FF00,Green Thread
*,0,STITCH,100.0,200.0
*,1,STITCH,110.0,210.0
*,2,JUMP,150.0,250.0
*,3,COLOR_CHANGE
@,name,Test Pattern
";

        let mut cursor = Cursor::new(csv_data.as_bytes());
        let mut pattern = EmbPattern::new();

        read(&mut cursor, &mut pattern).expect("Failed to read CSV");

        assert_eq!(pattern.threads().len(), 2);
        assert_eq!(pattern.threads()[0].red(), 255);
        assert_eq!(pattern.threads()[0].green(), 0);
        assert_eq!(pattern.threads()[0].blue(), 0);

        assert!(pattern.stitches().len() >= 4);
        assert_eq!(
            pattern.get_metadata("name"),
            Some(&"Test Pattern".to_string())
        );
    }

    #[test]
    fn test_parse_command_modifiers() {
        let cmd = parse_command("COLOR_CHANGE n2 t3").unwrap();
        assert_eq!(cmd & COMMAND_MASK, COLOR_CHANGE);
        // Check needle and thread encoding
        assert_eq!((cmd >> 16) & 0xFF, 3); // needle 2 + 1
        assert_eq!((cmd >> 8) & 0xFF, 4); // thread 3 + 1
    }

    #[test]
    fn test_embroidermodder_format() {
        let csv_data = "$,0,255,128,64,Orange Thread,EM001\n";

        let mut cursor = Cursor::new(csv_data.as_bytes());
        let mut pattern = EmbPattern::new();

        read(&mut cursor, &mut pattern).expect("Failed to read CSV");

        assert_eq!(pattern.threads().len(), 1);
        assert_eq!(pattern.threads()[0].red(), 255);
        assert_eq!(pattern.threads()[0].green(), 128);
        assert_eq!(pattern.threads()[0].blue(), 64);
        assert_eq!(
            pattern.threads()[0].description,
            Some("Orange Thread".to_string())
        );
    }
}
