//! Format Registry System
//!
//! Provides a registry for dynamic format discovery and handler lookup.
//! Simplifies format detection, reader/writer selection, and extensible format handling.

use crate::core::pattern::EmbPattern;
use crate::utils::error::{Error, Result};
use std::io::{Read, Seek, Write};
use std::path::Path;

/// Information about a supported format
#[derive(Debug, Clone)]
pub struct FormatInfo {
    /// Format name (e.g., "DST", "PES")
    pub name: &'static str,
    /// Supported file extensions
    pub extensions: &'static [&'static str],
    /// Whether this format supports reading
    pub can_read: bool,
    /// Whether this format supports writing
    pub can_write: bool,
    /// Human-readable description
    pub description: &'static str,
}

/// Registry for managing format information
pub struct FormatRegistry {
    formats: Vec<FormatInfo>,
}

impl FormatRegistry {
    /// Create a new registry with all built-in formats
    pub fn new() -> Self {
        Self {
            formats: vec![
                FormatInfo {
                    name: "DST",
                    extensions: &["dst"],
                    can_read: true,
                    can_write: true,
                    description: "Tajima DST format",
                },
                FormatInfo {
                    name: "PES",
                    extensions: &["pes"],
                    can_read: true,
                    can_write: true,
                    description: "Brother PES format",
                },
                FormatInfo {
                    name: "JEF",
                    extensions: &["jef"],
                    can_read: true,
                    can_write: true,
                    description: "Janome JEF format",
                },
                FormatInfo {
                    name: "EXP",
                    extensions: &["exp"],
                    can_read: true,
                    can_write: true,
                    description: "Melco EXP format",
                },
                FormatInfo {
                    name: "VP3",
                    extensions: &["vp3"],
                    can_read: true,
                    can_write: true,
                    description: "Pfaff VP3 format",
                },
                FormatInfo {
                    name: "PEC",
                    extensions: &["pec"],
                    can_read: true,
                    can_write: true,
                    description: "Brother PEC format",
                },
                FormatInfo {
                    name: "XXX",
                    extensions: &["xxx"],
                    can_read: true,
                    can_write: true,
                    description: "Singer XXX format",
                },
                FormatInfo {
                    name: "U01",
                    extensions: &["u01"],
                    can_read: true,
                    can_write: true,
                    description: "Barudan U01 format",
                },
                FormatInfo {
                    name: "TBF",
                    extensions: &["tbf"],
                    can_read: true,
                    can_write: true,
                    description: "Tajima TBF format",
                },
                FormatInfo {
                    name: "COL",
                    extensions: &["col"],
                    can_read: true,
                    can_write: true,
                    description: "Thread color list",
                },
                FormatInfo {
                    name: "EDR",
                    extensions: &["edr"],
                    can_read: true,
                    can_write: true,
                    description: "Embird color format",
                },
                FormatInfo {
                    name: "INF",
                    extensions: &["inf"],
                    can_read: true,
                    can_write: true,
                    description: "Thread information format",
                },
                FormatInfo {
                    name: "JSON",
                    extensions: &["json"],
                    can_read: true,
                    can_write: true,
                    description: "JSON embroidery data",
                },
                FormatInfo {
                    name: "CSV",
                    extensions: &["csv"],
                    can_read: true,
                    can_write: true,
                    description: "CSV embroidery data",
                },
                FormatInfo {
                    name: "GCODE",
                    extensions: &["gcode", "nc"],
                    can_read: true,
                    can_write: true,
                    description: "G-code embroidery format",
                },
                FormatInfo {
                    name: "SVG",
                    extensions: &["svg"],
                    can_read: false,
                    can_write: true,
                    description: "SVG vector graphics (write-only)",
                },
                FormatInfo {
                    name: "TXT",
                    extensions: &["txt"],
                    can_read: false,
                    can_write: true,
                    description: "Human-readable text (write-only)",
                },
            ],
        }
    }

    /// Get format info by name
    pub fn get_format(&self, name: &str) -> Option<&FormatInfo> {
        let name_lower = name.to_lowercase();
        self.formats
            .iter()
            .find(|f| f.name.to_lowercase() == name_lower)
    }

    /// Get format info by file extension
    pub fn get_format_by_extension(&self, extension: &str) -> Option<&FormatInfo> {
        let ext_lower = extension.trim_start_matches('.').to_lowercase();
        self.formats
            .iter()
            .find(|f| f.extensions.iter().any(|e| e.to_lowercase() == ext_lower))
    }

    /// Get format info from file path
    pub fn get_format_from_path<P: AsRef<Path>>(&self, path: P) -> Option<&FormatInfo> {
        path.as_ref()
            .extension()
            .and_then(|ext| ext.to_str())
            .and_then(|ext| self.get_format_by_extension(ext))
    }

    /// Get all registered formats
    pub fn all_formats(&self) -> &[FormatInfo] {
        &self.formats
    }

    /// Get all formats that support reading
    pub fn readable_formats(&self) -> Vec<&FormatInfo> {
        self.formats.iter().filter(|f| f.can_read).collect()
    }

    /// Get all formats that support writing
    pub fn writable_formats(&self) -> Vec<&FormatInfo> {
        self.formats.iter().filter(|f| f.can_write).collect()
    }

    /// Read a pattern from a file using the appropriate format
    pub fn read_pattern<R: Read + Seek>(&self, file: &mut R, format: &str) -> Result<EmbPattern> {
        let format_lower = format.to_lowercase();
        match format_lower.as_str() {
            "dst" => crate::formats::io::readers::dst::read(file, None),
            "pes" => {
                let mut pattern = EmbPattern::new();
                crate::formats::io::readers::pes::read(file, &mut pattern)?;
                Ok(pattern)
            }
            "jef" => crate::formats::io::readers::jef::read(file, None),
            "exp" => crate::formats::io::readers::exp::read(file),
            "vp3" => {
                let mut pattern = EmbPattern::new();
                crate::formats::io::readers::vp3::read(file, &mut pattern)?;
                Ok(pattern)
            }
            "pec" => crate::formats::io::readers::pec::read(file),
            "xxx" => {
                let mut pattern = EmbPattern::new();
                crate::formats::io::readers::xxx::read(file, &mut pattern)?;
                Ok(pattern)
            }
            "u01" => {
                let mut pattern = EmbPattern::new();
                crate::formats::io::readers::u01::read(file, &mut pattern)?;
                Ok(pattern)
            }
            "tbf" => {
                let mut pattern = EmbPattern::new();
                crate::formats::io::readers::tbf::read(file, &mut pattern)?;
                Ok(pattern)
            }
            "col" => {
                let mut pattern = EmbPattern::new();
                crate::formats::io::readers::col::read(file, &mut pattern)?;
                Ok(pattern)
            }
            "edr" => {
                let mut pattern = EmbPattern::new();
                crate::formats::io::readers::edr::read(file, &mut pattern)?;
                Ok(pattern)
            }
            "inf" => {
                let mut pattern = EmbPattern::new();
                crate::formats::io::readers::inf::read(file, &mut pattern)?;
                Ok(pattern)
            }
            "json" => crate::formats::io::readers::json::read(file),
            "csv" => {
                let mut pattern = EmbPattern::new();
                crate::formats::io::readers::csv::read(file, &mut pattern)?;
                Ok(pattern)
            }
            "gcode" => {
                let mut pattern = EmbPattern::new();
                crate::formats::io::readers::gcode::read(file, &mut pattern)?;
                Ok(pattern)
            }
            _ => Err(Error::UnsupportedFormat(format!(
                "Unsupported format: {}",
                format
            ))),
        }
    }

    /// Write a pattern to a file using the appropriate format
    pub fn write_pattern<W: Write + Seek>(
        &self,
        pattern: &EmbPattern,
        file: &mut W,
        format: &str,
    ) -> Result<()> {
        let format_lower = format.to_lowercase();
        match format_lower.as_str() {
            "dst" => crate::formats::io::writers::dst::write(file, pattern, true, 512),
            "pes" => crate::formats::io::writers::pes::write_pes(
                pattern,
                file,
                crate::formats::io::writers::pes::PesVersion::V1,
                false,
            ),
            "jef" => {
                crate::formats::io::writers::jef::write(file, pattern, true, 100, "2025-01-01")
            }
            "exp" => crate::formats::io::writers::exp::write(file, pattern),
            "vp3" => crate::formats::io::writers::vp3::write(file, pattern),
            "pec" => crate::formats::io::writers::pec::write(file, pattern),
            "xxx" => crate::formats::io::writers::xxx::write(pattern, file),
            "u01" => crate::formats::io::writers::u01::write(pattern, file),
            "tbf" => crate::formats::io::writers::tbf::write(pattern, file),
            "col" => crate::formats::io::writers::col::write(pattern, file),
            "edr" => crate::formats::io::writers::edr::write(pattern, file),
            "inf" => crate::formats::io::writers::inf::write(pattern, file),
            "json" => crate::formats::io::writers::json::write(file, pattern),
            "csv" => crate::formats::io::writers::csv::write(
                file,
                pattern,
                crate::formats::io::writers::csv::CsvVersion::Default,
            ),
            "gcode" => crate::formats::io::writers::gcode::write(pattern, file),
            "svg" => {
                // SVG doesn't require Seek
                let mut buf = Vec::new();
                crate::formats::io::writers::svg::write(pattern, &mut buf)?;
                file.write_all(&buf)?;
                Ok(())
            }
            "txt" => {
                // TXT doesn't require Seek either
                let mut buf = Vec::new();
                crate::formats::io::writers::txt::write(pattern, &mut buf)?;
                file.write_all(&buf)?;
                Ok(())
            }
            _ => Err(Error::UnsupportedFormat(format!(
                "Unsupported format: {}",
                format
            ))),
        }
    }
}

impl Default for FormatRegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_registry_creation() {
        let registry = FormatRegistry::new();
        assert!(!registry.all_formats().is_empty());
    }

    #[test]
    fn test_get_format_by_name() {
        let registry = FormatRegistry::new();

        let dst = registry.get_format("DST");
        assert!(dst.is_some());
        assert_eq!(dst.unwrap().name, "DST");

        // Case insensitive
        let pes = registry.get_format("pes");
        assert!(pes.is_some());
        assert_eq!(pes.unwrap().name, "PES");

        // Non-existent
        let none = registry.get_format("NONEXISTENT");
        assert!(none.is_none());
    }

    #[test]
    fn test_get_format_by_extension() {
        let registry = FormatRegistry::new();

        let dst = registry.get_format_by_extension("dst");
        assert!(dst.is_some());
        assert_eq!(dst.unwrap().name, "DST");

        // With dot prefix
        let pes = registry.get_format_by_extension(".pes");
        assert!(pes.is_some());

        // Case insensitive
        let jef = registry.get_format_by_extension("JEF");
        assert!(jef.is_some());

        // Non-existent
        let none = registry.get_format_by_extension("xyz");
        assert!(none.is_none());
    }

    #[test]
    fn test_get_format_from_path() {
        let registry = FormatRegistry::new();

        let dst = registry.get_format_from_path("pattern.dst");
        assert!(dst.is_some());
        assert_eq!(dst.unwrap().name, "DST");

        let pes = registry.get_format_from_path("/path/to/file.pes");
        assert!(pes.is_some());

        let none = registry.get_format_from_path("file.xyz");
        assert!(none.is_none());
    }

    #[test]
    fn test_readable_and_writable_formats() {
        let registry = FormatRegistry::new();

        let readable = registry.readable_formats();
        assert!(!readable.is_empty());

        let writable = registry.writable_formats();
        assert!(!writable.is_empty());

        // SVG and TXT should be write-only
        let svg = registry.get_format("SVG").unwrap();
        assert!(!svg.can_read);
        assert!(svg.can_write);
    }

    #[test]
    fn test_format_count() {
        let registry = FormatRegistry::new();
        // Should have all 17 formats (15 bidirectional + 2 write-only)
        assert_eq!(registry.all_formats().len(), 17);
    }
}
