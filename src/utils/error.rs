//! Error types for embroidery operations
//!
//! Provides custom error types with automatic conversions from common error sources
//! using the thiserror crate for ergonomic error handling throughout the library.

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
