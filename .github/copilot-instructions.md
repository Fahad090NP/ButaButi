# ButaButi Copilot Instructions

## Overview

ButaButi is a high-performance Rust library for reading, writing, and manipulating embroidery files with full read/write support. Core abstractions: `EmbPattern` (stitch sequences), `EmbThread` (colors), command constants (STITCH, JUMP, TRIM, etc.), and format-specific readers/writers.

**Project Type:** Library crate (no binary)  
**Target Users:** Embroidery software developers, digitizers, format conversion tools  
**Key Differentiator:** Full bidirectional format support - only formats with both readers AND writers are included

## Supported Formats

**Embroidery Formats (14 with full read/write):**

- **DST** - Tajima (most common industrial format)
- **PES** - Brother
- **EXP** - Melco
- **JEF** - Janome
- **VP3** - Pfaff
- **PEC** - Brother
- **XXX** - Singer
- **U01** - Barudan
- **TBF** - Tajima
- **COL** - Thread color list
- **EDR** - Embird color
- **INF** - Thread information
- **JSON** - JSON embroidery data
- **CSV** - CSV embroidery data
- **GCODE** - GCODE embroidery data

**Export-Only Formats:**

- **SVG** - Scalable vector graphics
- **PNG** - Raster image (requires `graphics` feature)
- **TXT** - Human-readable text

## Architecture

### Module Structure

- **`src/core/`** - Core types: `EmbPattern`, `EmbThread`, `EmbMatrix`, `Transcoder`, command constants
- **`src/formats/io/`** - Format readers (`readers/*.rs`) and writers (`writers/*.rs`)
- **`src/palettes/`** - Thread color palettes (JEF, PEC, SEW, SHV)
- **`src/utils/`** - Error handling (`Error`/`Result`), compression, processing utilities, batch conversion

### Key Design Patterns

#### Coordinate System

All coordinates in **0.1mm units** (tenths of millimeters). Example: `100.0` = 10mm.

```rust
pattern.stitch(100.0, 0.0);  // Move 10mm right
```

#### Command System

Commands are `u32` bit flags from `core/constants.rs`:

- Low byte (0xFF): Core command (STITCH=0, JUMP=1, TRIM=2, COLOR_CHANGE=5, END=4)
- Upper 24 bits: Metadata (thread index, needle number, sequencing)

```rust
const COMMAND_MASK: u32 = 0x0000_00FF;  // Extract core command
const THREAD_MASK: u32 = 0x0000_FF00;   // Thread info in bits 8-15
```

#### Reader/Writer Convention

**Readers** mutate an existing `EmbPattern` (critical: pattern must be pre-initialized):

```rust
// Most formats - mutate pattern in-place
pub fn read(file: &mut impl Read, pattern: &mut EmbPattern) -> Result<()>

// Formats requiring Seek (e.g., JEF, BRO, HUS) - read header/footer separately
pub fn read(file: &mut (impl Read + Seek), pattern: &mut EmbPattern) -> Result<()>

// Exception: VP3 and PES return new pattern (legacy API, pre-refactor)
pub fn read<R: Read + Seek>(reader: &mut R) -> Result<EmbPattern>
pub fn read_file(path: &str) -> Result<EmbPattern>  // Convenience wrapper
```

**Writers** write immutable pattern to stream:

```rust
pub fn write(pattern: &EmbPattern, file: &mut impl Write) -> Result<()>
```

**Why mutation?** Allows reusing pattern buffers in batch processing and avoids cloning large stitch arrays.

**Binary I/O Helpers**: Import `formats::io::utils::WriteHelper` for binary writes - provides methods like `write_u8()`, `write_u16_le()`, `write_string_fixed()` to enforce format-specific requirements.

#### Pattern Building API

Use convenience methods over raw stitch commands:

```rust
pattern.stitch(dx, dy);      // Relative stitch (incremental)
pattern.stitch_abs(x, y);    // Absolute stitch
pattern.jump(dx, dy);        // Jump without stitching
pattern.trim();              // Add trim command
pattern.color_change(0.0, 0.0); // Change thread color
pattern.end();               // End pattern
```

## Development Workflow

### Build & Test Commands

```powershell
cargo build                    # Standard build
cargo test --lib              # Run tests (excludes integration tests)
cargo clippy -- -D warnings   # Lint (MUST pass with zero warnings)
cargo fmt                     # Format code
.\validate.ps1                # Run all checks (build, test, clippy, fmt, docs)
```

**Critical**:

- Always run `cargo test --lib` (not `cargo test`) - project uses library-only tests
- **Always run `.\validate.ps1` before considering work complete** - this is the authoritative pre-commit check
- No binary targets (library-only project)

### Feature Flags

Enable optional features via Cargo.toml:

```toml
[dependencies]
butabuti = { version = "0.1.0", features = ["graphics", "parallel"] }
# or
butabuti = { version = "0.1.0", features = ["full"] }  # All features
```

- `graphics` - PNG export via `image` crate (adds `writers::png`)
- `parallel` - Parallel batch processing via `rayon` (speeds up `BatchConverter`)
- `full` - All optional features enabled

### Adding New Formats

#### Reader Template

1. Create `src/formats/io/readers/formatname.rs`
2. Implement signature: `pub fn read(file: &mut impl Read, pattern: &mut EmbPattern) -> Result<()>`
   - If format needs random access (headers/footers), use `impl Read + Seek`
3. Parse header → extract metadata → decode stitches → add to pattern
   - **Critical**: Use `pattern.add_stitch_relative()` for delta-encoded formats
   - Use `pattern.add_stitch_absolute()` for absolute coordinate formats
   - Add threads via `pattern.add_thread()` as discovered (order matters!)
   - Extract metadata with `pattern.add_metadata(key, value)` - see DST reader for examples
4. Export in `src/formats/io/readers.rs`: `pub mod formatname;`
5. Add tests with real file samples from `testing/` directory

#### Writer Template

1. Create `src/formats/io/writers/formatname.rs`
2. Implement: `pub fn write(pattern: &EmbPattern, file: &mut impl Write) -> Result<()>`
3. Write header → encode stitches → write footer
   - **Critical**: Most formats require fixed header sizes (DST=512, PES varies by version)
   - Use `WriteHelper` from `formats::io::utils` for binary writes
   - Import: `use crate::formats::io::utils::WriteHelper;`
   - Usage: `file.write_u16_le(value)?;` or `file.write_string_fixed("text", 16)?;`
4. Export in `src/formats/io/writers.rs`
5. Add round-trip test if reader exists (read → write → read → compare stitch counts)

### Format-Specific Encoding

Many formats use **bit-encoded deltas** (DST, PEC, etc.):

```rust
// DST encodes +/-121 range in 3 bytes with ternary encoding
fn decode_dx(b0: u8, b1: u8, b2: u8) -> i32 {
    let mut x = 0;
    x += get_bit(b2, 2) * 81;   // +81 if bit set
    x += get_bit(b2, 3) * -81;  // -81 if bit set
    // ... continues with ternary decomposition
}
```

See `readers/dst.rs` and `writers/dst.rs` for reference implementation.

## Critical Conventions

### Error Handling

Use `Result<T>` from `utils/error.rs`. Never panic in library code:

```rust
use crate::utils::error::{Error, Result};

// Prefer descriptive error contexts with format! for dynamic messages
Err(Error::Parse(format!("Invalid header size: expected 512, got {}", size)))

// Use appropriate error variants
Error::Io(io_error)           // I/O failures (auto-converted via From trait)
Error::Parse(msg)             // Format parsing issues
Error::UnsupportedFormat(msg) // Format not supported
Error::InvalidPattern(msg)    // Pattern validation failures
Error::Encoding(msg)          // Encoding/writing errors
```

### Thread Color Parsing

`EmbThread::from_string()` accepts hex or named colors:

- `EmbThread::from_string("red")?` - Named color
- `EmbThread::from_string("FF0000")?` - Hex color (with or without # prefix)

### Pattern Transformations

Use `Transcoder` for complex operations (splitting long stitches, handling trims):

```rust
let mut transcoder = Transcoder::with_settings(settings);
transcoder.transcode(&source_pattern, &mut dest_pattern)?;
```

For simple transforms, use pattern methods: `translate()`, `move_center_to_origin()`.

### Processing Utilities

`utils/processing.rs` provides common operations:

- `normalize(pattern)` - Move pattern to (0,0)
- `fix_color_count(pattern)` - Add missing threads for color changes
- `interpolate_trims(pattern, max_jump)` - Convert TRIMs to JUMPs for unsupported formats

## Testing Requirements

- All new features need unit tests in cfg(test) modules
- Test edge cases: empty patterns, single stitch, invalid data
- Format readers: test with real file samples from `examples/` or fixtures
- Round-trip tests: read → write → read → compare

## Known Gotchas

- **Coordinates**: Always in 0.1mm units, not pixels or mm
- **Y-axis**: Some formats flip Y (DST uses `y = -y`), handle in reader/writer
- **Thread indices**: Start at 0, not 1
- **Header sizes**: Fixed for many formats (DST=512, PES=48), enforce strictly
- **Huffman compression**: Required for HUS writer (see `utils/compress.rs`)

## Common Tasks

### Read a pattern

```rust
use butabuti::prelude::*;
use std::fs::File;

let mut pattern = EmbPattern::new();
let mut file = File::open("design.dst")?;

// Invoke format-specific reader from formats::io::readers
butabuti::formats::io::readers::dst::read(&mut file, &mut pattern)?;
```

### Create pattern programmatically

```rust
use butabuti::prelude::*;

let mut pattern = EmbPattern::new();
pattern.add_thread(EmbThread::from_string("red")?);
pattern.stitch(100.0, 0.0);  // 10mm right (relative to previous position)
pattern.stitch(0.0, 100.0);  // 10mm down
pattern.trim();
pattern.end();
```

### Write a pattern

```rust
use butabuti::prelude::*;
use std::fs::File;

let mut file = File::create("output.pes")?;
butabuti::formats::io::writers::pes::write(&pattern, &mut file)?;
```

### Get pattern statistics

```rust
let (min_x, min_y, max_x, max_y) = pattern.bounds();
let width_mm = (max_x - min_x) / 10.0;
let stitch_count = pattern.count_stitches();
let color_changes = pattern.count_color_changes();
```

### Batch conversion

```rust
use butabuti::prelude::*;

let converter = BatchConverter::new()
    .input_dir("input/")
    .output_dir("output/")
    .target_format("pes")
    .input_extensions(&["dst", "exp"])
    .parallel(true)
    .overwrite(true)
    .build();

let results = converter.convert_all()?;
results.print_summary();
```

## Do's and Don'ts

### ✅ DO

- **Run tests after every change**: `cargo test --lib` must pass with 0 failures
- **Fix clippy warnings**: `cargo clippy -- -D warnings` must produce zero warnings
- **Format code**: Run `cargo fmt` before committing
- **Write unit tests**: Every new function/feature needs cfg(test) module tests
- **Use builder patterns**: For complex configuration (see `BatchConverter`, `MultiFormatExporter`)
- **Handle errors gracefully**: Use `Result<T>` and proper error messages, never `panic!()` in library code
- **Document public APIs**: Add doc comments (`///`) for all public functions, structs, and methods
- **Follow coordinate system**: Always use 0.1mm units (100.0 = 10mm)
- **Validate inputs**: Check bounds, formats, and preconditions before processing
- **Update TODOS.md**: Mark features as `[x]` when completed

### ❌ DON'T

- **Don't create markdown files automatically**: Documentation files should only be created when explicitly requested
  - There is a `documentation/` folder for docs - only add files there when instructed
  - Don't create summary files like `IMPLEMENTATION.md`, `SUMMARY.md`, etc. after changes
  - README.md and TODOS.md are the only markdown files to update routinely
- **Don't use `panic!()` in library code**: Always return `Result` with descriptive errors
- **Don't use `unwrap()` without good reason**: Prefer `?` operator or proper error handling
- **Don't make breaking API changes**: Maintain backward compatibility for public APIs
- **Don't skip validation**: Always run `.\validate.ps1` before considering work complete
- **Don't commit with warnings**: Code must be clippy-clean with `-D warnings`
- **Don't use magic numbers**: Define constants for format-specific values
- **Don't forget Y-axis conventions**: Some formats flip Y-coordinates (document this)
- **Don't mix coordinate systems**: Stick to 0.1mm units throughout
- **Don't create docs without request**: Wait for explicit instruction to create documentation

## Resources

- Format specs in inline comments (e.g., `readers/dst.rs` documents DST encoding)
- Thread palettes: `palettes/thread_*.rs` (brand-specific color mappings)
- TODO list: `TODOS.md` (comprehensive feature roadmap)
- Contributing guide: `CONTRIBUTING.md` (PR requirements, code standards)
- Documentation folder: `documentation/` (only add files when explicitly requested)
