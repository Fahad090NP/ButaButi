//! Helper functions for encoding and decoding embroidery commands
//!
//! Provides utilities for packing metadata (thread index, needle number, order) into the
//! upper 24 bits of u32 command values, and extracting this information when needed.

use crate::core::constants::*;

/// Encode a thread change command with optional thread, needle, and order information
///
/// # Arguments
///
/// * `command` - The base command (e.g., COLOR_CHANGE, NEEDLE_SET)
/// * `thread` - Optional thread index (0-254, None means no thread specified)
/// * `needle` - Optional needle index (0-254, None means no needle specified)
/// * `order` - Optional order index (0-254, None means no order specified)
///
/// # Returns
///
/// The encoded command with thread/needle/order information in the high bits
///
/// # Example
///
/// ```
/// use rusty_petal::functions::encode_thread_change;
/// use rusty_petal::constants::*;
///
/// let cmd = encode_thread_change(COLOR_CHANGE, Some(2), None, None);
/// ```
pub fn encode_thread_change(
    command: u32,
    thread: Option<u8>,
    needle: Option<u8>,
    order: Option<u8>,
) -> u32 {
    // Values are stored as (index + 1) so 0 can represent None
    let thread_val = match thread {
        Some(t) => (t.wrapping_add(1)) as u32,
        None => 0,
    };

    let needle_val = match needle {
        Some(n) => (n.wrapping_add(1)) as u32,
        None => 0,
    };

    let order_val = match order {
        Some(o) => (o.wrapping_add(1)) as u32,
        None => 0,
    };

    // Pack into u32: [order:8][needle:8][thread:8][command:8]
    (command & COMMAND_MASK) | (order_val << 24) | (needle_val << 16) | (thread_val << 8)
}

/// Decode an embroidery command into its components
///
/// # Arguments
///
/// * `command` - The encoded command value
///
/// # Returns
///
/// A tuple of (command, thread, needle, order) where each value is `Option<u8>`
///
/// # Example
///
/// ```
/// use rusty_petal::functions::decode_embroidery_command;
/// use rusty_petal::constants::*;
///
/// let (cmd, thread, needle, order) = decode_embroidery_command(COLOR_CHANGE);
/// assert_eq!(cmd, COLOR_CHANGE);
/// assert_eq!(thread, None);
/// ```
pub fn decode_embroidery_command(command: u32) -> (u32, Option<u8>, Option<u8>, Option<u8>) {
    let flag = command & COMMAND_MASK;

    // Extract and decode thread (subtract 1 since we added 1 during encoding)
    let thread = {
        let mut t = ((command & THREAD_MASK) >> 8) as i32;
        t -= 1;
        if t == -1 {
            None
        } else {
            Some(t as u8)
        }
    };

    let needle = {
        let mut n = ((command & NEEDLE_MASK) >> 16) as i32;
        n -= 1;
        if n == -1 {
            None
        } else {
            Some(n as u8)
        }
    };

    let order = {
        let mut o = ((command & ORDER_MASK) >> 24) as i32;
        o -= 1;
        if o == -1 {
            None
        } else {
            Some(o as u8)
        }
    };

    (flag, thread, needle, order)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encode_thread_change_basic() {
        let cmd = encode_thread_change(COLOR_CHANGE, None, None, None);
        assert_eq!(cmd & COMMAND_MASK, COLOR_CHANGE);
    }

    #[test]
    fn test_encode_thread_change_with_values() {
        let cmd = encode_thread_change(COLOR_CHANGE, Some(5), Some(3), Some(10));
        let (flag, thread, needle, order) = decode_embroidery_command(cmd);

        assert_eq!(flag, COLOR_CHANGE);
        assert_eq!(thread, Some(5));
        assert_eq!(needle, Some(3));
        assert_eq!(order, Some(10));
    }

    #[test]
    fn test_decode_basic_command() {
        let (flag, thread, needle, order) = decode_embroidery_command(STITCH);

        assert_eq!(flag, STITCH);
        assert_eq!(thread, None);
        assert_eq!(needle, None);
        assert_eq!(order, None);
    }

    #[test]
    fn test_round_trip() {
        let original = encode_thread_change(NEEDLE_SET, Some(15), Some(7), Some(2));
        let (flag, thread, needle, order) = decode_embroidery_command(original);
        let reconstructed = encode_thread_change(flag, thread, needle, order);

        assert_eq!(original, reconstructed);
    }
}
