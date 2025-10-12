//! Error types for embroidery operations
//!
//! Provides custom error types with automatic conversions from common error sources
//! using the thiserror crate for ergonomic error handling throughout the library.
//!
//! # Error Context System
//!
//! Errors support context tracking via the `ErrorWithContext` trait, allowing you
//! to build a stack of contextual information as errors propagate up the call stack.
//!
//! ```rust
//! use butabuti::utils::error::{Error, ErrorWithContext};
//!
//! fn read_header() -> Result<(), Error> {
//!     // Error occurs here
//!     Err(Error::Parse("Invalid magic bytes".to_string()))
//!         .with_context("Reading DST header at offset 0")
//! }
//!
//! fn read_file() -> Result<(), Error> {
//!     read_header()
//!         .with_context("Processing design.dst")?;
//!     Ok(())
//! }
//! ```
//!
//! The resulting error will display:
//! ```text
//! Parse error: Invalid magic bytes
//! Context (innermost first):
//!   - Reading DST header at offset 0
//!   - Processing design.dst
//! ```
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

use std::fmt;
use std::io;

/// Main error type for embroidery operations with context tracking
#[derive(Debug, Clone)]
pub struct Error {
    /// The kind of error that occurred
    kind: ErrorKind,
    /// Stack of contextual information (innermost first)
    context: Vec<String>,
}

/// Different kinds of errors that can occur
#[derive(Debug, Clone)]
pub enum ErrorKind {
    /// I/O error occurred
    Io(String),

    /// Error parsing embroidery file
    Parse(String),

    /// Unsupported file format
    UnsupportedFormat(String),

    /// Invalid pattern data
    InvalidPattern(String),

    /// Thread index out of bounds
    ThreadIndexOutOfBounds(usize),

    /// Invalid color format
    InvalidColor(String),

    /// Encoding error
    Encoding(String),

    /// Unsupported operation
    Unsupported(String),

    /// JSON serialization/deserialization error
    Json(String),
}

impl Error {
    /// Create a new error with the given kind
    pub fn new(kind: ErrorKind) -> Self {
        Self {
            kind,
            context: Vec::new(),
        }
    }

    /// Create an I/O error
    pub fn io<S: Into<String>>(msg: S) -> Self {
        Self::new(ErrorKind::Io(msg.into()))
    }

    /// Create a parse error
    pub fn parse<S: Into<String>>(msg: S) -> Self {
        Self::new(ErrorKind::Parse(msg.into()))
    }

    /// Create an unsupported format error
    pub fn unsupported_format<S: Into<String>>(msg: S) -> Self {
        Self::new(ErrorKind::UnsupportedFormat(msg.into()))
    }

    /// Create an invalid pattern error
    pub fn invalid_pattern<S: Into<String>>(msg: S) -> Self {
        Self::new(ErrorKind::InvalidPattern(msg.into()))
    }

    /// Create a thread index out of bounds error
    pub fn thread_index_out_of_bounds(index: usize) -> Self {
        Self::new(ErrorKind::ThreadIndexOutOfBounds(index))
    }

    /// Create an invalid color error
    pub fn invalid_color<S: Into<String>>(msg: S) -> Self {
        Self::new(ErrorKind::InvalidColor(msg.into()))
    }

    /// Create an encoding error
    pub fn encoding<S: Into<String>>(msg: S) -> Self {
        Self::new(ErrorKind::Encoding(msg.into()))
    }

    /// Create an unsupported operation error
    pub fn unsupported<S: Into<String>>(msg: S) -> Self {
        Self::new(ErrorKind::Unsupported(msg.into()))
    }

    /// Create a JSON error
    pub fn json<S: Into<String>>(msg: S) -> Self {
        Self::new(ErrorKind::Json(msg.into()))
    }

    /// Get the error kind
    pub fn kind(&self) -> &ErrorKind {
        &self.kind
    }

    /// Get the context stack (innermost first)
    pub fn context(&self) -> &[String] {
        &self.context
    }
}

/// Trait for adding contextual information to errors
pub trait ErrorWithContext: Sized {
    /// Add a context message to this error
    ///
    /// Context is added to a stack, with the innermost (most specific)
    /// context appearing first in the display output.
    ///
    /// # Example
    ///
    /// ```
    /// use butabuti::utils::error::{Error, ErrorWithContext};
    ///
    /// let err = Error::parse("Invalid header")
    ///     .with_context("Reading DST file at offset 512");
    ///
    /// assert_eq!(err.context().len(), 1);
    /// ```
    fn with_context<S: Into<String>>(self, ctx: S) -> Self;

    /// Add context if a condition is true
    ///
    /// This is useful for conditionally adding context based on runtime conditions.
    ///
    /// # Example
    ///
    /// ```
    /// use butabuti::utils::error::{Error, ErrorWithContext};
    ///
    /// let verbose = true;
    /// let err = Error::parse("Invalid header")
    ///     .with_context_if(verbose, "Additional debug info");
    /// ```
    fn with_context_if<S: Into<String>>(self, condition: bool, ctx: S) -> Self {
        if condition {
            self.with_context(ctx)
        } else {
            self
        }
    }

    /// Remove all context from this error
    fn without_context(self) -> Self;
}

impl ErrorWithContext for Error {
    fn with_context<S: Into<String>>(mut self, ctx: S) -> Self {
        self.context.push(ctx.into());
        self
    }

    fn without_context(mut self) -> Self {
        self.context.clear();
        self
    }
}

/// Extension trait for Result types to add context
pub trait ResultExt<T> {
    /// Add context to an error if the result is Err
    ///
    /// # Example
    ///
    /// ```no_run
    /// use butabuti::utils::error::ResultExt;
    /// use std::fs::File;
    ///
    /// let file = File::open("design.dst")
    ///     .with_context("Opening embroidery file")?;
    /// # Ok::<(), butabuti::utils::error::Error>(())
    /// ```
    fn with_context<S: Into<String>>(self, ctx: S) -> Result<T>;
}

impl<T> ResultExt<T> for Result<T> {
    fn with_context<S: Into<String>>(self, ctx: S) -> Result<T> {
        self.map_err(|e| e.with_context(ctx))
    }
}

impl fmt::Display for ErrorKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ErrorKind::Io(msg) => write!(f, "I/O error: {}", msg),
            ErrorKind::Parse(msg) => write!(f, "Parse error: {}", msg),
            ErrorKind::UnsupportedFormat(msg) => write!(f, "Unsupported format: {}", msg),
            ErrorKind::InvalidPattern(msg) => write!(f, "Invalid pattern: {}", msg),
            ErrorKind::ThreadIndexOutOfBounds(idx) => {
                write!(f, "Thread index out of bounds: {}", idx)
            },
            ErrorKind::InvalidColor(msg) => write!(f, "Invalid color: {}", msg),
            ErrorKind::Encoding(msg) => write!(f, "Encoding error: {}", msg),
            ErrorKind::Unsupported(msg) => write!(f, "Unsupported operation: {}", msg),
            ErrorKind::Json(msg) => write!(f, "JSON error: {}", msg),
        }
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // Write the main error message
        write!(f, "{}", self.kind)?;

        // Add context if present
        if !self.context.is_empty() {
            write!(f, "\nContext (innermost first):")?;
            for (i, ctx) in self.context.iter().enumerate() {
                write!(f, "\n  {}: {}", i + 1, ctx)?;
            }
        }

        Ok(())
    }
}

impl std::error::Error for Error {}

// Automatic conversions from common error types
impl From<io::Error> for Error {
    fn from(err: io::Error) -> Self {
        Self::new(ErrorKind::Io(err.to_string()))
    }
}

impl From<serde_json::Error> for Error {
    fn from(err: serde_json::Error) -> Self {
        Self::new(ErrorKind::Json(err.to_string()))
    }
}

// Convenience constructors that match the old API
#[allow(non_snake_case)]
impl Error {
    /// Create a Parse error (backward compatibility)
    pub fn Parse(msg: String) -> Self {
        Self::parse(msg)
    }

    /// Create an UnsupportedFormat error (backward compatibility)
    pub fn UnsupportedFormat(msg: String) -> Self {
        Self::unsupported_format(msg)
    }

    /// Create an InvalidPattern error (backward compatibility)
    pub fn InvalidPattern(msg: String) -> Self {
        Self::invalid_pattern(msg)
    }

    /// Create a ThreadIndexOutOfBounds error (backward compatibility)
    pub fn ThreadIndexOutOfBounds(index: usize) -> Self {
        Self::thread_index_out_of_bounds(index)
    }

    /// Create an InvalidColor error (backward compatibility)
    pub fn InvalidColor(msg: String) -> Self {
        Self::invalid_color(msg)
    }

    /// Create an Encoding error (backward compatibility)
    pub fn Encoding(msg: String) -> Self {
        Self::encoding(msg)
    }

    /// Create an Unsupported error (backward compatibility)
    pub fn Unsupported(msg: String) -> Self {
        Self::unsupported(msg)
    }

    /// Create an Io error (backward compatibility)
    pub fn Io(err: io::Error) -> Self {
        Self::from(err)
    }

    /// Create a Json error (backward compatibility)
    pub fn Json(err: serde_json::Error) -> Self {
        Self::from(err)
    }
}

/// Result type alias for embroidery operations
pub type Result<T> = std::result::Result<T, Error>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_creation() {
        let err = Error::parse("test error");
        assert!(matches!(err.kind, ErrorKind::Parse(_)));
    }

    #[test]
    fn test_error_with_context() {
        let err = Error::parse("test error").with_context("outer context");
        assert_eq!(err.context().len(), 1);
        assert_eq!(err.context()[0], "outer context");
    }

    #[test]
    fn test_error_multiple_context() {
        let err = Error::parse("test error")
            .with_context("inner context")
            .with_context("middle context")
            .with_context("outer context");

        assert_eq!(err.context().len(), 3);
        assert_eq!(err.context()[0], "inner context");
        assert_eq!(err.context()[1], "middle context");
        assert_eq!(err.context()[2], "outer context");
    }

    #[test]
    fn test_error_without_context() {
        let err = Error::parse("test error")
            .with_context("context 1")
            .with_context("context 2")
            .without_context();

        assert_eq!(err.context().len(), 0);
    }

    #[test]
    fn test_error_with_context_if() {
        let err1 = Error::parse("test").with_context_if(true, "added");
        assert_eq!(err1.context().len(), 1);

        let err2 = Error::parse("test").with_context_if(false, "not added");
        assert_eq!(err2.context().len(), 0);
    }

    #[test]
    fn test_result_ext() {
        let result: Result<()> = Err(Error::parse("test error"));
        let with_ctx = result.with_context("file.dst");

        assert!(with_ctx.is_err());
        let err = with_ctx.unwrap_err();
        assert_eq!(err.context().len(), 1);
        assert_eq!(err.context()[0], "file.dst");
    }

    #[test]
    fn test_error_display() {
        let err = Error::parse("Invalid header")
            .with_context("Reading DST file")
            .with_context("Processing design.dst");

        let display = format!("{}", err);
        assert!(display.contains("Parse error: Invalid header"));
        assert!(display.contains("Context (innermost first):"));
        assert!(display.contains("Reading DST file"));
        assert!(display.contains("Processing design.dst"));
    }

    #[test]
    fn test_error_display_no_context() {
        let err = Error::parse("Invalid header");
        let display = format!("{}", err);
        assert!(display.contains("Parse error: Invalid header"));
        assert!(!display.contains("Context"));
    }

    #[test]
    fn test_backward_compat_constructors() {
        let _ = Error::Parse("msg".to_string());
        let _ = Error::UnsupportedFormat("msg".to_string());
        let _ = Error::InvalidPattern("msg".to_string());
        let _ = Error::ThreadIndexOutOfBounds(5);
        let _ = Error::InvalidColor("msg".to_string());
        let _ = Error::Encoding("msg".to_string());
        let _ = Error::Unsupported("msg".to_string());
    }

    #[test]
    fn test_from_io_error() {
        let io_err = io::Error::new(io::ErrorKind::NotFound, "file not found");
        let err: Error = io_err.into();
        assert!(matches!(err.kind, ErrorKind::Io(_)));
    }

    #[test]
    fn test_from_json_error() {
        let json_err = serde_json::from_str::<serde_json::Value>("{invalid").unwrap_err();
        let err: Error = json_err.into();
        assert!(matches!(err.kind, ErrorKind::Json(_)));
    }
}
