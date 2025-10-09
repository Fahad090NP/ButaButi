# ButaButi - Embroidery File Format Library

[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![Rust](https://img.shields.io/badge/rust-1.70%2B-orange.svg)](https://www.rust-lang.org/)

> **Note:** This project is in early stage development. Features and APIs may change. Contributions are welcome!

A high-performance Rust library for reading, writing, and manipulating embroidery machine files. ButaButi supports 40+ embroidery file formats with powerful batch conversion and pattern manipulation tools.

## Features

- **40+ Format Support** - Read from 47 formats, write to 18 formats
- **Batch Processing** - Convert multiple files with parallel processing
- **Pattern Manipulation** - Scale, rotate, translate, and transform designs
- **Thread Management** - Comprehensive color handling with 140+ named colors
- **Type Safety** - Leverage Rust's type system for correctness
- **Export Options** - PNG, SVG, JSON, CSV, and text formats

## Quick Start

Add ButaButi to your `Cargo.toml`:

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

### Read Support (47 formats)

**Major Formats:** DST, PES, JEF, VP3, EXP, XXX, U01, HUS, SEW, SHV, PEC, JSON, CSV

**Additional:** BRO, COL, DAT, DSB, DSZ, EDR, EMD, EXY, FXY, GCode, GT, INB, INF, JPX, KSM, MAX, MIT, NEW, PCD, PCM, PCQ, PCS, PHB, PHC, PMV, SPX, STC, STX, TAP, TBF, VIP, ZHS, ZXY, and more

### Write Support (18 formats)

**Machine Formats:** DST, PES, JEF, VP3, EXP, XXX, U01, PEC

**Export Formats:** SVG, PNG, JSON, CSV, TXT, GCode, COL, EDR, INF, TBF

## Documentation

📚 **[Complete Documentation](https://github.com/Fahad090NP/ButaButi/wiki)**

- [Installation Guide](https://github.com/Fahad090NP/ButaButi/wiki/Installation)
- [Quick Start Guide](https://github.com/Fahad090NP/ButaButi/wiki/Quick-Start)
- [API Reference](https://github.com/Fahad090NP/ButaButi/wiki/API-Reference)
- [Format Support](https://github.com/Fahad090NP/ButaButi/wiki/Format-Support)
- [Batch Conversion](https://github.com/Fahad090NP/ButaButi/wiki/Batch-Conversion)
- [Examples](https://github.com/Fahad090NP/ButaButi/wiki/Examples)
- [FAQ](https://github.com/Fahad090NP/ButaButi/wiki/FAQ)

## Examples

See the [Examples wiki page](https://github.com/Fahad090NP/ButaButi/wiki/Examples) for comprehensive examples including:

- Creating patterns (circles, stars, shapes)
- Reading and writing files
- Batch conversion
- Pattern manipulation
- Color management
- Statistics and analysis

## Support

- **Issues:** [GitHub Issues](https://github.com/Fahad090NP/ButaButi/issues)
- **Discussions:** [GitHub Discussions](https://github.com/Fahad090NP/ButaButi/discussions)
- **Wiki:** [Documentation](https://github.com/Fahad090NP/ButaButi/wiki)

---

Made with 🌸 by the Fahad Iftikhar
