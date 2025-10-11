//! Brother PES format reader
//!
//! PES is Brother's main embroidery format supporting multiple versions (1-6+).
//! Contains an embedded PEC section for machine compatibility and design metadata.
//!
//! ## Format Limitations
//! - PEC block offset must be within 0-100MB range (100,000,000 bytes)
//! - Supports versions 1-10 (#PES0001 through #PES0100)
//! - All PES files contain an embedded PEC section starting at specified offset

/// Maximum allowed PEC block position in bytes (100MB)
const MAX_PEC_OFFSET: i32 = 100_000_000;

use crate::core::pattern::EmbPattern;
use crate::core::thread::EmbThread;
use crate::formats::io::readers::pec;
use crate::formats::io::utils::ReadHelper;
use crate::utils::error::Result;
use std::io::{Read, Seek, SeekFrom};

/// Read a PES string (length-prefixed)
fn read_pes_string<R: Read>(helper: &mut ReadHelper<R>) -> Result<Option<String>> {
    let length = helper.read_u8()? as usize;
    if length == 0 {
        return Ok(None);
    }
    let s = helper.read_string(length)?;
    Ok(Some(s))
}

/// Read PES metadata fields
fn read_pes_metadata<R: Read>(helper: &mut ReadHelper<R>, pattern: &mut EmbPattern) -> Result<()> {
    if let Some(v) = read_pes_string(helper)? {
        if !v.is_empty() {
            pattern.add_metadata("name", v);
        }
    }
    if let Some(v) = read_pes_string(helper)? {
        if !v.is_empty() {
            pattern.add_metadata("category", v);
        }
    }
    if let Some(v) = read_pes_string(helper)? {
        if !v.is_empty() {
            pattern.add_metadata("author", v);
        }
    }
    if let Some(v) = read_pes_string(helper)? {
        if !v.is_empty() {
            pattern.add_metadata("keywords", v);
        }
    }
    if let Some(v) = read_pes_string(helper)? {
        if !v.is_empty() {
            pattern.add_metadata("comments", v);
        }
    }
    Ok(())
}

/// Read a PES thread definition
fn read_pes_thread<R: Read>(
    helper: &mut ReadHelper<R>,
    threadlist: &mut Vec<EmbThread>,
) -> Result<()> {
    let catalog_number = read_pes_string(helper)?;

    // Read color as 24-bit big-endian
    let b1 = helper.read_u8()? as u32;
    let b2 = helper.read_u8()? as u32;
    let b3 = helper.read_u8()? as u32;
    let color = 0xFF000000 | (b1 << 16) | (b2 << 8) | b3;

    // Skip 5 bytes
    helper.read_bytes(5)?;

    let description = read_pes_string(helper)?;
    let brand = read_pes_string(helper)?;
    let chart = read_pes_string(helper)?;

    let mut thread = EmbThread::new(color);
    if let Some(cat) = catalog_number {
        thread = thread.with_catalog_number(&cat);
    }
    if let Some(desc) = description {
        thread = thread.with_description(&desc);
    }
    if let Some(b) = brand {
        thread = thread.with_brand(&b);
    }
    if let Some(c) = chart {
        thread = thread.with_chart(&c);
    }

    threadlist.push(thread);
    Ok(())
}

/// Read PES header version 1
fn read_pes_header_version_1<R: Read>(
    _helper: &mut ReadHelper<R>,
    _pattern: &mut EmbPattern,
) -> Result<()> {
    // Version 1 has no additional data we care about
    Ok(())
}

/// Read PES header version 4
fn read_pes_header_version_4<R: Read>(
    helper: &mut ReadHelper<R>,
    pattern: &mut EmbPattern,
) -> Result<()> {
    helper.read_bytes(4)?;
    read_pes_metadata(helper, pattern)?;
    Ok(())
}

/// Read PES header version 5/5.5/5.6
fn read_pes_header_version_5<R: Read>(
    helper: &mut ReadHelper<R>,
    pattern: &mut EmbPattern,
    threadlist: &mut Vec<EmbThread>,
) -> Result<()> {
    helper.read_bytes(4)?;
    read_pes_metadata(helper, pattern)?;
    helper.read_bytes(24)?;

    if let Some(v) = read_pes_string(helper)? {
        if !v.is_empty() {
            pattern.add_metadata("image_file", v);
        }
    }

    helper.read_bytes(24)?;

    let count_programmable_fills = helper.read_u16_le()?;
    if count_programmable_fills != 0 {
        return Ok(());
    }

    let count_motifs = helper.read_u16_le()?;
    if count_motifs != 0 {
        return Ok(());
    }

    let count_feather_patterns = helper.read_u16_le()?;
    if count_feather_patterns != 0 {
        return Ok(());
    }

    let count_threads = helper.read_u16_le()?;
    for _ in 0..count_threads {
        read_pes_thread(helper, threadlist)?;
    }

    Ok(())
}

/// Read PES header version 6
fn read_pes_header_version_6<R: Read>(
    helper: &mut ReadHelper<R>,
    pattern: &mut EmbPattern,
    threadlist: &mut Vec<EmbThread>,
) -> Result<()> {
    helper.read_bytes(4)?;
    read_pes_metadata(helper, pattern)?;
    helper.read_bytes(36)?; // Different from v5

    if let Some(v) = read_pes_string(helper)? {
        if !v.is_empty() {
            pattern.add_metadata("image_file", v);
        }
    }

    helper.read_bytes(24)?;

    let count_programmable_fills = helper.read_u16_le()?;
    if count_programmable_fills != 0 {
        return Ok(());
    }

    let count_motifs = helper.read_u16_le()?;
    if count_motifs != 0 {
        return Ok(());
    }

    let count_feather_patterns = helper.read_u16_le()?;
    if count_feather_patterns != 0 {
        return Ok(());
    }

    let count_threads = helper.read_u16_le()?;
    for _ in 0..count_threads {
        read_pes_thread(helper, threadlist)?;
    }

    Ok(())
}

/// Read PES (Brother PES) format
///
/// Reads a PES embroidery file into the provided pattern.
/// PES files contain an embedded PEC section for machine compatibility.
///
/// # Arguments
///
/// * `file` - The input stream to read from (must support Seek for PEC block access)
/// * `pattern` - The pattern to populate with stitches and metadata
///
/// # Example
///
/// ```no_run
/// use butabuti::core::pattern::EmbPattern;
/// use butabuti::formats::io::readers::pes;
/// use std::fs::File;
///
/// let mut file = File::open("design.pes")?;
/// let mut pattern = EmbPattern::new();
/// pes::read(&mut file, &mut pattern)?;
/// # Ok::<(), butabuti::utils::error::Error>(())
/// ```
pub fn read(file: &mut (impl Read + Seek), pattern: &mut EmbPattern) -> Result<()> {
    let mut helper = ReadHelper::new(file);
    let mut loaded_thread_values = Vec::new();

    // Read PES header string (8 bytes)
    let pes_string = helper.read_string(8).map_err(|e| {
        if e.kind() == std::io::ErrorKind::UnexpectedEof {
            crate::utils::error::Error::Parse(
                "PES file too small: header must be at least 8 bytes".to_string(),
            )
        } else {
            crate::utils::error::Error::from(e)
        }
    })?;

    // Validate PES/PEC magic bytes
    if !pes_string.starts_with("#PES") && !pes_string.starts_with("#PEC") {
        return Err(crate::utils::error::Error::Parse(format!(
            "Invalid PES/PEC header: expected '#PES' or '#PEC', got '{}'",
            pes_string
        )));
    }

    // Check if it's actually a standalone PEC file
    if pes_string == "#PEC0001" {
        let mut reader = helper.into_inner();
        pec::read_pec(&mut reader, pattern, None)?;
        pattern.interpolate_duplicate_color_as_stop();
        return Ok(());
    }

    // Read PEC block position
    let pec_block_position = helper.read_i32_le()?;

    // Validate PEC block position is reasonable
    if !(0..=MAX_PEC_OFFSET).contains(&pec_block_position) {
        return Err(crate::utils::error::Error::Parse(format!(
            "Invalid PEC block position: {} (must be between 0 and {})",
            pec_block_position, MAX_PEC_OFFSET
        )));
    }

    // Parse version and read appropriate header
    match pes_string.as_str() {
        "#PES0100" => {
            pattern.add_metadata("version", "10");
            read_pes_header_version_6(&mut helper, pattern, &mut loaded_thread_values)?;
        }
        "#PES0090" => {
            pattern.add_metadata("version", "9");
            read_pes_header_version_6(&mut helper, pattern, &mut loaded_thread_values)?;
        }
        "#PES0080" => {
            pattern.add_metadata("version", "8");
            read_pes_header_version_6(&mut helper, pattern, &mut loaded_thread_values)?;
        }
        "#PES0070" => {
            pattern.add_metadata("version", "7");
            read_pes_header_version_6(&mut helper, pattern, &mut loaded_thread_values)?;
        }
        "#PES0060" => {
            pattern.add_metadata("version", "6");
            read_pes_header_version_6(&mut helper, pattern, &mut loaded_thread_values)?;
        }
        "#PES0050" | "#PES0055" | "#PES0056" => {
            pattern.add_metadata("version", "5");
            read_pes_header_version_5(&mut helper, pattern, &mut loaded_thread_values)?;
        }
        "#PES0040" => {
            pattern.add_metadata("version", "4");
            read_pes_header_version_4(&mut helper, pattern)?;
        }
        "#PES0030" => {
            pattern.add_metadata("version", "3");
        }
        "#PES0022" => {
            pattern.add_metadata("version", "2.2");
        }
        "#PES0020" => {
            pattern.add_metadata("version", "2");
        }
        "#PES0001" => {
            pattern.add_metadata("version", "1");
            read_pes_header_version_1(&mut helper, pattern)?;
        }
        _ => {
            // Unknown version, skip header
        }
    }

    // Seek to PEC block and read it
    let mut reader = helper.into_inner();
    reader.seek(SeekFrom::Start(pec_block_position as u64))?;

    pec::read_pec(&mut reader, pattern, Some(&mut loaded_thread_values))?;
    pattern.interpolate_duplicate_color_as_stop();

    Ok(())
}

/// Read a PES file from path
///
/// Convenience function to read a PES file directly from a file path.
///
/// # Example
///
/// ```no_run
/// use butabuti::formats::io::readers::pes;
///
/// let pattern = pes::read_file("design.pes")?;
/// # Ok::<(), butabuti::utils::error::Error>(())
/// ```
pub fn read_file(path: &str) -> Result<EmbPattern> {
    let file = std::fs::File::open(path)?;
    let mut reader = std::io::BufReader::new(file);
    let mut pattern = EmbPattern::new();
    read(&mut reader, &mut pattern)?;
    Ok(pattern)
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_pes_version_strings() {
        let versions = vec![
            "#PES0001", "#PES0020", "#PES0022", "#PES0030", "#PES0040", "#PES0050", "#PES0055",
            "#PES0056", "#PES0060", "#PES0070", "#PES0080", "#PES0090", "#PES0100",
        ];
        for v in versions {
            assert_eq!(v.len(), 8);
        }
    }
}
