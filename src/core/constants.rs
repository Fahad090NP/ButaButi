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

/// Trim thread command
pub const TRIM: u32 = 2;

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

/// Get the name of a command constant
pub fn command_name(command: u32) -> &'static str {
    match command & COMMAND_MASK {
        NO_COMMAND => "NO_COMMAND",
        STITCH => "STITCH",
        JUMP => "JUMP",
        TRIM => "TRIM",
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
    }
}
