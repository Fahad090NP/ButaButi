//! WebAssembly bindings for Butabuti
//!
//! This module provides browser-compatible API for embroidery file conversion.
//! Enables reading, writing, and converting embroidery patterns in the browser
//! without server-side processing.
//!
//! # Features
//!
//! - Convert between embroidery formats directly in the browser
//! - Get pattern statistics (stitch count, dimensions, colors)
//! - Export to SVG for visualization
//! - JSON serialization for pattern data
//!
//! # Example
//!
//! ```javascript
//! // Load WASM module
//! import init, { convert_pattern, get_pattern_info } from './pkg/butabuti.js';
//!
//! await init();
//!
//! // Convert DST to PES
//! const dstBytes = new Uint8Array([...]); // File contents
//! const pesBytes = convert_pattern(dstBytes, 'dst', 'pes');
//!
//! // Get pattern statistics
//! const info = get_pattern_info(dstBytes, 'dst');
//! console.log(JSON.parse(info));
//! ```

use crate::core::pattern::EmbPattern;
use crate::formats::io::readers;
use crate::formats::io::writers;
use crate::utils::error::{Error, Result};
use std::io::Cursor;
use wasm_bindgen::prelude::*;

/// Initialize panic hook for better error messages in browser console
#[wasm_bindgen(start)]
pub fn init_panic_hook() {
    console_error_panic_hook::set_once();
}

/// Internal helper: Unified reader that handles all format API variations
///
/// # API Inconsistency Note
///
/// Butabuti readers currently use two different patterns:
/// - **Legacy API**: Returns `Result<EmbPattern>` (DST, JEF, EXP, PEC, JSON)
/// - **Modern API**: Mutates `&mut EmbPattern` and returns `Result<()>` (PES, VP3, XXX, U01, TBF, etc.)
///
/// This helper abstracts the difference to provide a unified interface for WASM.
/// Future refactor: unify all readers to use the mutation pattern.
fn read_pattern(data: &[u8], format: &str) -> Result<EmbPattern> {
    let mut cursor = Cursor::new(data);
    let mut pattern = EmbPattern::new();

    match format.to_lowercase().as_str() {
        // Formats that return EmbPattern (legacy API)
        "dst" => {
            pattern = readers::dst::read(&mut cursor, None)?;
        },
        "jef" => {
            pattern = readers::jef::read(&mut cursor, None)?;
        },
        "exp" => {
            pattern = readers::exp::read(&mut cursor)?;
        },
        "pec" => {
            pattern = readers::pec::read(&mut cursor)?;
        },
        "json" => {
            pattern = readers::json::read(&mut cursor)?;
        },

        // Formats that mutate pattern (modern API)
        "pes" => {
            readers::pes::read(&mut cursor, &mut pattern)?;
        },
        "vp3" => {
            readers::vp3::read(&mut cursor, &mut pattern)?;
        },
        "xxx" => {
            readers::xxx::read(&mut cursor, &mut pattern)?;
        },
        "u01" => {
            readers::u01::read(&mut cursor, &mut pattern)?;
        },
        "tbf" => {
            readers::tbf::read(&mut cursor, &mut pattern)?;
        },
        "csv" => {
            readers::csv::read(&mut cursor, &mut pattern)?;
        },
        "col" => {
            readers::col::read(&mut cursor, &mut pattern)?;
        },
        "edr" => {
            readers::edr::read(&mut cursor, &mut pattern)?;
        },
        "inf" => {
            readers::inf::read(&mut cursor, &mut pattern)?;
        },
        "gcode" => {
            readers::gcode::read(&mut cursor, &mut pattern)?;
        },

        _ => {
            return Err(Error::UnsupportedFormat(format!(
                "Unsupported read format: {}",
                format
            )))
        },
    }

    Ok(pattern)
}

/// Internal helper: Unified writer that handles all format API variations
///
/// # API Inconsistency Note
///
/// Butabuti writers have varying signatures:
/// - Some require `Seek` capability (PES, PEC, XXX, TBF, INF) - use Cursor
/// - Some have extra parameters with defaults (DST, JEF, CSV)
/// - Most accept `impl Write` (EXP, VP3, U01, SVG, JSON, TXT, COL, EDR)
///
/// This helper uses appropriate defaults and Cursor wrappers as needed.
fn write_pattern(pattern: &EmbPattern, format: &str) -> Result<Vec<u8>> {
    let mut output = Vec::new();

    match format.to_lowercase().as_str() {
        // Formats with extra parameters - use defaults
        "dst" => {
            writers::dst::write(&mut output, pattern, false, 121)?;
        },
        "jef" => {
            writers::jef::write(&mut output, pattern, true, 127, "")?;
        },
        "csv" => {
            writers::csv::write(&mut output, pattern, writers::csv::CsvVersion::Default)?;
        },

        // Formats that need Cursor for Seek capability
        "pes" => {
            let mut cursor = Cursor::new(&mut output);
            writers::pes::write_pes(pattern, &mut cursor, writers::pes::PesVersion::V6, false)?;
        },
        "pec" => {
            let mut cursor = Cursor::new(&mut output);
            writers::pec::write(&mut cursor, pattern)?;
        },
        "xxx" => {
            let mut cursor = Cursor::new(&mut output);
            writers::xxx::write(pattern, &mut cursor)?;
        },
        "tbf" => {
            let mut cursor = Cursor::new(&mut output);
            writers::tbf::write(pattern, &mut cursor)?;
        },
        "inf" => {
            let mut cursor = Cursor::new(&mut output);
            writers::inf::write(pattern, &mut cursor)?;
        },

        // Standard writers (impl Write)
        "exp" => {
            writers::exp::write(&mut output, pattern)?;
        },
        "vp3" => {
            writers::vp3::write(&mut output, pattern)?;
        },
        "u01" => {
            writers::u01::write(pattern, &mut output)?;
        },
        "svg" => {
            writers::svg::write(pattern, &mut output)?;
        },
        "json" => {
            writers::json::write(&mut output, pattern)?;
        },
        "txt" => {
            writers::txt::write(pattern, &mut output)?;
        },
        "col" => {
            writers::col::write(pattern, &mut output)?;
        },
        "edr" => {
            writers::edr::write(pattern, &mut output)?;
        },

        _ => {
            return Err(Error::UnsupportedFormat(format!(
                "Unsupported write format: {}",
                format
            )))
        },
    }

    Ok(output)
}

/// Convert an embroidery pattern from one format to another
///
/// # Arguments
///
/// * `input_data` - Raw bytes of the input file
/// * `input_format` - Format of input file (e.g., "dst", "pes", "jef")
/// * `output_format` - Desired output format
///
/// # Returns
///
/// Raw bytes of the converted file, or error message as JSON
///
/// # Example
///
/// ```javascript
/// const dstBytes = new Uint8Array([...]);
/// const pesBytes = convert_pattern(dstBytes, 'dst', 'pes');
/// ```
#[wasm_bindgen]
pub fn convert_pattern(
    input_data: &[u8],
    input_format: &str,
    output_format: &str,
) -> std::result::Result<Vec<u8>, JsValue> {
    // Read input pattern using unified reader
    let pattern = read_pattern(input_data, input_format)
        .map_err(|e| JsValue::from_str(&format!("Failed to read {}: {}", input_format, e)))?;

    // Write output pattern using unified writer
    let output = write_pattern(&pattern, output_format)
        .map_err(|e| JsValue::from_str(&format!("Failed to write {}: {}", output_format, e)))?;

    Ok(output)
}

/// Get information about an embroidery pattern
///
/// Returns statistics about the pattern including stitch count, dimensions,
/// color information, and metadata.
///
/// # Arguments
///
/// * `input_data` - Raw bytes of the input file
/// * `format` - Format of input file (e.g., "dst", "pes", "jef")
///
/// # Returns
///
/// JSON string with pattern statistics:
/// ```json
/// {
///   "stitch_count": 1234,
///   "color_count": 5,
///   "bounds": {
///     "min_x": 0.0,
///     "min_y": 0.0,
///     "max_x": 1000.0,
///     "max_y": 800.0
///   },
///   "width_mm": 100.0,
///   "height_mm": 80.0,
///   "colors": [
///     {"red": 255, "green": 0, "blue": 0, "description": "Red"},
///     ...
///   ]
/// }
/// ```
#[wasm_bindgen]
pub fn get_pattern_info(input_data: &[u8], format: &str) -> std::result::Result<String, JsValue> {
    // Read pattern using unified reader
    let pattern = read_pattern(input_data, format)
        .map_err(|e| JsValue::from_str(&format!("Failed to read {}: {}", format, e)))?;

    // Build statistics
    let (min_x, min_y, max_x, max_y) = pattern.bounds();
    let width_mm = (max_x - min_x) / 10.0;
    let height_mm = (max_y - min_y) / 10.0;

    let colors: Vec<serde_json::Value> = pattern
        .threads()
        .iter()
        .map(|thread| {
            serde_json::json!({
                "red": thread.red(),
                "green": thread.green(),
                "blue": thread.blue(),
                "description": thread.description.as_deref().unwrap_or("Unknown")
            })
        })
        .collect();

    let info = serde_json::json!({
        "stitch_count": pattern.count_stitches(),
        "color_count": pattern.threads().len(),
        "color_changes": pattern.count_color_changes(),
        "jumps": pattern.count_jumps(),
        "trims": pattern.count_trims(),
        "bounds": {
            "min_x": min_x,
            "min_y": min_y,
            "max_x": max_x,
            "max_y": max_y
        },
        "width_mm": width_mm,
        "height_mm": height_mm,
        "colors": colors
    });

    Ok(info.to_string())
}

/// Export pattern to SVG for visualization
///
/// Converts embroidery pattern to SVG format for display in the browser.
///
/// # Arguments
///
/// * `input_data` - Raw bytes of the input file
/// * `format` - Format of input file (e.g., "dst", "pes", "jef")
///
/// # Returns
///
/// SVG string that can be embedded in HTML
#[wasm_bindgen]
pub fn export_to_svg(input_data: &[u8], format: &str) -> std::result::Result<String, JsValue> {
    // Read pattern using unified reader
    let pattern = read_pattern(input_data, format)
        .map_err(|e| JsValue::from_str(&format!("Failed to read {}: {}", format, e)))?;

    // Convert to SVG
    let mut svg_data = Vec::new();
    writers::svg::write(&pattern, &mut svg_data)
        .map_err(|e| JsValue::from_str(&format!("Failed to write SVG: {}", e)))?;

    String::from_utf8(svg_data)
        .map_err(|e| JsValue::from_str(&format!("Invalid UTF-8 in SVG: {}", e)))
}

/// List all supported formats
///
/// Returns a JSON array of supported input and output formats.
/// Dynamically queries the FormatRegistry for accurate format information.
///
/// # Returns
///
/// JSON string with format information:
/// ```json
/// {
///   "input_formats": [
///     {"name": "DST", "extensions": ["dst"], "description": "Tajima DST format"},
///     ...
///   ],
///   "output_formats": [...]
/// }
/// ```
#[wasm_bindgen]
pub fn list_formats() -> String {
    use crate::formats::registry::FormatRegistry;

    let registry = FormatRegistry::new();

    let input_formats: Vec<serde_json::Value> = registry
        .readable_formats()
        .iter()
        .map(|f| {
            serde_json::json!({
                "name": f.name.to_lowercase(),
                "display_name": f.name,
                "extensions": f.extensions,
                "description": f.description
            })
        })
        .collect();

    let output_formats: Vec<serde_json::Value> = registry
        .writable_formats()
        .iter()
        .map(|f| {
            serde_json::json!({
                "name": f.name.to_lowercase(),
                "display_name": f.name,
                "extensions": f.extensions,
                "description": f.description
            })
        })
        .collect();

    let info = serde_json::json!({
        "input_formats": input_formats,
        "output_formats": output_formats
    });

    info.to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_list_formats() {
        let formats = list_formats();
        assert!(formats.contains("dst"));
        assert!(formats.contains("pes"));
        assert!(formats.contains("svg"));
    }
}
