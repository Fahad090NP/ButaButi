//! Error types for embroidery operations
//!
//! Provides custom error types with automatic conversions from common error sources
//! using the thiserror crate for ergonomic error handling throughout the library.
//!
//! # Error Type Usage Guidelines
//!
//! Choose the appropriate error variant based on the failure context:
//!
//! ## `Error::Io`
//! - **When**: File system or stream I/O operations fail
//! - **Examples**: File not found, permission denied, disk full, read/write errors
//! - **Auto-converted**: From `std::io::Error` via `?` operator
//! - **Usage**: Generally handled automatically, no manual construction needed
//!
//! ```rust,ignore
//! let mut file = File::open(path)?; // Auto-converts io::Error to Error::Io
//! ```
//!
//! ## `Error::Parse`
//! - **When**: Format-specific parsing or decoding fails
//! - **Examples**: Invalid header signature, corrupted data structures, malformed binary
//! - **Context**: Include format name and expected vs actual values when possible
//!
//! ```rust,ignore
//! if signature != DST_SIGNATURE {
//!     return Err(Error::Parse(format!(
//!         "Invalid DST header: expected {:?}, got {:?}",
//!         DST_SIGNATURE, signature
//!     )));
//! }
//! ```
//!
//! ## `Error::UnsupportedFormat`
//! - **When**: File extension or format type is not recognized
//! - **Examples**: Unknown file extension, unsupported format variant
//! - **Context**: Specify what format was attempted or what's supported
//!
//! ```rust,ignore
//! Err(Error::UnsupportedFormat(
//!     format!("File extension '{}' not supported", ext)
//! ))
//! ```
//!
//! ## `Error::InvalidPattern`
//! - **When**: Pattern data is semantically invalid (not just malformed)
//! - **Examples**: No stitches, thread index out of sequence, invalid stitch sequence
//! - **Differs from Parse**: Parse = corrupt data, InvalidPattern = logically wrong data
//!
//! ```rust,ignore
//! if pattern.stitches().is_empty() {
//!     return Err(Error::InvalidPattern(
//!         "Pattern must contain at least one stitch".to_string()
//!     ));
//! }
//! ```
//!
//! ## `Error::InvalidColor`
//! - **When**: Color string parsing fails
//! - **Examples**: Invalid hex format, unknown color name
//! - **Context**: Show the invalid input and expected formats
//!
//! ```rust,ignore
//! Err(Error::InvalidColor(
//!     format!("Invalid color '{}': expected hex (FF0000) or name (red)", input)
//! ))
//! ```
//!
//! ## `Error::Encoding`
//! - **When**: Writing or encoding pattern data fails
//! - **Examples**: Stitch coordinates out of format range, too many colors for format
//! - **Differs from Parse**: Encoding = write failures, Parse = read failures
//!
//! ```rust,ignore
//! if stitch_count > MAX_DST_STITCHES {
//!     return Err(Error::Encoding(
//!         format!("DST format supports max {} stitches, got {}",
//!             MAX_DST_STITCHES, stitch_count)
//!     ));
//! }
//! ```
//!
//! ## `Error::ThreadIndexOutOfBounds`
//! - **When**: Accessing thread by index that doesn't exist
//! - **Specialized**: More specific than InvalidPattern for thread errors
//! - **Usage**: Pass the invalid index for debugging
//!
//! ```rust,ignore
//! if thread_index >= pattern.threads().len() {
//!     return Err(Error::ThreadIndexOutOfBounds(thread_index));
//! }
//! ```
//!
//! ## `Error::Unsupported`
//! - **When**: Feature or operation is valid but not implemented
//! - **Examples**: Optional format features, future functionality
//! - **Differs from UnsupportedFormat**: Unsupported = unimplemented feature
//!
//! ```rust,ignore
//! Err(Error::Unsupported(
//!     "Sequin data not supported in this format version".to_string()
//! ))
//! ```
//!
//! ## `Error::Json`
//! - **When**: JSON serialization/deserialization fails
//! - **Auto-converted**: From `serde_json::Error` via `?` operator
//! - **Usage**: Generally handled automatically

use std::io;
use thiserror::Error;

/// Main error type for Rusty Petal operations
#[derive(Error, Debug)]
pub enum Error {
    /// I/O error occurred
    #[error("I/O error: {0}")]
    Io(#[from] io::Error),

    /// Error parsing embroidery file
    #[error("Parse error: {0}")]
    Parse(String),

    /// Unsupported file format
    #[error("Unsupported format: {0}")]
    UnsupportedFormat(String),

    /// Invalid pattern data
    #[error("Invalid pattern: {0}")]
    InvalidPattern(String),

    /// Thread index out of bounds
    #[error("Thread index out of bounds: {0}")]
    ThreadIndexOutOfBounds(usize),

    /// Invalid color format
    #[error("Invalid color format: {0}")]
    InvalidColor(String),

    /// Encoding error
    #[error("Encoding error: {0}")]
    Encoding(String),

    /// Unsupported operation
    #[error("Unsupported operation: {0}")]
    Unsupported(String),

    /// JSON serialization/deserialization error
    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),
}

/// Result type alias for Rusty Petal operations
pub type Result<T> = std::result::Result<T, Error>;
