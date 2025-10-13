# Butabuti Todo List - Upcoming Features and Improvements

## File Format Support

- [ ] Add PES version 5 support
- [ ] Add PES version 7, 8, 9, 10 support
- [ ] Add EMB (Wilcom) format reader
- [ ] Add EMB (Wilcom) format writer
- [ ] Add CND (Poem/Huskygram/Singer EU) format reader
- [ ] Add CND format writer
- [ ] Add TAP (Happy) format writer
- [ ] Add STX (Data Stitch) format writer
- [ ] Add PHC (Brother) format writer
- [ ] Add PHB (Brother) format writer
- [ ] Add 10O (Toyota) format writer
- [ ] Add 100 (Toyota) format writer
- [ ] Add ART (Bernina) format writer
- [ ] Add DXF (AutoCAD) format reader
- [ ] Add DXF format writer
- [ ] Add PDF embroidery format export
- [ ] Add HUS format writer (with Huffman compression)
- [ ] Add SHV format writer
- [ ] Add SEW format writer
- [ ] Add OFM (Melco) format reader/writer
- [ ] Add CSD (Singer) format reader/writer
- [ ] Add XXX format improvements (better color handling)
- [ ] Add T01-T15 (Pfaff) format support
- [ ] Add ZSK format variants support

## Pattern Operations

- [ ] Implement pattern merging/combining
- [ ] Implement pattern splitting by color
- [ ] Implement pattern splitting by bounds
- [ ] Add pattern cropping functionality
- [ ] Add pattern rotation by arbitrary angles
- [ ] Add pattern mirroring (horizontal/vertical)
- [ ] Add pattern scaling with quality preservation
- [ ] Add stitch density analysis
- [ ] Add stitch count estimation
- [ ] Add thread length calculation
- [ ] Add estimated sewing time calculation
- [ ] Add pattern simplification (reduce stitch count)
- [ ] Add pattern smoothing algorithms
- [ ] Add auto-digitizing from vector graphics
- [ ] Add auto-digitizing from raster images
- [ ] Add pattern duplication/repetition
- [ ] Add pattern tiling functionality
- [ ] Add circular/radial pattern repetition
- [ ] Add pattern outline generation
- [ ] Add fill pattern generation (satin, zigzag, cross-stitch)
- [ ] Add running stitch conversion
- [ ] Add appliqué path generation

## Stitch Operations

- [ ] Implement stitch type conversion (normal to satin, etc.)
- [ ] Add underlay stitch generation
- [ ] Add pull compensation
- [ ] Add automatic jump trim insertion
- [ ] Add stitch angle optimization
- [ ] Add stitch length normalization
- [ ] Add tie-in/tie-off stitch generation
- [ ] Add bean stitch support
- [ ] Add moss stitch support
- [ ] Add triple stitch support
- [ ] Add manual stitch placement tools
- [ ] Add stitch reordering optimization
- [ ] Add stitch filtering (remove short jumps)
- [ ] Add sequin stitch support (for compatible formats)

## Color and Thread Management

- [ ] Add thread brand mapping (Madeira, Sulky, Robison-Anton, etc.)
- [ ] Add automatic color reduction
- [ ] Add color palette optimization
- [ ] Add closest thread color matching by brand
- [ ] Add thread cost estimation
- [ ] Add thread consumption calculator
- [ ] Add RGB to thread color database
- [ ] Add Pantone color matching
- [ ] Add thread substitution recommendations
- [ ] Add custom thread palette creation
- [ ] Add thread color sorting algorithms

## Encoding and Optimization

- [ ] Add encoder max stitch length per format
- [ ] Add encoder max jump length per format
- [ ] Add encoder stitch angle constraints
- [ ] Improve encoder quality settings
- [ ] Add pattern optimization (reduce file size)
- [ ] Add compression quality settings for formats
- [ ] Add encode/decode round-trip verification
- [ ] Add format conversion quality presets

## Graphics Export

- [ ] Add PNG export with custom DPI
- [ ] Add PNG export with dimension annotations
- [ ] Add SVG export with layers
- [ ] Add SVG export with thread color labels
- [ ] Add realistic 3D render export
- [ ] Add texture mapping for renders
- [ ] Add JPEG export
- [ ] Add BMP export
- [ ] Add TIFF export
- [ ] Add animated GIF export (stitch sequence)
- [ ] Add video export (MP4) showing stitching process
- [ ] Add WebP export
- [ ] Add thumbnail generation for all formats

## Analysis and Validation

- [ ] Add pattern validation (check for errors)
- [ ] Add stitch integrity checking
- [ ] Add problematic stitch detection (too long, too dense)
- [ ] Add hoop boundary validation
- [ ] Add design complexity scoring
- [ ] Add thread color change count optimization
- [ ] Add stitch statistics export (CSV, JSON)
- [ ] Add pattern comparison tools
- [ ] Add format compatibility checker

## API and Usability

- [ ] Add pattern builder API with fluent interface
- [ ] Add shape primitives (circle, rectangle, text)
- [ ] Add text-to-embroidery conversion
- [ ] Add font rendering to stitches
- [ ] Add command-line tool for conversions
- [x] Add batch conversion support
- [x] Add multi-format export functionality
- [ ] Add pattern preview generation
- [ ] Add progress callbacks for long operations
- [ ] Add pattern comparison/diff tools

## Performance

- [ ] Add parallel processing for large patterns
- [ ] Add streaming I/O for huge files
- [ ] Add memory-mapped file support
- [ ] Add lazy loading for pattern sections
- [ ] Optimize matrix transformations
- [ ] Add SIMD optimizations where applicable
- [ ] Add caching for expensive operations
- [ ] Benchmark and optimize hot paths

## Machine-Specific Features

- [ ] Add hoop size definitions for all major brands
- [ ] Add machine-specific metadata handling
- [ ] Add Brother-specific features (bobbin change, etc.)
- [ ] Add Janome-specific features
- [ ] Add Pfaff-specific features
- [ ] Add Husqvarna Viking-specific features
- [ ] Add Bernina-specific features
- [ ] Add Singer-specific features
- [ ] Add Tajima industrial features
- [ ] Add Barudan industrial features
- [ ] Add ZSK industrial features
- [ ] Add custom machine profile creation

## Testing and Quality

- [ ] Add property-based testing with proptest
- [ ] Add fuzzing for all readers
- [ ] Add format round-trip tests for all formats
- [ ] Add regression test suite
- [ ] Add performance benchmarks
- [ ] Add real-world pattern test suite
- [ ] Add code coverage reporting
- [ ] Add mutation testing

## Ecosystem Integration

- [ ] Add Python bindings (PyO3)
- [ ] Add Node.js bindings (Neon)
- [x] Add WebAssembly bindings
- [ ] Add C FFI bindings
- [ ] Add REST API server
- [ ] Add GraphQL API
- [ ] Add gRPC service

## WebAssembly/Browser Support

### Core Functionality

- [x] Basic WASM bindings for format conversion
- [x] Pattern info extraction in browser
- [x] SVG export for visualization
- [x] Format listing API (uses FormatRegistry)
- [x] Programmatic format population in HTML
- [ ] Batch conversion API in WASM
- [ ] Progress callbacks for long operations
- [ ] Web Worker support for background processing
- [ ] Streaming API for large files
- [ ] Memory optimization for constrained environments

### User Interface Improvements

- [ ] Add drag-and-drop file upload
- [ ] Add multi-file selection and batch processing
- [ ] Add file format auto-detection
- [ ] Add download progress indicator
- [ ] Add conversion queue management
- [ ] Add recent conversions history (localStorage)
- [ ] Add format presets (common conversions)
- [ ] Add keyboard shortcuts
- [ ] Add mobile-responsive design improvements
- [ ] Add pattern zoom and pan controls
- [ ] Add print-friendly SVG export option

### Advanced Features

- [ ] Add pattern editing capabilities in browser
  - [ ] Move/translate stitches
  - [ ] Rotate pattern
  - [ ] Scale pattern
  - [ ] Mirror/flip pattern
  - [ ] Change thread colors
  - [ ] Add/remove stitches
- [ ] Add pattern comparison view (side-by-side)
- [ ] Add hoop boundary overlay on preview
- [ ] Add stitch-by-stitch animation
- [ ] Add thread color picker with brand palettes
- [ ] Add pattern library/gallery with localStorage
- [ ] Add export to multiple formats at once (ZIP download)
- [ ] Add pattern merge/combine functionality
- [ ] Add pattern templates/blanks

### Performance & Optimization

- [ ] Implement WASM SIMD optimizations
- [ ] Add lazy loading for large patterns
- [ ] Optimize WASM binary size (tree-shaking)
- [ ] Add service worker for offline support
- [ ] Add WASM module caching
- [ ] Profile and optimize hot paths
- [ ] Add memory pooling for large conversions
- [ ] Implement progressive rendering for SVG preview

### Testing & Quality

- [ ] Add browser-based unit tests
- [ ] Add E2E tests for web interface
- [ ] Add cross-browser compatibility tests
- [ ] Add performance benchmarks in browser
- [ ] Add memory leak detection
- [ ] Add accessibility (a11y) improvements
- [ ] Add ARIA labels for screen readers
- [ ] Add keyboard navigation support

### Developer Experience

- [ ] Add TypeScript definitions for WASM API
- [ ] Add NPM package for WASM module
- [ ] Add CDN hosting for WASM binaries
- [ ] Add React component wrapper
- [ ] Add Vue component wrapper
- [ ] Add Svelte component wrapper
- [ ] Add usage examples for popular frameworks
- [ ] Add playground/sandbox environment
- [ ] Add API documentation generator

## Data Formats

- [ ] Add XML-based format support
- [ ] Add YAML pattern format
- [ ] Add TOML pattern format
- [ ] Add MessagePack format
- [ ] Add CBOR format
- [ ] Add Protocol Buffers format
- [ ] Add pattern database schema

## Utilities

- [ ] Add color space conversions (HSL, HSV, LAB)
- [ ] Add spline to stitch conversion
- [ ] Add polygon to stitch conversion
- [ ] Add path simplification algorithms
- [ ] Add noise reduction for digitized patterns
- [ ] Add pattern watermarking (small monogram will be embroidered in a corner)
- [ ] Add pattern metadata editor
- [ ] Add EXIF-like metadata support
- [ ] Add checksum/hash generation for patterns

## Developer Tools

- [ ] Add pattern diff viewer
- [ ] Add binary format inspector
- [ ] Add format converter wizard
- [ ] Add pattern debugger
- [ ] Add stitch visualizer with step-through
- [ ] Add performance profiler
- [ ] Add memory usage analyzer

## Code Automation & Refactoring

### File Naming & Organization

- [ ] **Rename `functions.rs` to `encoding.rs`** (optional, for clarity)
  - Currently: `src/utils/functions.rs` contains `encode_thread_change()`, `decode_*()` functions
  - Should: `src/utils/encoding.rs` better describes actual purpose
  - Benefit: More descriptive, matches content (encoding/decoding utilities)
  - Impact: Minimal - internal utility module, easy refactor
  - Note: Current name acceptable, but `encoding.rs` would be clearer

### Format Registry Integration (DRY Principle)

- [ ] **Refactor wasm.rs readers**: Replace manual match statements with FormatRegistry
  - Currently: Manual `match format.to_lowercase()` with 15+ hardcoded cases in `read_pattern()`
  - Should: Use `registry.read_pattern()` for unified API
  - Benefit: Automatically supports new formats without wasm.rs changes
  - Impact: Eliminates ~50 lines of boilerplate match code
  
- [ ] **Refactor wasm.rs writers**: Replace manual match statements with FormatRegistry
  - Currently: Manual `match format.to_lowercase()` with 14+ hardcoded cases in `write_pattern()`
  - Should: Use `registry.write_pattern()` for unified API
  - Benefit: Single source of truth for format capabilities
  - Impact: Eliminates ~60 lines of boilerplate match code
  
- [ ] **Add format parameter metadata to FormatRegistry**
  - Currently: Special parameters (DST max_jump=121, JEF hoop_size=127) hardcoded in wasm.rs
  - Should: Store default parameters in FormatInfo struct
  - Fields to add: `default_params: HashMap<String, serde_json::Value>`
  - Example: `{"max_jump": 121, "extended": false}` for DST
  - Benefit: Centralized format configuration, easier to expose in UI
  
- [ ] **Create CLI command registry using FormatRegistry**
  - Currently: CLI (src/bin/butabuti.rs) likely has manual format matching
  - Should: Query FormatRegistry for format validation and help text
  - Benefit: Auto-generated format list in `--help` output
  
- [ ] **Auto-generate format documentation**
  - Currently: Wiki format lists may be manually maintained
  - Should: Generate `Format-Support.md` from FormatRegistry at build time
  - Implementation: Add `build.rs` script or CLI command
  - Output: Table with Name | Extensions | Read | Write | Description
  
- [ ] **Auto-generate file extension to format mapping**
  - Currently: May have duplicate extension→format logic
  - Should: Single `get_format_from_extension(ext: &str)` in FormatRegistry
  - Already exists but verify all code uses it

### API Consistency (Unify Reader/Writer Patterns)

- [ ] **Migrate legacy readers to mutation pattern**
  - Legacy API (returns `Result<EmbPattern>`): DST, JEF, EXP, PEC, JSON
  - Modern API (mutates `&mut EmbPattern`): PES, VP3, XXX, U01, TBF, CSV, COL, EDR, INF, GCODE
  - Should: Standardize all readers to mutation pattern for consistency
  - Benefit: Enables pattern buffer reuse in batch operations (less allocation)
  - Breaking change: Update all reader signatures + tests
  
- [ ] **Standardize writer parameter order**
  - Inconsistency: Some writers take `(file, pattern)`, others `(pattern, file)`
  - Should: Standardize to `write(pattern: &EmbPattern, file: &mut impl Write)`
  - Benefit: Consistent API across all formats
  - Check: DST, JEF, CSV, XXX, TBF writers for parameter order
  
- [ ] **Abstract format-specific parameters into structs**
  - Currently: Some writers take many parameters (DST: extended, max_jump; JEF: extended, hoop_size, name)
  - Should: Create `DstWriteOptions`, `JefWriteOptions` structs
  - Benefit: Easier to add new parameters without breaking API
  - Example:

    ```rust
    pub struct DstWriteOptions {
        pub extended: bool,
        pub max_jump: i32,
    }
    impl Default for DstWriteOptions { ... }
    pub fn write(pattern: &EmbPattern, file: &mut impl Write, options: &DstWriteOptions)
    ```

### Palette Management Automation

- [ ] **Create thread palette registry**
  - Currently: Thread palettes (JEF, PEC, SEW, HUS, SHV) are separate modules
  - Should: Create `PaletteRegistry` similar to `FormatRegistry`
  - Methods: `get_palette(brand: &str)`, `find_closest_thread(rgb: (u8,u8,u8), brand: &str)`
  - Benefit: Programmatic palette discovery for UI dropdowns
  
- [ ] **Auto-generate palette documentation**
  - Currently: Thread color tables may be manually documented
  - Should: Generate markdown tables from palette data
  - Output: `palettes/README.md` with all thread colors per brand
  
- [ ] **Create unified thread matching API**
  - Currently: Color matching logic may be duplicated
  - Should: `ThreadMatcher` utility with configurable algorithms
  - Methods: `find_closest(rgb)`, `find_closest_in_brand(rgb, brand)`, `batch_match(colors)`

### Error Handling Standardization

- [ ] **Create format-specific error types**
  - Currently: Generic `Error::Parse(String)` for all format errors
  - Should: `Error::DstParse(DstError)`, `Error::PesParse(PesError)` with enums
  - Benefit: Better error messages, easier debugging, machine-readable error codes
  
- [ ] **Add error recovery hints**
  - Currently: Errors just report failure
  - Should: Include recovery suggestions in error messages
  - Example: `Error::Parse("Invalid header size: expected 512, got 256. File may be truncated or corrupted. Try re-exporting from source software.")`

### Metadata Management

- [ ] **Create metadata schema/registry**
  - Currently: Metadata keys are free-form strings
  - Should: Define metadata schema with known keys
  - Fields: `title`, `author`, `copyright`, `date`, `machine`, `hoop_size`, etc.
  - Benefit: Type-safe metadata access, auto-completion in IDEs
  
- [ ] **Add metadata propagation in format conversions**
  - Currently: Metadata may be lost during conversions
  - Should: Automatically copy compatible metadata between formats
  - Implementation: Metadata mapping table in FormatRegistry

### Testing Automation

- [ ] **Generate round-trip tests from FormatRegistry**
  - Currently: Manual round-trip tests per format
  - Should: Macro or build script to generate tests from registry
  - Pattern: For each format with `can_read && can_write`, test read→write→read→compare
  
- [ ] **Create test fixture registry**
  - Currently: Test files scattered in workspace root
  - Should: Organized `tests/fixtures/{format}/` structure
  - Manifest: `fixtures.toml` with test cases and expected results
  
- [ ] **Auto-generate format compatibility matrix**
  - Currently: Unknown which formats preserve which features
  - Should: Test all format pairs (A→B) and track data loss
  - Output: Compatibility matrix showing stitch count, color, metadata preservation

## Realistic Stitch Rendering

### SVG Export Enhancements

- [ ] **Integrate stitch.svg icon for realistic stitches**
  - Currently: SVG writer uses simple stroke paths
  - Should: Replace paths with repeated stitch.svg symbols
  - Implementation:
    1. Embed stitch.svg as SVG `<symbol id="stitch">` definition in header
    2. For each stitch point, add `<use xlink:href="#stitch" x="..." y="..." />`
    3. Replace color #808080ff in gradient with thread color dynamically
    4. Rotate stitch icon to match stitch angle
  - Benefit: Realistic embroidery preview in SVG exports
  
- [ ] **Calculate stitch angles for rotation**
  - Currently: Stitches rendered as simple lines
  - Should: Calculate angle between consecutive stitch points
  - Formula: `angle = atan2(dy, dx)` converted to degrees
  - Apply via SVG transform: `<use transform="rotate({angle} {x} {y})" .../>`
  
- [ ] **Add stitch density visualization**
  - Currently: All stitches rendered at same size
  - Should: Scale stitch icon based on local stitch density
  - Dense areas → smaller stitches, sparse areas → larger stitches
  - Benefit: Shows fabric texture variation
  
- [ ] **Add SVG export quality options**
  - Low quality: Simple paths (current implementation)
  - Medium quality: Colored paths with rounded caps
  - High quality: Realistic stitch icons with gradients
  - Ultra quality: 3D-effect stitches with shadows
  
- [ ] **Optimize SVG symbol reuse**
  - Currently: N/A (feature not implemented)
  - Should: Define stitch symbol once, reuse with `<use>`
  - Benefit: Smaller file size for large patterns (1000s of stitches)

### PNG Export Enhancements

- [ ] **Add realistic stitch rendering to PNG writer**
  - Currently: PNG writer (if graphics feature enabled) may use simple rendering
  - Should: Render stitch.svg icon at each stitch point
  - Implementation: Use `resvg` crate to rasterize SVG stitch icon per point
  - Alternative: Pre-render stitch icon at multiple angles (0-359°), cache as sprites
  
- [ ] **Add configurable DPI for PNG export**
  - Currently: Fixed resolution
  - Should: Accept DPI parameter (72, 150, 300, 600)
  - Benefit: Print-quality exports at 300+ DPI
  
- [ ] **Add anti-aliasing options**
  - Should: Quality presets (draft, normal, high, ultra)
  - Ultra mode: 4x supersampling for smooth edges

### Future Image/Video Formats

- [ ] **JPEG export with realistic stitches**
  - Strategy: Render to PNG with realistic stitches, then convert to JPEG
  - Add quality slider (1-100)
  
- [ ] **GIF export with realistic stitches**
  - Static GIF: Same as PNG but with palette quantization
  - Animated GIF: Show stitch sequence frame-by-frame
  - Frame rate option (1-30 fps)
  
- [ ] **MP4 video export of stitching process**
  - Render each stitch progressively
  - Add thread color changes as video segments
  - Show needle movement animation
  - Export options: Resolution (720p, 1080p, 4K), FPS (24, 30, 60)
  - Codec: H.264 for compatibility
  
- [ ] **WebP export with realistic stitches**
  - Supports both lossy and lossless compression
  - Better than PNG for web use (smaller files)
  
- [ ] **3D render export with texture mapping**
  - Render stitches with height/depth for 3D effect
  - Add fabric texture background
  - Export as PNG with normal maps or 3D formats (OBJ, glTF)

### Stitch Icon Customization

- [ ] **Create stitch icon variations**
  - Currently: Single stitch.svg icon
  - Should: Multiple stitch types (satin, running, cross, bean, moss)
  - Each type has unique visual appearance
  
- [ ] **Add stitch thickness scaling**
  - Currently: Fixed icon size
  - Should: Scale based on thread weight (30wt, 40wt, 60wt)
  - Heavier threads → thicker stitch icons
  
- [ ] **Add fabric texture backgrounds**
  - Option to overlay stitches on fabric textures
  - Textures: Cotton, linen, denim, felt, leather
  - Blend mode: Multiply or overlay for realistic appearance

### Performance Optimization

- [ ] **Cache rendered stitch sprites**
  - Pre-render stitch icon at all 360 rotation angles
  - Store as sprite sheet or texture atlas
  - Benefit: Faster rendering for large patterns
  
- [ ] **Add progressive rendering for large patterns**
  - Render in chunks (1000 stitches at a time)
  - Show progress indicator
  - Stream output for web display
  
- [ ] **Optimize SVG size with symbol reuse**
  - Define gradients and symbols once in `<defs>`
  - Reference via `<use>` throughout document
  - Benefit: 50-90% smaller file size vs. inline gradients
