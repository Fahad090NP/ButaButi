# 🌸 ButaButi - Embroidery File Format Library

**NOTE: THIS PROJECT IS ITS EARLY STAGE AND IT MAY CONTAIN INCOMPLETE FEATURES OR API CHANGES. I WELCOME ALL THE CONTRIBUTIONS WITH LOVE AND SUPPORT. MAY MY THIS ATTEMPT WILL SERVE HUMANITY WITH LOVE.**

[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![Rust](https://img.shields.io/badge/rust-1.70%2B-orange.svg)](https://www.rust-lang.org/)

A high-performance Rust library for reading, writing, and manipulating embroidery machine files. ButaButi supports 40+ embroidery file formats and provides powerful tools for pattern manipulation, format conversion, and embroidery design analysis.

---

## 📋 Table of Contents

- [Features](#-features)
- [Supported Formats](#-supported-formats)
- [Installation](#-installation)
- [Quick Start](#-quick-start)
- [Core Concepts](#-core-concepts)
- [Complete API Reference](#-complete-api-reference)
- [Advanced Features](#-advanced-features)
- [Examples](#-examples)
- [Building and Testing](#-building-and-testing)
- [Contributing](#-contributing)
- [License](#-license)

---

## ✨ Features

- **40+ Format Support**: Read from 47 embroidery formats, write to 18 formats
- **Format Conversion**: Seamless conversion between different embroidery file formats
- **Pattern Manipulation**: Scale, rotate, translate, and transform embroidery designs
- **Thread Management**: Comprehensive color handling with 140+ named colors
- **Pattern Analysis**: Calculate bounds, stitch counts, thread lengths, and statistics
- **Type Safety**: Leverage Rust's type system for correctness and performance
- **Zero Dependencies**: Core library has minimal dependencies for fast compilation
- **Export Options**: Export to PNG, SVG, JSON, CSV, and text formats

---

## 📁 Supported Formats

### Read Support (47 Formats)

**Industrial & Commercial Formats:**

- **DST** - Tajima (most widely used commercial format)
- **EXP** - Melco
- **PES** - Brother (versions 1-6 supported)
- **JEF** - Janome
- **VP3** - Pfaff/Husqvarna
- **HUS** - Husqvarna Viking
- **XXX** - Singer
- **U01** - Barudan
- **SEW** - Janome Sewing Machine
- **SHV** - Husqvarna SHV

**Additional Read-Only Formats:**
BRO, COL, CSV, DAT, DSB, DSZ, EDR, EMD, EXY, FXY, GCode, GT, INB, INF, JPX, JSON, KSM, MAX, MIT, NEW, PCD, PCM, PCQ, PCS, PEC, PHB, PHC, PMV, SPX, STC, STX, TAP, TBF, TYO-100, TYO-10O, ZHS, ZXY

### Write Support (18 Formats)

**Core Write Formats:**

- **DST** - Tajima (industrial standard)
- **PES** - Brother
- **EXP** - Melco
- **JEF** - Janome
- **VP3** - Pfaff/Husqvarna
- **XXX** - Singer
- **U01** - Barudan

**Export & Analysis Formats:**

- **PNG** - Raster image export
- **SVG** - Scalable vector graphics
- **JSON** - Structured data format
- **CSV** - Comma-separated values
- **TXT** - Human-readable text
- **GCode** - CNC/3D printer format

**Additional Formats:**
COL, EDR, INF, PEC, TBF

---

## 📦 Installation

Add ButaButi to your `Cargo.toml`:

```toml
[dependencies]
butabuti = "0.1.0"

# Optional features
butabuti = { version = "0.1.0", features = ["graphics", "parallel"] }
```

### Feature Flags

- **`graphics`** - Enables PNG export (requires `image` crate)
- **`parallel`** - Enables parallel processing with `rayon`
- **`full`** - Enables all features

---

## 🚀 Quick Start

### Create a Simple Pattern

```rust
use butabuti::prelude::*;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create a new pattern
    let mut pattern = EmbPattern::new();
    
    // Add a red thread
    let red_thread = EmbThread::from_string("red")?;
    pattern.add_thread(red_thread);
    
    // Create a 10mm x 10mm square
    pattern.stitch(100.0, 0.0);   // Move right 10mm
    pattern.stitch(0.0, 100.0);   // Move down 10mm
    pattern.stitch(-100.0, 0.0);  // Move left 10mm
    pattern.stitch(0.0, -100.0);  // Move up 10mm
    pattern.trim();               // Cut thread
    pattern.end();                // End pattern
    
    // Print pattern info
    println!("Stitches: {}", pattern.count_stitches());
    println!("Size: {:.1}mm x {:.1}mm", 
        pattern.bounds().2 / 10.0, 
        pattern.bounds().3 / 10.0);
    
    Ok(())
}
```

### Read and Convert a File

```rust
use butabuti::prelude::*;
use butabuti::formats::io::{readers, writers};
use std::fs::File;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Read a PES file
    let mut file = File::open("design.pes")?;
    let mut pattern = EmbPattern::new();
    readers::pes::read(&mut file, &mut pattern)?;
    
    // Convert to DST format
    let mut output = File::create("design.dst")?;
    writers::dst::write(&pattern, &mut output)?;
    
    println!("Converted PES to DST successfully!");
    Ok(())
}
```

---

## 🧩 Core Concepts

### Coordinate System

All coordinates in ButaButi use **0.1mm units** (tenths of millimeters). This provides precision while keeping calculations simple.

```rust
// Example: Moving 10mm to the right
pattern.stitch(100.0, 0.0);  // 100 units = 10mm

// Converting to millimeters
let mm = units / 10.0;  // 100.0 / 10.0 = 10mm

// Converting from millimeters
let units = mm * 10.0;  // 10mm * 10.0 = 100.0 units
```

### Stitch Commands

Embroidery patterns consist of stitches with associated commands:

| Command | Value | Description |
|---------|-------|-------------|
| `STITCH` | 0 | Regular stitch - needle down |
| `JUMP` | 1 | Move without stitching (needle up) |
| `TRIM` | 2 | Cut the thread |
| `STOP` | 3 | Stop machine (for applique, manual work) |
| `END` | 4 | End of pattern |
| `COLOR_CHANGE` | 5 | Change to next thread color |
| `SEQUIN_MODE` | 6 | Enter sequin mode |
| `SEQUIN_EJECT` | 7 | Eject sequin |

```rust
use butabuti::prelude::*;

let mut pattern = EmbPattern::new();

// Using convenience methods (recommended)
pattern.stitch(10.0, 0.0);       // Add stitch
pattern.jump(50.0, 0.0);         // Jump without stitching
pattern.trim();                  // Trim thread
pattern.color_change(0.0, 0.0);  // Change color
pattern.end();                   // End pattern

// Using low-level methods (for advanced use)
pattern.add_stitch_relative(10.0, 0.0, STITCH);
pattern.add_stitch_absolute(JUMP, 100.0, 50.0);
```

### Absolute vs Relative Positioning

ButaButi supports both absolute and relative positioning:

```rust
// Relative positioning (offset from last position)
pattern.stitch(10.0, 0.0);   // Move 10 units right from current position
pattern.stitch(0.0, 10.0);   // Move 10 units down from current position

// Absolute positioning (exact coordinates)
pattern.stitch_abs(100.0, 100.0);  // Go to exact position (100, 100)
pattern.stitch_abs(150.0, 150.0);  // Go to exact position (150, 150)
```

---

## 📖 Complete API Reference

### EmbPattern - The Main Pattern Class

`EmbPattern` is the core structure that holds all pattern data including stitches, threads, and metadata.

#### Creating Patterns

```rust
// Create a new empty pattern
let pattern = EmbPattern::new();

// Create from existing stitches and threads
let stitches = vec![Stitch::new(10.0, 10.0, STITCH)];
let threads = vec![EmbThread::new(0xFF0000)];
let pattern = EmbPattern::from_stitches(stitches, threads);
```

#### Adding Stitches

```rust
// Relative positioning (recommended for most use cases)
pattern.stitch(dx, dy);               // Add stitch relative to last position
pattern.jump(dx, dy);                 // Jump relative to last position
pattern.trim();                       // Add trim at current position
pattern.color_change(dx, dy);         // Change color with position offset
pattern.stop();                       // Add stop command
pattern.end();                        // Mark end of pattern

// Absolute positioning
pattern.stitch_abs(x, y);             // Stitch at exact coordinates
pattern.jump_abs(x, y);               // Jump to exact coordinates

// Low-level control
pattern.add_stitch_relative(dx, dy, STITCH);
pattern.add_stitch_absolute(STITCH, x, y);
pattern.add_command(command, x, y);
```

**Example - Creating a Triangle:**

```rust
let mut pattern = EmbPattern::new();
pattern.add_thread(EmbThread::from_string("blue")?);

// Draw a triangle (30mm sides)
pattern.stitch_abs(0.0, 0.0);         // Start at origin
pattern.stitch(300.0, 0.0);           // Right 30mm
pattern.stitch(-150.0, 260.0);        // Up-left (approximate 60°)
pattern.stitch(-150.0, -260.0);       // Down-left back to start
pattern.trim();
pattern.end();
```

#### Managing Threads

```rust
// Add threads with different methods
pattern.add_thread(EmbThread::new(0xFF0000));           // Hex color
pattern.add_thread(EmbThread::from_string("red")?);     // Named color
pattern.add_thread(EmbThread::from_rgb(255, 0, 0));     // RGB values

// Add thread with metadata
let thread = EmbThread::from_string("red")?
    .with_description("Bright Red")
    .with_brand("Madeira")
    .with_catalog_number("1147");
pattern.add_thread(thread);

// Access threads
let threads = pattern.threads();        // Get all threads
let thread_count = threads.len();       // Count threads
```

#### Metadata Management

```rust
// Add metadata
pattern.set_metadata("name", "My Design");
pattern.set_metadata("author", "Your Name");
pattern.set_metadata("copyright", "© 2025");
pattern.set_metadata("notes", "Created with ButaButi");

// Read metadata
if let Some(name) = pattern.get_metadata("name") {
    println!("Design name: {}", name);
}

// Iterate all metadata
for (key, value) in pattern.metadata() {
    println!("{}: {}", key, value);
}
```

#### Pattern Analysis

```rust
// Get pattern boundaries
let (min_x, min_y, max_x, max_y) = pattern.bounds();
let width_mm = (max_x - min_x) / 10.0;
let height_mm = (max_y - min_y) / 10.0;
println!("Size: {:.1}mm x {:.1}mm", width_mm, height_mm);

// Count operations
let stitch_count = pattern.count_stitches();        // Only STITCH commands
let color_changes = pattern.count_color_changes();  // COLOR_CHANGE commands
let total_commands = pattern.stitches().len();      // All commands

// Access raw stitches
for stitch in pattern.stitches() {
    println!("Position: ({}, {}), Command: {}", 
        stitch.x, stitch.y, stitch.command);
}
```

#### Transformations

```rust
// Translate (move) the entire pattern
pattern.translate(100.0, 50.0);  // Move 10mm right, 5mm down

// Center pattern at origin
pattern.move_center_to_origin();

// Example: Center and then move to specific position
pattern.move_center_to_origin();    // Center at (0, 0)
pattern.translate(500.0, 500.0);    // Move to (50mm, 50mm)
```

#### Pattern Processing

```rust
// Interpolate trims (convert TRIM to JUMPs for formats that don't support TRIM)
pattern.interpolate_trims(
    5,          // Insert trim after 5 consecutive jumps
    Some(50.0), // Minimum jump distance (5mm) to trigger trim
    false       // Clipping mode
);

// Handle duplicate color changes
pattern.interpolate_duplicate_color_as_stop();

// Get stitches grouped by color
let stitch_blocks = pattern.get_as_stitchblock();
for (stitches, thread) in stitch_blocks {
    println!("Block with {} stitches in color {}", 
        stitches.len(), thread.hex_color());
}
```

---

### EmbThread - Thread Color Management

`EmbThread` represents embroidery thread with color and metadata.

#### Creating Threads

```rust
// From hex color code
let thread = EmbThread::new(0xFF0000);              // Red (0xRRGGBB)

// From color string
let thread = EmbThread::from_string("red")?;        // Named color
let thread = EmbThread::from_string("#FF0000")?;    // Hex with #
let thread = EmbThread::from_string("ff0000")?;     // Hex without #
let thread = EmbThread::from_string("F00")?;        // Short hex (3 digits)

// From RGB components
let thread = EmbThread::from_rgb(255, 0, 0);        // R, G, B values
```

#### Named Colors

ButaButi supports 140+ CSS/X11 named colors:

```rust
// Common colors
"red", "green", "blue", "white", "black", "yellow", "orange", "purple"

// Extended colors
"crimson", "navy", "olive", "teal", "maroon", "lime", "aqua", "fuchsia"
"coral", "salmon", "khaki", "lavender", "turquoise", "indigo", "gold"

// Color variations
"darkred", "lightblue", "darkgreen", "lightgray", "mediumvioletred"

// All CSS color names are supported
let thread = EmbThread::from_string("cornflowerblue")?;
```

#### Thread Metadata

```rust
// Builder pattern for thread with metadata
let thread = EmbThread::from_string("red")?
    .with_description("Bright Cherry Red")
    .with_brand("Madeira")
    .with_catalog_number("1147")
    .with_chart("Polyneon");

// Access thread properties
println!("Color: {}", thread.hex_color());          // "#ff0000"
println!("Red: {}", thread.red());                  // 255
println!("Green: {}", thread.green());              // 0
println!("Blue: {}", thread.blue());                // 0

// Get RGBA color
let rgba = thread.opaque_color();  // 0xFFFF0000 (with alpha channel)
```

#### Color Operations

```rust
// Change thread color
let mut thread = EmbThread::new(0xFF0000);
thread.set_hex_color("#00FF00")?;  // Change to green

// Calculate color distance (perceptual difference)
let thread1 = EmbThread::from_string("red")?;
let thread2 = EmbThread::from_string("crimson")?;
let distance = thread1.color_distance(thread2.color);
println!("Color difference: {}", distance);

// Find nearest color in palette
let palette = vec![
    EmbThread::from_string("red")?,
    EmbThread::from_string("blue")?,
    EmbThread::from_string("green")?,
];
let my_color = EmbThread::from_string("#FF3333")?;
if let Some(index) = my_color.find_nearest_color_index(&palette) {
    println!("Nearest color: {}", palette[index]);
}
```

---

### EmbMatrix - 2D Transformations

`EmbMatrix` provides affine transformations for patterns.

#### Creating and Using Matrices

```rust
use butabuti::core::matrix::EmbMatrix;

// Create identity matrix
let mut matrix = EmbMatrix::new();

// Apply transformations
matrix.post_translate(100.0, 50.0);    // Move 10mm right, 5mm down
matrix.post_scale(2.0, None, 0.0, 0.0); // Scale 2x from origin
matrix.post_rotate(45.0, 0.0, 0.0);    // Rotate 45° around origin

// Transform a point
let (new_x, new_y) = matrix.transform_point(10.0, 20.0);

// Apply to array in-place
let mut point = [10.0, 20.0];
matrix.apply(&mut point);
```

#### Advanced Transformations

```rust
// Scale with custom origin
let mut matrix = EmbMatrix::new();
matrix.post_scale(
    2.0,        // X scale factor
    Some(1.5),  // Y scale factor (different from X)
    100.0,      // Origin X
    100.0       // Origin Y
);

// Rotate around a point
matrix.post_rotate(
    90.0,   // Degrees (not radians!)
    50.0,   // Center X
    50.0    // Center Y
);

// Check if matrix is identity (no transformation)
if matrix.is_identity() {
    println!("No transformation applied");
}

// Get raw matrix values
let values = matrix.matrix();  // Returns [f64; 9]

// Reset to identity
matrix.reset();

// Compute inverse
matrix.inverse();  // Reverses the transformation
```

---

### Transcoder - Advanced Pattern Processing

The `Transcoder` applies transformations and handles format-specific requirements.

#### Basic Transcoding

```rust
use butabuti::core::encoder::{Transcoder, EncoderSettings};

// Create transcoder with default settings
let mut transcoder = Transcoder::new();

// Transcode pattern
let source = pattern_from_somewhere();
let mut destination = EmbPattern::new();
transcoder.transcode(&source, &mut destination)?;
```

#### Custom Encoder Settings

```rust
let mut settings = EncoderSettings::default();

// Stitch length limits
settings.max_stitch = 120.0;  // Maximum 12mm stitch
settings.max_jump = 500.0;    // Maximum 50mm jump

// Format-specific options
settings.needle_count = 15;                           // Number of needles available
settings.thread_change_command = COLOR_CHANGE;        // Use COLOR_CHANGE command
settings.explicit_trim = true;                        // Add TRIM before color change
settings.round = true;                                // Round coordinates to integers

// Contingency handling
settings.long_stitch_contingency = CONTINGENCY_LONG_STITCH_SEW_TO;
settings.sequin_contingency = CONTINGENCY_SEQUIN_JUMP;

// Create transcoder with custom settings
let mut transcoder = Transcoder::with_settings(settings);
```

#### Contingency Modes

```rust
// Long stitch handling
CONTINGENCY_LONG_STITCH_JUMP_NEEDLE  // Jump to position, then stitch
CONTINGENCY_LONG_STITCH_SEW_TO       // Break into smaller stitches

// Sequin handling
CONTINGENCY_SEQUIN_UTILIZE    // Keep sequin commands
CONTINGENCY_SEQUIN_JUMP       // Convert to jumps
CONTINGENCY_SEQUIN_STITCH     // Convert to stitches
CONTINGENCY_SEQUIN_REMOVE     // Remove sequin commands
```

#### Applying Transformations

```rust
// Create transcoder with transformation matrix
let mut transcoder = Transcoder::new();

// Create and apply matrix
let mut matrix = EmbMatrix::new();
matrix.post_scale(2.0, None, 0.0, 0.0);  // Scale 2x
matrix.post_rotate(45.0, 0.0, 0.0);      // Rotate 45°
transcoder.set_matrix(matrix);

// Transcode with transformation
let mut output = EmbPattern::new();
transcoder.transcode(&input_pattern, &mut output)?;
```

---

### Processing Utilities

The `utils::processing` module provides common pattern operations.

#### Normalize Pattern

```rust
use butabuti::utils::processing;

// Move pattern so minimum coordinates are at (0, 0)
processing::normalize(&mut pattern);

let (min_x, min_y, _, _) = pattern.bounds();
assert_eq!(min_x, 0.0);
assert_eq!(min_y, 0.0);
```

#### Fix Color Count

```rust
// Ensure pattern has enough threads for all color changes
processing::fix_color_count(&mut pattern);

// Adds default colored threads if needed:
// Black, Red, Green, Blue, Yellow, Magenta, Cyan (cycles)
```

#### Remove Duplicates

```rust
// Remove consecutive stitches at same position
processing::remove_duplicates(&mut pattern);
```

#### Calculate Statistics

```rust
use butabuti::utils::processing::{calculate_stats, PatternStats};

let stats = calculate_stats(&pattern);

println!("Stitches: {}", stats.stitch_count);
println!("Jumps: {}", stats.jump_count);
println!("Trims: {}", stats.trim_count);
println!("Color changes: {}", stats.color_change_count);
println!("Total thread length: {:.2}mm", stats.total_length);
println!("Bounds: ({:.1}, {:.1}) to ({:.1}, {:.1})",
    stats.min_x, stats.min_y, stats.max_x, stats.max_y);
```

---

## 🔧 Advanced Features

### Format-Specific Reading

Each format has its own reader module:

```rust
use butabuti::formats::io::readers;
use std::fs::File;

// Read DST file
let mut file = File::open("design.dst")?;
let mut pattern = EmbPattern::new();
readers::dst::read(&mut file, &mut pattern)?;

// Read PES file
let mut file = File::open("design.pes")?;
let mut pattern = EmbPattern::new();
readers::pes::read(&mut file, &mut pattern)?;

// Read JEF file
let mut file = File::open("design.jef")?;
let mut pattern = EmbPattern::new();
readers::jef::read(&mut file, &mut pattern)?;
```

### Format-Specific Writing

```rust
use butabuti::formats::io::writers;
use std::fs::File;

// Write DST file
let mut file = File::create("output.dst")?;
writers::dst::write(&pattern, &mut file)?;

// Write PES file
let mut file = File::create("output.pes")?;
writers::pes::write(&pattern, &mut file)?;

// Export to SVG
let mut file = File::create("output.svg")?;
writers::svg::write(&pattern, &mut file)?;

// Export to PNG (requires "graphics" feature)
#[cfg(feature = "graphics")]
{
    let mut file = File::create("output.png")?;
    writers::png::write(&pattern, &mut file)?;
}
```

### JSON Serialization

Patterns can be serialized to/from JSON:

```rust
use std::fs::File;

// Write as JSON
let mut file = File::create("pattern.json")?;
writers::json::write(&pattern, &mut file)?;

// Read from JSON
let mut file = File::open("pattern.json")?;
let mut pattern = EmbPattern::new();
readers::json::read(&mut file, &mut pattern)?;
```

### CSV Export

Export pattern data to CSV for analysis:

```rust
let mut file = File::create("pattern.csv")?;
writers::csv::write(&pattern, &mut file)?;

// CSV format:
// x,y,command
// 0.0,0.0,0
// 10.0,0.0,0
// ...
```

---

## 📚 Examples

### Complete Format Conversion Tool

```rust
use butabuti::prelude::*;
use butabuti::formats::io::{readers, writers};
use std::fs::File;
use std::path::Path;

fn convert_embroidery_file(
    input_path: &str,
    output_path: &str
) -> Result<(), Box<dyn std::error::Error>> {
    // Read input file
    let mut input = File::open(input_path)?;
    let mut pattern = EmbPattern::new();
    
    // Detect format from extension and read
    let input_ext = Path::new(input_path)
        .extension()
        .and_then(|s| s.to_str())
        .unwrap_or("");
    
    match input_ext.to_lowercase().as_str() {
        "pes" => readers::pes::read(&mut input, &mut pattern)?,
        "dst" => readers::dst::read(&mut input, &mut pattern)?,
        "jef" => readers::jef::read(&mut input, &mut pattern)?,
        "vp3" => {
            drop(input);
            pattern = readers::vp3::read_file(input_path)?;
        },
        _ => return Err("Unsupported input format".into()),
    }
    
    // Write output file
    let mut output = File::create(output_path)?;
    let output_ext = Path::new(output_path)
        .extension()
        .and_then(|s| s.to_str())
        .unwrap_or("");
    
    match output_ext.to_lowercase().as_str() {
        "pes" => writers::pes::write(&pattern, &mut output)?,
        "dst" => writers::dst::write(&pattern, &mut output)?,
        "jef" => writers::jef::write(&pattern, &mut output)?,
        "svg" => writers::svg::write(&pattern, &mut output)?,
        "json" => writers::json::write(&pattern, &mut output)?,
        _ => return Err("Unsupported output format".into()),
    }
    
    println!("Converted {} to {}", input_path, output_path);
    Ok(())
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    convert_embroidery_file("design.pes", "design.dst")?;
    convert_embroidery_file("design.jef", "design.svg")?;
    Ok(())
}
```

### Pattern Transformation Pipeline

```rust
use butabuti::prelude::*;
use butabuti::core::encoder::{Transcoder, EncoderSettings};
use butabuti::core::matrix::EmbMatrix;
use butabuti::utils::processing;

fn transform_pattern(input: &EmbPattern) -> Result<EmbPattern, Error> {
    // Step 1: Normalize position
    let mut working = input.clone();
    processing::normalize(&mut working);
    
    // Step 2: Fix color count
    processing::fix_color_count(&mut working);
    
    // Step 3: Remove duplicates
    processing::remove_duplicates(&mut working);
    
    // Step 4: Apply transformations
    let mut settings = EncoderSettings::default();
    settings.max_stitch = 120.0;    // 12mm max stitch
    settings.round = true;          // Round coordinates
    
    let mut transcoder = Transcoder::with_settings(settings);
    
    // Create transformation matrix
    let mut matrix = EmbMatrix::new();
    matrix.post_scale(1.5, None, 0.0, 0.0);  // Scale 150%
    matrix.post_rotate(15.0, 0.0, 0.0);       // Rotate 15°
    transcoder.set_matrix(matrix);
    
    // Apply transformation
    let mut output = EmbPattern::new();
    transcoder.transcode(&working, &mut output)?;
    
    // Step 5: Center the result
    output.move_center_to_origin();
    
    Ok(output)
}
```

### Pattern Statistics Report

```rust
use butabuti::prelude::*;
use butabuti::utils::processing::calculate_stats;

fn print_pattern_report(pattern: &EmbPattern) {
    let stats = calculate_stats(pattern);
    
    println!("═══════════════════════════════════");
    println!("  EMBROIDERY PATTERN REPORT");
    println!("═══════════════════════════════════");
    
    // Basic info
    if let Some(name) = pattern.get_metadata("name") {
        println!("Name: {}", name);
    }
    if let Some(author) = pattern.get_metadata("author") {
        println!("Author: {}", author);
    }
    
    println!();
    println!("STITCH INFORMATION:");
    println!("  Total stitches: {}", stats.stitch_count);
    println!("  Jump stitches: {}", stats.jump_count);
    println!("  Trim commands: {}", stats.trim_count);
    println!("  Color changes: {}", stats.color_change_count);
    println!("  Total commands: {}", 
        stats.stitch_count + stats.jump_count + 
        stats.trim_count + stats.color_change_count);
    
    println!();
    println!("THREAD INFORMATION:");
    println!("  Number of colors: {}", pattern.threads().len());
    for (i, thread) in pattern.threads().iter().enumerate() {
        print!("  Color {}: {}", i + 1, thread.hex_color());
        if let Some(desc) = &thread.description {
            print!(" ({})", desc);
        }
        println!();
    }
    
    println!();
    println!("DIMENSIONS:");
    let width = (stats.max_x - stats.min_x) / 10.0;
    let height = (stats.max_y - stats.min_y) / 10.0;
    println!("  Width: {:.2}mm", width);
    println!("  Height: {:.2}mm", height);
    println!("  Bounds: ({:.1}, {:.1}) to ({:.1}, {:.1})",
        stats.min_x / 10.0, stats.min_y / 10.0,
        stats.max_x / 10.0, stats.max_y / 10.0);
    
    println!();
    println!("THREAD USAGE:");
    println!("  Total length: {:.2}mm ({:.2}m)", 
        stats.total_length, stats.total_length / 1000.0);
    
    let estimated_time = stats.stitch_count as f64 / 800.0; // ~800 stitches/min
    println!("  Estimated time: {:.1} minutes", estimated_time);
    
    println!("═══════════════════════════════════");
}
```

### Create Complex Shapes

```rust
use butabuti::prelude::*;
use std::f64::consts::PI;

fn create_circle(radius: f64, segments: usize) -> EmbPattern {
    let mut pattern = EmbPattern::new();
    pattern.add_thread(EmbThread::from_string("blue").unwrap());
    
    for i in 0..=segments {
        let angle = 2.0 * PI * (i as f64) / (segments as f64);
        let x = radius * angle.cos();
        let y = radius * angle.sin();
        
        if i == 0 {
            pattern.stitch_abs(x, y);
        } else {
            pattern.stitch_abs(x, y);
        }
    }
    
    pattern.trim();
    pattern.end();
    pattern
}

fn create_star(radius: f64, points: usize) -> EmbPattern {
    let mut pattern = EmbPattern::new();
    pattern.add_thread(EmbThread::from_string("gold").unwrap());
    
    let inner_radius = radius * 0.4;
    
    for i in 0..=(points * 2) {
        let angle = PI * (i as f64) / (points as f64);
        let r = if i % 2 == 0 { radius } else { inner_radius };
        let x = r * angle.cos();
        let y = r * angle.sin();
        pattern.stitch_abs(x, y);
    }
    
    pattern.trim();
    pattern.end();
    pattern
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create a 20mm diameter circle
    let circle = create_circle(100.0, 36);  // radius=10mm, 36 segments
    
    // Create a 5-pointed star
    let star = create_star(100.0, 5);  // radius=10mm, 5 points
    
    Ok(())
}
```

---

## 🔨 Building and Testing

### Build Commands

```powershell
# Standard build
cargo build

# Release build (optimized)
cargo build --release

# With all features
cargo build --features full

# Check without building
cargo check
```

### Testing

```powershell
# Run tests (library only)
cargo test --lib

# Run tests with output
cargo test --lib -- --nocapture

# Test specific module
cargo test --lib pattern::tests

# Run with all features
cargo test --lib --features full
```

### Quality Checks

```powershell
# Lint with Clippy (must pass with zero warnings)
cargo clippy -- -D warnings

# Format code
cargo fmt

# Check formatting
cargo fmt --check

# Generate documentation
cargo doc --no-deps --open
```

### Validation Script

Run all checks at once:

```powershell
.\validate.ps1
```

This runs: build → tests → clippy → format check → docs

---

## 🤝 Contributing

We welcome contributions! Please see [CONTRIBUTING.md](CONTRIBUTING.md) for guidelines.

### Quick Contribution Checklist

- ✅ All tests pass (`cargo test --lib`)
- ✅ No clippy warnings (`cargo clippy -- -D warnings`)
- ✅ Code is formatted (`cargo fmt`)
- ✅ New features have tests
- ✅ Public APIs are documented
- ✅ Clear commit messages

---

## 📄 License

This project is licensed under the MIT License - see the [LICENSE.md](LICENSE.md) file for details.

---

## 🙏 Acknowledgments

- Inspired by the [pyembroidery](https://github.com/EmbroidePy/pyembroidery) project
- Format specifications from the embroidery community
- Built with ❤️ for the embroidery and maker communities

---

## 📞 Support & Contact

- **Issues**: [GitHub Issues](https://github.com/Fahad090NP/ButaButi/issues)
- **Discussions**: [GitHub Discussions](https://github.com/Fahad090NP/ButaButi/discussions)
- **Author**: [Fahad Iftikhar](https://github.com/Fahad090NP)

---

### Made with 🌸 by the ButaButi Team
