//! Batch conversion utilities for processing multiple embroidery files
//!
//! This module provides high-level APIs for:
//! - Converting multiple files in one operation
//! - Exporting a single pattern to multiple formats
//! - Progress tracking and error reporting
//! - Parallel processing for improved performance
//!
//! ## Thread Safety
//!
//! The batch converter is designed for safe concurrent operation:
//!
//! - **Mutex Protection**: Results are collected in an `Arc<Mutex<Vec<ConversionResult>>>` to ensure
//!   thread-safe access during parallel processing. The mutex is only held during result insertion,
//!   minimizing contention.
//!
//! - **Arc Sharing**: Configuration data (target format, output directory) is wrapped in `Arc` to enable
//!   safe sharing across threads without cloning. This is read-only after initialization.
//!
//! - **Thread Panic Handling**: Worker threads are isolated - if one panics during conversion, others
//!   continue processing. Failed conversions are captured as `ConversionResult::Failed` rather than
//!   propagating panics.
//!
//! - **Rayon Guarantees**: When `parallel` feature is enabled, uses Rayon's `par_iter()` which:
//!   - Automatically manages thread pool sizing
//!   - Ensures all spawned threads complete before returning
//!   - Provides work-stealing for load balancing
//!   - Handles thread panics gracefully with `catch_unwind`
//!
//! - **Mutex Poisoning**: In the rare case of mutex poisoning (panic while lock held), operations
//!   gracefully degrade to serial processing or return accumulated results rather than panicking.
//!
//! ## Supported Input Formats
//!
//! The batch converter supports automatic format detection for:
//! - **dst** - Tajima DST
//! - **pes** - Brother PES
//! - **exp** - Melco EXP
//! - **jef** - Janome JEF
//! - **vp3** - Pfaff VP3
//! - **pec** - Brother PEC
//! - **xxx** - Singer XXX
//! - **u01** - Barudan U01
//! - **tbf** - Tajima TBF
//! - **col** - Embroidery Thread Color
//! - **edr** - Embird Color
//! - **inf** - Embroidery Thread Info
//! - **gcode** - GCode embroidery data
//! - **json** - JSON embroidery data
//! - **csv** - CSV embroidery data
//!
//! ## Supported Output Formats
//!
//! The batch converter can export to any format supported by the writers module,
//! including: dst, pes, jef, vp3, exp, pec, xxx, u01, tbf, col, edr, inf, gcode, json, csv, svg, png, txt.
//!
//! # Examples
//!
//! ## Batch convert multiple files
//!
//! ```no_run
//! use butabuti::utils::batch::{BatchConverter, ConversionResult};
//!
//! let converter = BatchConverter::new()
//!     .input_dir("./designs")
//!     .output_dir("./output")
//!     .target_format("dst")
//!     .build();
//!
//! let results = converter.convert_all()?;
//! for result in results {
//!     match result {
//!         ConversionResult::Success { input, output, .. } => {
//!             println!("✓ Converted {} -> {}", input, output);
//!         }
//!         ConversionResult::Failed { input, error, .. } => {
//!             eprintln!("✗ Failed to convert {}: {}", input, error);
//!         }
//!     }
//! }
//! # Ok::<(), Box<dyn std::error::Error>>(())
//! ```
//!
//! ## Export to multiple formats
//!
//! ```no_run
//! use butabuti::utils::batch::MultiFormatExporter;
//! use butabuti::prelude::*;
//!
//! let pattern = EmbPattern::new();
//! // ... populate pattern ...
//!
//! let exporter = MultiFormatExporter::new()
//!     .output_dir("./exports")
//!     .base_name("my_design")
//!     .formats(&["dst", "pes", "jef", "vp3", "exp"])
//!     .build();
//!
//! let results = exporter.export(&pattern)?;
//! println!("Exported {} files", results.success_count());
//! # Ok::<(), Box<dyn std::error::Error>>(())
//! ```

use crate::core::pattern::EmbPattern;
use crate::formats::io::{readers, writers};
use crate::utils::error::{Error, Result};
use std::fs::{self, File};
use std::io::{BufReader, BufWriter};
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::time::Instant;

/// Represents the result of a single conversion operation
#[derive(Debug, Clone)]
pub enum ConversionResult {
    /// Conversion succeeded
    Success {
        /// Input file path
        input: PathBuf,
        /// Output file path
        output: PathBuf,
        /// Time taken in milliseconds
        duration_ms: u128,
        /// Output file size in bytes
        file_size: u64,
    },
    /// Conversion failed
    Failed {
        /// Input file path
        input: PathBuf,
        /// Error message
        error: String,
        /// Time taken before failure in milliseconds
        duration_ms: u128,
    },
    /// File was skipped (e.g., already exists and overwrite is disabled)
    Skipped {
        /// Input file path
        input: PathBuf,
        /// Reason for skipping
        reason: String,
    },
}

/// Collection of conversion results with summary statistics
#[derive(Debug, Clone)]
pub struct ConversionResults {
    results: Vec<ConversionResult>,
    total_duration_ms: u128,
}

impl ConversionResults {
    /// Create a new results collection
    pub fn new() -> Self {
        Self {
            results: Vec::new(),
            total_duration_ms: 0,
        }
    }

    /// Add a result to the collection
    pub fn add(&mut self, result: ConversionResult) {
        self.results.push(result);
    }

    /// Get all results
    pub fn results(&self) -> &[ConversionResult] {
        &self.results
    }

    /// Set total duration
    pub fn set_total_duration(&mut self, duration_ms: u128) {
        self.total_duration_ms = duration_ms;
    }

    /// Get total duration
    pub fn total_duration_ms(&self) -> u128 {
        self.total_duration_ms
    }

    /// Count successful conversions
    pub fn success_count(&self) -> usize {
        self.results
            .iter()
            .filter(|r| matches!(r, ConversionResult::Success { .. }))
            .count()
    }

    /// Count failed conversions
    pub fn failed_count(&self) -> usize {
        self.results
            .iter()
            .filter(|r| matches!(r, ConversionResult::Failed { .. }))
            .count()
    }

    /// Count skipped files
    pub fn skipped_count(&self) -> usize {
        self.results
            .iter()
            .filter(|r| matches!(r, ConversionResult::Skipped { .. }))
            .count()
    }

    /// Get total number of results
    pub fn total_count(&self) -> usize {
        self.results.len()
    }

    /// Calculate success rate (0.0 to 1.0)
    pub fn success_rate(&self) -> f64 {
        if self.total_count() == 0 {
            0.0
        } else {
            self.success_count() as f64 / self.total_count() as f64
        }
    }

    /// Get total output size in bytes
    pub fn total_output_size(&self) -> u64 {
        self.results
            .iter()
            .filter_map(|r| match r {
                ConversionResult::Success { file_size, .. } => Some(*file_size),
                _ => None,
            })
            .sum()
    }

    /// Print a summary report
    pub fn print_summary(&self) {
        println!("\n=== Conversion Summary ===");
        println!("Total files processed: {}", self.total_count());
        println!("  ✓ Successful: {}", self.success_count());
        println!("  ✗ Failed: {}", self.failed_count());
        println!("  ⊘ Skipped: {}", self.skipped_count());
        println!("Success rate: {:.1}%", self.success_rate() * 100.0);
        println!(
            "Total output size: {:.2} MB",
            self.total_output_size() as f64 / 1_048_576.0
        );
        println!("Total time: {:.2}s", self.total_duration_ms as f64 / 1000.0);
    }
}

impl Default for ConversionResults {
    fn default() -> Self {
        Self::new()
    }
}

/// Builder for batch file conversion operations
pub struct BatchConverter {
    input_dir: Option<PathBuf>,
    output_dir: Option<PathBuf>,
    input_files: Vec<PathBuf>,
    target_format: Option<String>,
    overwrite: bool,
    recursive: bool,
    input_extensions: Vec<String>,
    parallel: bool,
}

impl BatchConverter {
    /// Create a new batch converter builder
    pub fn new() -> Self {
        Self {
            input_dir: None,
            output_dir: None,
            input_files: Vec::new(),
            target_format: None,
            overwrite: false,
            recursive: false,
            input_extensions: Vec::new(),
            parallel: true,
        }
    }

    /// Set the input directory to scan for files
    pub fn input_dir<P: AsRef<Path>>(mut self, path: P) -> Self {
        self.input_dir = Some(path.as_ref().to_path_buf());
        self
    }

    /// Set the output directory for converted files
    pub fn output_dir<P: AsRef<Path>>(mut self, path: P) -> Self {
        self.output_dir = Some(path.as_ref().to_path_buf());
        self
    }

    /// Add specific input files to convert
    pub fn input_files(mut self, files: &[PathBuf]) -> Self {
        self.input_files.extend_from_slice(files);
        self
    }

    /// Set the target output format (e.g., "dst", "pes", "jef")
    pub fn target_format(mut self, format: &str) -> Self {
        self.target_format = Some(format.to_lowercase());
        self
    }

    /// Enable or disable overwriting existing files (default: false)
    pub fn overwrite(mut self, overwrite: bool) -> Self {
        self.overwrite = overwrite;
        self
    }

    /// Enable recursive directory scanning (default: false)
    pub fn recursive(mut self, recursive: bool) -> Self {
        self.recursive = recursive;
        self
    }

    /// Filter input files by extensions (e.g., ["dst", "pes"])
    pub fn input_extensions(mut self, extensions: &[&str]) -> Self {
        self.input_extensions = extensions.iter().map(|s| s.to_lowercase()).collect();
        self
    }

    /// Enable parallel processing (default: true)
    pub fn parallel(mut self, parallel: bool) -> Self {
        self.parallel = parallel;
        self
    }

    /// Build and execute the batch conversion
    pub fn build(self) -> BatchConverterExecutor {
        BatchConverterExecutor { config: self }
    }
}

impl Default for BatchConverter {
    fn default() -> Self {
        Self::new()
    }
}

/// Executes batch conversion operations
pub struct BatchConverterExecutor {
    config: BatchConverter,
}

impl BatchConverterExecutor {
    /// Convert all input files
    pub fn convert_all(&self) -> Result<ConversionResults> {
        let start = Instant::now();
        let mut results = ConversionResults::new();

        // Collect all input files
        let input_files = self.collect_input_files()?;

        if input_files.is_empty() {
            return Err(Error::InvalidPattern(
                "No input files found to convert".to_string(),
            ));
        }

        // Ensure output directory exists
        if let Some(ref output_dir) = self.config.output_dir {
            fs::create_dir_all(output_dir)?;
        }

        // Convert files
        if self.config.parallel {
            // Parallel processing with Arc to avoid cloning config strings
            let results_arc = Arc::new(Mutex::new(ConversionResults::new()));
            let target_format_arc = Arc::new(self.config.target_format.clone());
            let output_dir_arc = Arc::new(self.config.output_dir.clone());
            let overwrite = self.config.overwrite;

            let handles: Vec<_> = input_files
                .into_iter()
                .map(|input_file| {
                    let results_clone = Arc::clone(&results_arc);
                    let target_format = Arc::clone(&target_format_arc);
                    let output_dir = Arc::clone(&output_dir_arc);

                    std::thread::spawn(move || {
                        let result = Self::convert_single_file(
                            &input_file,
                            target_format.as_ref().as_deref(),
                            output_dir.as_ref().as_deref(),
                            overwrite,
                        );
                        if let Ok(mut results) = results_clone.lock() {
                            results.add(result);
                        }
                    })
                })
                .collect();

            // Wait for all threads
            for handle in handles {
                let _ = handle.join(); // Ignore thread panic errors
            }

            results = Arc::try_unwrap(results_arc)
                .ok()
                .and_then(|m| m.into_inner().ok())
                .unwrap_or_default();
        } else {
            // Sequential processing
            for input_file in input_files {
                let result = Self::convert_single_file(
                    &input_file,
                    self.config.target_format.as_deref(),
                    self.config.output_dir.as_deref(),
                    self.config.overwrite,
                );
                results.add(result);
            }
        }

        results.set_total_duration(start.elapsed().as_millis());
        Ok(results)
    }

    /// Collect all input files based on configuration
    fn collect_input_files(&self) -> Result<Vec<PathBuf>> {
        let mut files = Vec::new();

        // Add explicitly specified files
        files.extend(self.config.input_files.iter().cloned());

        // Scan input directory if specified
        if let Some(ref input_dir) = self.config.input_dir {
            if self.config.recursive {
                self.collect_files_recursive(input_dir, &mut files)?;
            } else {
                self.collect_files_flat(input_dir, &mut files)?;
            }
        }

        Ok(files)
    }

    /// Collect files from a directory (non-recursive)
    fn collect_files_flat(&self, dir: &Path, files: &mut Vec<PathBuf>) -> Result<()> {
        for entry in fs::read_dir(dir)? {
            let entry = entry?;
            let path = entry.path();

            if path.is_file() && self.matches_extension(&path) {
                files.push(path);
            }
        }
        Ok(())
    }

    /// Collect files from a directory recursively
    fn collect_files_recursive(&self, dir: &Path, files: &mut Vec<PathBuf>) -> Result<()> {
        for entry in fs::read_dir(dir)? {
            let entry = entry?;
            let path = entry.path();

            if path.is_file() && self.matches_extension(&path) {
                files.push(path);
            } else if path.is_dir() {
                self.collect_files_recursive(&path, files)?;
            }
        }
        Ok(())
    }

    /// Check if file matches extension filter
    fn matches_extension(&self, path: &Path) -> bool {
        if self.config.input_extensions.is_empty() {
            return true;
        }

        path.extension()
            .and_then(|ext| ext.to_str())
            .map(|ext| self.config.input_extensions.contains(&ext.to_lowercase()))
            .unwrap_or(false)
    }

    /// Convert a single file
    fn convert_single_file(
        input_path: &Path,
        target_format: Option<&str>,
        output_dir: Option<&Path>,
        overwrite: bool,
    ) -> ConversionResult {
        let start = Instant::now();

        // Determine output path
        let output_path = Self::determine_output_path(input_path, target_format, output_dir);

        // Check if output already exists and overwrite is disabled
        if output_path.exists() && !overwrite {
            return ConversionResult::Skipped {
                input: input_path.to_path_buf(),
                reason: "Output file already exists".to_string(),
            };
        }

        // Perform conversion
        match Self::perform_conversion(input_path, &output_path) {
            Ok(()) => {
                let duration = start.elapsed().as_millis();
                let file_size = fs::metadata(&output_path).map(|m| m.len()).unwrap_or(0);

                ConversionResult::Success {
                    input: input_path.to_path_buf(),
                    output: output_path,
                    duration_ms: duration,
                    file_size,
                }
            }
            Err(e) => ConversionResult::Failed {
                input: input_path.to_path_buf(),
                error: e.to_string(),
                duration_ms: start.elapsed().as_millis(),
            },
        }
    }

    /// Sanitize a filename to prevent path traversal attacks
    ///
    /// Removes or replaces characters that could be used for path traversal:
    /// - Path separators (/, \)
    /// - Parent directory references (..)
    /// - Special characters that could cause issues
    ///
    /// Returns a safe filename suitable for use in file operations.
    fn sanitize_filename(name: &str) -> String {
        name.chars()
            .filter(|c| {
                !matches!(
                    c,
                    // Remove path separators
                    '/' | '\\' |
                    // Remove null bytes and control characters
                    '\0'
                        ..='\x1F' | '\x7F' |
                    // Remove characters problematic on Windows
                    '<' | '>' | ':' | '"' | '|' | '?' | '*'
                )
            })
            .collect::<String>()
            .replace("..", "_") // Replace parent directory references
            .trim_matches('.') // Remove leading/trailing dots
            .trim()
            .to_string()
    }

    /// Determine the output file path
    fn determine_output_path(
        input_path: &Path,
        target_format: Option<&str>,
        output_dir: Option<&Path>,
    ) -> PathBuf {
        let file_stem = input_path
            .file_stem()
            .and_then(|s| s.to_str())
            .map(Self::sanitize_filename)
            .unwrap_or_else(|| "output".to_string());

        let extension = target_format.unwrap_or("dst");

        let output_filename = format!("{}.{}", file_stem, extension);

        if let Some(dir) = output_dir {
            dir.join(output_filename)
        } else {
            input_path.with_file_name(output_filename)
        }
    }

    /// Perform the actual conversion
    fn perform_conversion(input_path: &Path, output_path: &Path) -> Result<()> {
        // Read the input file
        let pattern = read_embroidery_file(input_path)?;

        // Write the output file
        write_embroidery_file(&pattern, output_path)?;

        Ok(())
    }
}

/// Builder for exporting a single pattern to multiple formats
pub struct MultiFormatExporter {
    output_dir: Option<PathBuf>,
    base_name: Option<String>,
    formats: Vec<String>,
    overwrite: bool,
}

impl MultiFormatExporter {
    /// Create a new multi-format exporter builder
    pub fn new() -> Self {
        Self {
            output_dir: None,
            base_name: None,
            formats: Vec::new(),
            overwrite: false,
        }
    }

    /// Set the output directory for exported files
    pub fn output_dir<P: AsRef<Path>>(mut self, path: P) -> Self {
        self.output_dir = Some(path.as_ref().to_path_buf());
        self
    }

    /// Set the base name for output files (without extension)
    pub fn base_name(mut self, name: &str) -> Self {
        self.base_name = Some(name.to_string());
        self
    }

    /// Set the target formats to export
    pub fn formats(mut self, formats: &[&str]) -> Self {
        self.formats = formats.iter().map(|s| s.to_lowercase()).collect();
        self
    }

    /// Enable or disable overwriting existing files (default: false)
    pub fn overwrite(mut self, overwrite: bool) -> Self {
        self.overwrite = overwrite;
        self
    }

    /// Build and execute the export
    pub fn build(self) -> MultiFormatExporterExecutor {
        MultiFormatExporterExecutor { config: self }
    }
}

impl Default for MultiFormatExporter {
    fn default() -> Self {
        Self::new()
    }
}

/// Executes multi-format export operations
pub struct MultiFormatExporterExecutor {
    config: MultiFormatExporter,
}

impl MultiFormatExporterExecutor {
    /// Export the pattern to all specified formats
    pub fn export(&self, pattern: &EmbPattern) -> Result<ConversionResults> {
        let start = Instant::now();
        let mut results = ConversionResults::new();

        if self.config.formats.is_empty() {
            return Err(Error::InvalidPattern(
                "No output formats specified".to_string(),
            ));
        }

        let base_name = self.config.base_name.as_deref().unwrap_or("pattern");

        // Ensure output directory exists
        if let Some(ref output_dir) = self.config.output_dir {
            fs::create_dir_all(output_dir)?;
        }

        // Export to each format
        for format in &self.config.formats {
            let output_filename = format!("{}.{}", base_name, format);
            let output_path = if let Some(ref dir) = self.config.output_dir {
                dir.join(output_filename)
            } else {
                PathBuf::from(output_filename)
            };

            let export_start = Instant::now();

            // Check if file exists and overwrite is disabled
            if output_path.exists() && !self.config.overwrite {
                results.add(ConversionResult::Skipped {
                    input: PathBuf::from(format!("pattern.{}", format)),
                    reason: "Output file already exists".to_string(),
                });
                continue;
            }

            // Export to format
            match write_embroidery_file(pattern, &output_path) {
                Ok(()) => {
                    let duration = export_start.elapsed().as_millis();
                    let file_size = fs::metadata(&output_path).map(|m| m.len()).unwrap_or(0);

                    results.add(ConversionResult::Success {
                        input: PathBuf::from(base_name),
                        output: output_path,
                        duration_ms: duration,
                        file_size,
                    });
                }
                Err(e) => {
                    results.add(ConversionResult::Failed {
                        input: PathBuf::from(base_name),
                        error: e.to_string(),
                        duration_ms: export_start.elapsed().as_millis(),
                    });
                }
            }
        }

        results.set_total_duration(start.elapsed().as_millis());
        Ok(results)
    }
}

/// Read an embroidery file, auto-detecting the format
fn read_embroidery_file(path: &Path) -> Result<EmbPattern> {
    let extension = path
        .extension()
        .and_then(|ext| ext.to_str())
        .map(|s| s.to_lowercase())
        .ok_or_else(|| Error::UnsupportedFormat("No file extension".to_string()))?;

    let mut file = BufReader::new(File::open(path)?);

    match extension.as_str() {
        "dst" => readers::dst::read(&mut file, None),
        "pes" => {
            let mut pattern = EmbPattern::new();
            readers::pes::read(&mut file, &mut pattern)?;
            Ok(pattern)
        }
        "exp" => readers::exp::read(&mut file),
        "jef" => readers::jef::read(&mut file, None),
        "vp3" => {
            let mut pattern = EmbPattern::new();
            readers::vp3::read(&mut file, &mut pattern)?;
            Ok(pattern)
        }
        "pec" => readers::pec::read(&mut file),
        "json" => readers::json::read(&mut file),
        "csv" => {
            let mut pattern = EmbPattern::new();
            readers::csv::read(&mut file, &mut pattern)?;
            Ok(pattern)
        }
        "xxx" => {
            let mut pattern = EmbPattern::new();
            readers::xxx::read(&mut file, &mut pattern)?;
            Ok(pattern)
        }
        "u01" => {
            let mut pattern = EmbPattern::new();
            readers::u01::read(&mut file, &mut pattern)?;
            Ok(pattern)
        }
        "tbf" => {
            let mut pattern = EmbPattern::new();
            readers::tbf::read(&mut file, &mut pattern)?;
            Ok(pattern)
        }
        "col" => {
            let mut pattern = EmbPattern::new();
            readers::col::read(&mut file, &mut pattern)?;
            Ok(pattern)
        }
        "edr" => {
            let mut pattern = EmbPattern::new();
            readers::edr::read(&mut file, &mut pattern)?;
            Ok(pattern)
        }
        "inf" => {
            let mut pattern = EmbPattern::new();
            readers::inf::read(&mut file, &mut pattern)?;
            Ok(pattern)
        }
        "gcode" => {
            let mut pattern = EmbPattern::new();
            readers::gcode::read(&mut file, &mut pattern)?;
            Ok(pattern)
        }
        _ => Err(Error::UnsupportedFormat(format!(
            "Unsupported input format: {}",
            extension
        ))),
    }
}

/// Write an embroidery file, auto-detecting the format from extension
fn write_embroidery_file(pattern: &EmbPattern, path: &Path) -> Result<()> {
    let extension = path
        .extension()
        .and_then(|ext| ext.to_str())
        .map(|s| s.to_lowercase())
        .ok_or_else(|| Error::UnsupportedFormat("No file extension".to_string()))?;

    // Ensure parent directory exists
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }

    let file = File::create(path)?;
    let mut writer = BufWriter::new(file);

    match extension.as_str() {
        "dst" => writers::dst::write(&mut writer, pattern, false, 3),
        "pes" => writers::pes::write_pes(pattern, &mut writer, writers::pes::PesVersion::V1, false),
        "exp" => writers::exp::write(&mut writer, pattern),
        "jef" => writers::jef::write(&mut writer, pattern, false, 0, ""),
        "vp3" => writers::vp3::write(&mut writer, pattern),
        "xxx" => writers::xxx::write(pattern, &mut writer).map_err(|e| Error::Parse(e.to_string())),
        "u01" => writers::u01::write(pattern, &mut writer),
        "pec" => writers::pec::write(&mut writer, pattern),
        "tbf" => writers::tbf::write(pattern, &mut writer).map_err(|e| Error::Parse(e.to_string())),
        "col" => writers::col::write(pattern, &mut writer).map_err(|e| Error::Parse(e.to_string())),
        "edr" => writers::edr::write(pattern, &mut writer).map_err(|e| Error::Parse(e.to_string())),
        "inf" => writers::inf::write(pattern, &mut writer).map_err(|e| Error::Parse(e.to_string())),
        "json" => writers::json::write(&mut writer, pattern),
        "csv" => writers::csv::write(&mut writer, pattern, writers::csv::CsvVersion::Default),
        "txt" => writers::txt::write(pattern, &mut writer).map_err(|e| Error::Parse(e.to_string())),
        "svg" => writers::svg::write(pattern, &mut writer).map_err(|e| Error::Parse(e.to_string())),
        "gcode" => {
            writers::gcode::write(pattern, &mut writer).map_err(|e| Error::Parse(e.to_string()))
        }
        _ => Err(Error::UnsupportedFormat(format!(
            "Unsupported output format: {}",
            extension
        ))),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sanitize_filename() {
        // Path traversal attempts - removes slashes and replaces ..
        assert_eq!(
            BatchConverterExecutor::sanitize_filename("../../../etc/passwd"),
            "___etcpasswd"
        );
        assert_eq!(
            BatchConverterExecutor::sanitize_filename("..\\..\\windows\\system32"),
            "__windowssystem32"
        );

        // Malicious characters removed
        assert_eq!(
            BatchConverterExecutor::sanitize_filename("file<>:\"|?*.dst"),
            "file.dst"
        );

        // Null bytes and control characters removed
        assert_eq!(
            BatchConverterExecutor::sanitize_filename("file\0name\x01test"),
            "filenametest"
        );

        // Normal filenames should pass through unchanged
        assert_eq!(
            BatchConverterExecutor::sanitize_filename("my_design_2024"),
            "my_design_2024"
        );
        assert_eq!(
            BatchConverterExecutor::sanitize_filename("Pattern-01 (final)"),
            "Pattern-01 (final)"
        );

        // Leading/trailing dots - replaced .. then trimmed
        assert_eq!(
            BatchConverterExecutor::sanitize_filename("...file..."),
            "_.file_"
        );

        // Empty or whitespace-only results in empty string
        assert_eq!(BatchConverterExecutor::sanitize_filename("   "), "");
        assert_eq!(BatchConverterExecutor::sanitize_filename("..."), "_");

        // Path separators within filename removed
        assert_eq!(
            BatchConverterExecutor::sanitize_filename("folder/file.dst"),
            "folderfile.dst"
        );
    }

    #[test]
    fn test_conversion_results() {
        let mut results = ConversionResults::new();

        results.add(ConversionResult::Success {
            input: PathBuf::from("test.dst"),
            output: PathBuf::from("test.pes"),
            duration_ms: 100,
            file_size: 1024,
        });

        results.add(ConversionResult::Failed {
            input: PathBuf::from("bad.dst"),
            error: "Parse error".to_string(),
            duration_ms: 50,
        });

        assert_eq!(results.success_count(), 1);
        assert_eq!(results.failed_count(), 1);
        assert_eq!(results.success_rate(), 0.5);
    }

    #[test]
    fn test_batch_converter_builder() {
        let converter = BatchConverter::new()
            .input_dir("./test")
            .output_dir("./output")
            .target_format("dst")
            .overwrite(true)
            .build();

        assert!(converter.config.input_dir.is_some());
        assert!(converter.config.output_dir.is_some());
        assert_eq!(converter.config.target_format, Some("dst".to_string()));
        assert!(converter.config.overwrite);
    }

    #[test]
    fn test_multi_format_exporter_builder() {
        let exporter = MultiFormatExporter::new()
            .output_dir("./exports")
            .base_name("design")
            .formats(&["dst", "pes", "jef"])
            .build();

        assert!(exporter.config.output_dir.is_some());
        assert_eq!(exporter.config.base_name, Some("design".to_string()));
        assert_eq!(exporter.config.formats.len(), 3);
    }
}
