//! Comprehensive example demonstrating JSON format and pattern processing
//!
//! This example shows:
//! - Creating a pattern programmatically
//! - Writing to JSON format
//! - Reading from JSON format
//! - Using pattern processing utilities (normalize, stats, etc.)

use butabuti::core::constants::*;
use butabuti::core::pattern::EmbPattern;
use butabuti::core::thread::EmbThread;
use butabuti::formats::io::readers::json as json_reader;
use butabuti::formats::io::writers::json as json_writer;
use butabuti::utils::processing;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== JSON Format and Pattern Processing Example ===\n");

    // Create a simple heart pattern
    let mut pattern = create_heart_pattern();

    println!("Original pattern:");
    print_pattern_info(&pattern);

    // Normalize the pattern to start at (0, 0)
    println!("\n--- Normalizing pattern ---");
    processing::normalize(&mut pattern);
    print_pattern_info(&pattern);

    // Calculate and display statistics
    println!("\n--- Pattern Statistics ---");
    let stats = processing::calculate_stats(&pattern);
    println!("Total stitches: {}", stats.stitch_count);
    println!("Jumps: {}", stats.jump_count);
    println!("Trims: {}", stats.trim_count);
    println!("Color changes: {}", stats.color_change_count);
    println!("Total thread length: {:.2} mm", stats.total_length);
    println!(
        "Dimensions: {:.1}x{:.1} mm",
        stats.max_x - stats.min_x,
        stats.max_y - stats.min_y
    );

    // Write to JSON
    println!("\n--- Writing to JSON ---");
    let mut json_output = Vec::new();
    json_writer::write(&mut json_output, &pattern)?;
    let json_string = String::from_utf8(json_output)?;
    println!("JSON output ({} bytes):", json_string.len());
    println!("{}", json_string);

    // Read back from JSON
    println!("\n--- Reading from JSON ---");
    let mut cursor = std::io::Cursor::new(json_string.as_bytes());
    let read_pattern = json_reader::read(&mut cursor)?;
    println!(
        "Successfully read pattern with {} stitches",
        read_pattern.stitches().len()
    );

    // Verify round-trip
    println!("\n--- Verifying round-trip ---");
    if pattern.stitches().len() == read_pattern.stitches().len() {
        println!("✓ Stitch count matches");
    } else {
        println!("✗ Stitch count mismatch");
    }

    if pattern.threads().len() == read_pattern.threads().len() {
        println!("✓ Thread count matches");
    } else {
        println!("✗ Thread count mismatch");
    }

    // Demonstrate processing utilities
    println!("\n--- Processing Utilities Demo ---");

    // Create a pattern with duplicates
    let mut pattern_with_dupes = EmbPattern::new();
    pattern_with_dupes.add_thread(EmbThread::new(0xFF0000));
    pattern_with_dupes.add_stitch_absolute(STITCH, 0.0, 0.0);
    pattern_with_dupes.add_stitch_absolute(STITCH, 0.0, 0.0); // Duplicate
    pattern_with_dupes.add_stitch_absolute(STITCH, 10.0, 10.0);
    pattern_with_dupes.add_stitch_absolute(STITCH, 10.0, 10.0); // Duplicate

    println!(
        "Pattern with duplicates: {} stitches",
        pattern_with_dupes.stitches().len()
    );
    processing::remove_duplicates(&mut pattern_with_dupes);
    println!(
        "After removing duplicates: {} stitches",
        pattern_with_dupes.stitches().len()
    );

    // Demonstrate color count fixing
    let mut pattern_no_threads = EmbPattern::new();
    pattern_no_threads.add_stitch_absolute(STITCH, 0.0, 0.0);
    pattern_no_threads.add_stitch_absolute(COLOR_CHANGE, 10.0, 10.0);
    pattern_no_threads.add_stitch_absolute(STITCH, 20.0, 20.0);

    println!(
        "\nPattern with color changes but no threads: {} threads",
        pattern_no_threads.threads().len()
    );
    processing::fix_color_count(&mut pattern_no_threads);
    println!(
        "After fixing color count: {} threads",
        pattern_no_threads.threads().len()
    );

    println!("\n=== Example Complete ===");
    Ok(())
}

/// Create a simple heart-shaped pattern
fn create_heart_pattern() -> EmbPattern {
    let mut pattern = EmbPattern::new();

    // Add metadata
    pattern.add_metadata("name", "Simple Heart");
    pattern.add_metadata("author", "Rusty Petal");
    pattern.add_metadata("description", "A simple heart pattern for demonstration");

    // Add thread (red)
    pattern.add_thread(
        EmbThread::new(0xFF0000)
            .with_description("Red")
            .with_brand("Robison-Anton")
            .with_catalog_number("2219"),
    );

    // Draw a simple heart shape (simplified for demo)
    // Left curve
    pattern.add_stitch_absolute(STITCH, 50.0, 100.0);
    pattern.add_stitch_absolute(STITCH, 30.0, 80.0);
    pattern.add_stitch_absolute(STITCH, 20.0, 60.0);
    pattern.add_stitch_absolute(STITCH, 20.0, 40.0);
    pattern.add_stitch_absolute(STITCH, 30.0, 20.0);
    pattern.add_stitch_absolute(STITCH, 50.0, 10.0);

    // Right curve
    pattern.add_stitch_absolute(STITCH, 70.0, 20.0);
    pattern.add_stitch_absolute(STITCH, 80.0, 40.0);
    pattern.add_stitch_absolute(STITCH, 80.0, 60.0);
    pattern.add_stitch_absolute(STITCH, 70.0, 80.0);
    pattern.add_stitch_absolute(STITCH, 50.0, 100.0);

    // End
    pattern.add_stitch_absolute(END, 50.0, 100.0);

    pattern
}

/// Print basic pattern information
fn print_pattern_info(pattern: &EmbPattern) {
    let (min_x, min_y, max_x, max_y) = pattern.bounds();
    println!("  Stitches: {}", pattern.stitches().len());
    println!("  Threads: {}", pattern.threads().len());
    println!(
        "  Bounds: ({:.1}, {:.1}) to ({:.1}, {:.1})",
        min_x, min_y, max_x, max_y
    );
}
