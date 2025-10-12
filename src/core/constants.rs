//! Constants and command definitions for embroidery operations
//!
//! This module defines all the low-level and middle-level commands used in embroidery patterns.
//! Commands below 0xFF are core commands, while higher values encode additional metadata like
//! thread index, needle number, and sequencing information in the upper 24 bits.

// Command masks
/// Mask for extracting the core command from a command value
pub const COMMAND_MASK: u32 = 0x0000_00FF;

/// Mask for extracting thread information
pub const THREAD_MASK: u32 = 0x0000_FF00;

/// Mask for extracting needle information
pub const NEEDLE_MASK: u32 = 0x00FF_0000;

/// Mask for extracting order information
pub const ORDER_MASK: u32 = 0xFF00_0000;

/// Mask for flags (same as THREAD_MASK for compatibility)
pub const FLAGS_MASK: u32 = 0x0000_FF00;

// Core embroidery commands (Low-level)
/// No command - placeholder
pub const NO_COMMAND: u32 = u32::MAX; // -1 in signed

/// Stitch command - move and drop needle
pub const STITCH: u32 = 0;

/// Jump command - move without dropping needle
pub const JUMP: u32 = 1;

/// Trim thread command - cut thread leaving a tail
///
/// This is the standard thread cut used in most embroidery machines.
/// The thread is cut but a small tail is left for easier threading and to prevent
/// the thread from pulling through the fabric.
pub const TRIM: u32 = 2;

/// Cut thread command - full cut with no tail
///
/// Some embroidery machines and formats support a stronger cut that leaves no tail.
/// This is less common than TRIM and may not be supported by all machines.
/// When not supported, CUT is typically treated the same as TRIM.
///
/// **Formats that use CUT**:
/// - XXX format: 0x88 attribute flag indicates CUT
/// - HUS format: Separate cut flag in some versions
pub const CUT: u32 = 0x20;

/// Stop machine command (for applique, thread change, etc.)
pub const STOP: u32 = 3;

/// End pattern command
pub const END: u32 = 4;

/// Color change command
pub const COLOR_CHANGE: u32 = 5;

/// Sequin mode command
pub const SEQUIN_MODE: u32 = 6;

/// Sequin eject command
pub const SEQUIN_EJECT: u32 = 7;

/// Needle set command (explicit needle selection)
pub const NEEDLE_SET: u32 = 9;

/// Slow speed command (U01 format)
pub const SLOW: u32 = 0x0B;

/// Fast speed command (U01 format)
pub const FAST: u32 = 0x0C;

// Middle-level commands

/// Set change sequence - preset/postset thread change sequence
pub const SET_CHANGE_SEQUENCE: u32 = 0x10;

/// Stitch with implied SEW_TO contingency
pub const SEW_TO: u32 = 0xB0;

/// Stitch with implied NEEDLE_AT contingency
pub const NEEDLE_AT: u32 = 0xB1;

/// Stitch break - reallocate jumps
pub const STITCH_BREAK: u32 = 0xE0;

/// Sequence break - break between stitch sequences
pub const SEQUENCE_BREAK: u32 = 0xE1;

/// Color break - break between color blocks
pub const COLOR_BREAK: u32 = 0xE2;

/// Tie on command
pub const TIE_ON: u32 = 0xE4;

/// Tie off command
pub const TIE_OFF: u32 = 0xE5;

/// Frame eject command
pub const FRAME_EJECT: u32 = 0xE9;

// Matrix transformation commands

/// Translate transformation
pub const MATRIX_TRANSLATE: u32 = 0xC0;

/// Scale from origin
pub const MATRIX_SCALE_ORIGIN: u32 = 0xC1;

/// Rotate from origin
pub const MATRIX_ROTATE_ORIGIN: u32 = 0xC2;

/// Reset matrix to identity
pub const MATRIX_RESET: u32 = 0xC3;

/// Scale from current position
pub const MATRIX_SCALE: u32 = 0xC4;

/// Rotate from current position
pub const MATRIX_ROTATE: u32 = 0xC5;

// Encoder options

/// Set maximum stitch length option
pub const OPTION_MAX_STITCH_LENGTH: u32 = 0xD5;

/// Set maximum jump length option
pub const OPTION_MAX_JUMP_LENGTH: u32 = 0xD6;

/// Enable explicit trim before color change
pub const OPTION_EXPLICIT_TRIM: u32 = 0xD7;

/// Trim is implicit in color change
pub const OPTION_IMPLICIT_TRIM: u32 = 0xD8;

// Tie-on contingencies

/// No tie-on
pub const CONTINGENCY_TIE_ON_NONE: u32 = 0xD3;

/// Tie on with three small stitches
pub const CONTINGENCY_TIE_ON_THREE_SMALL: u32 = 0xD1;

// Tie-off contingencies

/// No tie-off
pub const CONTINGENCY_TIE_OFF_NONE: u32 = 0xD4;

/// Tie off with three small stitches
pub const CONTINGENCY_TIE_OFF_THREE_SMALL: u32 = 0xD2;

// Long stitch contingencies

/// No contingency for long stitches
pub const CONTINGENCY_LONG_STITCH_NONE: u32 = 0xF0;

/// Jump needle to destination
pub const CONTINGENCY_LONG_STITCH_JUMP_NEEDLE: u32 = 0xF1;

/// Sew to destination with interpolated stitches
pub const CONTINGENCY_LONG_STITCH_SEW_TO: u32 = 0xF2;

// Sequin contingencies

/// Utilize sequin commands as-is
pub const CONTINGENCY_SEQUIN_UTILIZE: u32 = 0xF5;

/// Convert sequins to jumps
pub const CONTINGENCY_SEQUIN_JUMP: u32 = 0xF6;

/// Convert sequins to stitches
pub const CONTINGENCY_SEQUIN_STITCH: u32 = 0xF7;

/// Remove sequin commands
pub const CONTINGENCY_SEQUIN_REMOVE: u32 = 0xF8;

// Generic flags

/// Generic alternative form flag
pub const ALTERNATIVE: u32 = 0x100;

/// Categorization of stitch types based on command
///
/// This enum provides a high-level categorization of stitch commands,
/// making it easier to work with patterns without dealing with raw command constants.
///
/// # Example
///
/// ```
/// use butabuti::core::constants::StitchType;
/// use butabuti::core::pattern::Stitch;
///
/// let stitch = Stitch::new(10.0, 20.0, butabuti::core::constants::JUMP);
/// assert_eq!(stitch.stitch_type(), StitchType::Jump);
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum StitchType {
    /// Normal stitch - move and drop needle
    Normal,
    /// Jump stitch - move without dropping needle
    Jump,
    /// Trim thread command - cut with tail
    Trim,
    /// Cut thread command - full cut with no tail
    Cut,
    /// Color change command
    ColorChange,
    /// Stop machine command (for applique, manual changes, etc.)
    Stop,
    /// End of pattern command
    End,
    /// Sequin eject command
    SequinEject,
    /// Sequin mode command
    SequinMode,
    /// Needle set command (explicit needle selection)
    NeedleSet,
    /// Slow speed command
    Slow,
    /// Fast speed command
    Fast,
    /// Unknown or custom command
    Unknown,
}

impl StitchType {
    /// Get the StitchType from a command value
    ///
    /// # Example
    ///
    /// ```
    /// use butabuti::core::constants::{StitchType, JUMP, TRIM};
    ///
    /// assert_eq!(StitchType::from_command(JUMP), StitchType::Jump);
    /// assert_eq!(StitchType::from_command(TRIM), StitchType::Trim);
    /// ```
    #[inline]
    pub fn from_command(command: u32) -> Self {
        match command & COMMAND_MASK {
            STITCH => StitchType::Normal,
            JUMP => StitchType::Jump,
            TRIM => StitchType::Trim,
            CUT => StitchType::Cut,
            COLOR_CHANGE => StitchType::ColorChange,
            STOP => StitchType::Stop,
            END => StitchType::End,
            SEQUIN_EJECT => StitchType::SequinEject,
            SEQUIN_MODE => StitchType::SequinMode,
            NEEDLE_SET => StitchType::NeedleSet,
            SLOW => StitchType::Slow,
            FAST => StitchType::Fast,
            _ => StitchType::Unknown,
        }
    }

    /// Check if this stitch type represents actual needle movement
    ///
    /// Returns true for Normal, Jump, and ColorChange (which includes a stitch)
    #[inline]
    pub fn is_movement(&self) -> bool {
        matches!(
            self,
            StitchType::Normal | StitchType::Jump | StitchType::ColorChange
        )
    }

    /// Check if this is a thread management command
    ///
    /// Returns true for Trim, Cut, ColorChange, and Stop
    #[inline]
    pub fn is_thread_command(&self) -> bool {
        matches!(
            self,
            StitchType::Trim | StitchType::Cut | StitchType::ColorChange | StitchType::Stop
        )
    }

    /// Check if this is a control command
    ///
    /// Returns true for Stop, End, Slow, and Fast
    #[inline]
    pub fn is_control(&self) -> bool {
        matches!(
            self,
            StitchType::Stop | StitchType::End | StitchType::Slow | StitchType::Fast
        )
    }

    /// Check if this is a sequin-related command
    #[inline]
    pub fn is_sequin(&self) -> bool {
        matches!(self, StitchType::SequinEject | StitchType::SequinMode)
    }
}

impl std::fmt::Display for StitchType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            StitchType::Normal => write!(f, "Normal"),
            StitchType::Jump => write!(f, "Jump"),
            StitchType::Trim => write!(f, "Trim"),
            StitchType::Cut => write!(f, "Cut"),
            StitchType::ColorChange => write!(f, "ColorChange"),
            StitchType::Stop => write!(f, "Stop"),
            StitchType::End => write!(f, "End"),
            StitchType::SequinEject => write!(f, "SequinEject"),
            StitchType::SequinMode => write!(f, "SequinMode"),
            StitchType::NeedleSet => write!(f, "NeedleSet"),
            StitchType::Slow => write!(f, "Slow"),
            StitchType::Fast => write!(f, "Fast"),
            StitchType::Unknown => write!(f, "Unknown"),
        }
    }
}

/// Get the name of a command constant
pub fn command_name(command: u32) -> &'static str {
    match command & COMMAND_MASK {
        NO_COMMAND => "NO_COMMAND",
        STITCH => "STITCH",
        JUMP => "JUMP",
        TRIM => "TRIM",
        CUT => "CUT",
        STOP => "STOP",
        END => "END",
        COLOR_CHANGE => "COLOR_CHANGE",
        SEQUIN_MODE => "SEQUIN_MODE",
        SEQUIN_EJECT => "SEQUIN_EJECT",
        NEEDLE_SET => "NEEDLE_SET",
        SLOW => "SLOW",
        FAST => "FAST",
        SET_CHANGE_SEQUENCE => "SET_CHANGE_SEQUENCE",
        SEW_TO => "SEW_TO",
        NEEDLE_AT => "NEEDLE_AT",
        STITCH_BREAK => "STITCH_BREAK",
        SEQUENCE_BREAK => "SEQUENCE_BREAK",
        COLOR_BREAK => "COLOR_BREAK",
        TIE_ON => "TIE_ON",
        TIE_OFF => "TIE_OFF",
        FRAME_EJECT => "FRAME_EJECT",
        MATRIX_TRANSLATE => "MATRIX_TRANSLATE",
        MATRIX_SCALE_ORIGIN => "MATRIX_SCALE_ORIGIN",
        MATRIX_ROTATE_ORIGIN => "MATRIX_ROTATE_ORIGIN",
        MATRIX_RESET => "MATRIX_RESET",
        MATRIX_SCALE => "MATRIX_SCALE",
        MATRIX_ROTATE => "MATRIX_ROTATE",
        _ => "UNKNOWN",
    }
}

/// Check if a command is valid (within the defined range)
#[inline]
pub fn is_valid_command(command: u32) -> bool {
    let cmd = command & COMMAND_MASK;
    // Allow all values as formats may define custom commands
    // Just ensure it's within u8 range for the command byte
    cmd <= 0xFF
}

/// Extract the core command from a full command value
#[inline]
pub fn extract_command(command: u32) -> u32 {
    command & COMMAND_MASK
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_command_mask() {
        let encoded = 0x12345678u32;
        assert_eq!(encoded & COMMAND_MASK, 0x78);
    }

    #[test]
    fn test_command_names() {
        assert_eq!(command_name(STITCH), "STITCH");
        assert_eq!(command_name(JUMP), "JUMP");
        assert_eq!(command_name(TRIM), "TRIM");
        assert_eq!(command_name(CUT), "CUT");
    }

    #[test]
    fn test_is_valid_command() {
        assert!(is_valid_command(STITCH));
        assert!(is_valid_command(0xFF));
        assert!(is_valid_command(0x12345678)); // Upper bits don't affect validity
    }

    #[test]
    fn test_extract_command() {
        assert_eq!(extract_command(0x12345678), 0x78);
        assert_eq!(extract_command(STITCH), STITCH);
    }

    // StitchType tests
    #[test]
    fn test_stitch_type_from_command() {
        assert_eq!(StitchType::from_command(STITCH), StitchType::Normal);
        assert_eq!(StitchType::from_command(JUMP), StitchType::Jump);
        assert_eq!(StitchType::from_command(TRIM), StitchType::Trim);
        assert_eq!(StitchType::from_command(CUT), StitchType::Cut);
        assert_eq!(
            StitchType::from_command(COLOR_CHANGE),
            StitchType::ColorChange
        );
        assert_eq!(StitchType::from_command(STOP), StitchType::Stop);
        assert_eq!(StitchType::from_command(END), StitchType::End);
        assert_eq!(
            StitchType::from_command(SEQUIN_EJECT),
            StitchType::SequinEject
        );
        assert_eq!(
            StitchType::from_command(SEQUIN_MODE),
            StitchType::SequinMode
        );
        assert_eq!(StitchType::from_command(NEEDLE_SET), StitchType::NeedleSet);
        assert_eq!(StitchType::from_command(SLOW), StitchType::Slow);
        assert_eq!(StitchType::from_command(FAST), StitchType::Fast);
    }

    #[test]
    fn test_stitch_type_from_command_with_metadata() {
        // Commands with upper bits should still extract correctly
        assert_eq!(StitchType::from_command(0x12345600), StitchType::Normal);
        assert_eq!(StitchType::from_command(0xFF000001), StitchType::Jump);
        assert_eq!(StitchType::from_command(0x00FF0002), StitchType::Trim);
    }

    #[test]
    fn test_stitch_type_unknown() {
        assert_eq!(StitchType::from_command(0xFF), StitchType::Unknown);
        assert_eq!(StitchType::from_command(0x99), StitchType::Unknown);
    }

    #[test]
    fn test_stitch_type_is_movement() {
        assert!(StitchType::Normal.is_movement());
        assert!(StitchType::Jump.is_movement());
        assert!(StitchType::ColorChange.is_movement());
        assert!(!StitchType::Trim.is_movement());
        assert!(!StitchType::Cut.is_movement());
        assert!(!StitchType::Stop.is_movement());
        assert!(!StitchType::End.is_movement());
    }

    #[test]
    fn test_stitch_type_is_thread_command() {
        assert!(StitchType::Trim.is_thread_command());
        assert!(StitchType::Cut.is_thread_command());
        assert!(StitchType::ColorChange.is_thread_command());
        assert!(StitchType::Stop.is_thread_command());
        assert!(!StitchType::Normal.is_thread_command());
        assert!(!StitchType::Jump.is_thread_command());
        assert!(!StitchType::End.is_thread_command());
    }

    #[test]
    fn test_stitch_type_is_control() {
        assert!(StitchType::Stop.is_control());
        assert!(StitchType::End.is_control());
        assert!(StitchType::Slow.is_control());
        assert!(StitchType::Fast.is_control());
        assert!(!StitchType::Normal.is_control());
        assert!(!StitchType::Jump.is_control());
        assert!(!StitchType::Trim.is_control());
    }

    #[test]
    fn test_stitch_type_is_sequin() {
        assert!(StitchType::SequinEject.is_sequin());
        assert!(StitchType::SequinMode.is_sequin());
        assert!(!StitchType::Normal.is_sequin());
        assert!(!StitchType::Jump.is_sequin());
    }

    #[test]
    fn test_stitch_type_display() {
        assert_eq!(format!("{}", StitchType::Normal), "Normal");
        assert_eq!(format!("{}", StitchType::Jump), "Jump");
        assert_eq!(format!("{}", StitchType::Trim), "Trim");
        assert_eq!(format!("{}", StitchType::ColorChange), "ColorChange");
        assert_eq!(format!("{}", StitchType::Unknown), "Unknown");
    }

    #[test]
    fn test_stitch_type_equality() {
        assert_eq!(StitchType::Normal, StitchType::Normal);
        assert_ne!(StitchType::Normal, StitchType::Jump);

        // Test with from_command
        assert_eq!(
            StitchType::from_command(STITCH),
            StitchType::from_command(STITCH)
        );
        assert_ne!(
            StitchType::from_command(STITCH),
            StitchType::from_command(JUMP)
        );
    }

    #[test]
    fn test_cut_command() {
        // Test CUT constant
        assert_eq!(CUT, 0x20);
        assert_eq!(command_name(CUT), "CUT");

        // Test StitchType::Cut
        assert_eq!(StitchType::from_command(CUT), StitchType::Cut);
        assert_eq!(format!("{}", StitchType::Cut), "Cut");

        // Test that CUT is a thread command
        assert!(StitchType::Cut.is_thread_command());
        assert!(!StitchType::Cut.is_movement());
        assert!(!StitchType::Cut.is_control());
        assert!(!StitchType::Cut.is_sequin());
    }

    #[test]
    fn test_cut_vs_trim() {
        // Verify CUT and TRIM are distinct
        assert_ne!(CUT, TRIM);
        assert_ne!(StitchType::Cut, StitchType::Trim);

        // Both are thread commands
        assert!(StitchType::Cut.is_thread_command());
        assert!(StitchType::Trim.is_thread_command());

        // Test extraction from commands with metadata
        let cut_with_meta = CUT | 0xFF00;
        let trim_with_meta = TRIM | 0xFF00;
        assert_eq!(StitchType::from_command(cut_with_meta), StitchType::Cut);
        assert_eq!(StitchType::from_command(trim_with_meta), StitchType::Trim);
    }
}
