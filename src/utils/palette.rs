//! Thread palette management utilities
//!
//! Provides tools for loading, saving, and managing thread color palettes from various
//! sources including files (EDR, COL), built-in machine palettes, and custom collections.

use crate::core::pattern::EmbPattern;
use crate::core::thread::EmbThread;
use crate::formats::io::{readers, writers};
use crate::utils::error::{Error, Result};
use std::collections::HashMap;
use std::fs::File;
use std::io::{BufRead, BufReader, Read, Write};
use std::path::Path;

/// Supported palette file formats
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PaletteFormat {
    /// EDR format (Embird color list, binary RGB)
    Edr,
    /// COL format (text-based color list with catalog numbers)
    Col,
    /// INF format (thread information)
    Inf,
    /// RGB format (simple RGB text format)
    Rgb,
}

impl PaletteFormat {
    /// Detect format from file extension
    pub fn from_extension(ext: &str) -> Option<Self> {
        match ext.to_lowercase().as_str() {
            "edr" => Some(PaletteFormat::Edr),
            "col" => Some(PaletteFormat::Col),
            "inf" => Some(PaletteFormat::Inf),
            "rgb" => Some(PaletteFormat::Rgb),
            _ => None,
        }
    }

    /// Get file extension for format
    pub fn extension(&self) -> &'static str {
        match self {
            PaletteFormat::Edr => "edr",
            PaletteFormat::Col => "col",
            PaletteFormat::Inf => "inf",
            PaletteFormat::Rgb => "rgb",
        }
    }
}

/// Thread palette manager for loading, saving, and accessing color libraries
#[derive(Debug, Clone)]
pub struct ThreadPalette {
    /// Palette name
    pub name: String,
    /// Thread colors in palette
    pub threads: Vec<EmbThread>,
}

impl ThreadPalette {
    /// Create a new empty palette
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            threads: Vec::new(),
        }
    }

    /// Create palette from thread list
    pub fn from_threads(name: impl Into<String>, threads: Vec<EmbThread>) -> Self {
        Self {
            name: name.into(),
            threads,
        }
    }

    /// Load palette from file (auto-detects format from extension)
    pub fn load_file<P: AsRef<Path>>(path: P) -> Result<Self> {
        let path = path.as_ref();
        let ext = path
            .extension()
            .and_then(|e| e.to_str())
            .ok_or_else(|| Error::Parse("Palette file has no extension".to_string()))?;

        let format = PaletteFormat::from_extension(ext)
            .ok_or_else(|| Error::UnsupportedFormat(format!("Unknown palette format: .{}", ext)))?;

        let name = path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("Unknown")
            .to_string();

        let mut file = File::open(path)?;
        Self::load(&mut file, format, name)
    }

    /// Load palette from reader with specified format
    pub fn load(reader: &mut impl Read, format: PaletteFormat, name: String) -> Result<Self> {
        let mut pattern = EmbPattern::new();

        match format {
            PaletteFormat::Edr => readers::edr::read(reader, &mut pattern)?,
            PaletteFormat::Col => readers::col::read(reader, &mut pattern)?,
            PaletteFormat::Inf => readers::inf::read(reader, &mut pattern)?,
            PaletteFormat::Rgb => Self::read_rgb(reader, &mut pattern)?,
        }

        Ok(Self {
            name,
            threads: pattern.threads().to_vec(),
        })
    }

    /// Save palette to file (auto-detects format from extension)
    pub fn save_file<P: AsRef<Path>>(&self, path: P) -> Result<()> {
        let path = path.as_ref();
        let ext = path
            .extension()
            .and_then(|e| e.to_str())
            .ok_or_else(|| Error::Parse("Palette file has no extension".to_string()))?;

        let format = PaletteFormat::from_extension(ext)
            .ok_or_else(|| Error::UnsupportedFormat(format!("Unknown palette format: .{}", ext)))?;

        let mut file = File::create(path)?;
        self.save(&mut file, format)
    }

    /// Save palette to writer with specified format
    pub fn save(&self, writer: &mut impl Write, format: PaletteFormat) -> Result<()> {
        // Create temporary pattern to reuse existing format writers
        let mut pattern = EmbPattern::new();
        for thread in &self.threads {
            pattern.add_thread(thread.clone());
        }

        match format {
            PaletteFormat::Edr => writers::edr::write(&pattern, writer)?,
            PaletteFormat::Col => writers::col::write(&pattern, writer)?,
            PaletteFormat::Inf => {
                // INF requires Seek, so we need to buffer the output
                let mut buffer = Vec::new();
                let mut cursor = std::io::Cursor::new(&mut buffer);
                writers::inf::write(&pattern, &mut cursor)?;
                writer.write_all(&buffer)?;
            },
            PaletteFormat::Rgb => Self::write_rgb(&pattern, writer)?,
        }

        Ok(())
    }

    /// Read RGB format (simple text format: R G B per line)
    fn read_rgb(reader: &mut impl Read, pattern: &mut EmbPattern) -> Result<()> {
        let buf_reader = BufReader::new(reader);

        for (line_num, line) in buf_reader.lines().enumerate() {
            let line = line?;
            let line = line.trim();

            // Skip empty lines and comments
            if line.is_empty() || line.starts_with('#') {
                continue;
            }

            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() < 3 {
                return Err(Error::Parse(format!(
                    "RGB: Invalid line {} - expected 3 values (R G B), got {}",
                    line_num + 1,
                    parts.len()
                )));
            }

            let r = parts[0].parse::<u8>().map_err(|e| {
                Error::Parse(format!(
                    "RGB: Invalid red value on line {}: {}",
                    line_num + 1,
                    e
                ))
            })?;

            let g = parts[1].parse::<u8>().map_err(|e| {
                Error::Parse(format!(
                    "RGB: Invalid green value on line {}: {}",
                    line_num + 1,
                    e
                ))
            })?;

            let b = parts[2].parse::<u8>().map_err(|e| {
                Error::Parse(format!(
                    "RGB: Invalid blue value on line {}: {}",
                    line_num + 1,
                    e
                ))
            })?;

            pattern.add_thread(EmbThread::from_rgb(r, g, b));
        }

        Ok(())
    }

    /// Write RGB format (simple text format: R G B per line)
    fn write_rgb(pattern: &EmbPattern, writer: &mut impl Write) -> Result<()> {
        for thread in pattern.threads() {
            writeln!(
                writer,
                "{} {} {}",
                thread.red(),
                thread.green(),
                thread.blue()
            )?;
        }
        Ok(())
    }

    /// Add thread to palette
    pub fn add_thread(&mut self, thread: EmbThread) {
        self.threads.push(thread);
    }

    /// Get thread count
    pub fn len(&self) -> usize {
        self.threads.len()
    }

    /// Check if palette is empty
    pub fn is_empty(&self) -> bool {
        self.threads.is_empty()
    }

    /// Find closest matching thread from palette using color distance
    pub fn find_closest(&self, color: u32) -> Option<&EmbThread> {
        if self.threads.is_empty() {
            return None;
        }

        let target = EmbThread::new(color);
        self.threads.iter().min_by(|a, b| {
            let dist_a = target.color_distance(a.color);
            let dist_b = target.color_distance(b.color);
            dist_a
                .partial_cmp(&dist_b)
                .unwrap_or(std::cmp::Ordering::Equal)
        })
    }

    /// Find closest matching thread and return its index
    pub fn find_closest_index(&self, color: u32) -> Option<usize> {
        if self.threads.is_empty() {
            return None;
        }

        let target = EmbThread::new(color);
        self.threads
            .iter()
            .enumerate()
            .min_by(|(_, a), (_, b)| {
                let dist_a = target.color_distance(a.color);
                let dist_b = target.color_distance(b.color);
                dist_a
                    .partial_cmp(&dist_b)
                    .unwrap_or(std::cmp::Ordering::Equal)
            })
            .map(|(idx, _)| idx)
    }

    /// Convert pattern colors to closest matches from this palette
    pub fn quantize_pattern(&self, pattern: &mut EmbPattern) -> Result<()> {
        if self.threads.is_empty() {
            return Err(Error::InvalidPattern(
                "Cannot quantize with empty palette".to_string(),
            ));
        }

        let original_threads = pattern.threads().to_vec();
        let mut new_threads = Vec::new();
        let mut color_map: HashMap<u32, usize> = HashMap::new();

        // Map original colors to palette colors
        for original_thread in &original_threads {
            let color = original_thread.color;

            if let std::collections::hash_map::Entry::Vacant(e) = color_map.entry(color) {
                let closest_idx = self.find_closest_index(color).ok_or_else(|| {
                    Error::InvalidPattern("Failed to find closest color".to_string())
                })?;

                // Check if this palette color already exists in new threads
                let palette_color = self.threads[closest_idx].color;
                let new_idx = if let Some(pos) = new_threads
                    .iter()
                    .position(|t: &EmbThread| t.color == palette_color)
                {
                    pos
                } else {
                    new_threads.push(self.threads[closest_idx].clone());
                    new_threads.len() - 1
                };

                e.insert(new_idx);
            }
        }

        // Create a new pattern with quantized threads
        let stitches = pattern.stitches().to_vec();
        let metadata_pairs: Vec<_> = pattern
            .metadata()
            .map(|(k, v)| (k.clone(), v.clone()))
            .collect();

        // Build new pattern
        let mut new_pattern = EmbPattern::from_stitches(stitches, new_threads);
        for (key, value) in metadata_pairs {
            new_pattern.set_metadata(key, value);
        }

        // Replace the pattern contents
        *pattern = new_pattern;

        Ok(())
    }
}

/// Built-in palette library manager
pub struct PaletteLibrary;

impl PaletteLibrary {
    /// Get Brother PEC palette (64 colors)
    pub fn brother_pec() -> ThreadPalette {
        ThreadPalette::from_threads(
            "Brother PEC",
            crate::palettes::thread_pec::PEC_THREADS.clone(),
        )
    }

    /// Get Husqvarna Viking HUS palette (29 colors)
    pub fn husqvarna_hus() -> ThreadPalette {
        ThreadPalette::from_threads(
            "Husqvarna HUS",
            crate::palettes::thread_hus::get_thread_set(),
        )
    }

    /// Get Husqvarna Viking SHV palette (43 colors)
    pub fn husqvarna_shv() -> ThreadPalette {
        ThreadPalette::from_threads(
            "Husqvarna SHV",
            crate::palettes::thread_shv::get_thread_set(),
        )
    }

    /// Get Janome JEF palette (78 colors, skipping None entries)
    pub fn janome_jef() -> ThreadPalette {
        ThreadPalette::from_threads(
            "Janome JEF",
            crate::palettes::thread_jef::JEF_THREADS
                .iter()
                .filter_map(|t| t.clone())
                .collect(),
        )
    }

    /// Get Janome SEW palette (79 colors)
    pub fn janome_sew() -> ThreadPalette {
        ThreadPalette::from_threads("Janome SEW", crate::palettes::thread_sew::get_thread_set())
    }

    /// Get all available built-in palettes
    pub fn all_palettes() -> Vec<ThreadPalette> {
        vec![
            Self::brother_pec(),
            Self::husqvarna_hus(),
            Self::husqvarna_shv(),
            Self::janome_jef(),
            Self::janome_sew(),
        ]
    }

    /// Get palette by name (case-insensitive)
    pub fn get_by_name(name: &str) -> Option<ThreadPalette> {
        let name_lower = name.to_lowercase();
        if name_lower.contains("pec") || name_lower.contains("brother") {
            Some(Self::brother_pec())
        } else if name_lower.contains("hus") && !name_lower.contains("shv") {
            Some(Self::husqvarna_hus())
        } else if name_lower.contains("shv") {
            Some(Self::husqvarna_shv())
        } else if name_lower.contains("jef") {
            Some(Self::janome_jef())
        } else if name_lower.contains("sew") {
            Some(Self::janome_sew())
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    #[test]
    fn test_palette_format_detection() {
        assert_eq!(
            PaletteFormat::from_extension("edr"),
            Some(PaletteFormat::Edr)
        );
        assert_eq!(
            PaletteFormat::from_extension("EDR"),
            Some(PaletteFormat::Edr)
        );
        assert_eq!(
            PaletteFormat::from_extension("col"),
            Some(PaletteFormat::Col)
        );
        assert_eq!(
            PaletteFormat::from_extension("rgb"),
            Some(PaletteFormat::Rgb)
        );
        assert_eq!(
            PaletteFormat::from_extension("inf"),
            Some(PaletteFormat::Inf)
        );
        assert_eq!(PaletteFormat::from_extension("xyz"), None);
    }

    #[test]
    fn test_palette_creation() {
        let palette = ThreadPalette::new("Test Palette");
        assert_eq!(palette.name, "Test Palette");
        assert_eq!(palette.len(), 0);
        assert!(palette.is_empty());
    }

    #[test]
    fn test_palette_from_threads() {
        let threads = vec![
            EmbThread::from_rgb(255, 0, 0),
            EmbThread::from_rgb(0, 255, 0),
            EmbThread::from_rgb(0, 0, 255),
        ];

        let palette = ThreadPalette::from_threads("RGB Palette", threads);
        assert_eq!(palette.name, "RGB Palette");
        assert_eq!(palette.len(), 3);
    }

    #[test]
    fn test_palette_add_thread() {
        let mut palette = ThreadPalette::new("Test");
        palette.add_thread(EmbThread::from_rgb(255, 0, 0));
        palette.add_thread(EmbThread::from_rgb(0, 255, 0));

        assert_eq!(palette.len(), 2);
        assert!(!palette.is_empty());
    }

    #[test]
    fn test_palette_load_save_rgb() {
        let rgb_data = "255 0 0\n0 255 0\n0 0 255\n# Comment line\n\n128 128 128\n";
        let mut cursor = Cursor::new(rgb_data.as_bytes());

        let palette =
            ThreadPalette::load(&mut cursor, PaletteFormat::Rgb, "Test".to_string()).unwrap();

        assert_eq!(palette.len(), 4);
        assert_eq!(palette.threads[0].red(), 255);
        assert_eq!(palette.threads[1].green(), 255);
        assert_eq!(palette.threads[2].blue(), 255);
        assert_eq!(palette.threads[3].red(), 128);

        // Test saving
        let mut output = Vec::new();
        palette.save(&mut output, PaletteFormat::Rgb).unwrap();

        let output_str = String::from_utf8(output).unwrap();
        assert!(output_str.contains("255 0 0"));
        assert!(output_str.contains("0 255 0"));
        assert!(output_str.contains("128 128 128"));
    }

    #[test]
    fn test_palette_load_save_edr() {
        // EDR format: 4 bytes per thread [R, G, B, 0x00]
        let edr_data = vec![
            255, 0, 0, 0, // Red
            0, 255, 0, 0, // Green
            0, 0, 255, 0, // Blue
        ];

        let mut cursor = Cursor::new(&edr_data);
        let palette =
            ThreadPalette::load(&mut cursor, PaletteFormat::Edr, "Test".to_string()).unwrap();

        assert_eq!(palette.len(), 3);
        assert_eq!(palette.threads[0].red(), 255);
        assert_eq!(palette.threads[1].green(), 255);
        assert_eq!(palette.threads[2].blue(), 255);

        // Test saving
        let mut output = Vec::new();
        palette.save(&mut output, PaletteFormat::Edr).unwrap();

        assert_eq!(output, edr_data);
    }

    #[test]
    fn test_palette_find_closest() {
        let palette = ThreadPalette::from_threads(
            "Test",
            vec![
                EmbThread::from_rgb(255, 0, 0), // Red
                EmbThread::from_rgb(0, 255, 0), // Green
                EmbThread::from_rgb(0, 0, 255), // Blue
            ],
        );

        // Find closest to orange (closer to red)
        let orange = 0xFF8000; // RGB(255, 128, 0)
        let closest = palette.find_closest(orange).unwrap();
        assert_eq!(closest.red(), 255);
        assert_eq!(closest.green(), 0);
        assert_eq!(closest.blue(), 0);

        // Find closest to cyan (closer to blue)
        let cyan = 0x00FFFF;
        let closest_idx = palette.find_closest_index(cyan).unwrap();
        assert!(closest_idx == 1 || closest_idx == 2); // Could be green or blue
    }

    #[test]
    fn test_palette_find_closest_empty() {
        let palette = ThreadPalette::new("Empty");
        assert!(palette.find_closest(0xFF0000).is_none());
        assert!(palette.find_closest_index(0xFF0000).is_none());
    }

    #[test]
    fn test_palette_quantize_pattern() {
        let palette = ThreadPalette::from_threads(
            "Simple",
            vec![
                EmbThread::from_rgb(255, 0, 0), // Red
                EmbThread::from_rgb(0, 0, 255), // Blue
            ],
        );

        let mut pattern = EmbPattern::new();
        pattern.add_thread(EmbThread::from_rgb(250, 10, 10)); // Close to red
        pattern.add_thread(EmbThread::from_rgb(10, 10, 250)); // Close to blue
        pattern.add_thread(EmbThread::from_rgb(200, 50, 50)); // Also close to red

        palette.quantize_pattern(&mut pattern).unwrap();

        // Should have 2 threads now (red and blue)
        assert_eq!(pattern.threads().len(), 2);

        // Check colors are from palette
        assert!(pattern
            .threads()
            .iter()
            .any(|t| t.red() == 255 && t.green() == 0 && t.blue() == 0));
        assert!(pattern
            .threads()
            .iter()
            .any(|t| t.red() == 0 && t.green() == 0 && t.blue() == 255));
    }

    #[test]
    fn test_palette_library_brother_pec() {
        let palette = PaletteLibrary::brother_pec();
        assert_eq!(palette.name, "Brother PEC");
        assert_eq!(palette.len(), 64);
    }

    #[test]
    fn test_palette_library_husqvarna_hus() {
        let palette = PaletteLibrary::husqvarna_hus();
        assert_eq!(palette.name, "Husqvarna HUS");
        assert_eq!(palette.len(), 29);
    }

    #[test]
    fn test_palette_library_janome_jef() {
        let palette = PaletteLibrary::janome_jef();
        assert_eq!(palette.name, "Janome JEF");
        assert_eq!(palette.len(), 78); // 78 actual colors (79 entries with one None)
    }

    #[test]
    fn test_palette_library_get_by_name() {
        assert!(PaletteLibrary::get_by_name("brother").is_some());
        assert!(PaletteLibrary::get_by_name("PEC").is_some());
        assert!(PaletteLibrary::get_by_name("hus").is_some());
        assert!(PaletteLibrary::get_by_name("shv").is_some());
        assert!(PaletteLibrary::get_by_name("jef").is_some());
        assert!(PaletteLibrary::get_by_name("sew").is_some());
        assert!(PaletteLibrary::get_by_name("invalid").is_none());
    }

    #[test]
    fn test_palette_library_all_palettes() {
        let palettes = PaletteLibrary::all_palettes();
        assert_eq!(palettes.len(), 5);

        let names: Vec<_> = palettes.iter().map(|p| p.name.as_str()).collect();
        assert!(names.contains(&"Brother PEC"));
        assert!(names.contains(&"Husqvarna HUS"));
        assert!(names.contains(&"Husqvarna SHV"));
        assert!(names.contains(&"Janome JEF"));
        assert!(names.contains(&"Janome SEW"));
    }

    #[test]
    fn test_rgb_format_with_comments() {
        let rgb_data = "# Header comment\n255 0 0\n\n# Mid comment\n0 255 0\n";
        let mut cursor = Cursor::new(rgb_data.as_bytes());

        let palette =
            ThreadPalette::load(&mut cursor, PaletteFormat::Rgb, "Test".to_string()).unwrap();

        assert_eq!(palette.len(), 2);
        assert_eq!(palette.threads[0].red(), 255);
        assert_eq!(palette.threads[1].green(), 255);
    }
}
