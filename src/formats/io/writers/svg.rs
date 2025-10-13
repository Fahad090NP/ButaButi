//! SVG vector graphics format writer for embroidery patterns
//!
//! Renders embroidery patterns as scalable vector graphics with thread colors,
//! path elements for stitches, and proper viewBox for web and print use.
//!
//! Supports two rendering modes:
//! - **Simple paths**: Fast rendering with solid stroke paths (default)
//! - **Realistic stitches**: Uses stitch icons with gradients and rotation (opt-in)

use crate::core::pattern::EmbPattern;
use crate::utils::error::Result;
use crate::utils::stitch_renderer::{
    calculate_stitch_angle, create_colored_stitch_symbol, create_stitch_use_element,
    StitchRenderQuality,
};
use std::io::Write;

/// Write SVG (Scalable Vector Graphics) format with simple path rendering
///
/// This is the default export method for backward compatibility.
/// Uses simple stroke paths for fast rendering.
pub fn write(pattern: &EmbPattern, file: &mut impl Write) -> Result<()> {
    write_with_quality(pattern, file, StitchRenderQuality::Low)
}

/// Write SVG with configurable render quality
///
/// Allows choosing between simple paths and realistic stitch rendering.
///
/// # Arguments
///
/// * `pattern` - Embroidery pattern to export
/// * `file` - Output stream
/// * `quality` - Render quality level (Low, Medium, High, Ultra)
///
/// # Quality Levels
///
/// - **Low**: Simple paths with solid stroke (fastest, smallest file)
/// - **Medium**: Colored paths with rounded caps (smoother appearance)
/// - **High**: Realistic stitch icons with gradients (best quality)
/// - **Ultra**: 3D-effect stitches with shadows (future implementation)
///
/// # Example
///
/// ```rust
/// use butabuti::prelude::*;
/// use butabuti::formats::io::writers::svg;
/// use butabuti::utils::stitch_renderer::StitchRenderQuality;
/// use std::fs::File;
///
/// let pattern = EmbPattern::new();
/// let mut file = File::create("output.svg")?;
/// svg::write_with_quality(&pattern, &mut file, StitchRenderQuality::High)?;
/// ```
pub fn write_with_quality(
    pattern: &EmbPattern,
    file: &mut impl Write,
    quality: StitchRenderQuality,
) -> Result<()> {
    // Get pattern bounds
    let bounds = pattern.bounds();
    let min_x = bounds.0;
    let min_y = bounds.1;
    let max_x = bounds.2;
    let max_y = bounds.3;

    let width = max_x - min_x;
    let height = max_y - min_y;

    // Write SVG header
    writeln!(file, "<?xml version=\"1.0\" encoding=\"UTF-8\"?>")?;
    writeln!(
        file,
        "<svg version=\"1.1\" xmlns=\"http://www.w3.org/2000/svg\" xmlns:xlink=\"http://www.w3.org/1999/xlink\" xmlns:ev=\"http://www.w3.org/2001/xml-events\" width=\"{}\" height=\"{}\" viewBox=\"{} {} {} {}\">",
        width, height, min_x, min_y, width, height
    )?;

    // If using realistic stitch icons, define symbols in <defs>
    if quality.use_stitch_icons() {
        writeln!(file, "  <defs>")?;

        // Create a colored stitch symbol for each thread
        for (i, thread) in pattern.threads().iter().enumerate() {
            let symbol_id = format!("stitch_{}", i);
            let symbol = create_colored_stitch_symbol(thread, &symbol_id);
            writeln!(file, "    {}", symbol)?;
        }

        writeln!(file, "  </defs>")?;
    }

    // Get stitch blocks
    let stitch_blocks = pattern.get_as_stitchblock();

    // Render each stitch block
    for (block_idx, (block, thread)) in stitch_blocks.iter().enumerate() {
        if block.is_empty() {
            continue;
        }

        if quality.use_stitch_icons() {
            // Realistic rendering: use stitch icons
            render_block_with_icons(file, block, block_idx)?;
        } else {
            // Simple rendering: use paths
            render_block_with_paths(file, block, thread, &quality)?;
        }
    }

    // Close SVG
    writeln!(file, "</svg>")?;

    Ok(())
}

/// Render a stitch block as a simple path
fn render_block_with_paths(
    file: &mut impl Write,
    block: &[(f64, f64)],
    thread: &crate::core::thread::EmbThread,
    quality: &StitchRenderQuality,
) -> Result<()> {
    // Start path with M (move to)
    let mut path_data = String::from("M");

    for stitch in block {
        path_data.push_str(&format!(" {},{}", stitch.0, stitch.1));
    }

    // Get thread color
    let color = thread.hex_color();
    let stroke_width = quality.stroke_width();

    // Determine stroke cap style
    let stroke_cap = match quality {
        StitchRenderQuality::Low => "butt",
        _ => "round",
    };

    // Write path element
    writeln!(
        file,
        "  <path d=\"{}\" fill=\"none\" stroke=\"{}\" stroke-width=\"{}\" stroke-linecap=\"{}\"/>",
        path_data, color, stroke_width, stroke_cap
    )?;

    Ok(())
}

/// Render a stitch block with realistic stitch icons
fn render_block_with_icons(
    file: &mut impl Write,
    block: &[(f64, f64)],
    thread_idx: usize,
) -> Result<()> {
    let symbol_id = format!("stitch_{}", thread_idx);

    // Render each stitch with rotation based on angle to next stitch
    for i in 0..block.len() {
        let (x, y) = block[i];

        // Calculate angle to next stitch (or previous if last stitch)
        let angle = if i < block.len() - 1 {
            let (next_x, next_y) = block[i + 1];
            calculate_stitch_angle(x as f32, y as f32, next_x as f32, next_y as f32)
        } else if i > 0 {
            let (prev_x, prev_y) = block[i - 1];
            calculate_stitch_angle(prev_x as f32, prev_y as f32, x as f32, y as f32)
        } else {
            0.0 // Single stitch, no rotation
        };

        // Create <use> element for this stitch
        let use_elem = create_stitch_use_element(&symbol_id, x as f32, y as f32, angle);
        writeln!(file, "  {}", use_elem)?;
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::constants::*;
    use crate::core::thread::EmbThread;
    use std::io::Cursor;

    #[test]
    fn test_write_svg_basic() {
        let mut pattern = EmbPattern::new();

        // Add a thread
        pattern.add_thread(EmbThread::from_rgb(255, 0, 0));

        // Add some stitches
        pattern.add_stitch_absolute(STITCH, 0.0, 0.0);
        pattern.add_stitch_absolute(STITCH, 10.0, 10.0);
        pattern.add_stitch_absolute(STITCH, 20.0, 0.0);
        pattern.add_command(END, 0.0, 0.0);

        let mut output = Cursor::new(Vec::new());
        write(&pattern, &mut output).expect("Failed to write SVG");

        let svg_content = String::from_utf8(output.into_inner()).unwrap();

        assert!(svg_content.contains("<?xml"));
        assert!(svg_content.contains("<svg"));
        assert!(svg_content.contains("<path"));
        assert!(svg_content.contains("</svg>"));
    }

    #[test]
    fn test_svg_with_multiple_colors() {
        let mut pattern = EmbPattern::new();

        // Add two threads
        pattern.add_thread(EmbThread::from_rgb(255, 0, 0));
        pattern.add_thread(EmbThread::from_rgb(0, 0, 255));

        // Add stitches for first color
        pattern.add_stitch_absolute(STITCH, 0.0, 0.0);
        pattern.add_stitch_absolute(STITCH, 10.0, 10.0);
        pattern.add_command(COLOR_CHANGE, 0.0, 0.0);

        // Add stitches for second color
        pattern.add_stitch_absolute(STITCH, 10.0, 10.0);
        pattern.add_stitch_absolute(STITCH, 20.0, 20.0);
        pattern.add_command(END, 0.0, 0.0);

        let mut output = Cursor::new(Vec::new());
        write(&pattern, &mut output).expect("Failed to write SVG");

        let svg_content = String::from_utf8(output.into_inner()).unwrap();

        // Should have multiple path elements
        assert!(svg_content.matches("<path").count() >= 2);
    }

    #[test]
    fn test_svg_viewbox() {
        let mut pattern = EmbPattern::new();
        pattern.add_thread(EmbThread::from_rgb(0, 255, 0));

        pattern.add_stitch_absolute(STITCH, 100.0, 100.0);
        pattern.add_stitch_absolute(STITCH, 200.0, 200.0);
        pattern.add_command(END, 0.0, 0.0);

        let mut output = Cursor::new(Vec::new());
        write(&pattern, &mut output).expect("Failed to write SVG");

        let svg_content = String::from_utf8(output.into_inner()).unwrap();

        assert!(svg_content.contains("viewBox"));
        assert!(svg_content.contains("100")); // min coordinates
    }

    #[test]
    fn test_write_with_quality_high() {
        let mut pattern = EmbPattern::new();
        pattern.add_thread(EmbThread::from_rgb(255, 0, 0));

        pattern.add_stitch_absolute(STITCH, 0.0, 0.0);
        pattern.add_stitch_absolute(STITCH, 10.0, 10.0);
        pattern.add_stitch_absolute(STITCH, 20.0, 0.0);
        pattern.add_command(END, 0.0, 0.0);

        let mut output = Cursor::new(Vec::new());
        write_with_quality(&pattern, &mut output, StitchRenderQuality::High)
            .expect("Failed to write SVG");

        let svg_content = String::from_utf8(output.into_inner()).unwrap();

        // High quality should use stitch symbols
        assert!(svg_content.contains("<defs>"));
        assert!(svg_content.contains("<symbol"));
        assert!(svg_content.contains("<use"));
        assert!(svg_content.contains("xlink:href"));
    }

    #[test]
    fn test_write_with_quality_medium() {
        let mut pattern = EmbPattern::new();
        pattern.add_thread(EmbThread::from_rgb(0, 255, 0));

        pattern.add_stitch_absolute(STITCH, 0.0, 0.0);
        pattern.add_stitch_absolute(STITCH, 10.0, 10.0);
        pattern.add_command(END, 0.0, 0.0);

        let mut output = Cursor::new(Vec::new());
        write_with_quality(&pattern, &mut output, StitchRenderQuality::Medium)
            .expect("Failed to write SVG");

        let svg_content = String::from_utf8(output.into_inner()).unwrap();

        // Medium quality should use paths with rounded caps
        assert!(svg_content.contains("stroke-linecap=\"round\""));
        assert!(svg_content.contains("<path"));
    }
}
