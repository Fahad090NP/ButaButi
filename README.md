# Butabuti - Embroidery File Format Library

[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![Rust](https://img.shields.io/badge/rust-1.70%2B-orange.svg)](https://www.rust-lang.org/)
[![CI](https://github.com/Fahad090NP/Butabuti/workflows/CI/badge.svg)](https://github.com/Fahad090NP/Butabuti/actions)
[![codecov](https://codecov.io/gh/Fahad090NP/Butabuti/branch/main/graph/badge.svg)](https://codecov.io/gh/Fahad090NP/Butabuti)

> **Note:** This project is in active development. Features and APIs may change. Contributions are welcome!

A high-performance Rust library for reading, writing, and manipulating embroidery machine files. Butabuti supports 15 embroidery file formats with full read/write support, plus additional export formats.

## Features

- **15 Embroidery Formats** - Full bidirectional support (read & write)
- **Export Formats** - SVG (with realistic rendering), PNG, TXT for visualization
- **Realistic Stitch Rendering** - High-quality SVG export with gradient stitches and rotation
- **Color Group Architecture** - Organize threads into logical groups with auto-grouping by color similarity
- **CLI Tool** - Command-line converter for batch processing and analysis
- **WebAssembly** (Experimental) - Browser-based file conversion infrastructure (API refinement in progress)
- **Batch Processing** - Convert multiple files with parallel processing
- **Pattern Manipulation** - Scale, rotate, translate, and transform designs
- **Thread Management** - Comprehensive color handling with 140+ named colors
- **Type Safety** - Leverage Rust's type system for correctness

## Documentation

📚 **[Complete Documentation](https://github.com/Fahad090NP/Butabuti/wiki)** - User guides, tutorials, and examples

- [Installation Guide](https://github.com/Fahad090NP/Butabuti/wiki/Installation) - Setup and dependencies
- [Quick Start Guide](https://github.com/Fahad090NP/Butabuti/wiki/Quick-Start) - Get started in minutes
- [API Reference](https://github.com/Fahad090NP/Butabuti/wiki/API-Reference) - Complete API documentation
- [Format Support](https://github.com/Fahad090NP/Butabuti/wiki/Format-Support) - All supported formats
- [Examples](https://github.com/Fahad090NP/Butabuti/wiki/Examples) - Code examples and patterns
- [Features](https://github.com/Fahad090NP/Butabuti/wiki/Features) - Complete feature list
- [FAQ](https://github.com/Fahad090NP/Butabuti/wiki/FAQ) - Frequently asked questions

## Quick Start

### Command Line Tool

```bash
# Install the CLI tool
cargo install --path . --bin butabuti

# Convert files
butabuti convert input.dst output.pes

# Show pattern info
butabuti info design.dst

# Batch convert
butabuti batch ./input ./output pes
```

See [docs/CLI.md](docs/CLI.md) for complete CLI documentation.

### Library Usage

Add Butabuti to your `Cargo.toml`:

```toml
[dependencies]
butabuti = "0.1.0"
```

### Create a Pattern

```rust
use butabuti::prelude::*;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut pattern = EmbPattern::new();
    pattern.add_thread(EmbThread::from_string("red")?);
    
    // Create a 10mm square
    pattern.stitch(100.0, 0.0);   // Right 10mm
    pattern.stitch(0.0, 100.0);   // Down 10mm
    pattern.stitch(-100.0, 0.0);  // Left 10mm
    pattern.stitch(0.0, -100.0);  // Up 10mm
    pattern.end();
    
    Ok(())
}
```

### Convert Files

```rust
use butabuti::formats::io::{readers, writers};
use std::fs::File;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Read PES file
    let mut input = File::open("design.pes")?;
    let mut pattern = EmbPattern::new();
    readers::pes::read(&mut input, &mut pattern)?;
    
    // Write as DST
    let mut output = File::create("design.dst")?;
    writers::dst::write(&pattern, &mut output)?;
    
    Ok(())
}
```

## Supported Formats

### Read & Write Support (15 formats)

**Major Machine Formats:** DST (Tajima), PES (Brother), JEF (Janome), VP3 (Pfaff), EXP (Melco), PEC (Brother), XXX (Singer), U01 (Barudan), TBF (Tajima)

**Data Formats:** JSON, CSV, GCode, COL (color list), EDR (Embird color), INF (thread info)

### Export-Only Formats

**Visualization:** SVG (vector graphics), PNG (raster image - requires `graphics` feature), TXT (human-readable)

See [Format Support](https://github.com/Fahad090NP/Butabuti/wiki/Format-Support) for detailed format information.

## Examples

See the [Examples wiki page](https://github.com/Fahad090NP/Butabuti/wiki/Examples) for comprehensive examples including:

- Creating patterns (circles, stars, shapes, spirals, waves)
- Using Color Groups to organize threads
- Reading and writing files
- Batch conversion
- Multi-format export
- Pattern transformations
- Pattern manipulation
- Color management
- Statistics and analysis

Run example programs:

```bash
# Create basic patterns
cargo run --example basic_pattern

# Batch conversion
cargo run --example batch_conversion

# Multi-format export
cargo run --example multi_format_export

# JSON and processing
cargo run --example json_and_processing
```

## Support

- **Issues:** [GitHub Issues](https://github.com/Fahad090NP/Butabuti/issues)
- **Discussions:** [GitHub Discussions](https://github.com/Fahad090NP/Butabuti/discussions)
- **Wiki:** [Documentation](https://github.com/Fahad090NP/Butabuti/wiki)

---

Made with 🌸 by Fahad Iftikhar
