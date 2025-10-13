# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added

- Realistic stitch rendering utilities in `src/utils/stitch_renderer.rs`
- Quality levels for SVG export (Low, Medium, High, Ultra)
- `rustfmt.toml` for consistent code formatting
- `clippy.toml` for linting configuration
- Comprehensive file organization documentation

### Changed

- Enhanced SVG writer with `write_with_quality()` function
- Updated contribution guidelines with file naming conventions

## [0.1.0] - 2024-01-XX (Initial Release)

### Added Features

#### Core Features

- `EmbPattern` - Core pattern abstraction for stitch sequences
- `EmbThread` - Thread color and metadata management
- `ColorGroup` - Thread organization by categories
- `EmbCollection` - Pattern collection management
- Command constants (STITCH, JUMP, TRIM, COLOR_CHANGE, END)
- Coordinate system based on 0.1mm units (industry standard)
- Transformation matrices for pattern manipulation

#### Format Support (15 Bidirectional Formats)

**Readers & Writers:**

- DST (Tajima)
- PES (Brother)
- JEF (Janome)
- VP3 (Pfaff/Husqvarna)
- EXP (Melco)
- SEW (Janome)
- XXX (Singer)
- HUS (Husqvarna Viking)
- VIP (Pfaff)
- BRO (Bits & Volts)
- U01 (Barudan)
- DAT (Barudan)
- 100 (Toyota)
- 10O (Toyota)
- DSB (Barudan)

**Additional Writers:**

- SVG (Scalable Vector Graphics)
- PNG (Portable Network Graphics - requires `graphics` feature)
- TXT (Human-readable text format)

#### Format Infrastructure

- `FormatRegistry` - Dynamic format discovery and management
- Format detection from file content
- `ReadHelper`/`WriteHelper` traits for binary I/O
- Thread palettes for JEF, PEC, SEW, SHV, HUS formats

#### Pattern Processing

- `Transcoder` - Complex pattern transformations
- Pattern normalization and centering
- Stitch interpolation and trim handling
- Color count fixing
- Long stitch splitting
- Bounds calculation

#### Batch Processing

- `BatchConverter` - Multi-file conversion
- `MultiFormatExporter` - Export to multiple formats
- Parallel processing support (requires `parallel` feature)
- Overwrite protection and progress reporting

#### CLI Tool

- `butabuti` binary for command-line operations
- Commands: convert, info, validate, batch, list-formats
- Format validation and pattern inspection

#### WASM Support

- WebAssembly bindings for browser use
- JavaScript interop via wasm-bindgen
- Browser-based pattern conversion and rendering

#### Utilities

- Huffman compression (for HUS format)
- String encoding/decoding utilities
- Palette management
- Error handling with descriptive error types

### Documentation

- Comprehensive README with quick start guide
- API documentation with examples
- Format support matrix
- Contributing guidelines
- Code of conduct
- License (MIT OR Apache-2.0)
- TODO roadmap with 250+ items
- Wiki documentation:
  - Home
  - Quick Start
  - API Reference
  - Format Support
  - Features
  - Examples
  - FAQ
  - Terminology
  - Coordinate Systems
  - Color Groups
  - Installation
  - Gallery

### Testing

- 522 unit tests with 100% pass rate
- Property-based testing with proptest
- Fuzz testing infrastructure
- Format round-trip tests
- Real-world file samples

### Performance

- Benchmarks for format I/O operations
- Benchmarks for pattern operations
- Benchmarks for thread operations
- Optimized binary I/O with helper traits

### Developer Experience

- `.editorconfig` for cross-editor consistency
- `validate.ps1` script for pre-commit checks
- Example programs demonstrating API usage
- Inline documentation with code examples

### Features Flags

- `graphics` - PNG export support
- `parallel` - Parallel batch processing
- `wasm` - WebAssembly bindings
- `full` - All optional features

## Release Notes

### Version 0.1.0 - Initial Release

This is the first public release of Butabuti, a high-performance Rust library for embroidery file manipulation. The library provides comprehensive support for reading, writing, and processing embroidery files with 15 bidirectional format implementations.

**Key Highlights:**

- **Full Format Support**: Only formats with BOTH readers AND writers are included
- **Memory Safe**: Written in Rust with comprehensive error handling
- **Performance**: Optimized I/O with optional parallel processing
- **Cross-Platform**: Works on Windows, macOS, Linux, and in browsers via WASM
- **Well Tested**: 522 tests covering core functionality and edge cases
- **Production Ready**: Used in real-world embroidery software

**Breaking Changes:** N/A (initial release)

**Migration Guide:** N/A (initial release)

**Contributors:**

- Core development: [Your Name/Team]
- Format specifications: Various embroidery machine manufacturers
- Community testing: [List contributors]

**Known Limitations:**

- WASM build requires external HTTP server (browser security)
- PNG export requires `graphics` feature flag
- Some format metadata fields not fully supported (see format docs)
- Thread palette coverage varies by brand

**Next Steps:**

- See [TODOS.md](TODOS.md) for planned features
- See [ROADMAP.md](ROADMAP.md) for release schedule
- Join discussions on GitHub for feature requests

## Versioning Policy

- **Major (X.0.0)**: Breaking API changes, major architectural changes
- **Minor (0.X.0)**: New features, new format support, non-breaking changes
- **Patch (0.0.X)**: Bug fixes, performance improvements, documentation updates

## How to Upgrade

### From Pre-release to 0.1.0

This is the first stable release. No migration needed.

### Future Upgrades

Check the changelog entry for your target version:

- Review "Breaking Changes" section
- Follow "Migration Guide" if provided
- Update `Cargo.toml` dependency version
- Run `cargo update` and `cargo test`

## Links

- [Repository](https://github.com/Fahad090NP/butabuti)
- [Documentation](https://docs.rs/butabuti)
- [Crate](https://crates.io/crates/butabuti)
- [Issues](https://github.com/Fahad090NP/butabuti/issues)
- [Discussions](https://github.com/Fahad090NP/butabuti/discussions)

---

**Legend:**

- `Added` - New features
- `Changed` - Changes in existing functionality
- `Deprecated` - Soon-to-be removed features
- `Removed` - Removed features
- `Fixed` - Bug fixes
- `Security` - Security vulnerability fixes
