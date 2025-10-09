//! # ButaButi
//!
//! A high-performance Rust library for reading, writing, and manipulating embroidery files.
//!
//! ## Features
//!
//! - **Read Support**: 40+ embroidery file formats
//! - **Write Support**: 10+ embroidery file formats
//! - **Format Conversion**: Seamless conversion between formats
//! - **Pattern Manipulation**: Transformations, scaling, rotation, translation
//! - **Thread Management**: Comprehensive thread color handling
//! - **Type Safety**: Leverage Rust's type system for correctness
//! - **Performance**: Zero-cost abstractions and optimized operations
//!
//! ## Quick Start
//!
//! ```rust,no_run
//! use rusty_petal::prelude::*;
//!
//! // Read an embroidery file
//! let pattern = EmbPattern::read("design.pes")?;
//!
//! // Manipulate the pattern
//! let mut modified = pattern.clone();
//! modified.translate(100.0, 100.0);
//!
//! // Write to a different format
//! modified.write("design.dst")?;
//! # Ok::<(), Box<dyn std::error::Error>>(())
//! ```
//!
//! ## Supported Formats
//!
//! ### Mandated Formats (Full Read/Write Support)
//! - **PES** - Brother Embroidery Format
//! - **DST** - Tajima Embroidery Format
//! - **EXP** - Melco Embroidery Format
//! - **JEF** - Janome Embroidery Format
//! - **VP3** - Pfaff Embroidery Format
//!
//! ### Additional Formats
//! Over 35 additional formats for reading, including:
//! - PEC, XXX, U01, SEW, HUS, and many more
//!
//! See the documentation for a complete list.

#![warn(missing_docs)]
#![warn(clippy::all)]

// Core modules
pub mod core;
pub mod formats;
pub mod palettes;
pub mod utils;

// Re-export commonly used types at the crate root
pub use core::constants::*;
pub use core::matrix::EmbMatrix;
pub use core::pattern::EmbPattern;
pub use core::thread::EmbThread;
pub use utils::error::Error;

/// Prelude module for convenient imports
pub mod prelude {
    pub use crate::core::constants::*;
    pub use crate::core::matrix::EmbMatrix;
    pub use crate::core::pattern::EmbPattern;
    pub use crate::core::thread::EmbThread;
    pub use crate::utils::error::*;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_pattern_creation() {
        let pattern = EmbPattern::new();
        assert_eq!(pattern.stitches().len(), 0);
    }
}
