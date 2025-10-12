//! UTF-8 string utilities for embroidery format handling.
//!
//! This module provides string manipulation functions commonly needed when reading
//! and writing embroidery file formats, which often use C-style null-terminated
//! strings and fixed-width fields. Also includes a byte iterator with error handling
//! for convenient format parsing.

use std::io::{Bytes, Read};

/// Iterator over bytes from a reader with error handling.
///
/// This wrapper around `std::io::Bytes` provides convenient error handling
/// by storing errors internally and allowing clean iteration without Result unwrapping.
/// Useful for format readers that need to process byte streams.
///
/// # Examples
///
/// ```
/// use std::io::Cursor;
/// use butabuti::utils::string::ReadByteIterator;
///
/// let data = vec![1, 2, 3, 4, 5];
/// let reader = Cursor::new(data);
/// let mut iter = ReadByteIterator::new(reader);
///
/// assert_eq!(iter.next(), Some(1));
/// assert_eq!(iter.next(), Some(2));
/// assert_eq!(iter.next(), Some(3));
/// assert!(!iter.closed);
/// ```
pub struct ReadByteIterator<T: Read> {
    reader: Bytes<T>,
    /// Whether the iterator has been closed (EOF or error)
    pub closed: bool,
    /// Error encountered during iteration, if any
    pub error: Option<std::io::Error>,
}

impl<T: Read> ReadByteIterator<T> {
    /// Create a new byte iterator from a reader.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::io::Cursor;
    /// use butabuti::utils::string::ReadByteIterator;
    ///
    /// let data = vec![65, 66, 67];  // "ABC"
    /// let reader = Cursor::new(data);
    /// let iter = ReadByteIterator::new(reader);
    /// ```
    #[allow(clippy::unbuffered_bytes)] // This is intentionally a bytes iterator wrapper
    pub fn new(reader: T) -> Self {
        Self {
            reader: reader.bytes(),
            closed: false,
            error: None,
        }
    }

    /// Mark the iterator as closed.
    fn close(&mut self) {
        self.closed = true;
    }

    /// Check if an error occurred during iteration.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::io::Cursor;
    /// use butabuti::utils::string::ReadByteIterator;
    ///
    /// let data = vec![1, 2, 3];
    /// let reader = Cursor::new(data);
    /// let mut iter = ReadByteIterator::new(reader);
    ///
    /// // Read all bytes
    /// while let Some(_) = iter.next() {}
    ///
    /// assert!(!iter.has_error());
    /// ```
    pub fn has_error(&self) -> bool {
        self.error.is_some()
    }
}

impl<T: Read> Iterator for ReadByteIterator<T> {
    type Item = u8;

    fn next(&mut self) -> Option<u8> {
        if self.closed {
            return None;
        }

        match self.reader.next() {
            Some(Ok(value)) => Some(value),
            Some(Err(error)) => {
                self.error = Some(error);
                self.close();
                None
            },
            None => {
                self.close();
                None
            },
        }
    }
}

/// Trim null bytes and whitespace from C-style null-terminated strings.
///
/// Many embroidery formats use C-style strings with null padding. This function
/// removes trailing null bytes and any remaining whitespace.
///
/// # Examples
///
/// ```
/// use butabuti::utils::string::c_trim;
///
/// assert_eq!(c_trim("Hello\0\0\0"), "Hello");
/// assert_eq!(c_trim("Test  \0\0"), "Test");
/// assert_eq!(c_trim("\0\0\0"), "");
/// ```
pub fn c_trim(s: &str) -> &str {
    s.trim_end_matches('\0').trim()
}

/// Truncate string to maximum number of characters, respecting UTF-8 boundaries.
///
/// Unlike simple byte slicing which can panic or produce invalid UTF-8, this
/// function properly handles multi-byte UTF-8 characters by truncating at
/// character boundaries.
///
/// # Examples
///
/// ```
/// use butabuti::utils::string::char_truncate;
///
/// assert_eq!(char_truncate("Hello", 3), "Hel");
/// assert_eq!(char_truncate("Hello", 10), "Hello");
/// assert_eq!(char_truncate("日本語", 2), "日本");
/// assert_eq!(char_truncate("", 5), "");
/// ```
pub fn char_truncate(s: &str, max_chars: usize) -> &str {
    match s.char_indices().nth(max_chars) {
        Some((idx, _)) => &s[..idx],
        None => s,
    }
}

/// Convert null-padded byte array to String.
///
/// Reads bytes until the first null byte (or end of array) and converts
/// to a UTF-8 string. Useful for reading fixed-width text fields from
/// binary embroidery formats.
///
/// # Examples
///
/// ```
/// use butabuti::utils::string::from_null_padded;
///
/// let bytes = b"Hello\0\0\0";
/// assert_eq!(from_null_padded(bytes), "Hello");
///
/// let bytes = b"Test";
/// assert_eq!(from_null_padded(bytes), "Test");
///
/// let bytes = b"\0\0\0";
/// assert_eq!(from_null_padded(bytes), "");
/// ```
pub fn from_null_padded(bytes: &[u8]) -> String {
    let null_pos = bytes.iter().position(|&b| b == 0).unwrap_or(bytes.len());
    String::from_utf8_lossy(&bytes[..null_pos]).to_string()
}

/// Pad string to fixed width with null bytes.
///
/// Truncates if string is too long, pads with nulls if too short.
/// Useful for writing fixed-width text fields to binary formats.
///
/// # Examples
///
/// ```
/// use butabuti::utils::string::to_null_padded;
///
/// let padded = to_null_padded("Hello", 8);
/// assert_eq!(padded, b"Hello\0\0\0");
///
/// let padded = to_null_padded("TooLongString", 5);
/// assert_eq!(padded, b"TooLo");
///
/// let padded = to_null_padded("", 3);
/// assert_eq!(padded, b"\0\0\0");
/// ```
pub fn to_null_padded(s: &str, width: usize) -> Vec<u8> {
    let mut result = vec![0u8; width];
    let bytes = s.as_bytes();
    let copy_len = bytes.len().min(width);
    result[..copy_len].copy_from_slice(&bytes[..copy_len]);
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_c_trim_basic() {
        assert_eq!(c_trim("Hello\0\0\0"), "Hello");
        assert_eq!(c_trim("Test\0"), "Test");
        assert_eq!(c_trim("NoNulls"), "NoNulls");
    }

    #[test]
    fn test_c_trim_whitespace() {
        assert_eq!(c_trim("Hello  \0\0"), "Hello");
        assert_eq!(c_trim("  Test\0"), "Test");
        assert_eq!(c_trim("\t\tTab\0\0"), "Tab");
    }

    #[test]
    fn test_c_trim_empty() {
        assert_eq!(c_trim(""), "");
        assert_eq!(c_trim("\0\0\0"), "");
        assert_eq!(c_trim("   \0\0"), "");
    }

    #[test]
    fn test_c_trim_utf8() {
        assert_eq!(c_trim("日本語\0\0"), "日本語");
        assert_eq!(c_trim("Émily\0"), "Émily");
        assert_eq!(c_trim("🎨\0\0\0"), "🎨");
    }

    #[test]
    fn test_char_truncate_basic() {
        assert_eq!(char_truncate("Hello", 3), "Hel");
        assert_eq!(char_truncate("Hello", 5), "Hello");
        assert_eq!(char_truncate("Hello", 10), "Hello");
    }

    #[test]
    fn test_char_truncate_empty() {
        assert_eq!(char_truncate("", 0), "");
        assert_eq!(char_truncate("", 5), "");
        assert_eq!(char_truncate("Test", 0), "");
    }

    #[test]
    fn test_char_truncate_utf8() {
        // Japanese characters (3 bytes each in UTF-8)
        assert_eq!(char_truncate("日本語", 2), "日本");
        assert_eq!(char_truncate("日本語", 3), "日本語");
        assert_eq!(char_truncate("日本語", 10), "日本語");

        // Emoji (4 bytes in UTF-8)
        assert_eq!(char_truncate("🎨🎭🎪", 2), "🎨🎭");
        assert_eq!(char_truncate("Hello🌍", 6), "Hello🌍");
        assert_eq!(char_truncate("Hello🌍", 5), "Hello");
    }

    #[test]
    fn test_char_truncate_mixed() {
        assert_eq!(char_truncate("Hello世界", 5), "Hello");
        assert_eq!(char_truncate("Hello世界", 6), "Hello世");
        assert_eq!(char_truncate("Hello世界", 7), "Hello世界");
    }

    #[test]
    fn test_from_null_padded_basic() {
        assert_eq!(from_null_padded(b"Hello\0\0\0"), "Hello");
        assert_eq!(from_null_padded(b"Test\0"), "Test");
        assert_eq!(from_null_padded(b"NoNull"), "NoNull");
    }

    #[test]
    fn test_from_null_padded_empty() {
        assert_eq!(from_null_padded(b""), "");
        assert_eq!(from_null_padded(b"\0\0\0"), "");
    }

    #[test]
    fn test_from_null_padded_utf8() {
        // Valid UTF-8 sequences
        let japanese = "日本語".as_bytes();
        let mut padded = japanese.to_vec();
        padded.extend_from_slice(&[0, 0, 0]);
        assert_eq!(from_null_padded(&padded), "日本語");
    }

    #[test]
    fn test_from_null_padded_invalid_utf8() {
        // Invalid UTF-8 should use replacement characters
        let invalid = &[0xFF, 0xFE, b'H', b'i', 0x00];
        let result = from_null_padded(invalid);
        assert!(result.contains("Hi"));
    }

    #[test]
    fn test_to_null_padded_basic() {
        assert_eq!(to_null_padded("Hello", 8), b"Hello\0\0\0");
        assert_eq!(to_null_padded("Test", 4), b"Test");
        assert_eq!(to_null_padded("", 3), b"\0\0\0");
    }

    #[test]
    fn test_to_null_padded_truncate() {
        assert_eq!(to_null_padded("TooLongString", 5), b"TooLo");
        assert_eq!(to_null_padded("Hello", 3), b"Hel");
    }

    #[test]
    fn test_to_null_padded_exact() {
        assert_eq!(to_null_padded("Hello", 5), b"Hello");
        assert_eq!(to_null_padded("Test", 4), b"Test");
    }

    #[test]
    fn test_to_null_padded_utf8() {
        let result = to_null_padded("日本", 10);
        assert_eq!(result.len(), 10);
        // "日本" is 6 bytes in UTF-8
        assert_eq!(&result[0..6], "日本".as_bytes());
        assert_eq!(&result[6..10], &[0, 0, 0, 0]);
    }

    #[test]
    fn test_roundtrip_null_padding() {
        let original = "Hello";
        let padded = to_null_padded(original, 10);
        let recovered = from_null_padded(&padded);
        assert_eq!(recovered, original);
    }

    #[test]
    fn test_roundtrip_utf8() {
        let original = "日本語🎨";
        let padded = to_null_padded(original, 20);
        let recovered = from_null_padded(&padded);
        assert_eq!(recovered, original);
    }

    #[test]
    fn test_edge_case_single_char() {
        assert_eq!(char_truncate("A", 1), "A");
        assert_eq!(char_truncate("A", 0), "");
        assert_eq!(c_trim("A\0"), "A");
        assert_eq!(from_null_padded(b"A\0"), "A");
    }

    #[test]
    fn test_edge_case_all_nulls() {
        assert_eq!(c_trim("\0\0\0\0"), "");
        assert_eq!(from_null_padded(b"\0\0\0\0"), "");
    }

    #[test]
    fn test_edge_case_whitespace_preservation() {
        // c_trim should remove trailing whitespace after null removal
        assert_eq!(c_trim("Hello World\0"), "Hello World");
        assert_eq!(c_trim("Hello World  \0"), "Hello World");

        // But char_truncate and from_null_padded preserve it
        assert_eq!(char_truncate("Hello  ", 10), "Hello  ");
        assert_eq!(from_null_padded(b"Hi  \0"), "Hi  ");
    }

    // ReadByteIterator tests
    #[test]
    fn test_byte_iterator_basic() {
        use std::io::Cursor;

        let data = vec![1, 2, 3, 4, 5];
        let reader = Cursor::new(data);
        let mut iter = ReadByteIterator::new(reader);

        assert_eq!(iter.next(), Some(1));
        assert_eq!(iter.next(), Some(2));
        assert_eq!(iter.next(), Some(3));
        assert_eq!(iter.next(), Some(4));
        assert_eq!(iter.next(), Some(5));
        assert_eq!(iter.next(), None);
        assert!(iter.closed);
        assert!(!iter.has_error());
    }

    #[test]
    fn test_byte_iterator_empty() {
        use std::io::Cursor;

        let data: Vec<u8> = vec![];
        let reader = Cursor::new(data);
        let mut iter = ReadByteIterator::new(reader);

        assert_eq!(iter.next(), None);
        assert!(iter.closed);
        assert!(!iter.has_error());
    }

    #[test]
    fn test_byte_iterator_collect() {
        use std::io::Cursor;

        let data = vec![65, 66, 67, 68, 69]; // "ABCDE"
        let reader = Cursor::new(data.clone());
        let iter = ReadByteIterator::new(reader);

        let collected: Vec<u8> = iter.collect();
        assert_eq!(collected, data);
    }

    #[test]
    fn test_byte_iterator_partial_read() {
        use std::io::Cursor;

        let data = vec![10, 20, 30, 40, 50];
        let reader = Cursor::new(data);
        let mut iter = ReadByteIterator::new(reader);

        assert_eq!(iter.next(), Some(10));
        assert_eq!(iter.next(), Some(20));
        assert!(!iter.closed);

        // Read remaining
        let remaining: Vec<u8> = iter.collect();
        assert_eq!(remaining, vec![30, 40, 50]);
    }

    #[test]
    fn test_byte_iterator_after_closed() {
        use std::io::Cursor;

        let data = vec![1, 2];
        let reader = Cursor::new(data);
        let mut iter = ReadByteIterator::new(reader);

        assert_eq!(iter.next(), Some(1));
        assert_eq!(iter.next(), Some(2));
        assert_eq!(iter.next(), None); // Should close
        assert!(iter.closed);

        // Further calls should return None
        assert_eq!(iter.next(), None);
        assert_eq!(iter.next(), None);
    }

    #[test]
    fn test_byte_iterator_error_simulation() {
        use std::io::{self, Error, ErrorKind, Read};

        // Create a reader that will error after 3 bytes
        struct ErrorReader {
            count: usize,
        }

        impl Read for ErrorReader {
            fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
                if self.count >= 3 {
                    return Err(Error::new(ErrorKind::Other, "simulated error"));
                }
                if buf.is_empty() {
                    return Ok(0);
                }
                buf[0] = self.count as u8;
                self.count += 1;
                Ok(1)
            }
        }

        let reader = ErrorReader { count: 0 };
        let mut iter = ReadByteIterator::new(reader);

        assert_eq!(iter.next(), Some(0));
        assert_eq!(iter.next(), Some(1));
        assert_eq!(iter.next(), Some(2));
        assert_eq!(iter.next(), None); // Error occurs here
        assert!(iter.closed);
        assert!(iter.has_error());
        assert!(iter.error.is_some());
    }

    #[test]
    fn test_byte_iterator_map() {
        use std::io::Cursor;

        let data = vec![1, 2, 3, 4, 5];
        let reader = Cursor::new(data);
        let iter = ReadByteIterator::new(reader);

        let doubled: Vec<u8> = iter.map(|b| b * 2).collect();
        assert_eq!(doubled, vec![2, 4, 6, 8, 10]);
    }

    #[test]
    fn test_byte_iterator_filter() {
        use std::io::Cursor;

        let data = vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10];
        let reader = Cursor::new(data);
        let iter = ReadByteIterator::new(reader);

        let evens: Vec<u8> = iter.filter(|&b| b % 2 == 0).collect();
        assert_eq!(evens, vec![2, 4, 6, 8, 10]);
    }

    #[test]
    fn test_byte_iterator_take() {
        use std::io::Cursor;

        let data = vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10];
        let reader = Cursor::new(data);
        let iter = ReadByteIterator::new(reader);

        let first_five: Vec<u8> = iter.take(5).collect();
        assert_eq!(first_five, vec![1, 2, 3, 4, 5]);
    }

    #[test]
    fn test_byte_iterator_single_byte() {
        use std::io::Cursor;

        let data = vec![42];
        let reader = Cursor::new(data);
        let mut iter = ReadByteIterator::new(reader);

        assert_eq!(iter.next(), Some(42));
        assert_eq!(iter.next(), None);
        assert!(iter.closed);
    }
}
