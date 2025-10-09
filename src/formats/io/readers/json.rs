//! JSON format reader for embroidery patterns
//!
//! Provides lossless interchange format preserving all pattern data including extras,
//! metadata, and complete stitch information in human-readable JSON structure.

use crate::core::constants::*;
use crate::core::pattern::EmbPattern;
use crate::core::thread::EmbThread;
use crate::utils::error::{Error, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::io::Read;

/// JSON representation of an embroidery pattern
#[derive(Debug, Serialize, Deserialize)]
struct JsonPattern {
    #[serde(default)]
    metadata: HashMap<String, String>,

    #[serde(default)]
    threads: Vec<JsonThread>,

    #[serde(default)]
    stitches: Vec<JsonStitch>,
}

/// JSON representation of a thread
#[derive(Debug, Serialize, Deserialize)]
struct JsonThread {
    color: String,

    #[serde(skip_serializing_if = "Option::is_none")]
    description: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    catalog_number: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    brand: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    chart: Option<String>,
}

/// JSON representation of a stitch
#[derive(Debug, Serialize, Deserialize)]
struct JsonStitch {
    command: String,
    x: f64,
    y: f64,
}

/// Read a JSON embroidery pattern
pub fn read<R: Read>(reader: &mut R) -> Result<EmbPattern> {
    let json_pattern: JsonPattern = serde_json::from_reader(reader)
        .map_err(|e| Error::Parse(format!("JSON parse error: {}", e)))?;

    let mut pattern = EmbPattern::new();

    // Add metadata
    for (key, value) in json_pattern.metadata {
        pattern.add_metadata(&key, &value);
    }

    // Add threads
    for json_thread in json_pattern.threads {
        let color = parse_color(&json_thread.color)?;
        let mut thread = EmbThread::new(color);

        if let Some(desc) = json_thread.description {
            thread = thread.with_description(&desc);
        }
        if let Some(cat) = json_thread.catalog_number {
            thread = thread.with_catalog_number(&cat);
        }
        if let Some(brand) = json_thread.brand {
            thread = thread.with_brand(&brand);
        }
        if let Some(chart) = json_thread.chart {
            thread = thread.with_chart(&chart);
        }

        pattern.add_thread(thread);
    }

    // Add stitches
    for json_stitch in json_pattern.stitches {
        let command = parse_command(&json_stitch.command)?;
        pattern.add_stitch_absolute(command, json_stitch.x, json_stitch.y);
    }

    Ok(pattern)
}

/// Parse color from hex string
fn parse_color(color_str: &str) -> Result<u32> {
    if let Some(hex) = color_str.strip_prefix('#') {
        u32::from_str_radix(hex, 16)
            .map_err(|_| Error::Parse(format!("Invalid color: {}", color_str)))
    } else if let Some(hex) = color_str.strip_prefix("0x") {
        u32::from_str_radix(hex, 16)
            .map_err(|_| Error::Parse(format!("Invalid color: {}", color_str)))
    } else {
        u32::from_str_radix(color_str, 16)
            .map_err(|_| Error::Parse(format!("Invalid color: {}", color_str)))
    }
}

/// Parse command from string
fn parse_command(cmd_str: &str) -> Result<u32> {
    match cmd_str.to_uppercase().as_str() {
        "STITCH" => Ok(STITCH),
        "JUMP" => Ok(JUMP),
        "TRIM" => Ok(TRIM),
        "COLOR_CHANGE" => Ok(COLOR_CHANGE),
        "NEEDLE_SET" => Ok(NEEDLE_SET),
        "STOP" => Ok(STOP),
        "END" => Ok(END),
        "SEQUENCE_BREAK" => Ok(SEQUENCE_BREAK),
        "COLOR_BREAK" => Ok(COLOR_BREAK),
        "SLOW" => Ok(SLOW),
        "FAST" => Ok(FAST),
        "SEQUIN_MODE" => Ok(SEQUIN_MODE),
        "SEQUIN_EJECT" => Ok(SEQUIN_EJECT),
        _ => Err(Error::Parse(format!("Unknown command: {}", cmd_str))),
    }
}

/// Read JSON file from path
pub fn read_file(path: &str) -> Result<EmbPattern> {
    let file = std::fs::File::open(path)?;
    let reader = std::io::BufReader::new(file);
    read(&mut std::io::BufReader::new(reader))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_color() {
        assert_eq!(parse_color("#FF0000").unwrap(), 0xFF0000);
        assert_eq!(parse_color("0xFF0000").unwrap(), 0xFF0000);
        assert_eq!(parse_color("FF0000").unwrap(), 0xFF0000);
    }

    #[test]
    fn test_parse_command() {
        assert_eq!(parse_command("STITCH").unwrap(), STITCH);
        assert_eq!(parse_command("JUMP").unwrap(), JUMP);
        assert_eq!(parse_command("COLOR_CHANGE").unwrap(), COLOR_CHANGE);
        assert_eq!(parse_command("END").unwrap(), END);
    }

    #[test]
    fn test_read_simple_json() {
        let json = r##"{
            "metadata": {
                "name": "Test Pattern"
            },
            "threads": [
                {
                    "color": "#FF0000"
                }
            ],
            "stitches": [
                {"command": "STITCH", "x": 10.0, "y": 10.0},
                {"command": "STITCH", "x": 20.0, "y": 20.0},
                {"command": "END", "x": 20.0, "y": 20.0}
            ]
        }"##;

        let mut cursor = std::io::Cursor::new(json.as_bytes());
        let pattern = read(&mut cursor).unwrap();

        assert_eq!(
            pattern.get_metadata("name"),
            Some(&"Test Pattern".to_string())
        );
        assert_eq!(pattern.threads().len(), 1);
        assert_eq!(pattern.stitches().len(), 3);
    }

    #[test]
    fn test_read_empty_json() {
        let json = r##"{}"##;
        let mut cursor = std::io::Cursor::new(json.as_bytes());
        let pattern = read(&mut cursor).unwrap();

        assert_eq!(pattern.threads().len(), 0);
        assert_eq!(pattern.stitches().len(), 0);
    }

    #[test]
    fn test_invalid_json() {
        let json = r##"{ invalid json"##;
        let mut cursor = std::io::Cursor::new(json.as_bytes());
        let result = read(&mut cursor);
        assert!(result.is_err());
    }
}
