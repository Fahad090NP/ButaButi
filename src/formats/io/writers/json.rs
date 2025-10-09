//! JSON format writer for embroidery patterns
//!
//! Writes lossless interchange format preserving all pattern data including stitches,
//! threads, extras, and metadata in human-readable JSON structure.

use crate::core::constants::*;
use crate::core::pattern::EmbPattern;
use crate::utils::error::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::io::Write;

/// JSON representation of an embroidery pattern
#[derive(Debug, Serialize, Deserialize)]
struct JsonPattern {
    #[serde(skip_serializing_if = "HashMap::is_empty")]
    metadata: HashMap<String, String>,

    #[serde(skip_serializing_if = "Vec::is_empty")]
    threads: Vec<JsonThread>,

    #[serde(skip_serializing_if = "Vec::is_empty")]
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

/// Write an embroidery pattern to JSON
pub fn write<W: Write>(writer: &mut W, pattern: &EmbPattern) -> Result<()> {
    let json_pattern = to_json_pattern(pattern);
    serde_json::to_writer_pretty(writer, &json_pattern)?;
    Ok(())
}

/// Convert EmbPattern to JSON representation
fn to_json_pattern(pattern: &EmbPattern) -> JsonPattern {
    let mut metadata = HashMap::new();
    for (key, value) in pattern.metadata() {
        metadata.insert(key.clone(), value.clone());
    }

    let threads = pattern
        .threads()
        .iter()
        .map(|thread| JsonThread {
            color: format!("#{:06X}", thread.color),
            description: thread.description.clone(),
            catalog_number: thread.catalog_number.clone(),
            brand: thread.brand.clone(),
            chart: thread.chart.clone(),
        })
        .collect();

    let stitches = pattern
        .stitches()
        .iter()
        .map(|stitch| JsonStitch {
            command: command_to_string(stitch.command),
            x: stitch.x,
            y: stitch.y,
        })
        .collect();

    JsonPattern {
        metadata,
        threads,
        stitches,
    }
}

/// Convert command constant to string
fn command_to_string(command: u32) -> String {
    match command {
        STITCH => "STITCH".to_string(),
        JUMP => "JUMP".to_string(),
        TRIM => "TRIM".to_string(),
        COLOR_CHANGE => "COLOR_CHANGE".to_string(),
        NEEDLE_SET => "NEEDLE_SET".to_string(),
        STOP => "STOP".to_string(),
        END => "END".to_string(),
        SEQUENCE_BREAK => "SEQUENCE_BREAK".to_string(),
        COLOR_BREAK => "COLOR_BREAK".to_string(),
        SLOW => "SLOW".to_string(),
        FAST => "FAST".to_string(),
        SEQUIN_MODE => "SEQUIN_MODE".to_string(),
        SEQUIN_EJECT => "SEQUIN_EJECT".to_string(),
        _ => format!("UNKNOWN_{}", command),
    }
}

/// Write JSON file to path
pub fn write_file(path: &str, pattern: &EmbPattern) -> Result<()> {
    let mut file = std::fs::File::create(path)?;
    write(&mut file, pattern)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::thread::EmbThread;

    #[test]
    fn test_command_to_string() {
        assert_eq!(command_to_string(STITCH), "STITCH");
        assert_eq!(command_to_string(JUMP), "JUMP");
        assert_eq!(command_to_string(COLOR_CHANGE), "COLOR_CHANGE");
        assert_eq!(command_to_string(END), "END");
    }

    #[test]
    fn test_write_simple_pattern() {
        let mut pattern = EmbPattern::new();
        pattern.add_metadata("name", "Test Pattern");
        pattern.add_thread(EmbThread::new(0xFF0000));
        pattern.add_stitch_absolute(STITCH, 10.0, 10.0);
        pattern.add_stitch_absolute(STITCH, 20.0, 20.0);
        pattern.add_stitch_absolute(END, 20.0, 20.0);

        let mut output = Vec::new();
        write(&mut output, &pattern).unwrap();

        let json_str = String::from_utf8(output).unwrap();
        assert!(json_str.contains("Test Pattern"));
        assert!(json_str.contains("FF0000"));
        assert!(json_str.contains("STITCH"));
        assert!(json_str.contains("END"));
    }

    #[test]
    fn test_write_empty_pattern() {
        let pattern = EmbPattern::new();
        let mut output = Vec::new();
        write(&mut output, &pattern).unwrap();

        let json_str = String::from_utf8(output).unwrap();
        assert!(json_str.contains("{}") || json_str.contains("{\n}"));
    }

    #[test]
    fn test_write_thread_details() {
        let mut pattern = EmbPattern::new();
        pattern.add_thread(
            EmbThread::new(0xFF0000)
                .with_description("Red Thread")
                .with_brand("Test Brand")
                .with_catalog_number("123"),
        );

        let mut output = Vec::new();
        write(&mut output, &pattern).unwrap();

        let json_str = String::from_utf8(output).unwrap();
        assert!(json_str.contains("Red Thread"));
        assert!(json_str.contains("Test Brand"));
        assert!(json_str.contains("123"));
    }
}
