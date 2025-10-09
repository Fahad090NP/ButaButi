//! Embroidery file format writers
//!
//! Provides writers for 20+ embroidery file formats including DST, PES, JEF, VP3, and others.
//! Each writer module exposes a `write()` function that encodes an `EmbPattern` to the target format.

pub mod col;
pub mod csv;
pub mod dst;
/// EDR (Embird Color) format writer
pub mod edr;
pub mod exp;
pub mod gcode;
pub mod inf;
pub mod jef;
pub mod json;
pub mod pec;
pub mod pes;
/// PNG (Portable Network Graphics) raster format writer
pub mod png;
/// SVG (Scalable Vector Graphics) format writer
pub mod svg;
pub mod tbf;
/// TXT text format writer (human-readable)
pub mod txt;
pub mod u01;
pub mod vp3;
pub mod xxx;

// Additional writers to be implemented:
// ... etc
