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
    pub fn new(x: f64, y: f64, command: u32) -> Self {
        Self { x, y, command }
    }
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
}
