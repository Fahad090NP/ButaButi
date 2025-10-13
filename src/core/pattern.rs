//! Embroidery pattern structure and manipulation
//!
//! The core `EmbPattern` type stores stitches, threads, and metadata for embroidery designs.
//! Supports reading/writing multiple formats, transformations, and pattern analysis.

use crate::core::constants::*;
use crate::core::thread::EmbThread;
use crate::utils::error::{Error, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// A single stitch with position and command
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Stitch {
    /// X coordinate (in 0.1mm units)
    pub x: f64,
    /// Y coordinate (in 0.1mm units)
    pub y: f64,
    /// Command (STITCH, JUMP, TRIM, etc.)
    pub command: u32,
}

impl Stitch {
    /// Create a new stitch
    pub const fn new(x: f64, y: f64, command: u32) -> Self {
        Self { x, y, command }
    }

    /// Calculate the relative position (delta) from another stitch
    ///
    /// Returns (dx, dy) where dx = self.x - other.x, dy = self.y - other.y
    ///
    /// # Example
    ///
    /// ```
    /// use butabuti::core::pattern::Stitch;
    ///
    /// let stitch1 = Stitch::new(100.0, 100.0, 0);
    /// let stitch2 = Stitch::new(50.0, 30.0, 0);
    /// let (dx, dy) = stitch1.relative_to(&stitch2);
    /// assert_eq!(dx, 50.0);
    /// assert_eq!(dy, 70.0);
    /// ```
    #[inline]
    pub fn relative_to(&self, other: &Self) -> (f64, f64) {
        (self.x - other.x, self.y - other.y)
    }

    /// Calculate the Euclidean distance to another stitch
    ///
    /// Uses the Pythagorean theorem: distance = √(dx² + dy²)
    ///
    /// # Example
    ///
    /// ```
    /// use butabuti::core::pattern::Stitch;
    ///
    /// let stitch1 = Stitch::new(0.0, 0.0, 0);
    /// let stitch2 = Stitch::new(30.0, 40.0, 0);
    /// let distance = stitch1.distance_to(&stitch2);
    /// assert_eq!(distance, 50.0); // 3-4-5 triangle
    /// ```
    #[inline]
    pub fn distance_to(&self, other: &Self) -> f64 {
        let (dx, dy) = self.relative_to(other);
        ((dx * dx) + (dy * dy)).sqrt()
    }

    /// Check if the stitch has valid coordinates
    ///
    /// Returns false if x or y is NaN or infinite
    ///
    /// # Example
    ///
    /// ```
    /// use butabuti::core::pattern::Stitch;
    ///
    /// let valid = Stitch::new(100.0, 200.0, 0);
    /// assert!(valid.is_valid());
    ///
    /// let invalid = Stitch::new(f64::NAN, 200.0, 0);
    /// assert!(!invalid.is_valid());
    ///
    /// let infinite = Stitch::new(f64::INFINITY, 200.0, 0);
    /// assert!(!infinite.is_valid());
    /// ```
    #[inline]
    pub fn is_valid(&self) -> bool {
        self.x.is_finite() && self.y.is_finite()
    }

    /// Create a zero-position stitch (at origin)
    ///
    /// Useful as a starting reference point
    #[inline]
    pub const fn zero() -> Self {
        Self::new(0.0, 0.0, STITCH)
    }

    /// Get the type of this stitch based on its command
    ///
    /// Returns a `StitchType` enum value that categorizes the stitch command.
    ///
    /// # Example
    ///
    /// ```
    /// use butabuti::core::pattern::Stitch;
    /// use butabuti::core::constants::{StitchType, JUMP, TRIM};
    ///
    /// let jump = Stitch::new(10.0, 20.0, JUMP);
    /// assert_eq!(jump.stitch_type(), StitchType::Jump);
    ///
    /// let trim = Stitch::new(10.0, 20.0, TRIM);
    /// assert_eq!(trim.stitch_type(), StitchType::Trim);
    /// assert!(trim.stitch_type().is_thread_command());
    /// ```
    #[inline]
    pub fn stitch_type(&self) -> crate::core::constants::StitchType {
        crate::core::constants::StitchType::from_command(self.command)
    }
}

impl std::fmt::Display for Stitch {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let cmd_name = match self.command & COMMAND_MASK {
            STITCH => "STITCH",
            JUMP => "JUMP",
            TRIM => "TRIM",
            CUT => "CUT",
            COLOR_CHANGE => "COLOR",
            STOP => "STOP",
            END => "END",
            SEQUIN_EJECT => "SEQUIN",
            _ => "UNKNOWN",
        };
        write!(f, "Stitch({:.2}, {:.2}, {})", self.x, self.y, cmd_name)
    }
}

/// Thread usage statistics for a single thread color
#[derive(Debug, Clone, PartialEq)]
pub struct ThreadUsage {
    /// The thread with color and metadata
    pub thread: EmbThread,
    /// Total stitch length for this thread in millimeters
    pub length_mm: f64,
    /// Number of stitches using this thread
    pub stitch_count: usize,
}

/// Comprehensive pattern statistics
#[derive(Debug, Clone, PartialEq)]
pub struct PatternStatistics {
    /// Total number of actual stitches (excludes jumps, trims, etc.)
    pub stitch_count: usize,
    /// Number of jump stitches
    pub jump_count: usize,
    /// Number of trim commands
    pub trim_count: usize,
    /// Number of color change commands
    pub color_change_count: usize,
    /// Total stitch length in millimeters
    pub total_length_mm: f64,
    /// Total stitch length in inches
    pub total_length_inches: f64,
    /// Estimated sewing time in minutes (based on 800 stitches/min default machine speed)
    pub estimated_time_minutes: f64,
    /// Per-thread usage statistics
    pub thread_usage: Vec<ThreadUsage>,
    /// Stitch density (stitches per square centimeter)
    pub density: f64,
    /// Pattern width in millimeters
    pub width_mm: f64,
    /// Pattern height in millimeters
    pub height_mm: f64,
    /// Average stitch length in millimeters
    pub avg_stitch_length_mm: f64,
    /// Maximum stitch length in millimeters
    pub max_stitch_length_mm: f64,
}

/// Main embroidery pattern structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmbPattern {
    /// List of stitches
    stitches: Vec<Stitch>,

    /// List of threads used in the pattern
    thread_list: Vec<EmbThread>,

    /// Additional metadata
    extras: HashMap<String, String>,

    /// Previous X position (for relative stitching)
    previous_x: f64,

    /// Previous Y position (for relative stitching)
    previous_y: f64,

    /// Thread color grouping (optional)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    color_grouping: Option<crate::core::color_group::ThreadGrouping>,
}

/// Command type for pattern iteration
///
/// This enum represents the different types of commands that can appear in a pattern
/// when iterating through stitches. It provides a high-level view of the stitch sequence
/// that is useful for format writers and pattern analysis.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum StitchCommand<'a> {
    /// Regular stitch (move and drop needle)
    Stitch(&'a Stitch),
    /// Jump stitch (move without dropping needle)
    Jump(&'a Stitch),
    /// Color change with optional thread reference
    ColorChange(Option<&'a EmbThread>, &'a Stitch),
    /// Trim thread (cut with tail)
    Trim(&'a Stitch),
    /// Cut thread (full cut with no tail)
    Cut(&'a Stitch),
    /// Stop machine (for applique, manual thread change, etc.)
    Stop(&'a Stitch),
    /// End of pattern
    End(&'a Stitch),
}

/// Iterator over pattern commands
///
/// This iterator converts the flat stitch list into a stream of high-level commands,
/// making it easier to write format-specific encoders and analyze patterns.
pub struct StitchCommandIterator<'a> {
    pattern: &'a EmbPattern,
    index: usize,
}

impl<'a> StitchCommandIterator<'a> {
    fn new(pattern: &'a EmbPattern) -> Self {
        Self { pattern, index: 0 }
    }
}

impl<'a> Iterator for StitchCommandIterator<'a> {
    type Item = StitchCommand<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.index >= self.pattern.stitches.len() {
            return None;
        }

        let stitch = &self.pattern.stitches[self.index];
        let command = extract_command(stitch.command);
        self.index += 1;

        // Determine which thread is active for color changes
        let thread = if command == COLOR_CHANGE {
            // Count color changes up to this point to determine thread index
            let color_changes = self.pattern.stitches[..self.index]
                .iter()
                .filter(|s| extract_command(s.command) == COLOR_CHANGE)
                .count();
            self.pattern.thread_list.get(color_changes)
        } else {
            None
        };

        Some(match command {
            STITCH => StitchCommand::Stitch(stitch),
            JUMP => StitchCommand::Jump(stitch),
            COLOR_CHANGE => StitchCommand::ColorChange(thread, stitch),
            TRIM => StitchCommand::Trim(stitch),
            CUT => StitchCommand::Cut(stitch),
            STOP => StitchCommand::Stop(stitch),
            END => StitchCommand::End(stitch),
            _ => StitchCommand::Stitch(stitch), // Treat unknown as stitch
        })
    }
}

impl EmbPattern {
    /// Create a new empty pattern
    ///
    /// # Example
    ///
    /// ```
    /// use rusty_petal::pattern::EmbPattern;
    ///
    /// let pattern = EmbPattern::new();
    /// assert_eq!(pattern.stitches().len(), 0);
    /// ```
    pub fn new() -> Self {
        Self {
            stitches: Vec::new(),
            thread_list: Vec::new(),
            extras: HashMap::new(),
            previous_x: 0.0,
            previous_y: 0.0,
            color_grouping: None,
        }
    }

    /// Create a pattern from existing stitches and threads
    pub fn from_stitches(stitches: Vec<Stitch>, threads: Vec<EmbThread>) -> Self {
        Self {
            stitches,
            thread_list: threads,
            extras: HashMap::new(),
            previous_x: 0.0,
            previous_y: 0.0,
            color_grouping: None,
        }
    }

    /// Get reference to stitches
    pub fn stitches(&self) -> &[Stitch] {
        &self.stitches
    }

    /// Get reference to thread list
    pub fn threads(&self) -> &[EmbThread] {
        &self.thread_list
    }

    /// Get reference to extras/metadata
    pub fn extras(&self) -> &HashMap<String, String> {
        &self.extras
    }

    /// Add a stitch at absolute position
    ///
    /// # Arguments
    ///
    /// * `command` - The stitch command (STITCH, JUMP, etc.)
    /// * `x` - Absolute X coordinate
    /// * `y` - Absolute Y coordinate
    pub fn add_stitch_absolute(&mut self, command: u32, x: f64, y: f64) {
        self.stitches.push(Stitch::new(x, y, command));
        self.previous_x = x;
        self.previous_y = y;
    }

    /// Add a stitch relative to previous position
    ///
    /// # Arguments
    ///
    /// * `dx` - Relative X offset
    /// * `dy` - Relative Y offset
    /// * `command` - The stitch command (STITCH, JUMP, etc.)
    pub fn add_stitch_relative(&mut self, dx: f64, dy: f64, command: u32) {
        let x = self.previous_x + dx;
        let y = self.previous_y + dy;
        self.add_stitch_absolute(command, x, y);
    }

    /// Add a command without updating position
    pub fn add_command(&mut self, command: u32, x: f64, y: f64) {
        self.stitches.push(Stitch::new(x, y, command));
    }

    /// Add a thread to the pattern
    pub fn add_thread(&mut self, thread: EmbThread) {
        self.thread_list.push(thread);
    }

    /// Set metadata value
    pub fn set_metadata(&mut self, key: impl Into<String>, value: impl Into<String>) {
        self.extras.insert(key.into(), value.into());
    }

    /// Get metadata value
    pub fn get_metadata(&self, key: &str) -> Option<&String> {
        self.extras.get(key)
    }

    /// Get all metadata as an iterator
    pub fn metadata(&self) -> impl Iterator<Item = (&String, &String)> {
        self.extras.iter()
    }

    // ========== Structured Property Accessors ==========
    // These provide typed access to common embroidery pattern properties

    /// Get pattern title/name
    pub fn title(&self) -> Option<&str> {
        self.get_metadata("name")
            .or_else(|| self.get_metadata("title"))
            .map(|s| s.as_str())
    }

    /// Set pattern title/name
    pub fn set_title(&mut self, title: impl Into<String>) {
        self.set_metadata("name", title);
    }

    /// Get pattern author/designer
    pub fn author(&self) -> Option<&str> {
        self.get_metadata("author").map(|s| s.as_str())
    }

    /// Set pattern author/designer
    pub fn set_author(&mut self, author: impl Into<String>) {
        self.set_metadata("author", author);
    }

    /// Get copyright information
    pub fn copyright(&self) -> Option<&str> {
        self.get_metadata("copyright").map(|s| s.as_str())
    }

    /// Set copyright information
    pub fn set_copyright(&mut self, copyright: impl Into<String>) {
        self.set_metadata("copyright", copyright);
    }

    /// Get pattern description
    pub fn description(&self) -> Option<&str> {
        self.get_metadata("description").map(|s| s.as_str())
    }

    /// Set pattern description
    pub fn set_description(&mut self, description: impl Into<String>) {
        self.set_metadata("description", description);
    }

    /// Get pattern keywords (comma-separated string)
    pub fn keywords(&self) -> Option<Vec<String>> {
        self.get_metadata("keywords").map(|s| {
            s.split(',')
                .map(|k| k.trim().to_string())
                .filter(|k| !k.is_empty())
                .collect()
        })
    }

    /// Set pattern keywords from a vector
    pub fn set_keywords(&mut self, keywords: &[impl AsRef<str>]) {
        let keywords_str = keywords
            .iter()
            .map(|k| k.as_ref())
            .collect::<Vec<_>>()
            .join(", ");
        self.set_metadata("keywords", keywords_str);
    }

    /// Get creation date (ISO 8601 format recommended)
    pub fn date(&self) -> Option<&str> {
        self.get_metadata("date").map(|s| s.as_str())
    }

    /// Set creation date
    pub fn set_date(&mut self, date: impl Into<String>) {
        self.set_metadata("date", date);
    }

    /// Get notes/comments
    pub fn notes(&self) -> Option<&str> {
        self.get_metadata("notes")
            .or_else(|| self.get_metadata("comments"))
            .map(|s| s.as_str())
    }

    /// Set notes/comments
    pub fn set_notes(&mut self, notes: impl Into<String>) {
        self.set_metadata("notes", notes);
    }

    /// Get creating software name
    pub fn software(&self) -> Option<&str> {
        self.get_metadata("software").map(|s| s.as_str())
    }

    /// Set creating software name
    pub fn set_software(&mut self, software: impl Into<String>) {
        self.set_metadata("software", software);
    }

    /// Get software version
    pub fn software_version(&self) -> Option<&str> {
        self.get_metadata("software_version")
            .or_else(|| self.get_metadata("version"))
            .map(|s| s.as_str())
    }

    /// Set software version
    pub fn set_software_version(&mut self, version: impl Into<String>) {
        self.set_metadata("software_version", version);
    }

    /// Get hoop size/type (e.g., "4x4", "5x7", "100x100")
    pub fn hoop_size(&self) -> Option<&str> {
        self.get_metadata("hoop_size")
            .or_else(|| self.get_metadata("hoop"))
            .map(|s| s.as_str())
    }

    /// Set hoop size/type
    pub fn set_hoop_size(&mut self, hoop: impl Into<String>) {
        self.set_metadata("hoop_size", hoop);
    }

    /// Get design width in millimeters (from metadata or calculated)
    pub fn design_width(&self) -> Option<f64> {
        self.get_metadata("design_width")
            .and_then(|s| s.parse::<f64>().ok())
            .or_else(|| {
                if self.stitches.is_empty() {
                    return None;
                }
                let (min_x, _, max_x, _) = self.bounds();
                if min_x.is_finite() && max_x.is_finite() {
                    Some((max_x - min_x) / 10.0) // Convert 0.1mm to mm
                } else {
                    None
                }
            })
    }

    /// Get design height in millimeters (from metadata or calculated)
    pub fn design_height(&self) -> Option<f64> {
        self.get_metadata("design_height")
            .and_then(|s| s.parse::<f64>().ok())
            .or_else(|| {
                if self.stitches.is_empty() {
                    return None;
                }
                let (_, min_y, _, max_y) = self.bounds();
                if min_y.is_finite() && max_y.is_finite() {
                    Some((max_y - min_y) / 10.0) // Convert 0.1mm to mm
                } else {
                    None
                }
            })
    }

    /// Get fabric type
    pub fn fabric_type(&self) -> Option<&str> {
        self.get_metadata("fabric_type")
            .or_else(|| self.get_metadata("fabric"))
            .map(|s| s.as_str())
    }

    /// Set fabric type
    pub fn set_fabric_type(&mut self, fabric: impl Into<String>) {
        self.set_metadata("fabric_type", fabric);
    }

    /// Get thread brand/manufacturer
    pub fn thread_brand(&self) -> Option<&str> {
        self.get_metadata("thread_brand").map(|s| s.as_str())
    }

    /// Set thread brand/manufacturer
    pub fn set_thread_brand(&mut self, brand: impl Into<String>) {
        self.set_metadata("thread_brand", brand);
    }

    /// Get company/organization name
    pub fn company(&self) -> Option<&str> {
        self.get_metadata("company")
            .or_else(|| self.get_metadata("organization"))
            .map(|s| s.as_str())
    }

    /// Set company/organization name
    pub fn set_company(&mut self, company: impl Into<String>) {
        self.set_metadata("company", company);
    }

    /// Iterate over pattern commands
    ///
    /// Returns an iterator that yields high-level commands (Stitch, Jump, ColorChange, etc.)
    /// instead of raw stitches with command flags. This is useful for format writers and
    /// pattern analysis.
    ///
    /// # Example
    ///
    /// ```
    /// use butabuti::prelude::*;
    /// use butabuti::core::pattern::StitchCommand;
    ///
    /// let mut pattern = EmbPattern::new();
    /// pattern.add_thread(EmbThread::from_string("red").unwrap());
    /// pattern.stitch(10.0, 0.0);
    /// pattern.trim();
    /// pattern.end();
    ///
    /// for cmd in pattern.iter_commands() {
    ///     match cmd {
    ///         StitchCommand::Stitch(s) => println!("Stitch at ({}, {})", s.x, s.y),
    ///         StitchCommand::Trim(s) => println!("Trim at ({}, {})", s.x, s.y),
    ///         StitchCommand::End(s) => println!("End at ({}, {})", s.x, s.y),
    ///         _ => {}
    ///     }
    /// }
    /// ```
    pub fn iter_commands(&self) -> StitchCommandIterator<'_> {
        StitchCommandIterator::new(self)
    }

    /// Calculate pattern bounds
    ///
    /// Returns (min_x, min_y, max_x, max_y)
    pub fn bounds(&self) -> (f64, f64, f64, f64) {
        if self.stitches.is_empty() {
            return (0.0, 0.0, 0.0, 0.0);
        }

        let mut min_x = f64::INFINITY;
        let mut min_y = f64::INFINITY;
        let mut max_x = f64::NEG_INFINITY;
        let mut max_y = f64::NEG_INFINITY;

        for stitch in &self.stitches {
            // Only consider finite coordinates
            if stitch.x.is_finite() && stitch.y.is_finite() {
                if stitch.x < min_x {
                    min_x = stitch.x;
                }
                if stitch.x > max_x {
                    max_x = stitch.x;
                }
                if stitch.y < min_y {
                    min_y = stitch.y;
                }
                if stitch.y > max_y {
                    max_y = stitch.y;
                }
            }
        }

        // If all coordinates were non-finite, return zeros
        if !min_x.is_finite() {
            return (0.0, 0.0, 0.0, 0.0);
        }

        (min_x, min_y, max_x, max_y)
    }

    /// Translate pattern by given offset
    pub fn translate(&mut self, dx: f64, dy: f64) {
        // Guard against non-finite translations
        if !dx.is_finite() || !dy.is_finite() {
            return;
        }

        for stitch in &mut self.stitches {
            stitch.x += dx;
            stitch.y += dy;
        }
        self.previous_x += dx;
        self.previous_y += dy;
    }

    /// Move pattern center to origin
    pub fn move_center_to_origin(&mut self) {
        let (min_x, min_y, max_x, max_y) = self.bounds();
        let cx = ((max_x + min_x) / 2.0).round();
        let cy = ((max_y + min_y) / 2.0).round();
        self.translate(-cx, -cy);
    }

    /// Rotate pattern around origin by angle in degrees
    ///
    /// # Arguments
    ///
    /// * `angle_degrees` - Rotation angle in degrees (positive = counterclockwise)
    ///
    /// # Example
    ///
    /// ```
    /// use butabuti::prelude::*;
    ///
    /// let mut pattern = EmbPattern::new();
    /// pattern.stitch(100.0, 0.0);
    /// pattern.rotate(90.0);
    /// // Point is now at (0, 100) due to 90° rotation
    /// ```
    pub fn rotate(&mut self, angle_degrees: f64) {
        if !angle_degrees.is_finite() {
            return;
        }

        let angle_rad = angle_degrees.to_radians();
        let cos_a = angle_rad.cos();
        let sin_a = angle_rad.sin();

        for stitch in &mut self.stitches {
            let x = stitch.x;
            let y = stitch.y;
            stitch.x = x * cos_a - y * sin_a;
            stitch.y = x * sin_a + y * cos_a;
        }

        // Update previous position
        let prev_x = self.previous_x;
        let prev_y = self.previous_y;
        self.previous_x = prev_x * cos_a - prev_y * sin_a;
        self.previous_y = prev_x * sin_a + prev_y * cos_a;
    }

    /// Rotate pattern around a specific point
    ///
    /// # Arguments
    ///
    /// * `angle_degrees` - Rotation angle in degrees
    /// * `cx` - Center X coordinate
    /// * `cy` - Center Y coordinate
    pub fn rotate_around_point(&mut self, angle_degrees: f64, cx: f64, cy: f64) {
        if !angle_degrees.is_finite() || !cx.is_finite() || !cy.is_finite() {
            return;
        }

        // Translate to origin, rotate, translate back
        self.translate(-cx, -cy);
        self.rotate(angle_degrees);
        self.translate(cx, cy);
    }

    /// Scale pattern by given factors
    ///
    /// # Arguments
    ///
    /// * `sx` - X scale factor
    /// * `sy` - Y scale factor
    ///
    /// # Example
    ///
    /// ```
    /// use butabuti::prelude::*;
    ///
    /// let mut pattern = EmbPattern::new();
    /// pattern.stitch(100.0, 50.0);
    /// pattern.scale(2.0, 1.5);
    /// // Pattern is now 2x wider and 1.5x taller
    /// ```
    pub fn scale(&mut self, sx: f64, sy: f64) {
        if !sx.is_finite() || !sy.is_finite() || sx == 0.0 || sy == 0.0 {
            return;
        }

        for stitch in &mut self.stitches {
            stitch.x *= sx;
            stitch.y *= sy;
        }

        self.previous_x *= sx;
        self.previous_y *= sy;
    }

    /// Scale pattern uniformly
    ///
    /// # Arguments
    ///
    /// * `factor` - Uniform scale factor
    pub fn scale_uniform(&mut self, factor: f64) {
        self.scale(factor, factor);
    }

    /// Flip pattern horizontally (mirror across Y axis)
    ///
    /// # Example
    ///
    /// ```
    /// use butabuti::prelude::*;
    ///
    /// let mut pattern = EmbPattern::new();
    /// pattern.stitch(100.0, 50.0);
    /// pattern.flip_horizontal();
    /// // X coordinates are negated
    /// ```
    pub fn flip_horizontal(&mut self) {
        for stitch in &mut self.stitches {
            stitch.x = -stitch.x;
        }
        self.previous_x = -self.previous_x;
    }

    /// Flip pattern vertically (mirror across X axis)
    ///
    /// # Example
    ///
    /// ```
    /// use butabuti::prelude::*;
    ///
    /// let mut pattern = EmbPattern::new();
    /// pattern.stitch(100.0, 50.0);
    /// pattern.flip_vertical();
    /// // Y coordinates are negated
    /// ```
    pub fn flip_vertical(&mut self) {
        for stitch in &mut self.stitches {
            stitch.y = -stitch.y;
        }
        self.previous_y = -self.previous_y;
    }

    /// Apply an affine transformation matrix to all stitches
    ///
    /// Applies the given transformation matrix to every stitch in the pattern,
    /// enabling complex transformations like combined rotation, scaling, and translation.
    ///
    /// # Arguments
    ///
    /// * `matrix` - The transformation matrix to apply
    ///
    /// # Example
    ///
    /// ```
    /// use butabuti::prelude::*;
    ///
    /// let mut pattern = EmbPattern::new();
    /// pattern.stitch(100.0, 0.0);
    ///
    /// // Create a transformation: rotate 90° then scale 2x
    /// let mut matrix = EmbMatrix::new();
    /// matrix.post_rotate(90.0, 0.0, 0.0);
    /// matrix.post_scale(2.0, None, 0.0, 0.0);
    ///
    /// pattern.apply_matrix(&matrix);
    /// ```
    pub fn apply_matrix(&mut self, matrix: &crate::core::matrix::EmbMatrix) {
        for stitch in &mut self.stitches {
            let (new_x, new_y) = matrix.transform_point(stitch.x, stitch.y);
            stitch.x = new_x;
            stitch.y = new_y;
        }

        // Update previous position
        let (new_prev_x, new_prev_y) = matrix.transform_point(self.previous_x, self.previous_y);
        self.previous_x = new_prev_x;
        self.previous_y = new_prev_y;
    }

    /// Split long stitches to comply with format constraints
    ///
    /// Automatically splits stitches exceeding the specified maximum length
    /// into multiple shorter stitches, preserving the overall path.
    ///
    /// # Arguments
    ///
    /// * `max_length` - Maximum allowed stitch length in pattern units (0.1mm)
    ///
    /// # Example
    ///
    /// ```
    /// use butabuti::prelude::*;
    ///
    /// let mut pattern = EmbPattern::new();
    /// pattern.stitch(200.0, 0.0);  // Long stitch
    /// pattern.split_long_stitches(100.0)?;
    /// // Now split into multiple stitches
    /// # Ok::<(), butabuti::utils::error::Error>(())
    /// ```
    pub fn split_long_stitches(&mut self, max_length: f64) -> crate::utils::error::Result<()> {
        use crate::utils::error::Error;

        if max_length <= 0.0 || !max_length.is_finite() {
            return Err(Error::InvalidPattern(format!(
                "Invalid max_length: {}",
                max_length
            )));
        }

        let mut new_stitches = Vec::new();
        let mut prev_x = 0.0;
        let mut prev_y = 0.0;

        for stitch in &self.stitches {
            let dx = stitch.x - prev_x;
            let dy = stitch.y - prev_y;
            let length = (dx * dx + dy * dy).sqrt();

            // Only split STITCH commands; preserve all others
            if stitch.command == STITCH && length > max_length {
                // Calculate number of segments needed
                let num_segments = (length / max_length).ceil() as usize;
                let segment_dx = dx / num_segments as f64;
                let segment_dy = dy / num_segments as f64;

                // Create intermediate stitches
                for i in 1..=num_segments {
                    let new_x = prev_x + segment_dx * i as f64;
                    let new_y = prev_y + segment_dy * i as f64;
                    new_stitches.push(Stitch::new(new_x, new_y, STITCH));
                }

                prev_x = stitch.x;
                prev_y = stitch.y;
            } else {
                // Keep stitch as-is
                new_stitches.push(*stitch);
                prev_x = stitch.x;
                prev_y = stitch.y;
            }
        }

        self.stitches = new_stitches;
        Ok(())
    }

    /// Split stitches based on format-specific constraints
    ///
    /// Automatically applies the correct max stitch length for the specified format.
    ///
    /// # Arguments
    ///
    /// * `format` - Format name (e.g., "dst", "pes", "jef")
    ///
    /// # Example
    ///
    /// ```
    /// use butabuti::prelude::*;
    ///
    /// let mut pattern = EmbPattern::new();
    /// pattern.stitch(200.0, 0.0);
    /// pattern.split_to_format_limits("dst")?;  // Splits to DST's ±121 unit limit
    /// # Ok::<(), butabuti::utils::error::Error>(())
    /// ```
    pub fn split_to_format_limits(&mut self, format: &str) -> crate::utils::error::Result<()> {
        use crate::utils::error::Error;

        let max_length = match format.to_lowercase().as_str() {
            "dst" => 121.0,         // DST format: ±121 units (12.1mm)
            "pes" | "pec" => 127.0, // PES/PEC: ±127 units (12.7mm)
            "jef" => 127.0,         // JEF: ±127 units
            "exp" => 127.0,         // EXP: ±127 units
            "vp3" => 127.0,         // VP3: ±127 units
            "xxx" => 127.0,         // XXX: ±127 units
            "u01" => 127.0,         // U01: ±127 units
            _ => {
                return Err(Error::UnsupportedFormat(format!(
                    "Unknown format for stitch splitting: {}",
                    format
                )))
            },
        };

        self.split_long_stitches(max_length)
    }

    /// Remove consecutive duplicate stitches
    ///
    /// Removes stitches that are at the exact same position as the previous stitch,
    /// optimizing file size and machine efficiency. Preserves all command stitches
    /// (jumps, trims, color changes) even if they're at the same position.
    ///
    /// # Example
    ///
    /// ```
    /// use butabuti::prelude::*;
    ///
    /// let mut pattern = EmbPattern::new();
    /// pattern.stitch_abs(10.0, 10.0);
    /// pattern.stitch_abs(10.0, 10.0);  // Duplicate - will be removed
    /// pattern.stitch_abs(20.0, 20.0);
    /// pattern.remove_duplicates();
    /// assert_eq!(pattern.count_stitches(), 2);  // Only 2 stitches remain
    /// ```
    pub fn remove_duplicates(&mut self) {
        if self.stitches.is_empty() {
            return;
        }

        let mut new_stitches = Vec::new();
        new_stitches.push(self.stitches[0]);

        for i in 1..self.stitches.len() {
            let current = &self.stitches[i];
            let previous = &self.stitches[i - 1];

            // Keep stitch if position changed or if it's a command (not just a stitch)
            if current.x != previous.x
                || current.y != previous.y
                || (current.command & !STITCH) != 0
            {
                new_stitches.push(*current);
            }
        }

        self.stitches = new_stitches;
        // Update previous position to match last stitch
        if let Some(last) = self.stitches.last() {
            self.previous_x = last.x;
            self.previous_y = last.y;
        }
    }

    /// Count the number of stitches (excluding non-stitch commands)
    pub fn count_stitches(&self) -> usize {
        self.stitches.iter().filter(|s| s.command == STITCH).count()
    }

    /// Count the number of color changes
    pub fn count_color_changes(&self) -> usize {
        self.stitches
            .iter()
            .filter(|s| s.command == COLOR_CHANGE)
            .count()
    }

    /// Calculate the total stitch length in pattern units (0.1mm)
    ///
    /// Sums the distance between consecutive stitches.
    ///
    /// # Example
    ///
    /// ```
    /// use butabuti::prelude::*;
    ///
    /// let mut pattern = EmbPattern::new();
    /// pattern.stitch(30.0, 40.0);  // 3-4-5 triangle = 50.0 units
    /// assert_eq!(pattern.total_stitch_length(), 50.0);
    /// ```
    #[inline]
    pub fn total_stitch_length(&self) -> f64 {
        let mut total = 0.0;
        let mut prev_x = 0.0;
        let mut prev_y = 0.0;

        for stitch in &self.stitches {
            // Only count actual stitches (not jumps, trims, etc.)
            if stitch.command == STITCH {
                let dx = stitch.x - prev_x;
                let dy = stitch.y - prev_y;
                total += (dx * dx + dy * dy).sqrt();
            }
            // Update position for all commands (stitches, jumps, etc.)
            prev_x = stitch.x;
            prev_y = stitch.y;
        }
        total
    }

    /// Find the maximum stitch length in the pattern
    ///
    /// Returns 0.0 if pattern has no stitches.
    ///
    /// # Example
    ///
    /// ```
    /// use butabuti::prelude::*;
    ///
    /// let mut pattern = EmbPattern::new();
    /// pattern.stitch(10.0, 0.0);
    /// pattern.stitch(50.0, 0.0);  // This is the longest (50.0)
    /// assert_eq!(pattern.max_stitch_length(), 50.0);
    /// ```
    #[inline]
    pub fn max_stitch_length(&self) -> f64 {
        let mut max_length = 0.0;
        let mut prev_x = 0.0;
        let mut prev_y = 0.0;

        for stitch in &self.stitches {
            if stitch.command == STITCH {
                let dx = stitch.x - prev_x;
                let dy = stitch.y - prev_y;
                let length = (dx * dx + dy * dy).sqrt();
                if length > max_length {
                    max_length = length;
                }
            }
            prev_x = stitch.x;
            prev_y = stitch.y;
        }
        max_length
    }

    /// Calculate the average stitch length
    ///
    /// Returns 0.0 if pattern has no stitches.
    ///
    /// # Example
    ///
    /// ```
    /// use butabuti::prelude::*;
    ///
    /// let mut pattern = EmbPattern::new();
    /// pattern.stitch(10.0, 0.0);  // Length: 10.0
    /// pattern.stitch(20.0, 0.0);  // Length: 20.0
    /// assert_eq!(pattern.avg_stitch_length(), 15.0);  // (10 + 20) / 2
    /// ```
    #[inline]
    pub fn avg_stitch_length(&self) -> f64 {
        let count = self.count_stitches();
        if count == 0 {
            return 0.0;
        }
        self.total_stitch_length() / count as f64
    }

    /// Count the number of jumps
    #[inline]
    pub fn count_jumps(&self) -> usize {
        self.stitches.iter().filter(|s| s.command == JUMP).count()
    }

    /// Count the number of trims
    #[inline]
    pub fn count_trims(&self) -> usize {
        self.stitches.iter().filter(|s| s.command == TRIM).count()
    }

    /// Get pattern width in pattern units (0.1mm)
    #[inline]
    pub fn width(&self) -> f64 {
        let (min_x, _, max_x, _) = self.bounds();
        max_x - min_x
    }

    /// Get pattern height in pattern units (0.1mm)
    #[inline]
    pub fn height(&self) -> f64 {
        let (_, min_y, _, max_y) = self.bounds();
        max_y - min_y
    }

    /// Convenience method: add a stitch
    pub fn stitch(&mut self, dx: f64, dy: f64) {
        self.add_stitch_relative(dx, dy, STITCH);
    }

    /// Convenience method: add a stitch at absolute position
    pub fn stitch_abs(&mut self, x: f64, y: f64) {
        self.add_stitch_absolute(STITCH, x, y);
    }

    /// Convenience method: add a jump
    pub fn jump(&mut self, dx: f64, dy: f64) {
        self.add_stitch_relative(dx, dy, JUMP);
    }

    /// Convenience method: add a jump at absolute position
    pub fn jump_abs(&mut self, x: f64, y: f64) {
        self.add_stitch_absolute(JUMP, x, y);
    }

    /// Convenience method: add a trim
    pub fn trim(&mut self) {
        self.add_stitch_relative(0.0, 0.0, TRIM);
    }

    /// Convenience method: add a cut (full thread cut with no tail)
    ///
    /// CUT is similar to TRIM but performs a complete thread cut leaving no tail.
    /// Not all machines support CUT; on machines that don't support it, CUT may
    /// be treated the same as TRIM.
    ///
    /// Use TRIM for standard thread cuts, and CUT only when you specifically need
    /// a full cut (e.g., for certain fabrics or when a cleaner finish is required).
    pub fn cut(&mut self) {
        self.add_stitch_relative(0.0, 0.0, CUT);
    }

    /// Convenience method: add a color change
    pub fn color_change(&mut self, dx: f64, dy: f64) {
        self.add_stitch_relative(dx, dy, COLOR_CHANGE);
    }

    /// Convenience method: add a stop
    pub fn stop(&mut self) {
        self.add_stitch_relative(0.0, 0.0, STOP);
    }

    /// Convenience method: add an end
    pub fn end(&mut self) {
        self.add_stitch_relative(0.0, 0.0, END);
    }

    /// Calculate comprehensive pattern statistics
    ///
    /// Returns detailed statistics including stitch counts, thread usage per color,
    /// estimated sewing time, and density calculations.
    ///
    /// # Arguments
    ///
    /// * `machine_speed_spm` - Machine speed in stitches per minute (default: 800)
    ///
    /// # Examples
    ///
    /// ```
    /// use butabuti::prelude::*;
    ///
    /// let mut pattern = EmbPattern::new();
    /// pattern.add_thread(EmbThread::from_string("red").unwrap());
    /// pattern.stitch(100.0, 0.0);
    /// pattern.stitch(100.0, 100.0);
    ///
    /// // Calculate stats with default machine speed (800 spm)
    /// let stats = pattern.calculate_statistics(800.0);
    ///
    /// assert_eq!(stats.stitch_count, 2);
    /// assert!(stats.total_length_mm > 0.0);
    /// assert!(stats.estimated_time_minutes > 0.0);
    /// ```
    pub fn calculate_statistics(&self, machine_speed_spm: f64) -> PatternStatistics {
        let stitch_count = self.count_stitches();
        let jump_count = self.count_jumps();
        let trim_count = self.count_trims();
        let color_change_count = self.count_color_changes();

        // Total length in 0.1mm units, convert to mm
        let total_length_0_1mm = self.total_stitch_length();
        let total_length_mm = total_length_0_1mm / 10.0;
        let total_length_inches = total_length_mm / 25.4;

        // Estimated time based on machine speed
        let estimated_time_minutes = if machine_speed_spm > 0.0 {
            stitch_count as f64 / machine_speed_spm
        } else {
            0.0
        };

        // Calculate thread usage per color
        let thread_usage = self.calculate_thread_usage();

        // Calculate density (stitches per square cm)
        let (min_x, min_y, max_x, max_y) = self.bounds();
        let width_0_1mm = max_x - min_x;
        let height_0_1mm = max_y - min_y;
        let width_mm = width_0_1mm / 10.0;
        let height_mm = height_0_1mm / 10.0;

        // Area in square centimeters
        let area_cm2 = (width_mm / 10.0) * (height_mm / 10.0);
        let density = if area_cm2 > 0.0 {
            stitch_count as f64 / area_cm2
        } else {
            0.0
        };

        // Average and max stitch lengths
        let avg_stitch_length_0_1mm = self.avg_stitch_length();
        let max_stitch_length_0_1mm = self.max_stitch_length();
        let avg_stitch_length_mm = avg_stitch_length_0_1mm / 10.0;
        let max_stitch_length_mm = max_stitch_length_0_1mm / 10.0;

        PatternStatistics {
            stitch_count,
            jump_count,
            trim_count,
            color_change_count,
            total_length_mm,
            total_length_inches,
            estimated_time_minutes,
            thread_usage,
            density,
            width_mm,
            height_mm,
            avg_stitch_length_mm,
            max_stitch_length_mm,
        }
    }

    /// Calculate thread usage statistics for each thread color
    ///
    /// Returns a vector of `ThreadUsage` with stitch count and length per thread.
    fn calculate_thread_usage(&self) -> Vec<ThreadUsage> {
        let mut usage_map: HashMap<usize, (usize, f64)> = HashMap::new();
        let mut current_thread_index = 0;
        let mut prev_x = 0.0;
        let mut prev_y = 0.0;

        for stitch in &self.stitches {
            let command = extract_command(stitch.command);

            // Track color changes
            if command == COLOR_CHANGE {
                current_thread_index += 1;
                prev_x = stitch.x;
                prev_y = stitch.y;
                continue;
            }

            // Only count actual stitches (not jumps, trims, etc.)
            if command == STITCH {
                let dx = stitch.x - prev_x;
                let dy = stitch.y - prev_y;
                let length = (dx * dx + dy * dy).sqrt();

                let entry = usage_map.entry(current_thread_index).or_insert((0, 0.0));
                entry.0 += 1; // stitch count
                entry.1 += length; // total length in 0.1mm
            }

            prev_x = stitch.x;
            prev_y = stitch.y;
        }

        // Convert to ThreadUsage vector
        let mut result = Vec::new();
        for (thread_idx, (count, length_0_1mm)) in usage_map {
            let thread = self
                .thread_list
                .get(thread_idx)
                .cloned()
                .unwrap_or_else(|| EmbThread::new(0x000000));

            result.push(ThreadUsage {
                thread,
                length_mm: length_0_1mm / 10.0,
                stitch_count: count,
            });
        }

        // Sort by thread index for consistent ordering
        result.sort_by_key(|usage| usage.thread.color);
        result
    }

    /// Add metadata
    pub fn add_metadata(&mut self, key: impl Into<String>, value: impl Into<String>) {
        self.extras.insert(key.into(), value.into());
    }

    /// Interpolate trims into the pattern
    ///
    /// This adds TRIM commands between long jumps
    pub fn interpolate_trims(
        &mut self,
        trim_at: usize,
        trim_distance: Option<f64>,
        _clipping: bool,
    ) {
        if self.stitches.is_empty() {
            return;
        }

        let mut new_stitches: Vec<Stitch> = Vec::new();
        let mut jump_count = 0;
        let mut last_was_jump = false;

        for &stitch in &self.stitches {
            let is_jump = stitch.command == JUMP;

            if is_jump {
                jump_count += 1;
                last_was_jump = true;

                // Check if we should add a trim after consecutive jumps
                if jump_count >= trim_at {
                    // Optionally check distance threshold
                    let should_trim = if let Some(dist) = trim_distance {
                        if let Some(last) = new_stitches.last() {
                            let dx = stitch.x - last.x;
                            let dy = stitch.y - last.y;
                            (dx * dx + dy * dy).sqrt() >= dist
                        } else {
                            true
                        }
                    } else {
                        true
                    };

                    if should_trim {
                        // Insert trim and reset jump counter
                        new_stitches.push(Stitch::new(stitch.x, stitch.y, TRIM));
                        jump_count = 0;
                        last_was_jump = false;
                        continue;
                    }
                }

                new_stitches.push(stitch);
            } else {
                // Reset jump counter on non-jump commands
                if last_was_jump && jump_count > 0 {
                    jump_count = 0;
                }
                last_was_jump = false;
                new_stitches.push(stitch);
            }
        }

        self.stitches = new_stitches;
    }

    /// Interpolate duplicate color changes as stops
    ///
    /// This converts consecutive color changes without stitches between them into STOP commands
    pub fn interpolate_duplicate_color_as_stop(&mut self) {
        if self.stitches.is_empty() {
            return;
        }

        let mut new_stitches: Vec<Stitch> = Vec::new();
        let mut last_was_color_change = false;

        for &stitch in &self.stitches {
            if stitch.command == COLOR_CHANGE {
                if last_was_color_change {
                    // Consecutive color changes: convert previous to STOP (for applique/manual operations)
                    if let Some(last) = new_stitches.last_mut() {
                        if last.command == COLOR_CHANGE {
                            last.command = STOP;
                        }
                    }
                }
                last_was_color_change = true;
                new_stitches.push(stitch);
            } else {
                last_was_color_change = false;
                new_stitches.push(stitch);
            }
        }

        self.stitches = new_stitches;
    }

    /// Read a pattern from file (stub - to be implemented with readers)
    pub fn read(_filename: &str) -> Result<Self> {
        Err(Error::Unsupported(
            "Reading not yet implemented".to_string(),
        ))
    }

    /// Write a pattern to file (stub - to be implemented with writers)
    pub fn write(&self, _filename: &str) -> Result<()> {
        Err(Error::Unsupported(
            "Writing not yet implemented".to_string(),
        ))
    }

    /// Get stitches grouped by color with their associated thread
    ///
    /// Returns an iterator of (stitch_block, thread) tuples where each block
    /// contains stitches of the same color
    pub fn get_as_stitchblock(&self) -> Vec<(Vec<(f64, f64)>, EmbThread)> {
        use crate::core::constants::*;

        let mut result = Vec::new();
        let mut current_block = Vec::new();
        let mut thread_index = 0;

        for stitch in &self.stitches {
            let flags = stitch.command & COMMAND_MASK;

            if flags == STITCH {
                current_block.push((stitch.x, stitch.y));
            } else {
                // Non-stitch command - yield current block if not empty
                if !current_block.is_empty() {
                    let thread = self.get_thread_or_filler(thread_index);
                    result.push((current_block.clone(), thread));
                    current_block.clear();
                }

                // Move to next thread on color change
                if flags == COLOR_CHANGE {
                    thread_index += 1;
                }
            }
        }

        // Don't forget the last block
        if !current_block.is_empty() {
            let thread = self.get_thread_or_filler(thread_index);
            result.push((current_block, thread));
        }

        result
    }

    /// Get thread or return a filler thread if index is out of bounds
    fn get_thread_or_filler(&self, index: usize) -> EmbThread {
        self.threads().get(index).cloned().unwrap_or_else(|| {
            // Generate a color based on index
            let r = ((index * 37) % 256) as u8;
            let g = ((index * 91) % 256) as u8;
            let b = ((index * 173) % 256) as u8;
            EmbThread::from_rgb(r, g, b)
        })
    }

    /// Validate pattern for DST format constraints
    ///
    /// DST format limitations:
    /// - Maximum 1,000,000 stitches
    /// - Stitch jumps limited to ±121 units per axis (±12.1mm)
    /// - Supports STITCH, JUMP, COLOR_CHANGE, END
    ///
    /// # Example
    ///
    /// ```
    /// use butabuti::core::pattern::EmbPattern;
    ///
    /// let pattern = EmbPattern::new();
    /// match pattern.validate_for_dst() {
    ///     Ok(_) => println!("Pattern valid for DST"),
    ///     Err(e) => println!("Validation failed: {}", e),
    /// }
    /// ```
    pub fn validate_for_dst(&self) -> Result<()> {
        const MAX_DST_STITCHES: usize = 1_000_000;
        const MAX_DST_JUMP: f64 = 121.0;

        if self.stitches.len() > MAX_DST_STITCHES {
            return Err(Error::Encoding(format!(
                "DST format supports max {} stitches, pattern has {}",
                MAX_DST_STITCHES,
                self.stitches.len()
            )));
        }

        // Check stitch jumps
        for i in 1..self.stitches.len() {
            let prev = &self.stitches[i - 1];
            let curr = &self.stitches[i];
            let dx = (curr.x - prev.x).abs();
            let dy = (curr.y - prev.y).abs();

            if dx > MAX_DST_JUMP || dy > MAX_DST_JUMP {
                return Err(Error::Encoding(format!(
                    "DST stitch jump too large at index {}: dx={:.1}, dy={:.1} (max {:.1})",
                    i, dx, dy, MAX_DST_JUMP
                )));
            }
        }

        Ok(())
    }

    /// Validate pattern for PES format constraints
    ///
    /// PES format limitations:
    /// - Embeds PEC data (inherits PEC constraints)
    /// - Maximum 1,000,000 stitches (practical limit)
    /// - Supports metadata fields
    ///
    /// # Example
    ///
    /// ```
    /// use butabuti::core::pattern::EmbPattern;
    ///
    /// let pattern = EmbPattern::new();
    /// pattern.validate_for_pes()?;
    /// # Ok::<(), butabuti::utils::error::Error>(())
    /// ```
    pub fn validate_for_pes(&self) -> Result<()> {
        const MAX_PES_STITCHES: usize = 1_000_000;

        if self.stitches.len() > MAX_PES_STITCHES {
            return Err(Error::Encoding(format!(
                "PES format supports max {} stitches, pattern has {}",
                MAX_PES_STITCHES,
                self.stitches.len()
            )));
        }

        Ok(())
    }

    /// Validate pattern for JEF format constraints
    ///
    /// JEF (Janome) format limitations:
    /// - Maximum 1,000 colors
    /// - Maximum 1,000,000 stitches
    ///
    /// # Example
    ///
    /// ```
    /// use butabuti::core::pattern::EmbPattern;
    ///
    /// let pattern = EmbPattern::new();
    /// pattern.validate_for_jef()?;
    /// # Ok::<(), butabuti::utils::error::Error>(())
    /// ```
    pub fn validate_for_jef(&self) -> Result<()> {
        const MAX_JEF_COLORS: usize = 1_000;
        const MAX_JEF_STITCHES: usize = 1_000_000;

        if self.thread_list.len() > MAX_JEF_COLORS {
            return Err(Error::Encoding(format!(
                "JEF format supports max {} colors, pattern has {}",
                MAX_JEF_COLORS,
                self.thread_list.len()
            )));
        }

        if self.stitches.len() > MAX_JEF_STITCHES {
            return Err(Error::Encoding(format!(
                "JEF format supports max {} stitches, pattern has {}",
                MAX_JEF_STITCHES,
                self.stitches.len()
            )));
        }

        Ok(())
    }

    /// Validate pattern has minimum required data
    ///
    /// Checks:
    /// - Pattern has at least one stitch
    /// - Pattern has at least one thread (will use default if missing)
    ///
    /// # Example
    ///
    /// ```
    /// use butabuti::core::pattern::EmbPattern;
    /// use butabuti::core::constants::STITCH;
    ///
    /// let mut pattern = EmbPattern::new();
    /// assert!(pattern.validate_basic().is_err()); // No stitches
    ///
    /// pattern.add_stitch_absolute(STITCH, 0.0, 0.0);
    /// assert!(pattern.validate_basic().is_ok()); // Has stitches
    /// ```
    pub fn validate_basic(&self) -> Result<()> {
        if self.stitches.is_empty() {
            return Err(Error::InvalidPattern(
                "Pattern must contain at least one stitch".to_string(),
            ));
        }

        Ok(())
    }

    /// Validate all stitches have valid coordinates
    ///
    /// Checks that all stitches have finite, non-NaN coordinates
    ///
    /// # Example
    ///
    /// ```
    /// use butabuti::core::pattern::{EmbPattern, Stitch};
    /// use butabuti::core::constants::STITCH;
    ///
    /// let mut pattern = EmbPattern::new();
    /// pattern.add_stitch_absolute(STITCH, 100.0, 200.0);
    /// assert!(pattern.validate_all_stitches().is_ok());
    /// ```
    pub fn validate_all_stitches(&self) -> Result<()> {
        for (i, stitch) in self.stitches.iter().enumerate() {
            if !stitch.is_valid() {
                return Err(Error::InvalidPattern(format!(
                    "Invalid stitch at index {}: ({}, {})",
                    i, stitch.x, stitch.y
                )));
            }
        }
        Ok(())
    }

    /// Comprehensive pattern validation
    ///
    /// Performs all basic validation checks:
    /// - Has at least one stitch
    /// - All stitches have valid coordinates
    /// - Pattern bounds are reasonable
    ///
    /// # Example
    ///
    /// ```
    /// use butabuti::core::pattern::EmbPattern;
    /// use butabuti::core::constants::STITCH;
    ///
    /// let mut pattern = EmbPattern::new();
    /// pattern.add_stitch_absolute(STITCH, 0.0, 0.0);
    /// assert!(pattern.validate().is_ok());
    /// ```
    pub fn validate(&self) -> Result<()> {
        // Check basic requirements
        self.validate_basic()?;

        // Validate all stitch coordinates
        self.validate_all_stitches()?;

        // Check bounds are reasonable (not too large)
        let (min_x, min_y, max_x, max_y) = self.bounds();
        const MAX_REASONABLE_COORD: f64 = 1_000_000.0; // 100 meters in 0.1mm units

        if min_x.abs() > MAX_REASONABLE_COORD
            || max_x.abs() > MAX_REASONABLE_COORD
            || min_y.abs() > MAX_REASONABLE_COORD
            || max_y.abs() > MAX_REASONABLE_COORD
        {
            return Err(Error::InvalidPattern(format!(
                "Pattern bounds exceed reasonable limits: ({:.1}, {:.1}) to ({:.1}, {:.1})",
                min_x, min_y, max_x, max_y
            )));
        }

        Ok(())
    }

    /// Validate pattern for EXP format constraints
    ///
    /// EXP (Melco) format limitations:
    /// - Maximum stitch delta: ±127 units (12.7mm)
    /// - Maximum 1,000,000 stitches
    ///
    /// # Example
    ///
    /// ```
    /// use butabuti::core::pattern::EmbPattern;
    ///
    /// let pattern = EmbPattern::new();
    /// match pattern.validate_for_exp() {
    ///     Ok(_) => println!("Valid for EXP"),
    ///     Err(e) => println!("Validation failed: {}", e),
    /// }
    /// # Ok::<(), butabuti::utils::error::Error>(())
    /// ```
    pub fn validate_for_exp(&self) -> Result<()> {
        const MAX_EXP_STITCHES: usize = 1_000_000;
        const MAX_EXP_DELTA: f64 = 127.0;

        if self.stitches.len() > MAX_EXP_STITCHES {
            return Err(Error::Encoding(format!(
                "EXP format supports max {} stitches, pattern has {}",
                MAX_EXP_STITCHES,
                self.stitches.len()
            )));
        }

        // Check stitch deltas
        for i in 1..self.stitches.len() {
            let prev = &self.stitches[i - 1];
            let curr = &self.stitches[i];
            let dx = (curr.x - prev.x).abs();
            let dy = (curr.y - prev.y).abs();

            if dx > MAX_EXP_DELTA || dy > MAX_EXP_DELTA {
                return Err(Error::Encoding(format!(
                    "EXP format stitch delta exceeds ±{} at index {}: ({:.1}, {:.1})",
                    MAX_EXP_DELTA, i, dx, dy
                )));
            }
        }

        Ok(())
    }

    /// Validate pattern for VP3 format constraints
    ///
    /// VP3 (Pfaff) format limitations:
    /// - Maximum stitch delta: ±127 units (12.7mm)
    /// - Maximum 1,000,000 stitches
    ///
    /// # Example
    ///
    /// ```
    /// use butabuti::core::pattern::EmbPattern;
    ///
    /// let pattern = EmbPattern::new();
    /// pattern.validate_for_vp3()?;
    /// # Ok::<(), butabuti::utils::error::Error>(())
    /// ```
    pub fn validate_for_vp3(&self) -> Result<()> {
        const MAX_VP3_STITCHES: usize = 1_000_000;
        const MAX_VP3_DELTA: f64 = 127.0;

        if self.stitches.len() > MAX_VP3_STITCHES {
            return Err(Error::Encoding(format!(
                "VP3 format supports max {} stitches, pattern has {}",
                MAX_VP3_STITCHES,
                self.stitches.len()
            )));
        }

        // Check stitch deltas
        for i in 1..self.stitches.len() {
            let prev = &self.stitches[i - 1];
            let curr = &self.stitches[i];
            let dx = (curr.x - prev.x).abs();
            let dy = (curr.y - prev.y).abs();

            if dx > MAX_VP3_DELTA || dy > MAX_VP3_DELTA {
                return Err(Error::Encoding(format!(
                    "VP3 format stitch delta exceeds ±{} at index {}: ({:.1}, {:.1})",
                    MAX_VP3_DELTA, i, dx, dy
                )));
            }
        }

        Ok(())
    }

    /// Validate pattern for XXX format constraints
    ///
    /// XXX (Singer) format limitations:
    /// - Maximum stitch delta: ±127 units (12.7mm)
    /// - Maximum 100,000 stitches
    ///
    /// # Example
    ///
    /// ```
    /// use butabuti::core::pattern::EmbPattern;
    ///
    /// let pattern = EmbPattern::new();
    /// pattern.validate_for_xxx()?;
    /// # Ok::<(), butabuti::utils::error::Error>(())
    /// ```
    pub fn validate_for_xxx(&self) -> Result<()> {
        const MAX_XXX_STITCHES: usize = 100_000;
        const MAX_XXX_DELTA: f64 = 127.0;

        if self.stitches.len() > MAX_XXX_STITCHES {
            return Err(Error::Encoding(format!(
                "XXX format supports max {} stitches, pattern has {}",
                MAX_XXX_STITCHES,
                self.stitches.len()
            )));
        }

        // Check stitch deltas
        for i in 1..self.stitches.len() {
            let prev = &self.stitches[i - 1];
            let curr = &self.stitches[i];
            let dx = (curr.x - prev.x).abs();
            let dy = (curr.y - prev.y).abs();

            if dx > MAX_XXX_DELTA || dy > MAX_XXX_DELTA {
                return Err(Error::Encoding(format!(
                    "XXX format stitch delta exceeds ±{} at index {}: ({:.1}, {:.1})",
                    MAX_XXX_DELTA, i, dx, dy
                )));
            }
        }

        Ok(())
    }

    /// Validate pattern for U01 format constraints
    ///
    /// U01 (Barudan) format limitations:
    /// - Maximum stitch delta: ±127 units (12.7mm)
    /// - Maximum 1,000,000 stitches
    ///
    /// # Example
    ///
    /// ```
    /// use butabuti::core::pattern::EmbPattern;
    ///
    /// let pattern = EmbPattern::new();
    /// pattern.validate_for_u01()?;
    /// # Ok::<(), butabuti::utils::error::Error>(())
    /// ```
    pub fn validate_for_u01(&self) -> Result<()> {
        const MAX_U01_STITCHES: usize = 1_000_000;
        const MAX_U01_DELTA: f64 = 127.0;

        if self.stitches.len() > MAX_U01_STITCHES {
            return Err(Error::Encoding(format!(
                "U01 format supports max {} stitches, pattern has {}",
                MAX_U01_STITCHES,
                self.stitches.len()
            )));
        }

        // Check stitch deltas
        for i in 1..self.stitches.len() {
            let prev = &self.stitches[i - 1];
            let curr = &self.stitches[i];
            let dx = (curr.x - prev.x).abs();
            let dy = (curr.y - prev.y).abs();

            if dx > MAX_U01_DELTA || dy > MAX_U01_DELTA {
                return Err(Error::Encoding(format!(
                    "U01 format stitch delta exceeds ±{} at index {}: ({:.1}, {:.1})",
                    MAX_U01_DELTA, i, dx, dy
                )));
            }
        }

        Ok(())
    }

    // ============================================================================
    // Color Group Management
    // ============================================================================

    /// Get reference to color grouping
    ///
    /// # Example
    ///
    /// ```
    /// use butabuti::core::pattern::EmbPattern;
    ///
    /// let pattern = EmbPattern::new();
    /// assert!(pattern.color_grouping().is_none());
    /// ```
    pub fn color_grouping(&self) -> Option<&crate::core::color_group::ThreadGrouping> {
        self.color_grouping.as_ref()
    }

    /// Get mutable reference to color grouping
    pub fn color_grouping_mut(&mut self) -> Option<&mut crate::core::color_group::ThreadGrouping> {
        self.color_grouping.as_mut()
    }

    /// Initialize color grouping with a default group
    ///
    /// If grouping already exists, this does nothing.
    ///
    /// # Example
    ///
    /// ```
    /// use butabuti::core::pattern::EmbPattern;
    ///
    /// let mut pattern = EmbPattern::new();
    /// pattern.init_color_grouping(Some("Ungrouped"));
    /// assert!(pattern.color_grouping().is_some());
    /// ```
    pub fn init_color_grouping(&mut self, default_group_name: Option<&str>) {
        if self.color_grouping.is_none() {
            self.color_grouping = Some(if let Some(name) = default_group_name {
                crate::core::color_group::ThreadGrouping::with_default_group(name)
            } else {
                crate::core::color_group::ThreadGrouping::new()
            });
        }
    }

    /// Set color grouping (replaces existing)
    ///
    /// # Example
    ///
    /// ```
    /// use butabuti::core::pattern::EmbPattern;
    /// use butabuti::core::color_group::ThreadGrouping;
    ///
    /// let mut pattern = EmbPattern::new();
    /// let grouping = ThreadGrouping::with_default_group("Default");
    /// pattern.set_color_grouping(Some(grouping));
    /// assert!(pattern.color_grouping().is_some());
    /// ```
    pub fn set_color_grouping(
        &mut self,
        grouping: Option<crate::core::color_group::ThreadGrouping>,
    ) {
        self.color_grouping = grouping;
    }

    /// Add a color group to the pattern
    ///
    /// Initializes grouping if not already present.
    ///
    /// # Example
    ///
    /// ```
    /// use butabuti::core::pattern::EmbPattern;
    /// use butabuti::core::color_group::ColorGroup;
    ///
    /// let mut pattern = EmbPattern::new();
    /// let group = ColorGroup::new("Foliage");
    /// pattern.add_color_group(group);
    ///
    /// assert!(pattern.color_grouping().unwrap().has_group("Foliage"));
    /// ```
    pub fn add_color_group(&mut self, group: crate::core::color_group::ColorGroup) {
        self.init_color_grouping(None);
        if let Some(grouping) = &mut self.color_grouping {
            grouping.add_group(group);
        }
    }

    /// Remove a color group by name
    pub fn remove_color_group(
        &mut self,
        name: &str,
    ) -> Option<crate::core::color_group::ColorGroup> {
        self.color_grouping
            .as_mut()
            .and_then(|g| g.remove_group(name))
    }

    /// Get a color group by name
    pub fn get_color_group(&self, name: &str) -> Option<&crate::core::color_group::ColorGroup> {
        self.color_grouping.as_ref().and_then(|g| g.get_group(name))
    }

    /// Get a mutable reference to a color group
    pub fn get_color_group_mut(
        &mut self,
        name: &str,
    ) -> Option<&mut crate::core::color_group::ColorGroup> {
        self.color_grouping
            .as_mut()
            .and_then(|g| g.get_group_mut(name))
    }

    /// Add a thread to a color group
    ///
    /// # Returns
    ///
    /// `Ok(true)` if added, `Ok(false)` if already in group, `Err` if group doesn't exist
    ///
    /// # Example
    ///
    /// ```
    /// use butabuti::core::pattern::EmbPattern;
    /// use butabuti::core::color_group::ColorGroup;
    /// use butabuti::core::thread::EmbThread;
    ///
    /// let mut pattern = EmbPattern::new();
    /// pattern.add_thread(EmbThread::from_string("red").unwrap());
    /// pattern.add_color_group(ColorGroup::new("Reds"));
    ///
    /// let result = pattern.add_thread_to_group("Reds", 0);
    /// assert!(result.is_ok());
    /// ```
    pub fn add_thread_to_group(&mut self, group_name: &str, thread_index: usize) -> Result<bool> {
        if thread_index >= self.thread_list.len() {
            return Err(Error::InvalidPattern(format!(
                "Thread index {} out of bounds (pattern has {} threads)",
                thread_index,
                self.thread_list.len()
            )));
        }

        self.init_color_grouping(None);
        self.color_grouping
            .as_mut()
            .ok_or_else(|| Error::InvalidPattern("Color grouping not initialized".to_string()))?
            .add_thread_to_group(group_name, thread_index)
            .map_err(Error::InvalidPattern)
    }

    /// Remove a thread from a color group
    pub fn remove_thread_from_group(
        &mut self,
        group_name: &str,
        thread_index: usize,
    ) -> Result<bool> {
        self.color_grouping
            .as_mut()
            .ok_or_else(|| Error::InvalidPattern("Color grouping not initialized".to_string()))?
            .remove_thread_from_group(group_name, thread_index)
            .map_err(Error::InvalidPattern)
    }

    /// Get all threads in a specific color group
    ///
    /// Returns a vector of (thread_index, thread_reference) pairs
    ///
    /// # Example
    ///
    /// ```
    /// use butabuti::core::pattern::EmbPattern;
    /// use butabuti::core::color_group::ColorGroup;
    /// use butabuti::core::thread::EmbThread;
    ///
    /// let mut pattern = EmbPattern::new();
    /// pattern.add_thread(EmbThread::from_string("red").unwrap());
    /// pattern.add_thread(EmbThread::from_string("darkred").unwrap());
    ///
    /// let mut group = ColorGroup::new("Reds");
    /// group.add_thread(0);
    /// group.add_thread(1);
    /// pattern.add_color_group(group);
    ///
    /// let threads = pattern.get_threads_by_group("Reds").unwrap();
    /// assert_eq!(threads.len(), 2);
    /// ```
    pub fn get_threads_by_group(&self, group_name: &str) -> Option<Vec<(usize, &EmbThread)>> {
        let group = self.color_grouping.as_ref()?.get_group(group_name)?;

        Some(
            group
                .thread_indices_sorted()
                .iter()
                .filter_map(|&idx| self.thread_list.get(idx).map(|thread| (idx, thread)))
                .collect(),
        )
    }

    /// Find all groups containing a specific thread
    pub fn find_groups_for_thread(&self, thread_index: usize) -> Vec<String> {
        self.color_grouping
            .as_ref()
            .map(|g| {
                g.find_group_names_with_thread(thread_index)
                    .into_iter()
                    .map(|s| s.to_string())
                    .collect()
            })
            .unwrap_or_default()
    }

    /// Assign ungrouped threads to default group
    ///
    /// Returns the number of threads assigned, or error if no default group configured
    pub fn assign_ungrouped_to_default(&mut self) -> Result<usize> {
        self.color_grouping
            .as_mut()
            .ok_or_else(|| Error::InvalidPattern("Color grouping not initialized".to_string()))?
            .assign_to_default_group(self.thread_list.len())
            .map_err(Error::InvalidPattern)
    }

    /// Auto-create color groups based on color similarity
    ///
    /// Groups threads with similar colors together using delta-E color distance.
    ///
    /// # Arguments
    ///
    /// * `threshold` - Maximum delta-E distance for grouping (typical: 10-30)
    /// * `group_prefix` - Prefix for generated group names (e.g., "Group")
    ///
    /// # Example
    ///
    /// ```
    /// use butabuti::core::pattern::EmbPattern;
    /// use butabuti::core::thread::EmbThread;
    ///
    /// let mut pattern = EmbPattern::new();
    /// pattern.add_thread(EmbThread::from_string("red").unwrap());
    /// pattern.add_thread(EmbThread::from_string("darkred").unwrap());
    /// pattern.add_thread(EmbThread::from_string("blue").unwrap());
    ///
    /// pattern.auto_group_by_color_similarity(20.0, "ColorGroup");
    ///
    /// // Should create groups for similar colors
    /// assert!(pattern.color_grouping().is_some());
    /// ```
    pub fn auto_group_by_color_similarity(&mut self, threshold: f64, group_prefix: &str) {
        if self.thread_list.is_empty() {
            return;
        }

        self.init_color_grouping(None);

        // Track which threads have been grouped
        let mut grouped = vec![false; self.thread_list.len()];
        let mut group_counter = 1;

        for i in 0..self.thread_list.len() {
            if grouped[i] {
                continue;
            }

            // Create a new group starting with this thread
            let group_name = format!("{} {}", group_prefix, group_counter);
            let mut group = crate::core::color_group::ColorGroup::new(&group_name);
            group.add_thread(i);
            grouped[i] = true;

            // Find similar threads
            #[allow(clippy::needless_range_loop)]
            for j in (i + 1)..self.thread_list.len() {
                if grouped[j] {
                    continue;
                }

                let distance = self.thread_list[i].delta_e(&self.thread_list[j]);
                if distance <= threshold as f32 {
                    group.add_thread(j);
                    grouped[j] = true;
                }
            }

            self.add_color_group(group);
            group_counter += 1;
        }
    }

    /// Clear all color groups
    pub fn clear_color_groups(&mut self) {
        if let Some(grouping) = &mut self.color_grouping {
            grouping.clear();
        }
    }

    /// Validate color grouping structure
    ///
    /// Returns a list of validation errors
    pub fn validate_color_grouping(&self) -> Vec<String> {
        self.color_grouping
            .as_ref()
            .map(|g| g.validate())
            .unwrap_or_default()
    }
}

impl Default for EmbPattern {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_pattern() {
        let pattern = EmbPattern::new();
        assert_eq!(pattern.stitches().len(), 0);
        assert_eq!(pattern.threads().len(), 0);
    }

    #[test]
    fn test_add_stitch_absolute() {
        let mut pattern = EmbPattern::new();
        pattern.add_stitch_absolute(STITCH, 100.0, 200.0);

        assert_eq!(pattern.stitches().len(), 1);
        assert_eq!(pattern.stitches()[0].x, 100.0);
        assert_eq!(pattern.stitches()[0].y, 200.0);
        assert_eq!(pattern.stitches()[0].command, STITCH);
    }

    #[test]
    fn test_add_stitch_relative() {
        let mut pattern = EmbPattern::new();
        pattern.add_stitch_absolute(STITCH, 100.0, 200.0);
        pattern.add_stitch_relative(50.0, 30.0, STITCH);

        assert_eq!(pattern.stitches().len(), 2);
        assert_eq!(pattern.stitches()[1].x, 150.0);
        assert_eq!(pattern.stitches()[1].y, 230.0);
    }

    #[test]
    fn test_bounds() {
        let mut pattern = EmbPattern::new();
        pattern.add_stitch_absolute(STITCH, 0.0, 0.0);
        pattern.add_stitch_absolute(STITCH, 100.0, 200.0);
        pattern.add_stitch_absolute(STITCH, -50.0, 50.0);

        let (min_x, min_y, max_x, max_y) = pattern.bounds();
        assert_eq!(min_x, -50.0);
        assert_eq!(min_y, 0.0);
        assert_eq!(max_x, 100.0);
        assert_eq!(max_y, 200.0);
    }

    #[test]
    fn test_translate() {
        let mut pattern = EmbPattern::new();
        pattern.add_stitch_absolute(STITCH, 100.0, 200.0);
        pattern.translate(50.0, -30.0);

        assert_eq!(pattern.stitches()[0].x, 150.0);
        assert_eq!(pattern.stitches()[0].y, 170.0);
    }

    #[test]
    fn test_convenience_methods() {
        let mut pattern = EmbPattern::new();
        pattern.stitch(10.0, 20.0);
        pattern.jump(5.0, 5.0);
        pattern.trim();
        pattern.color_change(0.0, 0.0);

        assert_eq!(pattern.stitches().len(), 4);
        assert_eq!(pattern.stitches()[0].command, STITCH);
        assert_eq!(pattern.stitches()[1].command, JUMP);
        assert_eq!(pattern.stitches()[2].command, TRIM);
        assert_eq!(pattern.stitches()[3].command, COLOR_CHANGE);
    }

    #[test]
    fn test_validate_basic() {
        let pattern = EmbPattern::new();
        assert!(
            pattern.validate_basic().is_err(),
            "Empty pattern should fail"
        );

        let mut pattern = EmbPattern::new();
        pattern.add_stitch_absolute(STITCH, 0.0, 0.0);
        assert!(
            pattern.validate_basic().is_ok(),
            "Pattern with stitches should pass"
        );
    }

    #[test]
    fn test_validate_for_dst() {
        let mut pattern = EmbPattern::new();
        pattern.add_stitch_absolute(STITCH, 0.0, 0.0);
        assert!(pattern.validate_for_dst().is_ok());

        // Test jump too large
        let mut pattern = EmbPattern::new();
        pattern.add_stitch_absolute(STITCH, 0.0, 0.0);
        pattern.add_stitch_absolute(STITCH, 150.0, 0.0); // 150 > 121 max
        assert!(
            pattern.validate_for_dst().is_err(),
            "Large jump should fail"
        );
    }

    #[test]
    fn test_validate_for_jef() {
        let mut pattern = EmbPattern::new();
        pattern.add_stitch_absolute(STITCH, 0.0, 0.0);
        assert!(pattern.validate_for_jef().is_ok());

        // Add many threads
        for _ in 0..1001 {
            pattern.add_thread(EmbThread::new(0xFF0000));
        }
        assert!(
            pattern.validate_for_jef().is_err(),
            "Too many colors should fail"
        );
    }

    #[test]
    fn test_validate_for_pes() {
        let mut pattern = EmbPattern::new();
        pattern.add_stitch_absolute(STITCH, 0.0, 0.0);
        assert!(pattern.validate_for_pes().is_ok());
    }

    #[test]
    fn test_validate_all_stitches() {
        let mut pattern = EmbPattern::new();
        pattern.add_stitch_absolute(STITCH, 100.0, 200.0);
        assert!(pattern.validate_all_stitches().is_ok());

        // Add invalid stitch
        pattern.stitches.push(Stitch::new(f64::NAN, 100.0, STITCH));
        assert!(pattern.validate_all_stitches().is_err());
    }

    #[test]
    fn test_validate() {
        let mut pattern = EmbPattern::new();
        assert!(pattern.validate().is_err(), "Empty pattern should fail");

        pattern.add_stitch_absolute(STITCH, 100.0, 200.0);
        assert!(pattern.validate().is_ok());

        // Add out-of-bounds stitch
        pattern.add_stitch_absolute(STITCH, 2_000_000.0, 0.0);
        assert!(pattern.validate().is_err(), "Excessive bounds should fail");
    }

    #[test]
    fn test_validate_for_exp() {
        let mut pattern = EmbPattern::new();
        pattern.add_stitch_absolute(STITCH, 0.0, 0.0);
        pattern.add_stitch_absolute(STITCH, 100.0, 100.0);
        assert!(pattern.validate_for_exp().is_ok());

        // Add stitch with large delta
        pattern.add_stitch_absolute(STITCH, 500.0, 500.0);
        assert!(
            pattern.validate_for_exp().is_err(),
            "Large delta should fail"
        );
    }

    #[test]
    fn test_validate_for_vp3() {
        let mut pattern = EmbPattern::new();
        pattern.add_stitch_absolute(STITCH, 0.0, 0.0);
        pattern.add_stitch_absolute(STITCH, 120.0, 120.0);
        assert!(pattern.validate_for_vp3().is_ok());

        // Add stitch with excessive delta
        let mut pattern2 = EmbPattern::new();
        pattern2.add_stitch_absolute(STITCH, 0.0, 0.0);
        pattern2.add_stitch_absolute(STITCH, 200.0, 0.0);
        assert!(pattern2.validate_for_vp3().is_err());
    }

    #[test]
    fn test_validate_for_xxx() {
        let mut pattern = EmbPattern::new();
        pattern.add_stitch_absolute(STITCH, 0.0, 0.0);
        pattern.add_stitch_absolute(STITCH, 100.0, 100.0);
        assert!(pattern.validate_for_xxx().is_ok());

        // Test stitch count limit
        let mut large_pattern = EmbPattern::new();
        for i in 0..100_001 {
            large_pattern.add_stitch_absolute(STITCH, i as f64, 0.0);
        }
        assert!(
            large_pattern.validate_for_xxx().is_err(),
            "Too many stitches should fail"
        );
    }

    #[test]
    fn test_validate_for_u01() {
        let mut pattern = EmbPattern::new();
        pattern.add_stitch_absolute(STITCH, 0.0, 0.0);
        pattern.add_stitch_absolute(STITCH, 125.0, 125.0);
        assert!(pattern.validate_for_u01().is_ok());

        // Test delta limit
        let mut pattern2 = EmbPattern::new();
        pattern2.add_stitch_absolute(STITCH, 0.0, 0.0);
        pattern2.add_stitch_absolute(STITCH, 150.0, 150.0);
        assert!(
            pattern2.validate_for_u01().is_err(),
            "Large delta should fail"
        );
    }

    // === Stitch Method Tests ===

    #[test]
    fn test_stitch_relative_to() {
        let stitch1 = Stitch::new(100.0, 200.0, STITCH);
        let stitch2 = Stitch::new(50.0, 80.0, STITCH);

        let (dx, dy) = stitch1.relative_to(&stitch2);
        assert_eq!(dx, 50.0);
        assert_eq!(dy, 120.0);
    }

    #[test]
    fn test_stitch_relative_to_negative() {
        let stitch1 = Stitch::new(50.0, 80.0, STITCH);
        let stitch2 = Stitch::new(100.0, 200.0, STITCH);

        let (dx, dy) = stitch1.relative_to(&stitch2);
        assert_eq!(dx, -50.0);
        assert_eq!(dy, -120.0);
    }

    #[test]
    fn test_stitch_relative_to_zero() {
        let stitch1 = Stitch::new(100.0, 200.0, STITCH);
        let stitch2 = Stitch::new(100.0, 200.0, STITCH);

        let (dx, dy) = stitch1.relative_to(&stitch2);
        assert_eq!(dx, 0.0);
        assert_eq!(dy, 0.0);
    }

    #[test]
    fn test_stitch_distance_to() {
        // 3-4-5 triangle
        let stitch1 = Stitch::new(0.0, 0.0, STITCH);
        let stitch2 = Stitch::new(30.0, 40.0, STITCH);

        let distance = stitch1.distance_to(&stitch2);
        assert_eq!(distance, 50.0);
    }

    #[test]
    fn test_stitch_distance_to_same_point() {
        let stitch1 = Stitch::new(100.0, 200.0, STITCH);
        let stitch2 = Stitch::new(100.0, 200.0, STITCH);

        let distance = stitch1.distance_to(&stitch2);
        assert_eq!(distance, 0.0);
    }

    #[test]
    fn test_stitch_distance_symmetric() {
        let stitch1 = Stitch::new(0.0, 0.0, STITCH);
        let stitch2 = Stitch::new(30.0, 40.0, STITCH);

        // Distance should be symmetric
        assert_eq!(stitch1.distance_to(&stitch2), stitch2.distance_to(&stitch1));
    }

    #[test]
    fn test_stitch_distance_large_values() {
        let stitch1 = Stitch::new(0.0, 0.0, STITCH);
        let stitch2 = Stitch::new(1000.0, 1000.0, STITCH);

        let distance = stitch1.distance_to(&stitch2);
        let expected = (1000.0_f64 * 1000.0 * 2.0).sqrt();
        assert!((distance - expected).abs() < 0.0001);
    }

    #[test]
    fn test_stitch_is_valid() {
        let valid = Stitch::new(100.0, 200.0, STITCH);
        assert!(valid.is_valid());
    }

    #[test]
    fn test_stitch_is_valid_zero() {
        let valid = Stitch::new(0.0, 0.0, STITCH);
        assert!(valid.is_valid());
    }

    #[test]
    fn test_stitch_is_valid_negative() {
        let valid = Stitch::new(-100.0, -200.0, STITCH);
        assert!(valid.is_valid());
    }

    #[test]
    fn test_stitch_invalid_nan_x() {
        let invalid = Stitch::new(f64::NAN, 200.0, STITCH);
        assert!(!invalid.is_valid());
    }

    #[test]
    fn test_stitch_invalid_nan_y() {
        let invalid = Stitch::new(100.0, f64::NAN, STITCH);
        assert!(!invalid.is_valid());
    }

    #[test]
    fn test_stitch_invalid_both_nan() {
        let invalid = Stitch::new(f64::NAN, f64::NAN, STITCH);
        assert!(!invalid.is_valid());
    }

    #[test]
    fn test_stitch_invalid_infinity_x() {
        let invalid = Stitch::new(f64::INFINITY, 200.0, STITCH);
        assert!(!invalid.is_valid());
    }

    #[test]
    fn test_stitch_invalid_infinity_y() {
        let invalid = Stitch::new(100.0, f64::INFINITY, STITCH);
        assert!(!invalid.is_valid());
    }

    #[test]
    fn test_stitch_invalid_neg_infinity() {
        let invalid = Stitch::new(f64::NEG_INFINITY, 200.0, STITCH);
        assert!(!invalid.is_valid());
    }

    #[test]
    fn test_stitch_zero() {
        let zero = Stitch::zero();
        assert_eq!(zero.x, 0.0);
        assert_eq!(zero.y, 0.0);
        assert_eq!(zero.command, STITCH);
        assert!(zero.is_valid());
    }

    #[test]
    fn test_stitch_zero_is_const() {
        // Test that zero() is const and can be used in const contexts
        const ZERO_STITCH: Stitch = Stitch::zero();
        assert_eq!(ZERO_STITCH.x, 0.0);
    }

    // Pattern statistics tests
    #[test]
    fn test_total_stitch_length_empty() {
        let pattern = EmbPattern::new();
        assert_eq!(pattern.total_stitch_length(), 0.0);
    }

    #[test]
    fn test_total_stitch_length_single() {
        let mut pattern = EmbPattern::new();
        pattern.stitch(10.0, 0.0);
        assert_eq!(pattern.total_stitch_length(), 10.0);
    }

    #[test]
    fn test_total_stitch_length_multiple() {
        let mut pattern = EmbPattern::new();
        pattern.stitch(30.0, 40.0); // 3-4-5 triangle = 50.0
        pattern.stitch(30.0, -40.0); // Another 50.0
        assert_eq!(pattern.total_stitch_length(), 100.0);
    }

    #[test]
    fn test_total_stitch_length_ignores_jumps() {
        let mut pattern = EmbPattern::new();
        pattern.stitch(10.0, 0.0); // Count this (10.0)
        pattern.jump(100.0, 0.0); // Don't count jumps
        pattern.stitch(10.0, 0.0); // Count this (10.0)
        assert_eq!(pattern.total_stitch_length(), 20.0);
    }

    #[test]
    fn test_max_stitch_length_empty() {
        let pattern = EmbPattern::new();
        assert_eq!(pattern.max_stitch_length(), 0.0);
    }

    #[test]
    fn test_max_stitch_length_single() {
        let mut pattern = EmbPattern::new();
        pattern.stitch(25.0, 0.0);
        assert_eq!(pattern.max_stitch_length(), 25.0);
    }

    #[test]
    fn test_max_stitch_length_multiple() {
        let mut pattern = EmbPattern::new();
        pattern.stitch(10.0, 0.0); // 10.0
        pattern.stitch(50.0, 0.0); // 50.0 (max)
        pattern.stitch(20.0, 0.0); // 20.0
        assert_eq!(pattern.max_stitch_length(), 50.0);
    }

    #[test]
    fn test_avg_stitch_length_empty() {
        let pattern = EmbPattern::new();
        assert_eq!(pattern.avg_stitch_length(), 0.0);
    }

    #[test]
    fn test_avg_stitch_length_single() {
        let mut pattern = EmbPattern::new();
        pattern.stitch(20.0, 0.0);
        assert_eq!(pattern.avg_stitch_length(), 20.0);
    }

    #[test]
    fn test_avg_stitch_length_multiple() {
        let mut pattern = EmbPattern::new();
        pattern.stitch(10.0, 0.0); // 10.0
        pattern.stitch(20.0, 0.0); // 20.0
        pattern.stitch(30.0, 0.0); // 30.0
                                   // Average: (10 + 20 + 30) / 3 = 20.0
        assert_eq!(pattern.avg_stitch_length(), 20.0);
    }

    #[test]
    fn test_count_jumps() {
        let mut pattern = EmbPattern::new();
        pattern.stitch(10.0, 0.0);
        pattern.jump(50.0, 0.0);
        pattern.jump(30.0, 0.0);
        pattern.stitch(10.0, 0.0);
        assert_eq!(pattern.count_jumps(), 2);
    }

    #[test]
    fn test_count_trims() {
        let mut pattern = EmbPattern::new();
        pattern.stitch(10.0, 0.0);
        pattern.trim();
        pattern.stitch(10.0, 0.0);
        pattern.trim();
        pattern.trim();
        assert_eq!(pattern.count_trims(), 3);
    }

    #[test]
    fn test_cut_command() {
        let mut pattern = EmbPattern::new();
        pattern.add_thread(EmbThread::from_string("red").unwrap());
        pattern.stitch(10.0, 0.0);
        pattern.cut();
        pattern.stitch(10.0, 0.0);

        assert_eq!(pattern.stitches().len(), 3);
        let cut_stitch = &pattern.stitches()[1];
        assert_eq!(cut_stitch.command & COMMAND_MASK, CUT);
        assert_eq!(
            cut_stitch.stitch_type(),
            crate::core::constants::StitchType::Cut
        );
    }

    #[test]
    fn test_cut_vs_trim() {
        let mut pattern = EmbPattern::new();
        pattern.add_thread(EmbThread::from_string("red").unwrap());
        pattern.trim();
        pattern.cut();

        assert_eq!(pattern.stitches().len(), 2);

        let trim_stitch = &pattern.stitches()[0];
        let cut_stitch = &pattern.stitches()[1];

        assert_eq!(trim_stitch.command & COMMAND_MASK, TRIM);
        assert_eq!(cut_stitch.command & COMMAND_MASK, CUT);
        assert_ne!(trim_stitch.command, cut_stitch.command);

        // Both are thread commands
        assert!(trim_stitch.stitch_type().is_thread_command());
        assert!(cut_stitch.stitch_type().is_thread_command());
    }

    #[test]
    fn test_width_empty() {
        let pattern = EmbPattern::new();
        assert_eq!(pattern.width(), 0.0);
    }

    #[test]
    fn test_width_single_stitch() {
        let mut pattern = EmbPattern::new();
        pattern.stitch_abs(100.0, 50.0);
        assert_eq!(pattern.width(), 0.0);
    }

    #[test]
    fn test_width_multiple_stitches() {
        let mut pattern = EmbPattern::new();
        pattern.stitch_abs(10.0, 0.0);
        pattern.stitch_abs(100.0, 0.0);
        pattern.stitch_abs(50.0, 0.0);
        assert_eq!(pattern.width(), 90.0); // 100 - 10
    }

    #[test]
    fn test_height_empty() {
        let pattern = EmbPattern::new();
        assert_eq!(pattern.height(), 0.0);
    }

    #[test]
    fn test_height_single_stitch() {
        let mut pattern = EmbPattern::new();
        pattern.stitch_abs(50.0, 100.0);
        assert_eq!(pattern.height(), 0.0);
    }

    #[test]
    fn test_height_multiple_stitches() {
        let mut pattern = EmbPattern::new();
        pattern.stitch_abs(0.0, 20.0);
        pattern.stitch_abs(0.0, 150.0);
        pattern.stitch_abs(0.0, 75.0);
        assert_eq!(pattern.height(), 130.0); // 150 - 20
    }

    #[test]
    fn test_pattern_dimensions() {
        let mut pattern = EmbPattern::new();
        pattern.stitch_abs(10.0, 20.0);
        pattern.stitch_abs(100.0, 120.0);
        assert_eq!(pattern.width(), 90.0); // 100 - 10
        assert_eq!(pattern.height(), 100.0); // 120 - 20
    }

    // Pattern transformation tests
    #[test]
    fn test_rotate_0_degrees() {
        let mut pattern = EmbPattern::new();
        pattern.stitch_abs(100.0, 50.0);
        pattern.rotate(0.0);
        assert_eq!(pattern.stitches[0].x, 100.0);
        assert_eq!(pattern.stitches[0].y, 50.0);
    }

    #[test]
    fn test_rotate_90_degrees() {
        let mut pattern = EmbPattern::new();
        pattern.stitch_abs(100.0, 0.0);
        pattern.rotate(90.0);
        // After 90° rotation: (100, 0) -> (0, 100)
        assert!((pattern.stitches[0].x - 0.0).abs() < 0.01);
        assert!((pattern.stitches[0].y - 100.0).abs() < 0.01);
    }

    #[test]
    fn test_rotate_180_degrees() {
        let mut pattern = EmbPattern::new();
        pattern.stitch_abs(100.0, 50.0);
        pattern.rotate(180.0);
        // After 180° rotation: (100, 50) -> (-100, -50)
        assert!((pattern.stitches[0].x + 100.0).abs() < 0.01);
        assert!((pattern.stitches[0].y + 50.0).abs() < 0.01);
    }

    #[test]
    fn test_rotate_270_degrees() {
        let mut pattern = EmbPattern::new();
        pattern.stitch_abs(100.0, 0.0);
        pattern.rotate(270.0);
        // After 270° rotation: (100, 0) -> (0, -100)
        assert!((pattern.stitches[0].x - 0.0).abs() < 0.01);
        assert!((pattern.stitches[0].y + 100.0).abs() < 0.01);
    }

    #[test]
    fn test_rotate_360_degrees() {
        let mut pattern = EmbPattern::new();
        pattern.stitch_abs(100.0, 50.0);
        pattern.rotate(360.0);
        // After 360° rotation: back to original
        assert!((pattern.stitches[0].x - 100.0).abs() < 0.01);
        assert!((pattern.stitches[0].y - 50.0).abs() < 0.01);
    }

    #[test]
    fn test_rotate_45_degrees() {
        let mut pattern = EmbPattern::new();
        pattern.stitch_abs(100.0, 0.0);
        pattern.rotate(45.0);
        // After 45° rotation: (100, 0) -> (70.71, 70.71)
        let expected = 100.0 / 2.0_f64.sqrt();
        assert!((pattern.stitches[0].x - expected).abs() < 0.01);
        assert!((pattern.stitches[0].y - expected).abs() < 0.01);
    }

    #[test]
    fn test_rotate_around_point() {
        let mut pattern = EmbPattern::new();
        pattern.stitch_abs(150.0, 100.0); // 50 units right of center (100, 100)
        pattern.rotate_around_point(90.0, 100.0, 100.0);
        // Point should be 50 units above center
        assert!((pattern.stitches[0].x - 100.0).abs() < 0.01);
        assert!((pattern.stitches[0].y - 150.0).abs() < 0.01);
    }

    #[test]
    fn test_rotate_preserves_stitch_count() {
        let mut pattern = EmbPattern::new();
        pattern.stitch(10.0, 20.0);
        pattern.stitch(30.0, 40.0);
        pattern.trim();
        pattern.stitch(50.0, 60.0);
        let count = pattern.stitches.len();
        pattern.rotate(45.0);
        assert_eq!(pattern.stitches.len(), count);
    }

    #[test]
    fn test_scale_basic() {
        let mut pattern = EmbPattern::new();
        pattern.stitch_abs(100.0, 50.0);
        pattern.scale(2.0, 3.0);
        assert_eq!(pattern.stitches[0].x, 200.0);
        assert_eq!(pattern.stitches[0].y, 150.0);
    }

    #[test]
    fn test_scale_uniform() {
        let mut pattern = EmbPattern::new();
        pattern.stitch_abs(100.0, 50.0);
        pattern.scale_uniform(2.0);
        assert_eq!(pattern.stitches[0].x, 200.0);
        assert_eq!(pattern.stitches[0].y, 100.0);
    }

    #[test]
    fn test_scale_negative() {
        let mut pattern = EmbPattern::new();
        pattern.stitch_abs(100.0, 50.0);
        pattern.scale(-1.0, -1.0);
        assert_eq!(pattern.stitches[0].x, -100.0);
        assert_eq!(pattern.stitches[0].y, -50.0);
    }

    #[test]
    fn test_scale_preserves_stitch_count() {
        let mut pattern = EmbPattern::new();
        pattern.stitch(10.0, 20.0);
        pattern.stitch(30.0, 40.0);
        let count = pattern.stitches.len();
        pattern.scale(2.0, 2.0);
        assert_eq!(pattern.stitches.len(), count);
    }

    #[test]
    fn test_flip_horizontal() {
        let mut pattern = EmbPattern::new();
        pattern.stitch_abs(100.0, 50.0);
        pattern.stitch_abs(-30.0, 75.0);
        pattern.flip_horizontal();
        assert_eq!(pattern.stitches[0].x, -100.0);
        assert_eq!(pattern.stitches[0].y, 50.0);
        assert_eq!(pattern.stitches[1].x, 30.0);
        assert_eq!(pattern.stitches[1].y, 75.0);
    }

    #[test]
    fn test_flip_vertical() {
        let mut pattern = EmbPattern::new();
        pattern.stitch_abs(100.0, 50.0);
        pattern.stitch_abs(-30.0, -75.0);
        pattern.flip_vertical();
        assert_eq!(pattern.stitches[0].x, 100.0);
        assert_eq!(pattern.stitches[0].y, -50.0);
        assert_eq!(pattern.stitches[1].x, -30.0);
        assert_eq!(pattern.stitches[1].y, 75.0);
    }

    #[test]
    fn test_flip_horizontal_roundtrip() {
        let mut pattern = EmbPattern::new();
        pattern.stitch_abs(100.0, 50.0);
        let orig_x = pattern.stitches[0].x;
        let orig_y = pattern.stitches[0].y;
        pattern.flip_horizontal();
        pattern.flip_horizontal();
        assert_eq!(pattern.stitches[0].x, orig_x);
        assert_eq!(pattern.stitches[0].y, orig_y);
    }

    #[test]
    fn test_flip_vertical_roundtrip() {
        let mut pattern = EmbPattern::new();
        pattern.stitch_abs(100.0, 50.0);
        let orig_x = pattern.stitches[0].x;
        let orig_y = pattern.stitches[0].y;
        pattern.flip_vertical();
        pattern.flip_vertical();
        assert_eq!(pattern.stitches[0].x, orig_x);
        assert_eq!(pattern.stitches[0].y, orig_y);
    }

    #[test]
    fn test_combined_transformations() {
        let mut pattern = EmbPattern::new();
        pattern.stitch_abs(100.0, 0.0);
        // Scale, rotate, then flip
        pattern.scale_uniform(2.0); // -> (200, 0)
        pattern.rotate(90.0); // -> (0, 200)
        pattern.flip_horizontal(); // -> (0, 200)
        assert!((pattern.stitches[0].x - 0.0).abs() < 0.01);
        assert!((pattern.stitches[0].y - 200.0).abs() < 0.01);
    }

    #[test]
    fn test_transformation_on_empty_pattern() {
        let mut pattern = EmbPattern::new();
        pattern.rotate(45.0);
        pattern.scale(2.0, 2.0);
        pattern.flip_horizontal();
        assert_eq!(pattern.stitches.len(), 0);
    }

    #[test]
    fn test_rotate_invalid_angle() {
        let mut pattern = EmbPattern::new();
        pattern.stitch_abs(100.0, 50.0);
        let orig_x = pattern.stitches[0].x;
        let orig_y = pattern.stitches[0].y;
        pattern.rotate(f64::NAN);
        // Should be unchanged
        assert_eq!(pattern.stitches[0].x, orig_x);
        assert_eq!(pattern.stitches[0].y, orig_y);
    }

    #[test]
    fn test_scale_zero() {
        let mut pattern = EmbPattern::new();
        pattern.stitch_abs(100.0, 50.0);
        let orig_x = pattern.stitches[0].x;
        let orig_y = pattern.stitches[0].y;
        pattern.scale(0.0, 1.0);
        // Should be unchanged (zero scale is invalid)
        assert_eq!(pattern.stitches[0].x, orig_x);
        assert_eq!(pattern.stitches[0].y, orig_y);
    }

    #[test]
    fn test_apply_matrix_basic() {
        use crate::core::matrix::EmbMatrix;

        let mut pattern = EmbPattern::new();
        pattern.stitch_abs(10.0, 0.0);

        let mut matrix = EmbMatrix::new();
        matrix.post_translate(5.0, 10.0);

        pattern.apply_matrix(&matrix);
        assert_eq!(pattern.stitches[0].x, 15.0);
        assert_eq!(pattern.stitches[0].y, 10.0);
    }

    #[test]
    fn test_apply_matrix_rotation() {
        use crate::core::matrix::EmbMatrix;

        let mut pattern = EmbPattern::new();
        pattern.stitch_abs(10.0, 0.0);

        let mut matrix = EmbMatrix::new();
        matrix.post_rotate(90.0, 0.0, 0.0);

        pattern.apply_matrix(&matrix);
        // After 90° rotation, (10, 0) -> (0, 10)
        assert!((pattern.stitches[0].x - 0.0).abs() < 1e-10);
        assert!((pattern.stitches[0].y - 10.0).abs() < 1e-10);
    }

    #[test]
    fn test_apply_matrix_complex_transform() {
        use crate::core::matrix::EmbMatrix;

        let mut pattern = EmbPattern::new();
        pattern.stitch_abs(10.0, 0.0);
        pattern.stitch_abs(10.0, 10.0);

        // Create complex transformation: scale then rotate then translate
        let mut matrix = EmbMatrix::new();
        matrix.post_scale(2.0, None, 0.0, 0.0); // Scale 2x
        matrix.post_rotate(90.0, 0.0, 0.0); // Rotate 90°
        matrix.post_translate(5.0, 5.0); // Translate

        pattern.apply_matrix(&matrix);

        // Verify pattern has 2 stitches
        assert_eq!(pattern.stitches.len(), 2);

        // Verify transformation was applied (exact values depend on matrix math)
        assert!(pattern.stitches[0].x != 10.0 || pattern.stitches[0].y != 0.0);
        assert!(pattern.stitches[1].x != 10.0 || pattern.stitches[1].y != 10.0);
    }

    #[test]
    fn test_apply_matrix_identity() {
        use crate::core::matrix::EmbMatrix;

        let mut pattern = EmbPattern::new();
        pattern.stitch_abs(15.0, 25.0);

        let matrix = EmbMatrix::new(); // Identity matrix

        pattern.apply_matrix(&matrix);
        // Should be unchanged
        assert_eq!(pattern.stitches[0].x, 15.0);
        assert_eq!(pattern.stitches[0].y, 25.0);
    }

    #[test]
    fn test_apply_matrix_empty_pattern() {
        use crate::core::matrix::EmbMatrix;

        let mut pattern = EmbPattern::new();
        let mut matrix = EmbMatrix::new();
        matrix.post_rotate(45.0, 0.0, 0.0);

        pattern.apply_matrix(&matrix); // Should not crash
        assert_eq!(pattern.stitches.len(), 0);
    }

    #[test]
    fn test_apply_matrix_updates_previous_position() {
        use crate::core::matrix::EmbMatrix;

        let mut pattern = EmbPattern::new();
        pattern.stitch_abs(10.0, 10.0);

        let mut matrix = EmbMatrix::new();
        matrix.post_scale(2.0, None, 0.0, 0.0);

        pattern.apply_matrix(&matrix);

        // Previous position should also be transformed
        assert_eq!(pattern.previous_x, 20.0);
        assert_eq!(pattern.previous_y, 20.0);
    }

    // Stitch splitting tests
    #[test]
    fn test_split_long_stitches_no_split_needed() {
        let mut pattern = EmbPattern::new();
        pattern.stitch(50.0, 0.0);
        pattern.stitch(50.0, 0.0);
        let orig_count = pattern.stitches.len();
        pattern.split_long_stitches(100.0).unwrap();
        assert_eq!(pattern.stitches.len(), orig_count);
    }

    #[test]
    fn test_split_long_stitches_exact_split() {
        let mut pattern = EmbPattern::new();
        pattern.stitch(200.0, 0.0); // Length 200, should split into 2 segments of 100
        pattern.split_long_stitches(100.0).unwrap();
        assert_eq!(pattern.stitches.len(), 2);
        // Check intermediate points
        assert!((pattern.stitches[0].x - 100.0).abs() < 0.01);
        assert!((pattern.stitches[1].x - 200.0).abs() < 0.01);
    }

    #[test]
    fn test_split_long_stitches_multiple_segments() {
        let mut pattern = EmbPattern::new();
        pattern.stitch(300.0, 0.0); // Length 300, should split into 3 segments of 100
        pattern.split_long_stitches(100.0).unwrap();
        assert_eq!(pattern.stitches.len(), 3);
        // Check all intermediate points
        assert!((pattern.stitches[0].x - 100.0).abs() < 0.01);
        assert!((pattern.stitches[1].x - 200.0).abs() < 0.01);
        assert!((pattern.stitches[2].x - 300.0).abs() < 0.01);
    }

    #[test]
    fn test_split_long_stitches_diagonal() {
        let mut pattern = EmbPattern::new();
        pattern.stitch(300.0, 400.0); // 3-4-5 triangle, length = 500
        pattern.split_long_stitches(250.0).unwrap();
        // Should split into 2 segments
        assert_eq!(pattern.stitches.len(), 2);
    }

    #[test]
    fn test_split_long_stitches_preserves_jumps() {
        let mut pattern = EmbPattern::new();
        pattern.stitch(50.0, 0.0);
        pattern.jump(200.0, 0.0); // Long jump - should NOT be split
        pattern.stitch(50.0, 0.0);
        pattern.split_long_stitches(100.0).unwrap();
        // Jump should be preserved, only stitches split
        assert_eq!(pattern.count_jumps(), 1);
    }

    #[test]
    fn test_split_long_stitches_preserves_trims() {
        let mut pattern = EmbPattern::new();
        pattern.stitch(50.0, 0.0);
        pattern.trim();
        pattern.stitch(200.0, 0.0); // This should be split
        pattern.split_long_stitches(100.0).unwrap();
        assert_eq!(pattern.count_trims(), 1);
    }

    #[test]
    fn test_split_long_stitches_very_long() {
        let mut pattern = EmbPattern::new();
        pattern.stitch(1000.0, 0.0); // 10x the max length
        pattern.split_long_stitches(100.0).unwrap();
        assert_eq!(pattern.stitches.len(), 10);
    }

    #[test]
    fn test_split_long_stitches_negative_coords() {
        let mut pattern = EmbPattern::new();
        pattern.stitch_abs(100.0, 100.0);
        pattern.stitch_abs(-100.0, -100.0); // Long diagonal
        pattern.split_long_stitches(150.0).unwrap();
        // Should have split the second stitch
        assert!(pattern.stitches.len() > 2);
    }

    #[test]
    fn test_split_long_stitches_invalid_max_length() {
        let mut pattern = EmbPattern::new();
        pattern.stitch(100.0, 0.0);
        // Zero max length
        assert!(pattern.split_long_stitches(0.0).is_err());
        // Negative max length
        assert!(pattern.split_long_stitches(-10.0).is_err());
        // NaN
        assert!(pattern.split_long_stitches(f64::NAN).is_err());
    }

    #[test]
    fn test_split_to_format_limits_dst() {
        let mut pattern = EmbPattern::new();
        pattern.stitch(250.0, 0.0); // Exceeds DST limit of 121
        pattern.split_to_format_limits("dst").unwrap();
        // Should be split into at least 3 segments
        assert!(pattern.stitches.len() >= 3);
    }

    #[test]
    fn test_split_to_format_limits_pes() {
        let mut pattern = EmbPattern::new();
        pattern.stitch(250.0, 0.0); // Exceeds PES limit of 127
        pattern.split_to_format_limits("pes").unwrap();
        assert!(pattern.stitches.len() >= 2);
    }

    #[test]
    fn test_split_to_format_limits_case_insensitive() {
        let mut pattern = EmbPattern::new();
        pattern.stitch(250.0, 0.0);
        pattern.split_to_format_limits("DST").unwrap(); // Uppercase
        assert!(pattern.stitches.len() >= 3);

        let mut pattern2 = EmbPattern::new();
        pattern2.stitch(250.0, 0.0);
        pattern2.split_to_format_limits("PeS").unwrap(); // Mixed case
        assert!(pattern2.stitches.len() >= 2);
    }

    #[test]
    fn test_split_to_format_limits_unknown_format() {
        let mut pattern = EmbPattern::new();
        pattern.stitch(250.0, 0.0);
        assert!(pattern.split_to_format_limits("unknown").is_err());
    }

    #[test]
    fn test_split_preserves_endpoint() {
        let mut pattern = EmbPattern::new();
        pattern.stitch(300.0, 400.0);
        let end_x = pattern.stitches.last().unwrap().x;
        let end_y = pattern.stitches.last().unwrap().y;
        pattern.split_long_stitches(100.0).unwrap();
        // Final stitch should be at the same endpoint
        assert_eq!(pattern.stitches.last().unwrap().x, end_x);
        assert_eq!(pattern.stitches.last().unwrap().y, end_y);
    }

    #[test]
    fn test_split_maintains_path() {
        let mut pattern = EmbPattern::new();
        pattern.stitch(100.0, 0.0);
        pattern.stitch(100.0, 100.0);
        pattern.stitch(-100.0, 0.0);
        pattern.split_long_stitches(75.0).unwrap();
        // All stitches should maintain the original path direction
        // Just verify the final position matches
        let last = pattern.stitches.last().unwrap();
        assert_eq!(last.x, 100.0);
        assert_eq!(last.y, 100.0);
    }

    // Remove duplicates tests
    #[test]
    fn test_remove_duplicates_empty_pattern() {
        let mut pattern = EmbPattern::new();
        pattern.remove_duplicates();
        assert_eq!(pattern.stitches.len(), 0);
    }

    #[test]
    fn test_remove_duplicates_no_duplicates() {
        let mut pattern = EmbPattern::new();
        pattern.stitch_abs(10.0, 10.0);
        pattern.stitch_abs(20.0, 20.0);
        pattern.stitch_abs(30.0, 30.0);
        pattern.remove_duplicates();
        assert_eq!(pattern.count_stitches(), 3);
    }

    #[test]
    fn test_remove_duplicates_consecutive_duplicates() {
        let mut pattern = EmbPattern::new();
        pattern.stitch_abs(10.0, 10.0);
        pattern.stitch_abs(10.0, 10.0); // Duplicate
        pattern.stitch_abs(10.0, 10.0); // Duplicate
        pattern.stitch_abs(20.0, 20.0);
        pattern.remove_duplicates();
        assert_eq!(pattern.count_stitches(), 2);
    }

    #[test]
    fn test_remove_duplicates_preserves_commands() {
        let mut pattern = EmbPattern::new();
        pattern.stitch_abs(10.0, 10.0);
        pattern.trim(); // Trim at same position - should be preserved
        pattern.stitch_abs(10.0, 10.0); // Duplicate stitch - removed
        pattern.remove_duplicates();
        // Should have: stitch, trim (duplicate stitch removed)
        assert_eq!(pattern.stitches.len(), 2);
        assert_eq!(pattern.count_stitches(), 1);
        assert_eq!(pattern.count_trims(), 1);
    }

    #[test]
    fn test_remove_duplicates_preserves_jumps() {
        let mut pattern = EmbPattern::new();
        pattern.stitch_abs(10.0, 10.0);
        pattern.jump_abs(10.0, 10.0); // Jump at same position - preserved
        pattern.stitch_abs(20.0, 20.0);
        pattern.remove_duplicates();
        assert_eq!(pattern.count_jumps(), 1);
    }

    #[test]
    fn test_remove_duplicates_preserves_color_changes() {
        let mut pattern = EmbPattern::new();
        pattern.stitch_abs(10.0, 10.0);
        pattern.color_change(0.0, 0.0); // Color change - preserved
        pattern.stitch_abs(10.0, 10.0);
        pattern.remove_duplicates();
        assert_eq!(pattern.count_color_changes(), 1);
    }

    #[test]
    fn test_remove_duplicates_mixed_pattern() {
        let mut pattern = EmbPattern::new();
        pattern.stitch_abs(10.0, 10.0);
        pattern.stitch_abs(10.0, 10.0); // Duplicate - removed
        pattern.stitch_abs(20.0, 20.0);
        pattern.stitch_abs(20.0, 20.0); // Duplicate - removed
        pattern.jump_abs(30.0, 30.0);
        pattern.stitch_abs(30.0, 30.0); // Duplicate position but after jump - removed
        pattern.stitch_abs(40.0, 40.0); // Different position - kept
        pattern.remove_duplicates();
        assert_eq!(pattern.count_stitches(), 3); // stitches at 10, 20, 40
        assert_eq!(pattern.count_jumps(), 1);
    }

    #[test]
    fn test_remove_duplicates_updates_previous_position() {
        let mut pattern = EmbPattern::new();
        pattern.stitch_abs(10.0, 10.0);
        pattern.stitch_abs(20.0, 20.0);
        pattern.stitch_abs(20.0, 20.0); // Duplicate
        pattern.remove_duplicates();
        // Previous position should be updated to last stitch
        assert_eq!(pattern.previous_x, 20.0);
        assert_eq!(pattern.previous_y, 20.0);
    }

    #[test]
    fn test_remove_duplicates_single_stitch() {
        let mut pattern = EmbPattern::new();
        pattern.stitch_abs(10.0, 10.0);
        pattern.remove_duplicates();
        assert_eq!(pattern.count_stitches(), 1);
    }

    #[test]
    fn test_remove_duplicates_all_duplicates() {
        let mut pattern = EmbPattern::new();
        pattern.stitch_abs(10.0, 10.0);
        pattern.stitch_abs(10.0, 10.0);
        pattern.stitch_abs(10.0, 10.0);
        pattern.stitch_abs(10.0, 10.0);
        pattern.remove_duplicates();
        assert_eq!(pattern.count_stitches(), 1);
    }

    #[test]
    fn test_remove_duplicates_alternating() {
        let mut pattern = EmbPattern::new();
        pattern.stitch_abs(10.0, 10.0);
        pattern.stitch_abs(20.0, 20.0);
        pattern.stitch_abs(10.0, 10.0); // Not consecutive - keep
        pattern.stitch_abs(20.0, 20.0); // Not consecutive - keep
        pattern.remove_duplicates();
        assert_eq!(pattern.count_stitches(), 4);
    }

    // Property-based tests
    mod proptests {
        use super::*;
        use proptest::prelude::*;

        // Strategy for generating valid stitches
        prop_compose! {
            fn stitch_strategy()
                (x in -10000.0..10000.0,
                 y in -10000.0..10000.0,
                 cmd in 0u32..16u32)  // Limit to valid command range
                -> Stitch {
                Stitch::new(x, y, cmd)
            }
        }

        // Strategy for generating patterns with multiple stitches
        prop_compose! {
            fn pattern_strategy()
                (stitches in prop::collection::vec(stitch_strategy(), 0..20))
                -> EmbPattern {
                let mut pattern = EmbPattern::new();
                for stitch in stitches {
                    pattern.add_stitch_absolute(stitch.command, stitch.x, stitch.y);
                }
                pattern
            }
        }

        proptest! {
            #[test]
            fn translate_preserves_stitch_count(
                pattern in pattern_strategy(),
                dx in -1000.0..1000.0,
                dy in -1000.0..1000.0
            ) {
                let orig_count = pattern.stitches().len();
                let mut translated = pattern.clone();
                translated.translate(dx, dy);
                prop_assert_eq!(translated.stitches().len(), orig_count);
            }

            #[test]
            fn translate_updates_positions(
                mut pattern in pattern_strategy(),
                dx in -100.0..100.0,
                dy in -100.0..100.0
            ) {
                if pattern.stitches().is_empty() {
                    return Ok(());
                }

                let orig_first = pattern.stitches()[0];
                pattern.translate(dx, dy);
                let new_first = pattern.stitches()[0];

                // Check translation worked (within floating point precision)
                prop_assert!((new_first.x - (orig_first.x + dx)).abs() < 0.001);
                prop_assert!((new_first.y - (orig_first.y + dy)).abs() < 0.001);
            }

            #[test]
            fn bounds_always_valid(pattern in pattern_strategy()) {
                let (min_x, min_y, max_x, max_y) = pattern.bounds();
                prop_assert!(min_x <= max_x);
                prop_assert!(min_y <= max_y);
            }

            #[test]
            fn rotate_preserves_stitch_count(
                pattern in pattern_strategy(),
                angle in -360.0..360.0
            ) {
                let orig_count = pattern.stitches().len();
                let mut rotated = pattern.clone();
                rotated.rotate(angle);
                prop_assert_eq!(rotated.stitches().len(), orig_count);
            }

            #[test]
            fn rotate_360_is_identity(
                mut pattern in pattern_strategy()
            ) {
                if pattern.stitches().is_empty() {
                    return Ok(());
                }

                let orig = pattern.stitches()[0];
                pattern.rotate(360.0);
                let new = pattern.stitches()[0];

                // Should be back to original (within floating point error)
                prop_assert!((new.x - orig.x).abs() < 0.01);
                prop_assert!((new.y - orig.y).abs() < 0.01);
            }

            #[test]
            fn scale_preserves_stitch_count(
                pattern in pattern_strategy(),
                sx in 0.1..10.0,
                sy in 0.1..10.0
            ) {
                let orig_count = pattern.stitches().len();
                let mut scaled = pattern.clone();
                scaled.scale(sx, sy);
                prop_assert_eq!(scaled.stitches().len(), orig_count);
            }

            #[test]
            fn scale_affects_bounds(
                mut pattern in pattern_strategy(),
                factor in 1.5..3.0
            ) {
                if pattern.stitches().is_empty() {
                    return Ok(());
                }

                let (min_x, min_y, max_x, max_y) = pattern.bounds();
                let orig_width = max_x - min_x;
                let orig_height = max_y - min_y;

                pattern.scale_uniform(factor);

                let (new_min_x, new_min_y, new_max_x, new_max_y) = pattern.bounds();
                let new_width = new_max_x - new_min_x;
                let new_height = new_max_y - new_min_y;

                // Width and height should scale by factor (within precision)
                if orig_width > 0.0 {
                    let width_ratio = new_width / orig_width;
                    prop_assert!((width_ratio - factor).abs() < 0.01);
                }
                if orig_height > 0.0 {
                    let height_ratio = new_height / orig_height;
                    prop_assert!((height_ratio - factor).abs() < 0.01);
                }
            }

            #[test]
            fn flip_horizontal_is_involution(
                mut pattern in pattern_strategy()
            ) {
                if pattern.stitches().is_empty() {
                    return Ok(());
                }

                let orig = pattern.stitches()[0];
                pattern.flip_horizontal();
                pattern.flip_horizontal();
                let new = pattern.stitches()[0];

                // Flipping twice should return to original
                prop_assert_eq!(new.x, orig.x);
                prop_assert_eq!(new.y, orig.y);
            }

            #[test]
            fn flip_vertical_is_involution(
                mut pattern in pattern_strategy()
            ) {
                if pattern.stitches().is_empty() {
                    return Ok(());
                }

                let orig = pattern.stitches()[0];
                pattern.flip_vertical();
                pattern.flip_vertical();
                let new = pattern.stitches()[0];

                // Flipping twice should return to original
                prop_assert_eq!(new.x, orig.x);
                prop_assert_eq!(new.y, orig.y);
            }

            #[test]
            fn stitch_distance_is_symmetric(
                s1 in stitch_strategy(),
                s2 in stitch_strategy()
            ) {
                let d1 = s1.distance_to(&s2);
                let d2 = s2.distance_to(&s1);
                prop_assert!((d1 - d2).abs() < 0.001);
            }

            #[test]
            fn stitch_distance_is_non_negative(
                s1 in stitch_strategy(),
                s2 in stitch_strategy()
            ) {
                let dist = s1.distance_to(&s2);
                prop_assert!(dist >= 0.0);
            }

            #[test]
            fn stitch_is_valid_for_finite_coords(
                x in -10000.0..10000.0,
                y in -10000.0..10000.0
            ) {
                let stitch = Stitch::new(x, y, STITCH);
                prop_assert!(stitch.is_valid());
            }

            #[test]
            fn width_is_non_negative(pattern in pattern_strategy()) {
                let width = pattern.width();
                prop_assert!(width >= 0.0);
            }

            #[test]
            fn height_is_non_negative(pattern in pattern_strategy()) {
                let height = pattern.height();
                prop_assert!(height >= 0.0);
            }

            #[test]
            fn total_stitch_length_is_non_negative(pattern in pattern_strategy()) {
                let length = pattern.total_stitch_length();
                prop_assert!(length >= 0.0);
            }

            #[test]
            fn max_stitch_length_is_non_negative(pattern in pattern_strategy()) {
                let max_length = pattern.max_stitch_length();
                prop_assert!(max_length >= 0.0);
            }

            #[test]
            fn avg_stitch_length_is_non_negative(pattern in pattern_strategy()) {
                let avg = pattern.avg_stitch_length();
                prop_assert!(avg >= 0.0);
            }

            #[test]
            fn split_increases_or_maintains_stitch_count(
                mut pattern in pattern_strategy(),
                max_length in 10.0..500.0
            ) {
                let orig_count = pattern.stitches().len();
                let _ = pattern.split_long_stitches(max_length);
                prop_assert!(pattern.stitches().len() >= orig_count);
            }

            #[test]
            fn split_preserves_final_position(
                mut pattern in pattern_strategy(),
                max_length in 50.0..200.0
            ) {
                if pattern.stitches().is_empty() {
                    return Ok(());
                }

                let last = pattern.stitches().last().cloned().unwrap();
                let _ = pattern.split_long_stitches(max_length);

                if !pattern.stitches().is_empty() {
                    let new_last = pattern.stitches().last().unwrap();
                    // Allow for floating point precision errors
                    prop_assert!((new_last.x - last.x).abs() < 0.001);
                    prop_assert!((new_last.y - last.y).abs() < 0.001);
                }
            }

            #[test]
            fn split_respects_max_length(
                mut pattern in pattern_strategy(),
                max_length in 50.0..200.0
            ) {
                let _ = pattern.split_long_stitches(max_length);

                // Check that no stitch exceeds max_length
                let mut prev_x = 0.0;
                let mut prev_y = 0.0;
                for stitch in pattern.stitches() {
                    if stitch.command == STITCH {
                        let dx = stitch.x - prev_x;
                        let dy = stitch.y - prev_y;
                        let length = (dx * dx + dy * dy).sqrt();
                        // Allow small floating point error
                        prop_assert!(length <= max_length + 0.1);
                    }
                    prev_x = stitch.x;
                    prev_y = stitch.y;
                }
            }

            #[test]
            fn remove_duplicates_reduces_or_maintains_count(
                mut pattern in pattern_strategy()
            ) {
                let orig_count = pattern.stitches().len();
                pattern.remove_duplicates();
                prop_assert!(pattern.stitches().len() <= orig_count);
            }

            #[test]
            fn remove_duplicates_preserves_endpoints(
                mut pattern in pattern_strategy()
            ) {
                if pattern.stitches().is_empty() {
                    return Ok(());
                }

                let first = pattern.stitches().first().cloned().unwrap();
                let last = pattern.stitches().last().cloned().unwrap();
                pattern.remove_duplicates();

                if !pattern.stitches().is_empty() {
                    let new_first = pattern.stitches().first().unwrap();
                    let new_last = pattern.stitches().last().unwrap();
                    prop_assert_eq!(new_first.x, first.x);
                    prop_assert_eq!(new_first.y, first.y);
                    prop_assert_eq!(new_last.x, last.x);
                    prop_assert_eq!(new_last.y, last.y);
                }
            }

            #[test]
            fn remove_duplicates_is_idempotent(
                mut pattern in pattern_strategy()
            ) {
                pattern.remove_duplicates();
                let count_after_first = pattern.stitches().len();
                pattern.remove_duplicates();
                let count_after_second = pattern.stitches().len();
                // Running twice should give same result
                prop_assert_eq!(count_after_first, count_after_second);
            }
        }
    }

    // ========== Property Accessor Tests ==========

    #[test]
    fn test_title_property() {
        let mut pattern = EmbPattern::new();
        assert!(pattern.title().is_none());

        pattern.set_title("My Design");
        assert_eq!(pattern.title(), Some("My Design"));

        // Should also work with "title" key
        pattern.set_metadata("title", "Another Name");
        assert_eq!(pattern.title(), Some("My Design")); // "name" takes precedence
    }

    #[test]
    fn test_author_property() {
        let mut pattern = EmbPattern::new();
        assert!(pattern.author().is_none());

        pattern.set_author("Jane Doe");
        assert_eq!(pattern.author(), Some("Jane Doe"));
    }

    #[test]
    fn test_copyright_property() {
        let mut pattern = EmbPattern::new();
        assert!(pattern.copyright().is_none());

        pattern.set_copyright("Copyright 2025");
        assert_eq!(pattern.copyright(), Some("Copyright 2025"));
    }

    #[test]
    fn test_description_property() {
        let mut pattern = EmbPattern::new();
        assert!(pattern.description().is_none());

        pattern.set_description("A beautiful floral design");
        assert_eq!(pattern.description(), Some("A beautiful floral design"));
    }

    #[test]
    fn test_keywords_property() {
        let mut pattern = EmbPattern::new();
        assert!(pattern.keywords().is_none());

        pattern.set_keywords(&["floral", "embroidery", "red"]);
        let keywords = pattern.keywords().unwrap();
        assert_eq!(keywords.len(), 3);
        assert!(keywords.contains(&"floral".to_string()));
        assert!(keywords.contains(&"embroidery".to_string()));
        assert!(keywords.contains(&"red".to_string()));

        // Test parsing comma-separated string
        pattern.set_metadata("keywords", "vintage, lace, white");
        let keywords2 = pattern.keywords().unwrap();
        assert_eq!(keywords2.len(), 3);
        assert_eq!(keywords2[0], "vintage");
        assert_eq!(keywords2[1], "lace");
        assert_eq!(keywords2[2], "white");
    }

    #[test]
    fn test_date_property() {
        let mut pattern = EmbPattern::new();
        assert!(pattern.date().is_none());

        pattern.set_date("2025-10-11");
        assert_eq!(pattern.date(), Some("2025-10-11"));
    }

    #[test]
    fn test_notes_property() {
        let mut pattern = EmbPattern::new();
        assert!(pattern.notes().is_none());

        pattern.set_notes("Use stabilizer on stretchy fabrics");
        assert_eq!(pattern.notes(), Some("Use stabilizer on stretchy fabrics"));

        // Should also work with "comments" key
        pattern.set_metadata("comments", "Another note");
        assert_eq!(pattern.notes(), Some("Use stabilizer on stretchy fabrics"));
        // "notes" takes precedence
    }

    #[test]
    fn test_software_properties() {
        let mut pattern = EmbPattern::new();
        assert!(pattern.software().is_none());
        assert!(pattern.software_version().is_none());

        pattern.set_software("Butabuti");
        pattern.set_software_version("0.1.0");

        assert_eq!(pattern.software(), Some("Butabuti"));
        assert_eq!(pattern.software_version(), Some("0.1.0"));

        // Test version fallback
        pattern.set_metadata("version", "1.0.0");
        assert_eq!(pattern.software_version(), Some("0.1.0")); // "software_version" takes precedence
    }

    #[test]
    fn test_hoop_size_property() {
        let mut pattern = EmbPattern::new();
        assert!(pattern.hoop_size().is_none());

        pattern.set_hoop_size("4x4");
        assert_eq!(pattern.hoop_size(), Some("4x4"));

        pattern.set_hoop_size("100mm x 100mm");
        assert_eq!(pattern.hoop_size(), Some("100mm x 100mm"));
    }

    #[test]
    fn test_design_dimensions() {
        let mut pattern = EmbPattern::new();

        // Empty pattern should return None
        assert!(pattern.design_width().is_none());
        assert!(pattern.design_height().is_none());

        // Add stitches to create bounds
        pattern.add_stitch_absolute(STITCH, 0.0, 0.0);
        pattern.add_stitch_absolute(STITCH, 100.0, 200.0); // 10mm x 20mm

        let width = pattern.design_width().unwrap();
        let height = pattern.design_height().unwrap();

        assert!(
            (width - 10.0).abs() < 0.01,
            "Expected 10mm width, got {}",
            width
        );
        assert!(
            (height - 20.0).abs() < 0.01,
            "Expected 20mm height, got {}",
            height
        );

        // Test explicit metadata override
        pattern.set_metadata("design_width", "15.5");
        pattern.set_metadata("design_height", "25.5");

        assert_eq!(pattern.design_width(), Some(15.5));
        assert_eq!(pattern.design_height(), Some(25.5));
    }

    #[test]
    fn test_fabric_type_property() {
        let mut pattern = EmbPattern::new();
        assert!(pattern.fabric_type().is_none());

        pattern.set_fabric_type("Cotton");
        assert_eq!(pattern.fabric_type(), Some("Cotton"));

        // Test fallback
        pattern.set_metadata("fabric", "Silk");
        assert_eq!(pattern.fabric_type(), Some("Cotton")); // "fabric_type" takes precedence
    }

    #[test]
    fn test_thread_brand_property() {
        let mut pattern = EmbPattern::new();
        assert!(pattern.thread_brand().is_none());

        pattern.set_thread_brand("Madeira");
        assert_eq!(pattern.thread_brand(), Some("Madeira"));
    }

    #[test]
    fn test_company_property() {
        let mut pattern = EmbPattern::new();
        assert!(pattern.company().is_none());

        pattern.set_company("Acme Embroidery");
        assert_eq!(pattern.company(), Some("Acme Embroidery"));

        // Test fallback
        pattern.set_metadata("organization", "Another Corp");
        assert_eq!(pattern.company(), Some("Acme Embroidery")); // "company" takes precedence
    }

    #[test]
    fn test_comprehensive_metadata() {
        let mut pattern = EmbPattern::new();

        // Set all properties
        pattern.set_title("Floral Design");
        pattern.set_author("Jane Designer");
        pattern.set_copyright("Copyright 2025 Jane Designer");
        pattern.set_description("Beautiful floral embroidery pattern");
        pattern.set_keywords(&["floral", "flowers", "nature"]);
        pattern.set_date("2025-10-11");
        pattern.set_notes("Use tear-away stabilizer");
        pattern.set_software("Butabuti");
        pattern.set_software_version("0.1.0");
        pattern.set_hoop_size("5x7");
        pattern.set_fabric_type("Cotton");
        pattern.set_thread_brand("Robison-Anton");
        pattern.set_company("Jane's Embroidery Studio");

        // Verify all properties
        assert_eq!(pattern.title(), Some("Floral Design"));
        assert_eq!(pattern.author(), Some("Jane Designer"));
        assert_eq!(pattern.copyright(), Some("Copyright 2025 Jane Designer"));
        assert_eq!(
            pattern.description(),
            Some("Beautiful floral embroidery pattern")
        );
        assert_eq!(pattern.keywords().unwrap().len(), 3);
        assert_eq!(pattern.date(), Some("2025-10-11"));
        assert_eq!(pattern.notes(), Some("Use tear-away stabilizer"));
        assert_eq!(pattern.software(), Some("Butabuti"));
        assert_eq!(pattern.software_version(), Some("0.1.0"));
        assert_eq!(pattern.hoop_size(), Some("5x7"));
        assert_eq!(pattern.fabric_type(), Some("Cotton"));
        assert_eq!(pattern.thread_brand(), Some("Robison-Anton"));
        assert_eq!(pattern.company(), Some("Jane's Embroidery Studio"));

        // Verify metadata iterator includes all
        let metadata_count = pattern.metadata().count();
        assert!(
            metadata_count >= 13,
            "Expected at least 13 metadata entries, got {}",
            metadata_count
        );
    }

    #[test]
    fn test_property_fallbacks() {
        let mut pattern = EmbPattern::new();

        // Test that "name" is preferred over "title"
        pattern.set_metadata("title", "Title Value");
        pattern.set_metadata("name", "Name Value");
        assert_eq!(pattern.title(), Some("Name Value"));

        // Test that "notes" is preferred over "comments"
        pattern.set_metadata("comments", "Comment Value");
        pattern.set_metadata("notes", "Notes Value");
        assert_eq!(pattern.notes(), Some("Notes Value"));

        // Test that "hoop_size" is preferred over "hoop"
        pattern.set_metadata("hoop", "Hoop Value");
        pattern.set_metadata("hoop_size", "HoopSize Value");
        assert_eq!(pattern.hoop_size(), Some("HoopSize Value"));
    }

    #[test]
    fn test_iter_commands_empty() {
        let pattern = EmbPattern::new();
        let commands: Vec<_> = pattern.iter_commands().collect();
        assert_eq!(commands.len(), 0);
    }

    #[test]
    fn test_iter_commands_basic_stitches() {
        let mut pattern = EmbPattern::new();
        pattern.add_thread(EmbThread::from_string("red").unwrap());
        pattern.stitch(10.0, 0.0);
        pattern.stitch(0.0, 10.0);
        pattern.end();

        let commands: Vec<_> = pattern.iter_commands().collect();
        assert_eq!(commands.len(), 3);

        match commands[0] {
            StitchCommand::Stitch(s) => {
                assert_eq!(s.x, 10.0);
                assert_eq!(s.y, 0.0);
            },
            _ => panic!("Expected Stitch"),
        }

        match commands[1] {
            StitchCommand::Stitch(s) => {
                assert_eq!(s.x, 10.0);
                assert_eq!(s.y, 10.0);
            },
            _ => panic!("Expected Stitch"),
        }

        match commands[2] {
            StitchCommand::End(_) => {},
            _ => panic!("Expected End"),
        }
    }

    #[test]
    fn test_iter_commands_jumps() {
        let mut pattern = EmbPattern::new();
        pattern.add_thread(EmbThread::from_string("red").unwrap());
        pattern.stitch(10.0, 0.0);
        pattern.jump(20.0, 0.0);
        pattern.stitch(10.0, 0.0);
        pattern.end();

        let commands: Vec<_> = pattern.iter_commands().collect();
        assert_eq!(commands.len(), 4);

        match commands[0] {
            StitchCommand::Stitch(_) => {},
            _ => panic!("Expected Stitch"),
        }

        match commands[1] {
            StitchCommand::Jump(s) => {
                assert_eq!(s.x, 30.0); // Accumulated position
                assert_eq!(s.y, 0.0);
            },
            _ => panic!("Expected Jump"),
        }

        match commands[2] {
            StitchCommand::Stitch(_) => {},
            _ => panic!("Expected Stitch"),
        }

        match commands[3] {
            StitchCommand::End(_) => {},
            _ => panic!("Expected End"),
        }
    }

    #[test]
    fn test_iter_commands_color_change() {
        let mut pattern = EmbPattern::new();
        let red = EmbThread::from_string("red").unwrap();
        let blue = EmbThread::from_string("blue").unwrap();

        pattern.add_thread(red.clone());
        pattern.add_thread(blue.clone());

        pattern.stitch(10.0, 0.0);
        pattern.color_change(0.0, 0.0);
        pattern.stitch(10.0, 0.0);
        pattern.end();

        let commands: Vec<_> = pattern.iter_commands().collect();
        assert_eq!(commands.len(), 4);

        match commands[0] {
            StitchCommand::Stitch(_) => {},
            _ => panic!("Expected Stitch"),
        }

        match commands[1] {
            StitchCommand::ColorChange(thread, _) => {
                assert!(thread.is_some());
                assert_eq!(thread.unwrap().color, blue.color);
            },
            _ => panic!("Expected ColorChange"),
        }

        match commands[2] {
            StitchCommand::Stitch(_) => {},
            _ => panic!("Expected Stitch"),
        }

        match commands[3] {
            StitchCommand::End(_) => {},
            _ => panic!("Expected End"),
        }
    }

    #[test]
    fn test_iter_commands_trim() {
        let mut pattern = EmbPattern::new();
        pattern.add_thread(EmbThread::from_string("red").unwrap());
        pattern.stitch(10.0, 0.0);
        pattern.trim();
        pattern.stitch(10.0, 0.0);
        pattern.end();

        let commands: Vec<_> = pattern.iter_commands().collect();
        assert_eq!(commands.len(), 4);

        match commands[0] {
            StitchCommand::Stitch(_) => {},
            _ => panic!("Expected Stitch"),
        }

        match commands[1] {
            StitchCommand::Trim(_) => {},
            _ => panic!("Expected Trim"),
        }

        match commands[2] {
            StitchCommand::Stitch(_) => {},
            _ => panic!("Expected Stitch"),
        }

        match commands[3] {
            StitchCommand::End(_) => {},
            _ => panic!("Expected End"),
        }
    }

    #[test]
    fn test_iter_commands_cut() {
        let mut pattern = EmbPattern::new();
        pattern.add_thread(EmbThread::from_string("blue").unwrap());
        pattern.stitch(10.0, 0.0);
        pattern.cut();
        pattern.stitch(10.0, 0.0);
        pattern.end();

        let commands: Vec<_> = pattern.iter_commands().collect();
        assert_eq!(commands.len(), 4);

        match commands[0] {
            StitchCommand::Stitch(_) => {},
            _ => panic!("Expected Stitch"),
        }

        match commands[1] {
            StitchCommand::Cut(_) => {},
            _ => panic!("Expected Cut"),
        }

        match commands[2] {
            StitchCommand::Stitch(_) => {},
            _ => panic!("Expected Stitch"),
        }

        match commands[3] {
            StitchCommand::End(_) => {},
            _ => panic!("Expected End"),
        }
    }

    #[test]
    fn test_iter_commands_stop() {
        let mut pattern = EmbPattern::new();
        pattern.add_thread(EmbThread::from_string("red").unwrap());
        pattern.stitch(10.0, 0.0);
        pattern.add_stitch_relative(0.0, 0.0, STOP);
        pattern.stitch(10.0, 0.0);
        pattern.end();

        let commands: Vec<_> = pattern.iter_commands().collect();
        assert_eq!(commands.len(), 4);

        match commands[1] {
            StitchCommand::Stop(_) => {},
            _ => panic!("Expected Stop"),
        }
    }

    #[test]
    fn test_iter_commands_comprehensive() {
        let mut pattern = EmbPattern::new();
        let red = EmbThread::from_string("FF0000").unwrap();
        let green = EmbThread::from_string("00FF00").unwrap();
        let blue = EmbThread::from_string("0000FF").unwrap();

        pattern.add_thread(red);
        pattern.add_thread(green);
        pattern.add_thread(blue);

        // Red section
        pattern.stitch(10.0, 0.0);
        pattern.stitch(0.0, 10.0);
        pattern.trim();

        // Change to green
        pattern.color_change(0.0, 0.0);
        pattern.jump(50.0, 0.0);
        pattern.stitch(10.0, 0.0);
        pattern.add_stitch_relative(0.0, 0.0, STOP);

        // Change to blue
        pattern.color_change(0.0, 0.0);
        pattern.stitch(10.0, 0.0);
        pattern.end();

        let commands: Vec<_> = pattern.iter_commands().collect();

        // Count command types
        let stitch_count = commands
            .iter()
            .filter(|c| matches!(c, StitchCommand::Stitch(_)))
            .count();
        let jump_count = commands
            .iter()
            .filter(|c| matches!(c, StitchCommand::Jump(_)))
            .count();
        let trim_count = commands
            .iter()
            .filter(|c| matches!(c, StitchCommand::Trim(_)))
            .count();
        let color_count = commands
            .iter()
            .filter(|c| matches!(c, StitchCommand::ColorChange(_, _)))
            .count();
        let stop_count = commands
            .iter()
            .filter(|c| matches!(c, StitchCommand::Stop(_)))
            .count();
        let end_count = commands
            .iter()
            .filter(|c| matches!(c, StitchCommand::End(_)))
            .count();

        assert_eq!(stitch_count, 4);
        assert_eq!(jump_count, 1);
        assert_eq!(trim_count, 1);
        assert_eq!(color_count, 2);
        assert_eq!(stop_count, 1);
        assert_eq!(end_count, 1);
    }

    #[test]
    fn test_iter_commands_multiple_iterations() {
        let mut pattern = EmbPattern::new();
        pattern.add_thread(EmbThread::from_string("red").unwrap());
        pattern.stitch(10.0, 0.0);
        pattern.trim();
        pattern.end();

        // Test that we can iterate multiple times
        let commands1: Vec<_> = pattern.iter_commands().collect();
        let commands2: Vec<_> = pattern.iter_commands().collect();

        assert_eq!(commands1.len(), commands2.len());
        assert_eq!(commands1.len(), 3);
    }

    // Stitch type tests
    #[test]
    fn test_stitch_type_basic() {
        let stitch = Stitch::new(10.0, 20.0, STITCH);
        assert_eq!(
            stitch.stitch_type(),
            crate::core::constants::StitchType::Normal
        );

        let jump = Stitch::new(10.0, 20.0, JUMP);
        assert_eq!(jump.stitch_type(), crate::core::constants::StitchType::Jump);

        let trim = Stitch::new(10.0, 20.0, TRIM);
        assert_eq!(trim.stitch_type(), crate::core::constants::StitchType::Trim);
    }

    #[test]
    fn test_stitch_type_all_commands() {
        use crate::core::constants::StitchType;

        assert_eq!(
            Stitch::new(0.0, 0.0, STITCH).stitch_type(),
            StitchType::Normal
        );
        assert_eq!(Stitch::new(0.0, 0.0, JUMP).stitch_type(), StitchType::Jump);
        assert_eq!(Stitch::new(0.0, 0.0, TRIM).stitch_type(), StitchType::Trim);
        assert_eq!(
            Stitch::new(0.0, 0.0, COLOR_CHANGE).stitch_type(),
            StitchType::ColorChange
        );
        assert_eq!(Stitch::new(0.0, 0.0, STOP).stitch_type(), StitchType::Stop);
        assert_eq!(Stitch::new(0.0, 0.0, END).stitch_type(), StitchType::End);
        assert_eq!(
            Stitch::new(0.0, 0.0, SEQUIN_EJECT).stitch_type(),
            StitchType::SequinEject
        );
        assert_eq!(
            Stitch::new(0.0, 0.0, SEQUIN_MODE).stitch_type(),
            StitchType::SequinMode
        );
    }

    #[test]
    fn test_stitch_type_with_metadata() {
        use crate::core::constants::StitchType;

        // Commands with metadata in upper bits should still extract correctly
        let stitch = Stitch::new(10.0, 20.0, 0x12345600); // STITCH with metadata
        assert_eq!(stitch.stitch_type(), StitchType::Normal);

        let jump = Stitch::new(10.0, 20.0, 0xFF000001); // JUMP with metadata
        assert_eq!(jump.stitch_type(), StitchType::Jump);
    }

    #[test]
    fn test_stitch_type_helper_methods() {
        let normal = Stitch::new(10.0, 20.0, STITCH);
        assert!(normal.stitch_type().is_movement());
        assert!(!normal.stitch_type().is_thread_command());
        assert!(!normal.stitch_type().is_control());

        let trim = Stitch::new(10.0, 20.0, TRIM);
        assert!(!trim.stitch_type().is_movement());
        assert!(trim.stitch_type().is_thread_command());
        assert!(!trim.stitch_type().is_control());

        let stop = Stitch::new(10.0, 20.0, STOP);
        assert!(!stop.stitch_type().is_movement());
        assert!(stop.stitch_type().is_thread_command());
        assert!(stop.stitch_type().is_control());
    }

    #[test]
    fn test_stitch_type_pattern_usage() {
        use crate::core::constants::StitchType;

        let mut pattern = EmbPattern::new();
        pattern.add_thread(EmbThread::from_string("red").unwrap());
        pattern.stitch(10.0, 0.0);
        pattern.jump(20.0, 0.0);
        pattern.trim();
        pattern.end();

        let types: Vec<StitchType> = pattern.stitches().iter().map(|s| s.stitch_type()).collect();

        assert_eq!(types[0], StitchType::Normal);
        assert_eq!(types[1], StitchType::Jump);
        assert_eq!(types[2], StitchType::Trim);
        assert_eq!(types[3], StitchType::End);
    }

    #[test]
    fn test_calculate_statistics_empty_pattern() {
        let pattern = EmbPattern::new();
        let stats = pattern.calculate_statistics(800.0);

        assert_eq!(stats.stitch_count, 0);
        assert_eq!(stats.jump_count, 0);
        assert_eq!(stats.trim_count, 0);
        assert_eq!(stats.color_change_count, 0);
        assert_eq!(stats.total_length_mm, 0.0);
        assert_eq!(stats.total_length_inches, 0.0);
        assert_eq!(stats.estimated_time_minutes, 0.0);
        assert_eq!(stats.thread_usage.len(), 0);
        assert_eq!(stats.density, 0.0);
        assert_eq!(stats.width_mm, 0.0);
        assert_eq!(stats.height_mm, 0.0);
        assert_eq!(stats.avg_stitch_length_mm, 0.0);
        assert_eq!(stats.max_stitch_length_mm, 0.0);
    }

    #[test]
    fn test_calculate_statistics_basic() {
        let mut pattern = EmbPattern::new();
        pattern.add_thread(EmbThread::from_string("red").unwrap());

        // Add stitches: 100 units = 10mm
        pattern.stitch(100.0, 0.0); // Move to (100, 0)
        pattern.stitch(0.0, 100.0); // Move to (100, 100)

        let stats = pattern.calculate_statistics(800.0);

        assert_eq!(stats.stitch_count, 2);
        assert_eq!(stats.jump_count, 0);
        assert_eq!(stats.trim_count, 0);
        assert_eq!(stats.color_change_count, 0);

        // Total length: 10mm + 10mm = 20mm
        assert!((stats.total_length_mm - 20.0).abs() < 0.1);

        // Inches: 20mm / 25.4 ≈ 0.787
        assert!((stats.total_length_inches - 0.787).abs() < 0.01);

        // Time: 2 stitches / 800 spm = 0.0025 minutes
        assert!((stats.estimated_time_minutes - 0.0025).abs() < 0.0001);

        // Thread usage: 1 thread with 2 stitches
        assert_eq!(stats.thread_usage.len(), 1);
        assert_eq!(stats.thread_usage[0].stitch_count, 2);
        assert!((stats.thread_usage[0].length_mm - 20.0).abs() < 0.1);

        // Bounds: from (100, 0) to (100, 100)
        // Width: 0mm (both stitches have same X), Height: 10mm
        assert_eq!(stats.width_mm, 0.0);
        assert!((stats.height_mm - 10.0).abs() < 0.1);

        // Density: 2 stitches / 0 area = 0 (avoid division by zero)
        // Actually density will be infinity or 0 depending on implementation
        // For zero area, we return 0.0
        assert_eq!(stats.density, 0.0);

        // Avg stitch length: 20mm / 2 = 10mm
        assert!((stats.avg_stitch_length_mm - 10.0).abs() < 0.1);

        // Max stitch length: 10mm
        assert!((stats.max_stitch_length_mm - 10.0).abs() < 0.1);
    }

    #[test]
    fn test_calculate_statistics_multiple_threads() {
        let mut pattern = EmbPattern::new();
        pattern.add_thread(EmbThread::from_string("red").unwrap());
        pattern.add_thread(EmbThread::from_string("blue").unwrap());

        // Red thread stitches
        pattern.stitch(100.0, 0.0); // (100, 0) - 10mm
        pattern.stitch(100.0, 0.0); // (200, 0) - 10mm

        // Color change
        pattern.color_change(0.0, 0.0);

        // Blue thread stitches
        pattern.stitch(0.0, 100.0); // (200, 100) - 10mm
        pattern.stitch(0.0, 100.0); // (200, 200) - 10mm
        pattern.stitch(0.0, 100.0); // (200, 300) - 10mm

        let stats = pattern.calculate_statistics(800.0);

        assert_eq!(stats.stitch_count, 5);
        assert_eq!(stats.color_change_count, 1);

        // Thread usage should show 2 threads even though color_change creates a gap
        // The calculate_thread_usage function tracks by index, not actual thread count
        assert!(stats.thread_usage.len() >= 2);

        // Find red and blue threads in the usage list
        let red_idx = stats
            .thread_usage
            .iter()
            .position(|u| u.stitch_count == 2)
            .expect("Red thread usage not found");
        let blue_idx = stats
            .thread_usage
            .iter()
            .position(|u| u.stitch_count == 3)
            .expect("Blue thread usage not found");

        // Red thread: 2 stitches, 20mm
        assert_eq!(stats.thread_usage[red_idx].stitch_count, 2);
        assert!((stats.thread_usage[red_idx].length_mm - 20.0).abs() < 0.1);

        // Blue thread: 3 stitches, 30mm
        assert_eq!(stats.thread_usage[blue_idx].stitch_count, 3);
        assert!((stats.thread_usage[blue_idx].length_mm - 30.0).abs() < 0.1);

        // Total length: 50mm
        assert!((stats.total_length_mm - 50.0).abs() < 0.1);
    }

    #[test]
    fn test_calculate_statistics_with_jumps_and_trims() {
        let mut pattern = EmbPattern::new();
        pattern.add_thread(EmbThread::from_string("red").unwrap());

        pattern.stitch(100.0, 0.0); // Stitch to (100, 0)
        pattern.jump(100.0, 0.0); // Jump to (200, 0)
        pattern.stitch(100.0, 0.0); // Stitch to (300, 0)
        pattern.trim(); // Trim at (300, 0)

        let stats = pattern.calculate_statistics(800.0);

        assert_eq!(stats.stitch_count, 2);
        assert_eq!(stats.jump_count, 1);
        assert_eq!(stats.trim_count, 1);

        // Thread usage should only count stitches, not jumps
        assert_eq!(stats.thread_usage.len(), 1);
        assert_eq!(stats.thread_usage[0].stitch_count, 2);

        // Thread usage length: only the 2 stitches count (20mm)
        // The jump doesn't contribute to thread usage
        assert!((stats.thread_usage[0].length_mm - 20.0).abs() < 0.1);

        // Total length: total_stitch_length() only counts STITCH commands, not jumps
        // First stitch: 100 units = 10mm
        // Second stitch: 100 units = 10mm
        // Total: 20mm (jump is not included in total_stitch_length)
        assert!((stats.total_length_mm - 20.0).abs() < 0.1);
    }

    #[test]
    fn test_calculate_statistics_custom_machine_speed() {
        let mut pattern = EmbPattern::new();
        pattern.add_thread(EmbThread::from_string("red").unwrap());

        // Add 1000 stitches
        for _ in 0..1000 {
            pattern.stitch(10.0, 0.0);
        }

        // Default speed: 800 spm
        let stats_800 = pattern.calculate_statistics(800.0);
        assert!((stats_800.estimated_time_minutes - 1.25).abs() < 0.01); // 1000/800 = 1.25

        // Fast machine: 1200 spm
        let stats_1200 = pattern.calculate_statistics(1200.0);
        assert!((stats_1200.estimated_time_minutes - 0.833).abs() < 0.01); // 1000/1200 ≈ 0.833

        // Slow machine: 400 spm
        let stats_400 = pattern.calculate_statistics(400.0);
        assert!((stats_400.estimated_time_minutes - 2.5).abs() < 0.01); // 1000/400 = 2.5
    }

    #[test]
    fn test_calculate_statistics_density() {
        let mut pattern = EmbPattern::new();
        pattern.add_thread(EmbThread::from_string("red").unwrap());

        // Create a 10mm x 10mm pattern (1cm x 1cm) with 100 stitches
        // We need to create a grid pattern
        for i in 0..10 {
            for j in 0..10 {
                pattern.stitch_abs((i * 10) as f64, (j * 10) as f64);
            }
        }

        let stats = pattern.calculate_statistics(800.0);

        // 100 stitches total
        assert_eq!(stats.stitch_count, 100);

        // Bounds should be 0 to 90 (10 positions * 10 units/position)
        // Width: 90 units = 9mm = 0.9cm
        // Height: 90 units = 9mm = 0.9cm
        // Area: 0.9cm * 0.9cm = 0.81 cm²
        // Density: 100 / 0.81 ≈ 123.5 stitches/cm²
        assert!((stats.density - 123.5).abs() < 5.0);
    }

    #[test]
    fn test_calculate_statistics_unit_conversions() {
        let mut pattern = EmbPattern::new();
        pattern.add_thread(EmbThread::from_string("red").unwrap());

        // Create a 254mm = 10 inches long stitch
        // Start at (0, 0), stitch to (2540, 0)
        pattern.stitch_abs(0.0, 0.0);
        pattern.stitch_abs(2540.0, 0.0); // 2540 * 0.1mm = 254mm

        let stats = pattern.calculate_statistics(800.0);

        // Length in mm
        assert!((stats.total_length_mm - 254.0).abs() < 0.1);

        // Length in inches: 254mm / 25.4 = 10 inches
        assert!((stats.total_length_inches - 10.0).abs() < 0.01);

        // Width in mm: from 0 to 2540 units = 254mm
        assert!((stats.width_mm - 254.0).abs() < 0.1);

        // Height should be 0 (both Y coordinates are 0)
        assert_eq!(stats.height_mm, 0.0);
    }

    #[test]
    fn test_thread_usage_empty_pattern() {
        let pattern = EmbPattern::new();
        let usage = pattern.calculate_thread_usage();

        assert_eq!(usage.len(), 0);
    }

    #[test]
    fn test_thread_usage_missing_thread() {
        let mut pattern = EmbPattern::new();
        // No thread added, but add stitches
        pattern.stitch(100.0, 0.0);

        let stats = pattern.calculate_statistics(800.0);

        // Should still calculate, using default thread
        assert_eq!(stats.thread_usage.len(), 1);
        assert_eq!(stats.thread_usage[0].stitch_count, 1);
        assert_eq!(stats.thread_usage[0].thread.color, 0x000000); // Default black
    }
}
