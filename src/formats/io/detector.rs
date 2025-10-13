//! Format detection for embroidery files
//!
//! Provides automatic format detection using magic bytes, file signatures,
//! and extension-based fallback detection.

use crate::core::pattern::EmbPattern;
use crate::utils::error::{Error, Result};
use std::io::{Read, Seek, SeekFrom};
use std::path::Path;

/// Supported embroidery file formats with detection capabilities
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Format {
    /// Tajima DST (no magic bytes, 512-byte header)
    DST,
    /// Brother PES (#PES or #PEC prefix)
    PES,
    /// Pfaff VP3 (%vsm% signature)
    VP3,
    /// Janome JEF (0x74 at offset 0)
    JEF,
    /// Melco EXP
    EXP,
    /// Brother PEC (#PEC prefix)
    PEC,
    /// Singer XXX
    XXX,
    /// Barudan U01
    U01,
    /// Tajima TBF
    TBF,
    /// Thread color list (COL)
    COL,
    /// Embird color (EDR)
    EDR,
    /// Thread information (INF)
    INF,
    /// JSON embroidery data
    JSON,
    /// CSV embroidery data
    CSV,
    /// G-code embroidery data
    GCODE,
    /// Husqvarna Viking HUS
    HUS,
    /// Unknown/unsupported format
    Unknown,
}

/// Format detector for automatic format recognition
pub struct FormatDetector;

impl FormatDetector {
    /// Detect format from file content by examining magic bytes/signatures
    ///
    /// This method reads the first several bytes of the file to identify
    /// the format based on known signatures:
    ///
    /// - **PES/PEC**: Starts with "#PES" or "#PEC"
    /// - **VP3**: Starts with "%vsm%"
    /// - **JEF**: First byte is 0x74
    /// - **JSON**: Starts with '{'
    /// - **CSV**: Contains commas in first line
    ///
    /// # Example
    ///
    /// ```no_run
    /// use butabuti::formats::io::detector::{FormatDetector, Format};
    /// use std::fs::File;
    ///
    /// let mut file = File::open("design.pes")?;
    /// let format = FormatDetector::detect_from_content(&mut file)?;
    /// assert_eq!(format, Format::PES);
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    pub fn detect_from_content<R: Read + Seek>(reader: &mut R) -> Result<Format> {
        // Save current position
        let start_pos = reader.stream_position().map_err(Error::Io)?;

        // Read first 512 bytes (enough for DST header and all magic bytes)
        let mut buffer = vec![0u8; 512];
        let bytes_read = reader.read(&mut buffer).map_err(Error::Io)?;

        // Restore original position
        reader.seek(SeekFrom::Start(start_pos)).map_err(Error::Io)?;

        if bytes_read < 4 {
            return Err(Error::Parse(
                "File too small to detect format (< 4 bytes)".to_string(),
            ));
        }

        // Check magic bytes/signatures

        // PES/PEC: "#PES" or "#PEC"
        if bytes_read >= 4 && buffer[0] == b'#' {
            let prefix = String::from_utf8_lossy(&buffer[0..4]);
            if prefix == "#PES" {
                return Ok(Format::PES);
            }
            if prefix == "#PEC" {
                return Ok(Format::PEC);
            }
        }

        // VP3: "%vsm%"
        if bytes_read >= 5 {
            let prefix = String::from_utf8_lossy(&buffer[0..5]);
            if prefix == "%vsm%" {
                return Ok(Format::VP3);
            }
        }

        // JEF: First byte 0x74 + additional validation to reduce false positives
        // JEF files have a specific structure with stitch count at offset 0x74
        if buffer[0] == 0x74 && bytes_read >= 128 {
            // Additional validation: check for reasonable header structure
            // JEF typically has low bytes in positions 1-3 for offset values
            if buffer[1] < 0x80 && buffer[2] < 0x80 && buffer[3] < 0x80 {
                return Ok(Format::JEF);
            }
        }

        // JSON: Starts with '{' (possibly with whitespace)
        let content_start = buffer.iter().position(|&b| !b.is_ascii_whitespace());
        if let Some(start) = content_start {
            if buffer[start] == b'{' {
                return Ok(Format::JSON);
            }
        }

        // CSV: Check for commas in first line
        if bytes_read >= 20 {
            let first_line = buffer[..bytes_read.min(100)]
                .iter()
                .position(|&b| b == b'\n')
                .unwrap_or(bytes_read.min(100));
            if buffer[..first_line].iter().filter(|&&b| b == b',').count() >= 2 {
                return Ok(Format::CSV);
            }
        }

        // DST: 512-byte header with specific structure (LA:, ST:, CO: markers)
        if bytes_read >= 512 {
            let header = String::from_utf8_lossy(&buffer[0..512]);
            if header.contains("LA:") || header.contains("ST:") || header.contains("CO:") {
                return Ok(Format::DST);
            }
        }

        // G-code: Look for typical G-code patterns
        if bytes_read >= 10 {
            let text = String::from_utf8_lossy(&buffer[0..bytes_read.min(100)]);
            if text.contains("G0") || text.contains("G1") || text.contains("M3") {
                return Ok(Format::GCODE);
            }
        }

        // If no signature detected, return Unknown
        Ok(Format::Unknown)
    }

    /// Detect format from file extension (fallback method)
    ///
    /// This provides a reasonable guess based on the file extension
    /// when content-based detection is not possible or has failed.
    ///
    /// # Example
    ///
    /// ```
    /// use butabuti::formats::io::detector::{FormatDetector, Format};
    /// use std::path::Path;
    ///
    /// let format = FormatDetector::detect_from_extension(Path::new("design.dst"))?;
    /// assert_eq!(format, Format::DST);
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    pub fn detect_from_extension(path: &Path) -> Result<Format> {
        let extension = path
            .extension()
            .and_then(|e| e.to_str())
            .ok_or_else(|| Error::Parse("File has no extension".to_string()))?;

        let format = match extension.to_lowercase().as_str() {
            "dst" => Format::DST,
            "pes" => Format::PES,
            "pec" => Format::PEC,
            "vp3" => Format::VP3,
            "jef" => Format::JEF,
            "exp" => Format::EXP,
            "xxx" => Format::XXX,
            "u01" => Format::U01,
            "tbf" => Format::TBF,
            "col" => Format::COL,
            "edr" => Format::EDR,
            "inf" => Format::INF,
            "json" => Format::JSON,
            "csv" => Format::CSV,
            "gcode" | "nc" => Format::GCODE,
            "hus" | "vip" => Format::HUS,
            _ => Format::Unknown,
        };

        Ok(format)
    }

    /// Detect format and read pattern automatically
    ///
    /// Combines format detection with pattern reading in one convenient method.
    /// Tries content-based detection first, falls back to extension-based detection,
    /// then reads the pattern using the appropriate reader.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use butabuti::formats::io::detector::FormatDetector;
    /// use butabuti::core::pattern::EmbPattern;
    /// use std::fs::File;
    ///
    /// let mut file = File::open("design.pes")?;
    /// let mut pattern = EmbPattern::new();
    /// FormatDetector::detect_and_read(&mut file, &mut pattern, Some("design.pes"))?;
    /// println!("Detected and read {} stitches", pattern.count_stitches());
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    pub fn detect_and_read<R: Read + Seek>(
        reader: &mut R,
        pattern: &mut EmbPattern,
        filename_hint: Option<&str>,
    ) -> Result<Format> {
        // Try content-based detection first
        let mut format = Self::detect_from_content(reader)?;

        // If unknown and we have a filename, try extension
        if format == Format::Unknown {
            if let Some(filename) = filename_hint {
                let path = Path::new(filename);
                format = Self::detect_from_extension(path)?;
            }
        }

        if format == Format::Unknown {
            return Err(Error::UnsupportedFormat(
                "Unable to detect file format".to_string(),
            ));
        }

        // Read pattern using detected format
        Self::read_with_format(reader, pattern, format)?;

        Ok(format)
    }

    /// Read pattern using a specific format
    ///
    /// Internal helper to read a pattern once format is known.
    fn read_with_format<R: Read + Seek>(
        reader: &mut R,
        pattern: &mut EmbPattern,
        format: Format,
    ) -> Result<()> {
        match format {
            // Formats with old API that return Result<EmbPattern>
            Format::DST => {
                *pattern = crate::formats::io::readers::dst::read(reader, None)?;
                Ok(())
            }
            Format::EXP => {
                *pattern = crate::formats::io::readers::exp::read(reader)?;
                Ok(())
            }
            Format::JSON => {
                *pattern = crate::formats::io::readers::json::read(reader)?;
                Ok(())
            }
            Format::JEF => {
                *pattern = crate::formats::io::readers::jef::read(reader, None)?;
                Ok(())
            }
            Format::PEC => {
                *pattern = crate::formats::io::readers::pec::read(reader)?;
                Ok(())
            }
            // Formats with new API that mutate pattern and return Result<()>
            Format::PES => crate::formats::io::readers::pes::read(reader, pattern),
            Format::VP3 => crate::formats::io::readers::vp3::read(reader, pattern),
            Format::XXX => crate::formats::io::readers::xxx::read(reader, pattern),
            Format::U01 => crate::formats::io::readers::u01::read(reader, pattern),
            Format::TBF => crate::formats::io::readers::tbf::read(reader, pattern),
            Format::COL => crate::formats::io::readers::col::read(reader, pattern),
            Format::EDR => crate::formats::io::readers::edr::read(reader, pattern),
            Format::INF => crate::formats::io::readers::inf::read(reader, pattern),
            Format::CSV => crate::formats::io::readers::csv::read(reader, pattern),
            Format::GCODE => crate::formats::io::readers::gcode::read(reader, pattern),
            // HUS not yet supported (reader not exported)
            Format::HUS => Err(Error::UnsupportedFormat(
                "HUS format reader not yet available".to_string(),
            )),
            Format::Unknown => Err(Error::UnsupportedFormat(
                "Unknown format cannot be read".to_string(),
            )),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    #[test]
    fn test_detect_pes_from_content() {
        let data = b"#PES0001\x00\x00\x00\x00";
        let mut reader = Cursor::new(data);
        let format = FormatDetector::detect_from_content(&mut reader).unwrap();
        assert_eq!(format, Format::PES);
        // Verify reader position restored
        assert_eq!(reader.position(), 0);
    }

    #[test]
    fn test_detect_pec_from_content() {
        let data = b"#PEC0001\x00\x00\x00\x00";
        let mut reader = Cursor::new(data);
        let format = FormatDetector::detect_from_content(&mut reader).unwrap();
        assert_eq!(format, Format::PEC);
    }

    #[test]
    fn test_detect_vp3_from_content() {
        let data = b"%vsm%\x00\x00\x00\x00";
        let mut reader = Cursor::new(data);
        let format = FormatDetector::detect_from_content(&mut reader).unwrap();
        assert_eq!(format, Format::VP3);
    }

    #[test]
    fn test_detect_jef_from_content() {
        let data = [0x74u8; 512];
        let mut reader = Cursor::new(&data[..]);
        let format = FormatDetector::detect_from_content(&mut reader).unwrap();
        assert_eq!(format, Format::JEF);
    }

    #[test]
    fn test_detect_json_from_content() {
        let data = b"{\"stitches\": []}";
        let mut reader = Cursor::new(data);
        let format = FormatDetector::detect_from_content(&mut reader).unwrap();
        assert_eq!(format, Format::JSON);
    }

    #[test]
    fn test_detect_csv_from_content() {
        let data = b"x,y,command\n10,20,0\n";
        let mut reader = Cursor::new(data);
        let format = FormatDetector::detect_from_content(&mut reader).unwrap();
        assert_eq!(format, Format::CSV);
    }

    #[test]
    fn test_detect_dst_from_content() {
        let mut data = vec![0x20u8; 512]; // Space-filled header
        let header_text = "LA:Design Name  ST:1000  CO:123  ";
        data[..header_text.len()].copy_from_slice(header_text.as_bytes());
        let mut reader = Cursor::new(data);
        let format = FormatDetector::detect_from_content(&mut reader).unwrap();
        assert_eq!(format, Format::DST);
    }

    #[test]
    fn test_detect_gcode_from_content() {
        let data = b"G0 X10 Y20\nG1 X30 Y40\n";
        let mut reader = Cursor::new(data);
        let format = FormatDetector::detect_from_content(&mut reader).unwrap();
        assert_eq!(format, Format::GCODE);
    }

    #[test]
    fn test_detect_from_extension_dst() {
        let path = Path::new("design.dst");
        let format = FormatDetector::detect_from_extension(path).unwrap();
        assert_eq!(format, Format::DST);
    }

    #[test]
    fn test_detect_from_extension_pes() {
        let path = Path::new("design.PES"); // Test case insensitivity
        let format = FormatDetector::detect_from_extension(path).unwrap();
        assert_eq!(format, Format::PES);
    }

    #[test]
    fn test_detect_from_extension_unknown() {
        let path = Path::new("design.xyz");
        let format = FormatDetector::detect_from_extension(path).unwrap();
        assert_eq!(format, Format::Unknown);
    }

    #[test]
    fn test_detect_from_extension_no_extension() {
        let path = Path::new("design");
        let result = FormatDetector::detect_from_extension(path);
        assert!(result.is_err());
    }

    #[test]
    fn test_detect_too_small_file() {
        let data = b"123"; // Less than 4 bytes
        let mut reader = Cursor::new(data);
        let result = FormatDetector::detect_from_content(&mut reader);
        assert!(result.is_err());
    }

    #[test]
    fn test_detect_unknown_content() {
        let data = vec![0xFFu8; 512]; // Random bytes
        let mut reader = Cursor::new(data);
        let format = FormatDetector::detect_from_content(&mut reader).unwrap();
        assert_eq!(format, Format::Unknown);
    }

    #[test]
    fn test_position_restored_after_detection() {
        let mut data = b"#PES0001".to_vec();
        data.extend_from_slice(&[0u8; 504]);
        let mut reader = Cursor::new(data);
        reader.seek(SeekFrom::Start(10)).unwrap();
        let initial_pos = reader.position();

        FormatDetector::detect_from_content(&mut reader).unwrap();

        assert_eq!(
            reader.position(),
            initial_pos,
            "Reader position should be restored"
        );
    }
}
