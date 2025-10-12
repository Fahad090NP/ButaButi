//! Thread Palette Management Example
//!
//! Demonstrates how to:
//! - Load and save palette files
//! - Access built-in machine palettes
//! - Find closest thread colors
//! - Quantize patterns to palette colors

use butabuti::prelude::*;
use std::io::Cursor;

fn main() -> Result<()> {
    println!("=== Thread Palette Management Example ===\n");

    // Example 1: Create and save a custom palette
    println!("1. Creating custom palette...");
    let mut custom_palette = ThreadPalette::new("My Custom Palette");
    custom_palette.add_thread(EmbThread::from_rgb(255, 0, 0).with_description("Red"));
    custom_palette.add_thread(EmbThread::from_rgb(0, 255, 0).with_description("Green"));
    custom_palette.add_thread(EmbThread::from_rgb(0, 0, 255).with_description("Blue"));
    custom_palette.add_thread(EmbThread::from_rgb(255, 255, 0).with_description("Yellow"));

    println!("   Created palette with {} colors", custom_palette.len());

    // Save to different formats
    let mut edr_output = Vec::new();
    custom_palette.save(&mut edr_output, PaletteFormat::Edr)?;
    println!("   Saved as EDR format ({} bytes)", edr_output.len());

    let mut rgb_output = Vec::new();
    custom_palette.save(&mut rgb_output, PaletteFormat::Rgb)?;
    println!("   Saved as RGB format ({} bytes)", rgb_output.len());
    println!("   RGB content:\n{}", String::from_utf8_lossy(&rgb_output));

    // Example 2: Load palette from RGB format
    println!("\n2. Loading palette from RGB format...");
    let rgb_data = "255 0 0\n0 255 0\n0 0 255\n255 255 255\n0 0 0\n";
    let mut cursor = Cursor::new(rgb_data.as_bytes());
    let loaded_palette =
        ThreadPalette::load(&mut cursor, PaletteFormat::Rgb, "Loaded".to_string())?;
    println!("   Loaded {} colors from RGB data", loaded_palette.len());

    // Example 3: Access built-in machine palettes
    println!("\n3. Accessing built-in palettes...");
    let all_palettes = PaletteLibrary::all_palettes();
    for palette in &all_palettes {
        println!("   - {}: {} colors", palette.name, palette.len());
    }

    // Example 4: Find closest color
    println!("\n4. Finding closest thread color...");
    let brother_pec = PaletteLibrary::brother_pec();

    // Find closest to orange (RGB 255, 128, 0)
    let orange = 0xFF8000;
    if let Some(closest) = brother_pec.find_closest(orange) {
        println!("   Closest to orange (#{:06X}):", orange);
        println!(
            "   -> RGB({}, {}, {}) - {}",
            closest.red(),
            closest.green(),
            closest.blue(),
            closest.description.as_deref().unwrap_or("No description")
        );
    }

    // Find closest to cyan
    let cyan = 0x00FFFF;
    if let Some(closest) = brother_pec.find_closest(cyan) {
        println!("   Closest to cyan (#{:06X}):", cyan);
        println!(
            "   -> RGB({}, {}, {}) - {}",
            closest.red(),
            closest.green(),
            closest.blue(),
            closest.description.as_deref().unwrap_or("No description")
        );
    }

    // Example 5: Quantize pattern to palette colors
    println!("\n5. Quantizing pattern to palette...");
    let mut pattern = EmbPattern::new();

    // Add custom colors that don't exactly match palette
    pattern.add_thread(EmbThread::from_rgb(250, 10, 10)); // Close to red
    pattern.add_thread(EmbThread::from_rgb(10, 250, 10)); // Close to green
    pattern.add_thread(EmbThread::from_rgb(200, 50, 50)); // Reddish
    pattern.add_thread(EmbThread::from_rgb(5, 5, 250)); // Close to blue

    // Add some stitches
    pattern.stitch(100.0, 0.0);
    pattern.color_change(0.0, 0.0);
    pattern.stitch(0.0, 100.0);
    pattern.color_change(0.0, 0.0);
    pattern.stitch(-100.0, 0.0);
    pattern.color_change(0.0, 0.0);
    pattern.stitch(0.0, -100.0);

    println!(
        "   Pattern before quantization: {} threads",
        pattern.threads().len()
    );
    for (i, thread) in pattern.threads().iter().enumerate() {
        println!(
            "     Thread {}: RGB({}, {}, {})",
            i,
            thread.red(),
            thread.green(),
            thread.blue()
        );
    }

    // Quantize to RGB palette (Red, Green, Blue, Yellow)
    let rgb_palette = ThreadPalette::from_threads(
        "RGB",
        vec![
            EmbThread::from_rgb(255, 0, 0),
            EmbThread::from_rgb(0, 255, 0),
            EmbThread::from_rgb(0, 0, 255),
            EmbThread::from_rgb(255, 255, 0),
        ],
    );

    rgb_palette.quantize_pattern(&mut pattern)?;

    println!(
        "\n   Pattern after quantization: {} threads",
        pattern.threads().len()
    );
    for (i, thread) in pattern.threads().iter().enumerate() {
        println!(
            "     Thread {}: RGB({}, {}, {})",
            i,
            thread.red(),
            thread.green(),
            thread.blue()
        );
    }

    // Example 6: Get palette by name
    println!("\n6. Getting palettes by name...");
    let names = vec!["brother", "hus", "jef", "sew"];
    for name in names {
        if let Some(palette) = PaletteLibrary::get_by_name(name) {
            println!(
                "   Found '{}' -> {}: {} colors",
                name,
                palette.name,
                palette.len()
            );
        }
    }

    println!("\n=== Example Complete ===");
    Ok(())
}
