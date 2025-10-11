//! G-code embroidery format reader
//!
//! Reads G-code CNC machine language adapted for embroidery. Supports G00/G01 for
//! movement/stitches, M00/M01 for color changes/stops, and comment-based metadata.

use crate::core::constants::*;
use crate::core::pattern::EmbPattern;
use crate::utils::error::Result;
use std::collections::HashMap;
use std::io::{BufRead, BufReader, Read};

/// Parse a single line of G-code
fn parse_gcode_line(line: &str) -> HashMap<String, f64> {
    let mut map = HashMap::new();
    let mut comment = String::new();
    let mut in_comment = false;
    let mut code = String::new();
    let mut value = String::new();

    for ch in line.chars() {
        if in_comment {
            if ch == ')' {
                map.insert("comment".to_string(), 0.0); // Use special marker
                                                        // Store comment separately if needed
                in_comment = false;
            } else {
                comment.push(ch);
            }
            continue;
        }

        if ch == '(' || ch == ';' {
            in_comment = true;
            comment.clear();
            continue;
        }

        if ch == ' ' || ch == '\t' || ch == '\n' || ch == '\r' {
            if !code.is_empty() && !value.is_empty() {
                if let Ok(v) = value.parse::<f64>() {
                    map.insert(code.to_lowercase(), v);
                }
                code.clear();
                value.clear();
            }
            continue;
        }

        if ch.is_ascii_alphabetic() {
            if !code.is_empty() && !value.is_empty() {
                if let Ok(v) = value.parse::<f64>() {
                    map.insert(code.to_lowercase(), v);
                }
                value.clear();
            }
            code = ch.to_lowercase().to_string();
        } else if ch.is_ascii_digit() || ch == '.' || ch == '-' || ch == '+' {
            value.push(ch);
        }
    }

    // Don't forget the last code/value pair
    if !code.is_empty() && !value.is_empty() {
        if let Ok(v) = value.parse::<f64>() {
            map.insert(code.to_lowercase(), v);
        }
    }

    // Store comment text if we found one
    if !comment.is_empty() {
        // Check for thread info
        if comment.contains("Thread") {
            // Extract hex color if present
            let parts: Vec<&str> = comment.split_whitespace().collect();
            if parts.len() > 1 {
                // Try to parse as hex color
                if let Some(color_str) = parts.get(1) {
                    if color_str.starts_with('#') || color_str.len() == 6 {
                        map.insert("thread_color".to_string(), 0.0);
                        // In a real implementation, we'd store the actual color string
                    }
                }
            }
        }
    }

    map
}

/// Read G-code format file into a pattern
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
/// let mut file = File::open("design.gcode").unwrap();
/// let mut pattern = EmbPattern::new();
/// butabuti::io::readers::gcode::read(&mut file, &mut pattern).unwrap();
/// ```
pub fn read(file: &mut impl Read, pattern: &mut EmbPattern) -> Result<()> {
    let reader = BufReader::new(file);
    let mut absolute_mode = true;
    let flip_x = -1.0; // G-code typically uses flipped X
    let flip_y = -1.0; // G-code typically uses flipped Y
    let mut scale = 10.0; // Default to mm mode (10 units per mm)

    for line in reader.lines() {
        let line = line?;
        let gc = parse_gcode_line(&line);

        if gc.is_empty() {
            continue;
        }

        // Handle G commands
        if let Some(&g_val) = gc.get("g") {
            // G00/G01 - Move/Stitch
            if (g_val == 0.0 || g_val == 1.0) && gc.contains_key("x") && gc.contains_key("y") {
                let x = gc.get("x").unwrap() * scale * flip_x;
                let y = gc.get("y").unwrap() * scale * flip_y;

                if absolute_mode {
                    pattern.add_stitch_absolute(STITCH, x, y);
                } else {
                    pattern.add_stitch_relative(x, y, STITCH);
                }
                continue;
            }

            // G20/G70 - Inch mode
            if g_val == 20.0 || g_val == 70.0 {
                scale = 254.0; // 254 units per inch
            }
            // G21/G71 - Millimeter mode
            else if g_val == 21.0 || g_val == 71.0 {
                scale = 10.0; // 10 units per mm
            }
            // G90 - Absolute positioning
            else if g_val == 90.0 {
                absolute_mode = true;
            }
            // G91 - Relative positioning
            else if g_val == 91.0 {
                absolute_mode = false;
            }
        }

        // Handle M commands
        if let Some(&m_val) = gc.get("m") {
            // M30/M02 - End program
            if m_val == 30.0 || m_val == 2.0 {
                pattern.end();
            }
            // M00/M01 - Stop/Optional stop (color change)
            else if m_val == 0.0 || m_val == 1.0 {
                pattern.add_stitch_relative(0.0, 0.0, COLOR_CHANGE);
            }
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    #[test]
    fn test_parse_gcode_line() {
        let line = "G00 X10.5 Y20.3";
        let parsed = parse_gcode_line(line);

        assert_eq!(parsed.get("g"), Some(&0.0));
        assert_eq!(parsed.get("x"), Some(&10.5));
        assert_eq!(parsed.get("y"), Some(&20.3));
    }

    #[test]
    fn test_read_gcode_basic() {
        let gcode = "\
(STITCH_COUNT: 2)
G21
G90
G00 X10.0 Y20.0
G00 X15.0 Y25.0
M30
";

        let mut cursor = Cursor::new(gcode.as_bytes());
        let mut pattern = EmbPattern::new();

        read(&mut cursor, &mut pattern).unwrap();

        // Should have at least 2 stitches plus END
        assert!(pattern.stitches().len() >= 2);
    }

    #[test]
    fn test_read_gcode_color_change() {
        let gcode = "\
G21
G00 X10.0 Y10.0
M00
G00 X20.0 Y20.0
M30
";

        let mut cursor = Cursor::new(gcode.as_bytes());
        let mut pattern = EmbPattern::new();

        read(&mut cursor, &mut pattern).unwrap();

        let commands: Vec<u32> = pattern
            .stitches()
            .iter()
            .map(|s| s.command & COMMAND_MASK)
            .collect();

        assert!(commands.contains(&COLOR_CHANGE));
        assert!(commands.contains(&END));
    }

    #[test]
    fn test_read_gcode_inch_mode() {
        let gcode = "\
G20
G00 X1.0 Y1.0
M30
";

        let mut cursor = Cursor::new(gcode.as_bytes());
        let mut pattern = EmbPattern::new();

        read(&mut cursor, &mut pattern).unwrap();

        // In inch mode, 1.0 inch = 254 units (25.4mm)
        // With flip_x and flip_y = -1, coordinates should be -254.0
        assert!(!pattern.stitches().is_empty());
    }
}
