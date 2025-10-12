// File format I/O module - readers and writers for embroidery formats

//! File format readers and writers
//!
//! This module provides functionality for reading and writing various
//! embroidery file formats.

/// Binary I/O macros for cleaner reading/writing
#[macro_use]
pub mod macros;

/// Format detection and auto-loading
pub mod detector;

/// Format readers
pub mod readers;

/// Common I/O utilities
pub mod utils;

/// Format writers
pub mod writers;
