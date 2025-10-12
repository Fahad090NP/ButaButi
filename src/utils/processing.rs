//! Pattern processing and transformation utilities
//!
//! Provides functions for normalizing patterns, calculating statistics, interpolating stitches,
//! and other common pattern manipulation operations used across different file formats.

use crate::core::constants::*;
use crate::core::pattern::{EmbPattern, Stitch};

/// Normalize pattern to start at (0, 0)
///
/// Translates all stitches so the minimum coordinates are at (0, 0).
///
/// # Example
///
/// ```
/// use rusty_petal::pattern::EmbPattern;
/// use rusty_petal::processing;
///
/// let mut pattern = EmbPattern::new();
/// pattern.add_stitch_absolute(0x01, 100.0, 100.0);
/// pattern.add_stitch_absolute(0x01, 200.0, 200.0);
///
/// processing::normalize(&mut pattern);
///
/// let bounds = pattern.bounds();
/// assert_eq!(bounds.0, 0.0); // min_x
/// assert_eq!(bounds.1, 0.0); // min_y
/// ```
pub fn normalize(pattern: &mut EmbPattern) {
    let (min_x, min_y, _, _) = pattern.bounds();

    if min_x != 0.0 || min_y != 0.0 {
        pattern.translate(-min_x, -min_y);
    }
}

/// Fix color count by ensuring each color change has a corresponding thread
///
/// Adds default threads if there are more color changes than threads.
pub fn fix_color_count(pattern: &mut EmbPattern) {
    let mut max_color_index = 0;

    // Count color changes to determine how many threads we need
    for stitch in pattern.stitches() {
        if (stitch.command & COLOR_CHANGE) != 0 {
            max_color_index += 1;
        }
    }

    // Add default threads with cycling colors if needed
    while pattern.threads().len() <= max_color_index {
        use crate::core::thread::EmbThread;
        // Use a default palette color or black
        let color = match pattern.threads().len() % 7 {
            0 => 0x000000, // Black
            1 => 0xFF0000, // Red
            2 => 0x00FF00, // Green
            3 => 0x0000FF, // Blue
            4 => 0xFFFF00, // Yellow
            5 => 0xFF00FF, // Magenta
            6 => 0x00FFFF, // Cyan
            _ => 0x000000,
        };
        pattern.add_thread(EmbThread::new(color));
    }
}

/// Interpolate trim commands with jumps
///
/// For formats that don't support TRIM, this replaces TRIM commands
/// with JUMP commands, optionally adding intermediate jump stitches.
pub fn interpolate_trims(pattern: &mut EmbPattern, max_jump_length: f64) {
    let mut new_stitches = Vec::new();

    for i in 0..pattern.stitches().len() {
        let stitch = &pattern.stitches()[i];

        if (stitch.command & TRIM) != 0 {
            // Replace TRIM with JUMP
            let mut new_stitch = *stitch;
            new_stitch.command = (new_stitch.command & !TRIM) | JUMP;

            // If this is not the first stitch and we need to break long jumps
            if i > 0 && max_jump_length > 0.0 {
                let prev = &pattern.stitches()[i - 1];
                let dx = stitch.x - prev.x;
                let dy = stitch.y - prev.y;
                let distance = (dx * dx + dy * dy).sqrt();

                if distance > max_jump_length && distance > 0.0 {
                    // Add intermediate jumps
                    let num_segments = (distance / max_jump_length).ceil() as usize;

                    // Guard against excessive segmentation or division by zero
                    let num_segments = num_segments.clamp(1, 1000);

                    let step_x = dx / num_segments as f64;
                    let step_y = dy / num_segments as f64;

                    for j in 1..num_segments {
                        new_stitches.push(Stitch {
                            command: JUMP,
                            x: prev.x + step_x * j as f64,
                            y: prev.y + step_y * j as f64,
                        });
                    }
                }
            }

            new_stitches.push(new_stitch);
        } else {
            new_stitches.push(*stitch);
        }
    }

    // Replace pattern stitches
    *pattern = EmbPattern::from_stitches(new_stitches, pattern.threads().to_vec());
}

/// Remove duplicate consecutive stitches
///
/// Removes stitches that are at the same location as the previous stitch,
/// preserving command flags.
pub fn remove_duplicates(pattern: &mut EmbPattern) {
    if pattern.stitches().is_empty() {
        return;
    }

    let mut new_stitches = Vec::new();
    new_stitches.push(pattern.stitches()[0]);

    for i in 1..pattern.stitches().len() {
        let current = &pattern.stitches()[i];
        let previous = &pattern.stitches()[i - 1];

        // Keep stitch if position changed or if it's a command (not just a stitch)
        if current.x != previous.x || current.y != previous.y || (current.command & !STITCH) != 0 {
            new_stitches.push(*current);
        }
    }

    *pattern = EmbPattern::from_stitches(new_stitches, pattern.threads().to_vec());
}

/// Calculate pattern statistics
#[derive(Debug, Clone, PartialEq)]
pub struct PatternStats {
    /// Total number of stitches
    pub stitch_count: usize,
    /// Number of jump stitches
    pub jump_count: usize,
    /// Number of trim commands
    pub trim_count: usize,
    /// Number of color changes
    pub color_change_count: usize,
    /// Total thread length in mm
    pub total_length: f64,
    /// Minimum X coordinate
    pub min_x: f64,
    /// Minimum Y coordinate
    pub min_y: f64,
    /// Maximum X coordinate
    pub max_x: f64,
    /// Maximum Y coordinate
    pub max_y: f64,
}

/// Calculate statistics for a pattern
pub fn calculate_stats(pattern: &EmbPattern) -> PatternStats {
    let (min_x, min_y, max_x, max_y) = pattern.bounds();

    let mut stitch_count = 0;
    let mut jump_count = 0;
    let mut trim_count = 0;
    let mut color_change_count = 0;
    let mut total_length = 0.0;

    let stitches = pattern.stitches();
    for i in 0..stitches.len() {
        let stitch = &stitches[i];
        let cmd = stitch.command & COMMAND_MASK;

        // Count based on primary command
        match cmd {
            STITCH => stitch_count += 1,
            JUMP => jump_count += 1,
            TRIM => trim_count += 1,
            COLOR_CHANGE => color_change_count += 1,
            _ => {},
        }

        // Calculate length from previous stitch (only for STITCH or JUMP)
        if i > 0 && (cmd == STITCH || cmd == JUMP) {
            let prev = &stitches[i - 1];
            let dx = stitch.x - prev.x;
            let dy = stitch.y - prev.y;
            let segment_length = (dx * dx + dy * dy).sqrt();

            // Guard against NaN and infinity
            if segment_length.is_finite() && segment_length >= 0.0 {
                total_length += segment_length * 0.1; // Convert to mm (assuming 0.1mm units)
            }
        }
    }

    PatternStats {
        stitch_count,
        jump_count,
        trim_count,
        color_change_count,
        total_length,
        min_x,
        min_y,
        max_x,
        max_y,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::thread::EmbThread;

    #[test]
    fn test_normalize() {
        let mut pattern = EmbPattern::new();
        pattern.add_stitch_absolute(STITCH, 100.0, 100.0);
        pattern.add_stitch_absolute(STITCH, 200.0, 200.0);

        normalize(&mut pattern);

        let bounds = pattern.bounds();
        assert_eq!(bounds.0, 0.0); // min_x
        assert_eq!(bounds.1, 0.0); // min_y
        assert_eq!(bounds.2, 100.0); // max_x
        assert_eq!(bounds.3, 100.0); // max_y
    }

    #[test]
    fn test_fix_color_count() {
        let mut pattern = EmbPattern::new();
        pattern.add_stitch_absolute(STITCH, 0.0, 0.0);
        pattern.add_stitch_absolute(COLOR_CHANGE, 10.0, 10.0);
        pattern.add_stitch_absolute(STITCH, 20.0, 20.0);
        pattern.add_stitch_absolute(COLOR_CHANGE, 30.0, 30.0);
        pattern.add_stitch_absolute(STITCH, 40.0, 40.0);

        // Initially no threads
        assert_eq!(pattern.threads().len(), 0);

        fix_color_count(&mut pattern);

        // Should have added 3 threads (one for initial, two for color changes)
        assert!(pattern.threads().len() >= 2);
    }

    #[test]
    fn test_remove_duplicates() {
        let mut pattern = EmbPattern::new();
        pattern.add_stitch_absolute(STITCH, 10.0, 10.0);
        pattern.add_stitch_absolute(STITCH, 10.0, 10.0); // Duplicate
        pattern.add_stitch_absolute(STITCH, 20.0, 20.0);
        pattern.add_stitch_absolute(STITCH, 20.0, 20.0); // Duplicate

        assert_eq!(pattern.stitches().len(), 4);

        remove_duplicates(&mut pattern);

        assert_eq!(pattern.stitches().len(), 2);
    }

    #[test]
    fn test_calculate_stats() {
        let mut pattern = EmbPattern::new();
        pattern.add_thread(EmbThread::new(0xFF0000));
        pattern.add_stitch_absolute(STITCH, 0.0, 0.0);
        pattern.add_stitch_absolute(STITCH, 10.0, 0.0);
        pattern.add_stitch_absolute(JUMP, 20.0, 0.0);
        pattern.add_stitch_absolute(TRIM, 20.0, 0.0);
        pattern.add_stitch_absolute(COLOR_CHANGE, 30.0, 0.0);

        let stats = calculate_stats(&pattern);

        assert_eq!(stats.stitch_count, 2);
        assert_eq!(stats.jump_count, 1);
        assert_eq!(stats.trim_count, 1);
        assert_eq!(stats.color_change_count, 1);
        assert_eq!(stats.min_x, 0.0);
        assert_eq!(stats.max_x, 30.0);
    }

    #[test]
    fn test_interpolate_trims() {
        let mut pattern = EmbPattern::new();
        pattern.add_stitch_absolute(STITCH, 0.0, 0.0);
        pattern.add_stitch_absolute(TRIM, 100.0, 0.0);
        pattern.add_stitch_absolute(STITCH, 100.0, 0.0);

        interpolate_trims(&mut pattern, 30.0);

        // TRIM should be replaced with JUMP(s)
        let has_trim = pattern.stitches().iter().any(|s| (s.command & TRIM) != 0);
        assert!(!has_trim);

        // Should have intermediate jumps
        assert!(pattern.stitches().len() > 3);
    }
}
