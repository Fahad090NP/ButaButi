//! Basic example demonstrating pattern creation and manipulation

use butabuti::prelude::*;

fn main() -> std::result::Result<(), Box<dyn std::error::Error>> {
    println!("🌸 ButaButi Example - Pattern Creation\n");

    // Create a new pattern
    let mut pattern = EmbPattern::new();
    println!("Created new pattern");

    // Add metadata
    pattern.set_metadata("name", "Square Design");
    pattern.set_metadata("author", "ButaButi");
    println!("Added metadata: {}", pattern.get_metadata("name").unwrap());

    // Add a thread
    let red_thread = EmbThread::from_string("red")?
        .with_description("Ruby Red")
        .with_brand("Generic");
    pattern.add_thread(red_thread);
    println!("Added thread: {}", pattern.threads()[0]);

    // Create a simple square pattern (10mm x 10mm)
    println!("\nCreating square pattern:");
    pattern.stitch(100.0, 0.0); // Right (10mm in 0.1mm units)
    pattern.stitch(0.0, 100.0); // Down
    pattern.stitch(-100.0, 0.0); // Left
    pattern.stitch(0.0, -100.0); // Up (back to start)
    pattern.trim();
    pattern.end();

    // Display pattern info
    println!("Total stitches: {}", pattern.stitches().len());
    println!("Total threads: {}", pattern.threads().len());

    // Get bounds
    let (min_x, min_y, max_x, max_y) = pattern.bounds();
    println!("\nPattern bounds:");
    println!("  Min: ({:.1}, {:.1})", min_x, min_y);
    println!("  Max: ({:.1}, {:.1})", max_x, max_y);
    println!(
        "  Size: {:.1}mm x {:.1}mm",
        (max_x - min_x) / 10.0,
        (max_y - min_y) / 10.0
    );

    // Transform the pattern
    println!("\nTransforming pattern...");
    pattern.translate(500.0, 500.0); // Move 50mm right and down

    let (min_x, min_y, max_x, max_y) = pattern.bounds();
    println!("New bounds after translation:");
    println!("  Min: ({:.1}, {:.1})", min_x, min_y);
    println!("  Max: ({:.1}, {:.1})", max_x, max_y);

    // Center the pattern
    println!("\nCentering pattern at origin...");
    pattern.move_center_to_origin();

    let (min_x, min_y, max_x, max_y) = pattern.bounds();
    println!("Centered bounds:");
    println!("  Min: ({:.1}, {:.1})", min_x, min_y);
    println!("  Max: ({:.1}, {:.1})", max_x, max_y);

    println!("\n✅ Example completed successfully!");

    Ok(())
}
