//! Example: Export a single pattern to multiple formats
//!
//! This example demonstrates how to export one embroidery pattern
//! to multiple file formats simultaneously.

use butabuti::formats::io::readers;
use butabuti::prelude::*;
use std::fs::File;
use std::io::BufReader;

fn main() -> Result<()> {
    println!("=== Multi-Format Export Example ===\n");

    // Read a sample pattern
    println!("Reading sample pattern from tests/DST Files/017.DST...");
    let mut file = BufReader::new(File::open("tests/DST Files/017.DST")?);
    let pattern = readers::dst::read(&mut file, None)?;

    println!("✓ Pattern loaded successfully");
    println!("  - Stitches: {}", pattern.count_stitches());
    println!("  - Color changes: {}", pattern.count_color_changes());

    let (min_x, min_y, max_x, max_y) = pattern.bounds();
    println!(
        "  - Size: {:.1}mm × {:.1}mm\n",
        (max_x - min_x) / 10.0,
        (max_y - min_y) / 10.0
    );

    // Example 1: Export to all common formats
    println!("Example 1: Exporting to common embroidery formats");
    println!("--------------------------------------------------");

    let exporter = MultiFormatExporter::new()
        .output_dir("tests/Output/multi_format")
        .base_name("sample_design")
        .formats(&["dst", "pes", "jef", "vp3", "exp", "xxx", "pec"])
        .overwrite(true)
        .build();

    match exporter.export(&pattern) {
        Ok(results) => {
            println!("\n✓ Multi-format export completed!");
            results.print_summary();

            println!("\nGenerated files:");
            for result in results.results() {
                if let ConversionResult::Success {
                    output, file_size, ..
                } = result
                {
                    println!("  ✓ {} ({} KB)", output.display(), file_size / 1024);
                }
            }
        }
        Err(e) => {
            eprintln!("Error during export: {}", e);
            return Err(e);
        }
    }

    println!("\n");

    // Example 2: Export to analysis formats
    println!("Example 2: Exporting to analysis/interchange formats");
    println!("----------------------------------------------------");

    let exporter2 = MultiFormatExporter::new()
        .output_dir("tests/Output/analysis")
        .base_name("pattern_analysis")
        .formats(&["json", "csv", "txt", "svg"])
        .overwrite(true)
        .build();

    match exporter2.export(&pattern) {
        Ok(results) => {
            println!("\n✓ Analysis export completed!");
            println!(
                "Exported {} files ({:.2} MB total)",
                results.success_count(),
                results.total_output_size() as f64 / 1_048_576.0
            );
        }
        Err(e) => {
            eprintln!("Error: {}", e);
        }
    }

    println!("\n");

    // Example 3: Create a modified pattern and export it
    println!("Example 3: Creating and exporting a modified pattern");
    println!("----------------------------------------------------");

    let mut modified = pattern.clone();
    modified.translate(50.0, 50.0); // Move pattern by 5mm x 5mm

    let exporter3 = MultiFormatExporter::new()
        .output_dir("tests/Output/modified")
        .base_name("translated_design")
        .formats(&["dst", "pes", "jef"])
        .overwrite(true)
        .build();

    match exporter3.export(&modified) {
        Ok(results) => {
            println!("\n✓ Modified pattern exported!");
            println!("Success rate: {:.1}%", results.success_rate() * 100.0);
        }
        Err(e) => {
            eprintln!("Error: {}", e);
        }
    }

    println!("\n=== All exports completed successfully! ===");

    Ok(())
}
