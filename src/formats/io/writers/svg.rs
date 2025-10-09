//! SVG vector graphics format writer for embroidery patterns
//!
//! Renders embroidery patterns as scalable vector graphics with thread colors,
//! path elements for stitches, and proper viewBox for web and print use.

use crate::core::pattern::EmbPattern;
use anyhow::Result;
use std::io::Write;

/// Write SVG (Scalable Vector Graphics) format
pub fn write(pattern: &EmbPattern, file: &mut impl Write) -> Result<()> {
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

    // Get stitch blocks
    let stitch_blocks = pattern.get_as_stitchblock();

    // Draw each stitch block as a path
    for (block, thread) in stitch_blocks {
        if block.is_empty() {
            continue;
        }

        // Start path with M (move to)
        let mut path_data = String::from("M");

        for stitch in &block {
            path_data.push_str(&format!(" {},{}", stitch.0, stitch.1));
        }

        // Get thread color
        let color = thread.hex_color();

        // Write path element
        writeln!(
            file,
            "  <path d=\"{}\" fill=\"none\" stroke=\"{}\" stroke-width=\"3\"/>",
            path_data, color
        )?;
    }

    // Close SVG
    writeln!(file, "</svg>")?;

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
}
