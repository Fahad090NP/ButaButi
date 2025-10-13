# Butabuti Copilot Instructions

## Quick Reference (Critical Facts)

-   **Always run `.\validate.ps1`** before considering work complete (build + test + clippy + fmt + docs)
-   **Test command**: `cargo test --lib` (NOT `cargo test` - no integration tests)
-   **Coordinate units**: 0.1mm (so `100.0` = 10mm)
-   **Reader pattern**: Mutate existing `EmbPattern` via `pub fn read(file: &mut impl Read, pattern: &mut EmbPattern) -> Result<()>`
-   **Binary I/O**: Use `WriteHelper` trait from `formats::io::utils` (auto-implemented for all `Write` types)
-   **No auto-docs**: Never create markdown files after changes unless explicitly requested
-   **Error handling**: Always return `Result`, never `panic!()` in library code
-   **Test isolation**: Use cfg(test) modules, test with files from `testing/` directory
-   **File naming**: Prefer descriptive compound names (e.g., `stitch_renderer.rs` over `renderer.rs`) when parent folder alone is ambiguous
-   **No script automation for docs**: Never use scripts to auto-generate anything even documentation; update manually or add TODO items instead

## Overview

Butabuti is a high-performance Rust library for reading, writing, and manipulating embroidery files with full read/write support. Core abstractions: `EmbPattern` (stitch sequences), `EmbThread` (colors), command constants (STITCH, JUMP, TRIM, etc.), and format-specific readers/writers.

**Project Type:** Library crate + CLI binary (`src/bin/butabuti.rs`)  
**Target Users:** Embroidery software developers, digitizers, format conversion tools  
**Key Differentiator:** Full bidirectional format support - only formats with both readers AND writers are included

## Architecture

### Module Structure

```
src/
├── bin/                  # CLI binary
│   └── butabuti.rs       # Command-line tool (convert, info, validate, batch)
├── core/                 # Core abstractions
│   ├── pattern.rs        # EmbPattern (stitches + metadata)
│   ├── thread.rs         # EmbThread (colors)
│   ├── color_group.rs    # ColorGroup (organize threads by category)
│   ├── collection.rs     # Pattern collections
│   ├── constants.rs      # Command bit flags (STITCH, JUMP, TRIM)
│   ├── encoder.rs        # Transcoder for complex transforms
│   └── matrix.rs         # Transformation matrices
├── formats/              # Format I/O and registry
│   ├── registry.rs       # FormatRegistry (dynamic format discovery)
│   └── io/               # Format I/O (15 bidirectional formats)
│       ├── readers/      # DST, PES, JEF, VP3, EXP, etc.
│       ├── writers/      # Paired writers + SVG, PNG, TXT
│       ├── detector.rs   # Format detection from file content
│       └── utils.rs      # ReadHelper/WriteHelper for binary I/O
├── palettes/             # Thread color databases
│   └── thread_*.rs       # JEF, PEC, SEW, SHV, HUS brand palettes
└── utils/                # Cross-cutting concerns
    ├── error.rs          # Error/Result types
    ├── batch.rs          # BatchConverter/MultiFormatExporter
    ├── processing.rs     # Pattern utilities (normalize, fix_color_count)
    ├── palette.rs        # Thread palette management
    └── compress.rs       # Huffman compression (for HUS format)
```

### Key Design Patterns

#### Coordinate System

All coordinates in **0.1mm units** (tenths of millimeters). Example: `100.0` = 10mm.

```rust
pattern.stitch(100.0, 0.0);  // Move 10mm right
```

**Why 0.1mm?** Industry standard for embroidery machines - allows precise stitch placement without floating-point precision issues.

#### Command System

Commands are `u32` bit flags from `core/constants.rs`:

-   Low byte (0xFF): Core command (STITCH=0, JUMP=1, TRIM=2, COLOR_CHANGE=5, END=4)
-   Upper 24 bits: Metadata (thread index, needle number, sequencing)

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

**Binary I/O Helpers**: Import `formats::io::utils::WriteHelper` for binary writes - provides methods like `write_u8()`, `write_u16_le()`, `write_string_fixed()` to enforce format-specific requirements. The trait is automatically implemented for any type implementing `std::io::Write`.

```rust
use crate::formats::io::utils::WriteHelper;
use std::io::Write;

pub fn write(pattern: &EmbPattern, file: &mut impl Write) -> Result<()> {
    file.write_u16_le(0x1234)?;  // Little-endian u16
    file.write_string_fixed("DST", 3)?;  // Fixed-length string
    // ...
}
```

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

-   Always run `cargo test --lib` (not `cargo test`) - project uses library-only tests
-   **Always run `.\validate.ps1` before considering work complete** - this is the authoritative pre-commit check
-   CLI binary available: `cargo run --bin butabuti -- <command>` for testing CLI functionality

### CLI Tool Commands

The CLI binary in `src/bin/butabuti.rs` provides command-line utilities:

```sh
# Convert between formats
cargo run --bin butabuti -- convert input.dst output.pes

# Display pattern info
cargo run --bin butabuti -- info design.dst

# Validate pattern file
cargo run --bin butabuti -- validate pattern.pes

# Batch convert directory
cargo run --bin butabuti -- batch ./input ./output pes

# List supported formats
cargo run --bin butabuti -- list-formats
```

When adding CLI commands, update the match statement in `main()` and `print_usage()` function.

### WASM Build Commands

Build WebAssembly bindings for browser use:

```powershell
# Build WASM package
wasm-pack build --target web --features wasm

# Move output to wasm directory
Move-Item pkg wasm/pkg

# Or use build script
.\wasm\build.ps1

# Test locally with HTTP server
cd wasm
npx http-server -p 8000 -c-1
# Then open http://localhost:8000
```

**WASM Requirements**:

-   `wasm-pack` installed: `cargo install wasm-pack`
-   `[lib] crate-type = ["cdylib", "rlib"]` in Cargo.toml (already configured)
-   Must be served over HTTP (not `file://`) due to WASM security requirements
-   See `Butabuti.wiki/WASM-Browser-Support.md` for complete documentation

### Feature Flags

Enable optional features via Cargo.toml:

```toml
[dependencies]
butabuti = { version = "0.1.0", features = ["graphics", "parallel"] }
# or
butabuti = { version = "0.1.0", features = ["full"] }  # All features
```

-   `graphics` - PNG export via `image` crate (adds `writers::png`)
-   `parallel` - Parallel batch processing via `rayon` (speeds up `BatchConverter`)
-   `full` - All optional features enabled

### Adding New Formats

#### Reader Template

1. Create `src/formats/io/readers/formatname.rs`
2. Implement signature: `pub fn read(file: &mut impl Read, pattern: &mut EmbPattern) -> Result<()>`
    - If format needs random access (headers/footers), use `impl Read + Seek`
    - Pattern is mutated in-place (caller provides empty or pre-initialized pattern)
3. Parse header → extract metadata → decode stitches → add to pattern
    - **Critical**: Use `pattern.add_stitch_relative()` for delta-encoded formats (DST, PEC, etc.)
    - Use `pattern.add_stitch_absolute()` for absolute coordinate formats (rare)
    - Add threads via `pattern.add_thread()` as discovered (order matters!)
    - Extract metadata with `pattern.add_metadata(key, value)` - see DST reader for examples
    - Pattern tracks `previous_x`/`previous_y` internally for relative stitching
4. Export in `src/formats/io/readers.rs`: `pub mod formatname;`
5. **Register format**: Add to `FormatRegistry` in `src/formats/registry.rs` (see existing entries)
6. Add tests with real file samples (place test files in workspace root or examples/)
    - Test pattern: `cargo test --lib readers::formatname`

#### Writer Template

1. Create `src/formats/io/writers/formatname.rs`
2. Implement: `pub fn write(pattern: &EmbPattern, file: &mut impl Write) -> Result<()>`
    - Pattern is immutable (read-only access)
3. Write header → encode stitches → write footer
    - **Critical**: Most formats require fixed header sizes (DST=512, PES varies by version)
    - Use `WriteHelper` trait from `formats::io::utils` for binary writes
    - Import: `use crate::formats::io::utils::WriteHelper;`
    - The trait is automatically implemented for any `Write` type
    - Usage: `file.write_u16_le(value)?;` or `file.write_string_fixed("text", 16)?;`
    - Always check format specs for endianness (LE vs BE)
4. Export in `src/formats/io/writers.rs`: `pub mod formatname;`
5. **Register format**: Add to `FormatRegistry` in `src/formats/registry.rs` (see `write_pattern()` method)
6. Add round-trip test if reader exists (read → write → read → compare stitch counts)
    - Test pattern: `cargo test --lib writers::formatname`

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

-   `EmbThread::from_string("red")?` - Named color
-   `EmbThread::from_string("FF0000")?` - Hex color (with or without # prefix)

### Pattern Transformations

Use `Transcoder` for complex operations (splitting long stitches, handling trims):

```rust
let mut transcoder = Transcoder::with_settings(settings);
transcoder.transcode(&source_pattern, &mut dest_pattern)?;
```

For simple transforms, use pattern methods: `translate()`, `move_center_to_origin()`.

### Processing Utilities

`utils/processing.rs` provides common operations:

-   `normalize(pattern)` - Move pattern to (0,0)
-   `fix_color_count(pattern)` - Add missing threads for color changes
-   `interpolate_trims(pattern, max_jump)` - Convert TRIMs to JUMPs for unsupported formats

### Format Registry

`formats/registry.rs` provides centralized format management:

```rust
use butabuti::formats::registry::FormatRegistry;

let registry = FormatRegistry::new();

// Get format info
let format = registry.get_format_from_path("design.dst").unwrap();
assert_eq!(format.name, "DST");
assert!(format.can_read && format.can_write);

// Read/write using registry
let mut file = File::open("design.dst")?;
let pattern = registry.read_pattern(&mut file, "DST")?;

let mut output = File::create("design.pes")?;
registry.write_pattern(&pattern, &mut output, "PES")?;
```

When adding new formats, update registry in two places:

1. Add `FormatInfo` entry to `FormatRegistry::new()`
2. Add match arm to `read_pattern()` and `write_pattern()` methods

### Color Groups

Organize threads into logical categories using `ColorGroup`:

```rust
use butabuti::core::color_group::{ColorGroup, ThreadGrouping};

// Create groups manually
let mut skin = ColorGroup::new("Skin Tones")
    .with_description("All skin colors");
skin.add_thread(0);
skin.add_thread(1);

// Auto-group by similarity
let grouping = ThreadGrouping::from_pattern(&pattern);
let grouped = grouping.auto_group_by_similarity(0.15)?; // 15% color difference
```

See `src/core/color_group.rs` for full API including hierarchical groups, metadata, and visibility controls.

## Testing Requirements

-   All new features need unit tests in cfg(test) modules
-   Test edge cases: empty patterns, single stitch, invalid data
-   Format readers: test with real file samples (place test files in workspace root or `examples/`)
-   Round-trip tests: read → write → read → compare stitch counts
-   **Critical**: Always run `cargo test --lib` (not `cargo test`) - project has no integration tests
-   Test commands:
    -   `cargo test --lib` - Run all library tests
    -   `cargo test --lib pattern` - Test pattern-related code
    -   `cargo test --lib readers::dst` - Test specific format reader

## Known Gotchas

-   **Coordinates**: Always in 0.1mm units, not pixels or mm
-   **Y-axis**: Some formats flip Y (DST uses `y = -y`), handle in reader/writer
-   **Thread indices**: Start at 0, not 1
-   **Header sizes**: Fixed for many formats (DST=512, PES=48), enforce strictly
-   **Huffman compression**: Required for HUS writer (see `utils/compress.rs`)

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

### Implementing a new format reader

**Critical workflow**: When adding a new format, you MUST update multiple files in sequence:

1. **Create reader**: `src/formats/io/readers/formatname.rs`

    ```rust
    use crate::core::pattern::EmbPattern;
    use crate::utils::error::{Error, Result};
    use std::io::Read;

    pub fn read(file: &mut impl Read, pattern: &mut EmbPattern) -> Result<()> {
        // Parse header, decode stitches, populate pattern
        // Use pattern.add_stitch_relative() for delta formats
        // Use pattern.set_metadata() to store file metadata
        Ok(())
    }
    ```

2. **Export module**: Add to `src/formats/io/readers.rs`

    ```rust
    /// FORMATNAME format reader
    pub mod formatname;
    ```

3. **Create writer** (if bidirectional): `src/formats/io/writers/formatname.rs`

    ```rust
    use crate::core::pattern::EmbPattern;
    use crate::formats::io::utils::WriteHelper;
    use crate::utils::error::Result;
    use std::io::Write;

    pub fn write(pattern: &EmbPattern, file: &mut impl Write) -> Result<()> {
        // Write header, encode stitches, write footer
        // Use WriteHelper methods: file.write_u16_le(), etc.
        Ok(())
    }
    ```

4. **Export writer module**: Add to `src/formats/io/writers.rs`

5. **Add tests**: In both reader and writer files

    ```rust
    // Add cfg(test) attribute to test module
    mod tests {
        use super::*;
        use crate::core::pattern::EmbPattern;
        use std::io::Cursor;

        // Add test attribute to test function
        fn test_read_formatname() {
            let data = vec![/* test bytes */];
            let mut pattern = EmbPattern::new();
            read(&mut Cursor::new(data), &mut pattern).unwrap();
            assert!(pattern.stitches().len() > 0);
        }
    }
    ```

6. **Run validation**: `.\validate.ps1` must pass with zero errors

## Do's and Don'ts

### ✅ DO

-   **Update Documentations**: On every little change in our codebase, update documentation in `Butabuti.wiki/`
-   **Clean Documentation**: Make sure you write clean and concise documentation in the wiki, this documentation is for the end user.
-   **Run tests after every change**: `cargo test --lib` must pass with 0 failures
-   **Fix clippy warnings**: `cargo clippy -- -D warnings` must produce zero warnings
-   **Format code**: Run `cargo fmt` before committing
-   **Write unit tests**: Every new function/feature needs cfg(test) module tests
-   **Use builder patterns**: For complex configuration (see `BatchConverter`, `MultiFormatExporter`)
-   **Handle errors gracefully**: Use `Result<T>` and proper error messages, never `panic!()` in library code
-   **Document public APIs**: Add doc comments (`///`) for all public functions, structs, and methods
-   **Follow coordinate system**: Always use 0.1mm units (100.0 = 10mm)
-   **Validate inputs**: Check bounds, formats, and preconditions before processing
-   **Update TODOS.md**: Mark features as `[x]` when completed

### ❌ DON'T

-   **Don't create markdown files automatically**: Documentation files should only be created when explicitly requested
    -   Wiki documentation lives in `Butabuti.wiki/` folder
    -   Don't create summary files like `IMPLEMENTATION.md`, `SUMMARY.md`, etc. after changes
    -   README.md, IMPROVEMENTS.md, and TODOS.md are the only markdown files to update routinely
-   **Don't use scripts for automation of sensitive tasks**: Never automate documentation creation, file generation, or other sensitive operations with scripts
    -   Add TODO items instead of creating automated documentation generators
    -   Manual updates ensure quality and accuracy
    -   Scripts are OK for build/test/format tasks (validate.ps1, build.ps1)
-   **Don't use `panic!()` in library code**: Always return `Result` with descriptive errors
-   **Don't use `unwrap()` without good reason**: Prefer `?` operator or proper error handling
-   **Don't make breaking API changes**: Maintain backward compatibility for public APIs
-   **Don't skip validation**: Always run `.\validate.ps1` before considering work complete
-   **Don't commit with warnings**: Code must be clippy-clean with `-D warnings`
-   **Don't use magic numbers**: Define constants for format-specific values
-   **Don't forget Y-axis conventions**: Some formats flip Y-coordinates (document this)
-   **Don't mix coordinate systems**: Stick to 0.1mm units throughout
-   **Don't create docs without request**: Wait for explicit instruction to create documentation
-   **Don't use single-word file names when ambiguous**: Prefer descriptive compound names (e.g., `stitch_renderer.rs` over `renderer.rs`)
    -   Exception: When parent folder name provides sufficient context (e.g., `formats/registry.rs` is clear)
    -   Rationale: Searchability and clarity - `stitch_renderer` is more specific than `renderer`

## File Organization & Naming Conventions

### Naming Philosophy

**Prefer descriptive compound names over single words** to maximize clarity and searchability:

-   ✅ **GOOD**: `stitch_renderer.rs`, `color_group.rs`, `batch_converter.rs`
-   ❌ **BAD**: `renderer.rs`, `group.rs`, `converter.rs`

**Rationale:**

-   Parent folder name alone may not provide sufficient context
-   Compound names improve IDE search and codebase navigation
-   Clear intent: `stitch_renderer` is unambiguous, `renderer` could be anything
-   Grep/search friendliness: `stitch_renderer` has fewer false positives

### When Single Words Are Acceptable

Single-word names are OK when:

1. Parent folder provides full context: `formats/registry.rs` (clearly a format registry)
2. Module is universally understood: `error.rs`, `constants.rs`, `utils.rs`
3. No ambiguity exists: `pattern.rs` in `core/` (clearly the core pattern type)

### File Consolidation Guidelines

**Merge files when:**

-   < 200 lines each and closely related functionality
-   Tight coupling (one file can't exist without the other)
-   Shared test fixtures and dependencies

**Keep files separate when:**

-   > 300 lines (large, distinct modules)
-   Independent functionality (can evolve separately)
-   Different test requirements
-   Distinct conceptual boundaries

### Current File Organization Review

**✅ CORRECT naming (descriptive compounds):**

-   `src/utils/stitch_renderer.rs` - Renders stitches (not just any renderer)
-   `src/core/color_group.rs` - Color-specific groups (not generic groups)
-   `src/formats/io/detector.rs` - Format detection (clear purpose)

**✅ ACCEPTABLE single words (sufficient context):**

-   `src/core/pattern.rs` - Core pattern type (folder provides context)
-   `src/core/thread.rs` - Core thread type (folder provides context)
-   `src/core/encoder.rs` - Pattern encoder (clear in core/)
-   `src/core/matrix.rs` - Transformation matrix (mathematical concept)
-   `src/core/constants.rs` - Command constants (universal concept)
-   `src/core/collection.rs` - Pattern collection (clear in core/)
-   `src/utils/error.rs` - Error types (universal concept)
-   `src/utils/compress.rs` - Compression utilities (specific to HUS format)
-   `src/utils/batch.rs` - Batch processing (clear functionality)
-   `src/utils/palette.rs` - Palette utilities (clear functionality)
-   `src/utils/processing.rs` - Pattern processing (clear in utils/)
-   `src/utils/string.rs` - String utilities (universal concept)
-   `src/utils/functions.rs` - Encoding/decoding functions (could be renamed to `encoding.rs` for clarity)
-   `src/formats/registry.rs` - Format registry (folder provides context)

**⚠️ CONSIDER renaming (for consistency):**

-   `src/utils/functions.rs` → `src/utils/encoding.rs` (more descriptive of actual purpose: encode_thread_change, decode functions)
-   No other changes needed - current structure is well-organized

### File Merging Analysis

**DO NOT merge these pairs** (distinct responsibilities, sufficient size):

-   `error.rs` + `processing.rs` - Different concerns (error types vs pattern processing)
-   `batch.rs` + `processing.rs` - Batch operations vs single-pattern utilities
-   `palette.rs` + `compress.rs` - Palette management vs Huffman compression
-   `functions.rs` + `constants.rs` - Encoding helpers vs constant definitions

**All current files should remain separate** - each has a clear, distinct purpose and sufficient complexity.

### Automation Policy

**NEVER automate these with scripts:**

-   Documentation generation (markdown files, wikis)
-   Code file creation from templates
-   API documentation extraction
-   Changelog generation
-   Release note compilation

**Scripts are ONLY for:**

-   Build process (Cargo, wasm-pack)
-   Test execution (cargo test, validate.ps1)
-   Code formatting (cargo fmt)
-   Linting (cargo clippy)
-   Deployment (wasm/build.ps1 for WASM compilation)

**Instead of scripts, add TODO items:**

```markdown
-   [ ] Update API documentation for new feature X
-   [ ] Document format Y in wiki
-   [ ] Add example for use case Z
```

## Resources

-   Format specs in inline comments (e.g., `readers/dst.rs` documents DST encoding)
-   Thread palettes: `palettes/thread_*.rs` (brand-specific color mappings)
-   TODO list: `TODOS.md` (comprehensive feature roadmap)
-   Contributing guide: `CONTRIBUTING.md` (PR requirements, code standards)
-   Wiki documentation: `Butabuti.wiki/` (comprehensive user documentation)
