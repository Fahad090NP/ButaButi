//! Utility functions and helpers
//!
//! This module contains utility functions for compression, error handling,
//! pattern processing, and batch conversion operations.

/// Batch conversion and multi-format export utilities
pub mod batch;

/// Huffman compression for HUS format
pub mod compress;

/// Error types and handling
pub mod error;

/// Helper functions for encoding/decoding
pub mod functions;

/// Thread palette management and color library access
pub mod palette;

/// Pattern processing utilities
pub mod processing;

/// UTF-8 string utilities for format handling
pub mod string;
