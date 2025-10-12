//! Pattern encoding and transcoding
//!
//! This module provides functionality to encode patterns for writing to files,
//! applying transformations, and handling various contingencies.

use crate::core::constants::*;
use crate::core::matrix::EmbMatrix;
use crate::core::pattern::EmbPattern;
use crate::utils::error::Result;

/// Encoder settings for pattern transcoding
#[derive(Debug, Clone)]
pub struct EncoderSettings {
    /// Maximum stitch length
    pub max_stitch: f64,

    /// Maximum jump length
    pub max_jump: f64,

    /// Whether to use full jumps
    pub full_jump: bool,

    /// Whether to round coordinates
    pub round: bool,

    /// Number of needles available
    pub needle_count: usize,

    /// Thread change command to use
    pub thread_change_command: u32,

    /// Sequin contingency mode
    pub sequin_contingency: u32,

    /// Long stitch contingency mode
    pub long_stitch_contingency: u32,

    /// Tie-on contingency
    pub tie_on_contingency: u32,

    /// Tie-off contingency
    pub tie_off_contingency: u32,

    /// Write speed commands
    pub writes_speeds: bool,

    /// Explicit trim before color change
    pub explicit_trim: bool,
}

impl Default for EncoderSettings {
    fn default() -> Self {
        Self {
            max_stitch: f64::INFINITY,
            max_jump: f64::INFINITY,
            full_jump: false,
            round: false,
            needle_count: 5,
            thread_change_command: COLOR_CHANGE,
            sequin_contingency: CONTINGENCY_SEQUIN_JUMP,
            long_stitch_contingency: CONTINGENCY_LONG_STITCH_JUMP_NEEDLE,
            tie_on_contingency: CONTINGENCY_TIE_ON_NONE,
            tie_off_contingency: CONTINGENCY_TIE_OFF_NONE,
            writes_speeds: true,
            explicit_trim: false,
        }
    }
}

/// Pattern encoder/transcoder
pub struct Transcoder {
    settings: EncoderSettings,
    matrix: EmbMatrix,
}

impl Transcoder {
    /// Create a new transcoder with default settings
    pub fn new() -> Self {
        Self {
            settings: EncoderSettings::default(),
            matrix: EmbMatrix::new(),
        }
    }

    /// Create a new transcoder with custom settings
    pub fn with_settings(settings: EncoderSettings) -> Self {
        Self {
            settings,
            matrix: EmbMatrix::new(),
        }
    }

    /// Transcode a pattern, applying transformations and handling contingencies
    pub fn transcode(&mut self, source: &EmbPattern, destination: &mut EmbPattern) -> Result<()> {
        // Copy metadata
        for (key, value) in source.extras() {
            destination.add_metadata(key, value);
        }

        // Copy threads
        for thread in source.threads() {
            destination.add_thread(thread.clone());
        }

        // Check if matrix is non-identity
        let has_transform = !self.matrix.is_identity();

        // Process stitches with transformations
        let mut current_x = 0.0;
        let mut current_y = 0.0;

        for stitch in source.stitches() {
            let command = stitch.command & COMMAND_MASK;

            // Apply matrix transformation if any
            let (x, y) = if has_transform {
                self.matrix.transform_point(stitch.x, stitch.y)
            } else {
                (stitch.x, stitch.y)
            };

            // Round coordinates if requested
            let (x, y) = if self.settings.round {
                (x.round(), y.round())
            } else {
                (x, y)
            };

            match command {
                STITCH => {
                    self.handle_stitch(destination, &mut current_x, &mut current_y, x, y)?;
                },
                JUMP => {
                    self.handle_move(destination, &mut current_x, &mut current_y, x, y)?;
                },
                COLOR_CHANGE => {
                    if self.settings.explicit_trim {
                        destination.add_command(TRIM, current_x, current_y);
                    }
                    destination.add_command(command, x, y);
                    current_x = x;
                    current_y = y;
                },
                NEEDLE_SET => {
                    destination.add_command(command, x, y);
                    current_x = x;
                    current_y = y;
                },
                _ => {
                    // Handle other commands
                    if command == SEQUIN_MODE || command == SEQUIN_EJECT {
                        self.handle_sequin(destination, command, x, y)?;
                    } else if (command == SLOW || command == FAST) && self.settings.writes_speeds {
                        destination.add_command(command, x, y);
                    } else {
                        // Pass through other commands
                        destination.add_command(command, x, y);
                    }
                    current_x = x;
                    current_y = y;
                },
            }
        }

        Ok(())
    }

    /// Handle a stitch command with long stitch contingency
    fn handle_stitch(
        &self,
        destination: &mut EmbPattern,
        current_x: &mut f64,
        current_y: &mut f64,
        target_x: f64,
        target_y: f64,
    ) -> Result<()> {
        let dx = target_x - *current_x;
        let dy = target_y - *current_y;
        let distance = (dx * dx + dy * dy).sqrt();

        // Guard against NaN and infinity
        if !distance.is_finite() {
            *current_x = target_x;
            *current_y = target_y;
            return Ok(());
        }

        if distance > self.settings.max_stitch && distance > 0.0 {
            // Long stitch - apply contingency
            match self.settings.long_stitch_contingency {
                CONTINGENCY_LONG_STITCH_JUMP_NEEDLE => {
                    // Jump to position with needle
                    self.handle_move(destination, current_x, current_y, target_x, target_y)?;
                    destination.add_stitch_absolute(STITCH, target_x, target_y);
                },
                CONTINGENCY_LONG_STITCH_SEW_TO => {
                    // Sew incrementally to target
                    self.sew_to(destination, current_x, current_y, target_x, target_y)?;
                },
                _ => {
                    // Default: just add the stitch
                    destination.add_stitch_absolute(STITCH, target_x, target_y);
                },
            }
        } else {
            destination.add_stitch_absolute(STITCH, target_x, target_y);
        }

        *current_x = target_x;
        *current_y = target_y;
        Ok(())
    }

    /// Handle a move/jump command
    fn handle_move(
        &self,
        destination: &mut EmbPattern,
        current_x: &mut f64,
        current_y: &mut f64,
        target_x: f64,
        target_y: f64,
    ) -> Result<()> {
        let dx = target_x - *current_x;
        let dy = target_y - *current_y;
        let distance = (dx * dx + dy * dy).sqrt();

        // Guard against NaN and infinity
        if !distance.is_finite() {
            *current_x = target_x;
            *current_y = target_y;
            return Ok(());
        }

        if distance > self.settings.max_jump && distance > 0.0 {
            // Jump is too long - break it into smaller jumps
            let steps = (distance / self.settings.max_jump).ceil() as usize;
            let step_x = dx / steps as f64;
            let step_y = dy / steps as f64;

            for i in 1..=steps {
                let jump_x = *current_x + step_x * i as f64;
                let jump_y = *current_y + step_y * i as f64;
                destination.add_stitch_absolute(JUMP, jump_x, jump_y);
            }
        } else {
            destination.add_stitch_absolute(JUMP, target_x, target_y);
        }

        *current_x = target_x;
        *current_y = target_y;
        Ok(())
    }

    /// Sew incrementally to a target position
    fn sew_to(
        &self,
        destination: &mut EmbPattern,
        current_x: &mut f64,
        current_y: &mut f64,
        target_x: f64,
        target_y: f64,
    ) -> Result<()> {
        let dx = target_x - *current_x;
        let dy = target_y - *current_y;
        let distance = (dx * dx + dy * dy).sqrt();

        // Guard against NaN, infinity, and division by zero
        if !distance.is_finite() || distance == 0.0 || self.settings.max_stitch <= 0.0 {
            destination.add_stitch_absolute(STITCH, target_x, target_y);
            *current_x = target_x;
            *current_y = target_y;
            return Ok(());
        }

        let steps = (distance / self.settings.max_stitch).ceil() as usize;
        let steps = steps.clamp(1, 10000); // Prevent excessive loops

        if steps <= 1 {
            destination.add_stitch_absolute(STITCH, target_x, target_y);
        } else {
            let step_x = dx / steps as f64;
            let step_y = dy / steps as f64;

            for i in 1..=steps {
                let stitch_x = *current_x + step_x * i as f64;
                let stitch_y = *current_y + step_y * i as f64;
                destination.add_stitch_absolute(STITCH, stitch_x, stitch_y);
            }
        }

        *current_x = target_x;
        *current_y = target_y;
        Ok(())
    }

    /// Handle sequin commands based on contingency setting
    fn handle_sequin(
        &self,
        destination: &mut EmbPattern,
        command: u32,
        x: f64,
        y: f64,
    ) -> Result<()> {
        match self.settings.sequin_contingency {
            CONTINGENCY_SEQUIN_UTILIZE => {
                // Use sequin commands as-is
                destination.add_command(command, x, y);
            },
            CONTINGENCY_SEQUIN_JUMP => {
                // Convert sequin to jump
                if command == SEQUIN_EJECT {
                    destination.add_command(TRIM, x, y);
                }
            },
            CONTINGENCY_SEQUIN_STITCH => {
                // Convert sequin to stitch
                if command == SEQUIN_EJECT {
                    destination.add_stitch_absolute(STITCH, x, y);
                }
            },
            CONTINGENCY_SEQUIN_REMOVE => {
                // Simply ignore sequin commands
            },
            _ => {
                destination.add_command(command, x, y);
            },
        }
        Ok(())
    }

    /// Set the transformation matrix
    pub fn set_matrix(&mut self, matrix: EmbMatrix) {
        self.matrix = matrix;
    }

    /// Get a mutable reference to the settings
    pub fn settings_mut(&mut self) -> &mut EncoderSettings {
        &mut self.settings
    }

    /// Get a reference to the settings
    pub fn settings(&self) -> &EncoderSettings {
        &self.settings
    }
}

impl Default for Transcoder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_settings() {
        let settings = EncoderSettings::default();
        assert_eq!(settings.thread_change_command, COLOR_CHANGE);
        assert_eq!(settings.needle_count, 5);
    }

    #[test]
    fn test_transcoder_creation() {
        let transcoder = Transcoder::new();
        assert_eq!(transcoder.settings.needle_count, 5);
    }

    #[test]
    fn test_transcode_basic() {
        let mut source = EmbPattern::new();
        source.add_thread(crate::core::thread::EmbThread::new(0xFF0000));
        source.add_stitch_absolute(STITCH, 10.0, 10.0);
        source.add_stitch_absolute(STITCH, 20.0, 20.0);
        source.end();

        let mut destination = EmbPattern::new();
        let mut transcoder = Transcoder::new();

        let result = transcoder.transcode(&source, &mut destination);
        assert!(result.is_ok());
        assert_eq!(destination.threads().len(), 1);
        assert!(destination.stitches().len() >= 2);
    }

    #[test]
    fn test_transcode_with_matrix() {
        let mut source = EmbPattern::new();
        source.add_thread(crate::core::thread::EmbThread::new(0x00FF00));
        source.add_stitch_absolute(STITCH, 10.0, 0.0);
        source.end();

        let mut destination = EmbPattern::new();
        let mut transcoder = Transcoder::new();

        // Apply 90-degree rotation using post_rotate
        let mut matrix = EmbMatrix::new();
        matrix.post_rotate(90.0, 0.0, 0.0);
        transcoder.set_matrix(matrix);

        transcoder.transcode(&source, &mut destination).unwrap();

        // After 90-degree rotation, (10, 0) should become approximately (0, 10)
        let stitches = destination.stitches();
        if !stitches.is_empty() {
            let first_stitch = &stitches[0];
            assert!((first_stitch.x - 0.0).abs() < 0.1);
            assert!((first_stitch.y - 10.0).abs() < 0.1);
        }
    }

    #[test]
    fn test_long_stitch_contingency() {
        let mut source = EmbPattern::new();
        source.add_thread(crate::core::thread::EmbThread::new(0x0000FF));
        source.add_stitch_absolute(STITCH, 0.0, 0.0);
        source.add_stitch_absolute(STITCH, 100.0, 100.0);
        source.end();

        let mut destination = EmbPattern::new();
        let mut transcoder = Transcoder::new();

        // Set max stitch to 50 - should break long stitch
        transcoder.settings_mut().max_stitch = 50.0;
        transcoder.settings_mut().long_stitch_contingency = CONTINGENCY_LONG_STITCH_SEW_TO;

        transcoder.transcode(&source, &mut destination).unwrap();

        // Should have more stitches due to breaking long stitch
        assert!(destination.stitches().len() > source.stitches().len());
    }

    #[test]
    fn test_metadata_copy() {
        let mut source = EmbPattern::new();
        source.add_metadata("author", "Test Author");
        source.add_metadata("title", "Test Pattern");
        source.add_thread(crate::core::thread::EmbThread::new(0xFFFF00));
        source.end();

        let mut destination = EmbPattern::new();
        let mut transcoder = Transcoder::new();

        transcoder.transcode(&source, &mut destination).unwrap();

        assert_eq!(
            destination.get_metadata("author").map(|s| s.as_str()),
            Some("Test Author")
        );
        assert_eq!(
            destination.get_metadata("title").map(|s| s.as_str()),
            Some("Test Pattern")
        );
    }
}
