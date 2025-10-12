// Core module - fundamental embroidery types and operations

//! Core embroidery pattern structures and utilities
//!
//! This module contains the fundamental types and functionality for working
//! with embroidery patterns.

/// Pattern collection for multi-pattern files
pub mod collection;

/// Color group management for organizing threads
pub mod color_group;

/// Command definitions and constants
pub mod constants;

/// Encoder for pattern transcoding
pub mod encoder;

/// Affine transformation matrix
pub mod matrix;

/// Pattern structure and manipulation
pub mod pattern;

/// Thread color management
pub mod thread;
