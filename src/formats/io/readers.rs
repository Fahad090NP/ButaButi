//! Embroidery file format readers
//!
//! Provides readers for embroidery file formats with full read/write support.
//! Each reader module exposes a `read()` function that parses the format into an `EmbPattern`.

/// COL (Embroidery Thread Color) format reader
pub mod col;
/// CSV embroidery format reader (lossless debug format)
pub mod csv;
/// DST (Tajima) format reader
pub mod dst;
/// EDR (Embird Color) format reader
pub mod edr;
/// EXP (Melco) format reader
pub mod exp;
/// GCode format reader
pub mod gcode;
/// HUS (Husqvarna Viking) format reader
pub mod hus;
/// INF (Embroidery Thread Info) format reader
pub mod inf;
/// JEF (Janome) format reader
pub mod jef;
/// JSON embroidery format reader
pub mod json;
/// PEC (Brother) format reader
pub mod pec;
/// PES (Brother) format reader
pub mod pes;
/// TBF (Tajima) format reader
pub mod tbf;
/// U01 (Barudan) format reader
pub mod u01;
/// VP3 (Pfaff) format reader
pub mod vp3;
/// XXX (Singer) format reader
pub mod xxx;

#[cfg(test)]
mod tests;
