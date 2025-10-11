# ButaButi Code Improvements - Issues and Bugs

## Critical Bugs 🔴

- [x] **CSV Reader Thread Chart Duplication** (`src/formats/io/readers/csv.rs:116-122`)
  - Lines 116 and 120 both set `thread.chart` from parts[6] and parts[7]
  - This causes parts[6] to be overwritten by parts[7]
  - Should be: Line 116 sets a different field (likely `details` or similar)
  - Impact: Thread metadata corruption when reading CSV files

## Error Handling Issues ⚠️

- [x] **Unsafe `unwrap()` in Compress** (`src/utils/compress.rs:294,301`)
  - Huffman decompression uses `.unwrap()` without error handling
  - Fixed: Now uses `.expect()` with descriptive messages
  - Better panic messages if huffman tables aren't initialized

- [x] **Unsafe `unwrap()` in Batch Processing** (`src/utils/batch.rs:360,367,370`)
  - Multiple `.unwrap()` calls in thread synchronization code
  - Fixed: `results_clone.lock()` now uses `if let Ok()`
  - Fixed: `handle.join()` errors are now ignored with `let _ =`
  - Fixed: `Arc::try_unwrap()` now uses `.ok()` chain with `.unwrap_or_default()`
  - No longer panics on mutex poisoning or thread panic

- [ ] **Unsafe `unwrap()` in Palette Loading** (`src/palettes/thread_hus.rs`)
  - 29+ instances of `.unwrap()` when parsing hex colors (lines 12-152)
  - Should use `.unwrap_or()` with default color or return Result
  - Risk: Panic if palette data is corrupted

- [x] **File Path Unwrap** (`src/utils/batch.rs:494`)
  - `file_stem().unwrap()` could panic on paths without file names
  - Fixed: Now uses `.and_then()` chain with fallback to "output"

## Performance Issues 🐌

- [ ] **Excessive Clone Operations**
  - `src/utils/batch.rs:349-350` - Clones `target_format` and `output_dir` for each thread
  - `src/core/pattern.rs:395` - Clones entire stitch block for grouping
  - `src/core/encoder.rs:102` - Clones every thread during encoding
  - `src/formats/io/writers/pes.rs:478` - Clones stitch blocks during PES write
  - `src/formats/io/writers/json.rs:63,71-74` - Multiple string clones for JSON serialization
  - Solution: Use references with proper lifetimes or `Arc<T>` for shared data

- [ ] **HashMap Cloning in JSON Writer** (`src/formats/io/writers/json.rs:63`)
  - Clones both key and value when inserting into metadata HashMap
  - Could use `to_string()` or references instead

- [ ] **Thread Palette Cloning** (Multiple readers)
  - `pec.rs`, `jef.rs`, `hus.rs`, `sew.rs`, `shv.rs`, `phb.rs`, `phc.rs`
  - Repeatedly clones thread objects from static palettes
  - Solution: Consider using `Copy` trait for EmbThread or reference counting

## Code Quality Issues 📝

- [ ] **Inconsistent Error Messages**
  - Some readers use generic "Parse error" messages
  - Should include specific context (file format, offset, expected vs actual)

- [ ] **Magic Numbers**
  - Many format readers/writers use hardcoded values without constants
  - Examples: Header sizes, version numbers, color palette indices
  - Should define as named constants for maintainability

- [ ] **Missing Input Validation**
  - Many readers don't validate header magic bytes before parsing
  - File size checks are inconsistent
  - Could lead to confusing error messages on invalid files

- [ ] **Inconsistent Naming Conventions**
  - Some functions use `read_file()` while others use `read()`
  - Thread properties: `description`, `catalog_number`, `brand`, `chart` - unclear which is which
  - Should standardize naming across all format modules

## Documentation Issues 📚

- [ ] **Missing Safety Documentation**
  - Functions using `unwrap()` should document panic conditions
  - Thread-safety guarantees not documented in batch processor

- [ ] **Incomplete Examples in Doc Comments**
  - Some format writers have example code in comments that won't compile
  - Example: `src/formats/io/writers/tbf.rs:48-49` references old module path

- [ ] **Undocumented Format Limitations**
  - Some formats have stitch count limits, coordinate range limits
  - Not documented in module-level docs or function comments

## Testing Gaps 🧪

- [ ] **Missing Round-trip Tests**
  - Only a few formats have read→write→read tests
  - Should verify data integrity for all supported formats

- [ ] **No Fuzzing Tests**
  - Format readers should be fuzz-tested for crash resistance
  - Especially important for binary formats with complex encoding

- [ ] **Missing Error Case Tests**
  - Most tests only cover happy paths
  - Should test: truncated files, invalid headers, corrupted data

- [ ] **No Performance Benchmarks**
  - No benchmarks for large file processing
  - Batch conversion performance not measured

## API Design Issues 🎨

- [ ] **Inconsistent Reader Signatures**
  - Most: `read(file: &mut impl Read, pattern: &mut EmbPattern) -> Result<()>`
  - VP3/PES: `read<R: Read + Seek>(reader: &mut R) -> Result<EmbPattern>`
  - Should standardize to one approach

- [ ] **Mutable Pattern Parameter**
  - Readers mutate an existing pattern - could cause confusion
  - Consider builder pattern or return new pattern

- [ ] **Thread vs Palette Confusion**
  - `EmbThread` used for both pattern threads and palette references
  - Cloning from palettes is inefficient
  - Consider separate `PaletteColor` type

## Security Concerns 🔒

- [ ] **Integer Overflow Risks**
  - Coordinate calculations could overflow with malicious files
  - Stitch count multiplication without bounds checking
  - Should use checked arithmetic in critical paths

- [ ] **Unbounded Memory Allocation**
  - Readers allocate buffers based on file headers without limits
  - Could cause OOM with crafted files
  - Should enforce maximum pattern size limits

- [ ] **Path Traversal in Batch Converter**
  - Output paths constructed from input filenames
  - Could be exploited if processing untrusted files
  - Should sanitize file names before use

## Logical Issues 🤔

- [ ] **CSV Thread Property Overwrite Bug** ✅ CRITICAL
  - Lines 116-122 in csv.rs assign parts[7] to `chart` twice
  - First assignment from parts[6] is overwritten
  - Likely should map to different fields

- [ ] **Color Index Validation Missing**
  - Some readers access palette arrays by index without bounds check
  - Could panic or produce incorrect colors
  - Should validate against palette size

- [ ] **Coordinate System Inconsistencies**
  - Some formats flip Y-axis, others don't
  - Documentation says "0.1mm units" but some readers use different scales
  - Should audit all format converters for coordinate correctness

## Maintenance Issues 🔧

- [ ] **Duplicate Code in Format Writers**
  - Many writers have similar stitch encoding loops
  - Could extract common patterns into utilities

- [ ] **Inconsistent Error Types**
  - Mix of `Error::Parse`, `Error::Io`, `Error::Encoding`
  - Not always clear which to use when
  - Should document error type guidelines

- [ ] **Hard to Extend**
  - Adding new formats requires modifying multiple files
  - No trait-based plugin system
  - Consider format registry pattern

## Feature Completeness 🎯

- [ ] **Partial Format Support**
  - Some formats marked as "read/write" but missing advanced features
  - Metadata not preserved in many conversions
  - Should document feature matrix per format

- [ ] **Missing Validation Functions**
  - No public API to validate patterns before writing
  - Writers fail late instead of early
  - Should add `validate_for_format()` functions

## Priority Matrix

### High Priority (Fix First)

1. ✅ CSV thread chart duplication bug (data corruption)
2. Integer overflow/bounds checking in readers (security)
3. Unwrap() in error paths (stability)
4. Inconsistent reader API (developer experience)

### Medium Priority

1. Performance: excessive cloning
2. Missing input validation
3. Round-trip testing
4. Documentation improvements

### Low Priority

1. Code deduplication
2. API design consistency
3. Magic number constants
4. Extended format features

## Completion Status

Total Issues: 32

- [x] Completed: 5
- [ ] Pending: 27

Last Updated: 2025-10-11
