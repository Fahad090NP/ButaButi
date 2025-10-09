//! Embroidery file format readers
//!
//! Provides readers for 40+ embroidery file formats including DST, PES, JEF, VP3, and many others.
//! Each reader module exposes a `read()` function that parses the format into an `EmbPattern`.

/// BRO (Bits and Volts) format reader
pub mod bro;
pub mod col;
/// CSV embroidery format reader (lossless debug format)
pub mod csv;
/// DAT (Barudan/Sunstar) format reader
pub mod dat;
/// DSB (Barudan B-stitch) format reader
pub mod dsb;
pub mod dst;
/// DSZ (ZSK USA Design) format reader
pub mod dsz;
/// EDR (Embird Color) format reader
pub mod edr;
/// EMD (Elna) format reader
pub mod emd;
pub mod exp;
/// EXY (Eltac) format reader - DST with 256-byte header
pub mod exy;
/// FXY (Fortron) format reader - DSZ with 256-byte header
pub mod fxy;
pub mod gcode;
/// GT (Gold Thread) format reader
pub mod gt;
/// HUS (Husqvarna Viking) format reader
pub mod hus;
/// INB (Inbro) format reader
pub mod inb;
pub mod inf;
pub mod jef;
/// JPX (Janome) format reader
pub mod jpx;
pub mod json;
/// KSM (Pfaff) format reader
pub mod ksm;
/// MAX (Pfaff) format reader
pub mod max;
/// MIT (Mitsubishi) format reader
pub mod mit;
/// NEW (Ameco) format reader
pub mod new;
/// PCD (Pfaff) format reader
pub mod pcd;
/// PCM (Pfaff) format reader
pub mod pcm;
/// PCQ (Pfaff MAXI) format reader - same as PCD
pub mod pcq;
/// PCS (Pfaff Small/Large hoop) format reader - same as PCD
pub mod pcs;
pub mod pec;
pub mod pes;
/// PHB (Brother PHB) format reader - uses PEC encoding
pub mod phb;
/// PHC (Brother PHC) format reader - uses PEC encoding
pub mod phc;
/// PMV (Brother PMV) format reader
pub mod pmv;
/// SEW (Janome Sewing Machine) format reader
pub mod sew;
/// SHV (Husqvarna Viking SHV) format reader
pub mod shv;
/// SPX (Sperry) format reader
pub mod spx;
/// STC (Gunold) format reader
pub mod stc;
/// STX (Data Stitch) format reader - uses EXP encoding
pub mod stx;
/// TAP (Happy Embroidery) format reader - uses DST encoding
pub mod tap;
pub mod tbf;
/// Toyota 100 format reader
pub mod tyo_100;
/// Toyota 10O format reader
pub mod tyo_10o;
pub mod u01;
pub mod vp3;
pub mod xxx;
/// ZHS (Zeng Hsing) format reader
pub mod zhs;
/// ZXY (ZSK TC) format reader
pub mod zxy;

// Additional readers to be implemented:
// ... etc
