# ButaButi Improvement Roadmap

**Analysis Date**: October 11, 2025  
**Source**: Feature gap analysis comparing ButaButi with embroidery-rust library  
**Status**: New features identified for implementation

---

## Executive Summary

After comprehensive analysis of the embroidery-rust library, we've identified **45 high-value features** that can significantly enhance ButaButi's capabilities. These improvements span architecture, transformations, error handling, format support, and developer experience.

### Priority Breakdown

- 🔴 **Critical (P0)**: 8 features - Core functionality gaps
- 🟠 **High (P1)**: 15 features - Significant value additions
- 🟡 **Medium (P2)**: 12 features - Quality of life improvements
- 🟢 **Low (P3)**: 10 features - Nice-to-have enhancements

---

## 🔴 CRITICAL PRIORITY (P0)

### 1. Pattern Transformations 🎯

**Status**: ❌ Missing  
**Impact**: Very High - Essential for pattern manipulation  
**Complexity**: Medium

**Current State**:

- ✅ `translate()` - Basic translation exists
- ✅ `move_center_to_origin()` - Centering exists
- ❌ No `rotate()` method
- ❌ No `scale()` method
- ❌ No `flip_horizontal()` / `flip_vertical()` methods
- ❌ No advanced matrix transformations

**Missing Features**:

```rust
// Rotation
pub fn rotate(&mut self, angle_degrees: f64) -> Result<()>
pub fn rotate_around_point(&mut self, angle: f64, cx: f64, cy: f64) -> Result<()>

// Scaling
pub fn scale(&mut self, sx: f64, sy: f64) -> Result<()>
pub fn scale_uniform(&mut self, factor: f64) -> Result<()>

// Flipping
pub fn flip_horizontal(&mut self)
pub fn flip_vertical(&mut self)

// Advanced transforms
pub fn apply_matrix(&mut self, matrix: &EmbMatrix) -> Result<()>
```

**Implementation Notes**:

- Embroidery-rust doesn't have these either, but they're industry-standard
- Use EmbMatrix for transformations
- Preserve command flags during transforms
- Update bounds automatically

**Tests Required**:

- Rotation at 0°, 45°, 90°, 180°, 270°, 360°
- Scaling with positive/negative factors
- Flip roundtrips (flip twice = original)
- Combined transformations
- Edge cases: empty patterns, single stitch

---

### 2. Stitch Splitting / Long Stitch Handling 🎯

**Status**: ❌ Missing (Critical for format compliance)  
**Impact**: Very High - Required for format constraints  
**Complexity**: High

**Current State**:

- Encoder has `max_stitch` settings but no automatic splitting
- Many formats limit stitch length (DST: ±121, most others: ±127)
- Long stitches can cause malformed output

**embroidery-rust Implementation**:

```rust
pub trait SplitLongStitches {
    fn split_stitches(self, min_x: f64, max_x: f64, min_y: f64, max_y: f64) -> Self;
}

// Applied to Pattern, ColorGroup, StitchGroup
// Automatically splits stitches exceeding bounds
// Calculates optimal number of segments
// Preserves stitch attributes (trim, cut flags)
```

**Required Implementation**:

```rust
// In EmbPattern
pub fn split_long_stitches(&mut self, max_length: f64) -> Result<()>
pub fn split_to_format_limits(&mut self, format: &str) -> Result<()>

// Auto-splitting based on format:
// - DST: max_length = 121.0 (12.1mm)
// - PES/PEC: max_length = 127.0 (12.7mm)
// - JEF: max_length = 127.0
// - EXP: max_length = 127.0
```

**Algorithm** (from embroidery-rust):

1. Calculate delta (dx, dy) between consecutive stitches
2. If delta exceeds bounds, compute number of segments needed
3. Insert intermediate stitches with proportional spacing
4. Preserve stitch group attributes (trim, cut)
5. Handle edge cases (zero-length, very long stitches)

**Tests Required**:

- Split stitches at exactly max_length
- Split very long stitches (10x max_length)
- Asymmetric bounds
- Negative coordinates
- Property tests (proptest) for all valid input ranges

---

### 3. Remove Duplicate Stitches 🎯

**Status**: ❌ Missing  
**Impact**: High - File size optimization, machine efficiency  
**Complexity**: Low

**embroidery-rust Implementation**:

```rust
pub trait RemoveDuplicateStitches {
    fn remove_duplicate_stitches(self) -> Self;
}

impl RemoveDuplicateStitches for StitchGroup {
    fn remove_duplicate_stitches(self) -> Self {
        let mut stitches = Vec::with_capacity(self.stitches.len());
        if !self.stitches.is_empty() {
            let mut curr_stitch = self.stitches[0];
            stitches.push(curr_stitch);
            for stitch in self.stitches.into_iter().skip(1) {
                if stitch != curr_stitch {
                    stitches.push(stitch);
                    curr_stitch = stitch;
                }
            }
        }
        StitchGroup { stitches, ..self }
    }
}
```

**Our Implementation** (exists in processing.rs):

```rust
// ✅ We already have this!
pub fn remove_duplicates(pattern: &mut EmbPattern)
```

**Enhancement Needed**:

- Add as method to EmbPattern: `pattern.remove_duplicates()`
- Currently only in utils::processing module
- Make more discoverable

---

### 4. Pattern Collection / Multi-Pattern Files 🎯

**Status**: ❌ Missing  
**Impact**: High - Some formats contain multiple patterns  
**Complexity**: Medium

**embroidery-rust Design**:

```rust
#[derive(Clone, Debug, Default, PartialEq)]
pub struct PatternCollection {
    pub patterns: BTreeMap<String, Pattern>,
}

pub trait CollectionReader {
    fn read_collection(&self, item: &mut dyn Read) -> ReadResult<PatternCollection>;
}

pub trait CollectionWriter {
    fn write_collection(&self, collection: &PatternCollection, writer: &mut dyn Write) 
        -> WriteResult<()>;
}
```

**Formats Supporting Collections**:

- **HUS** - Can contain multiple designs
- **VP3** - Multi-part designs
- Some archive formats (.zip with multiple .dst files)

**Required Implementation**:

```rust
// In src/core/collection.rs
pub struct EmbPatternCollection {
    patterns: HashMap<String, EmbPattern>,
}

impl EmbPatternCollection {
    pub fn new() -> Self
    pub fn add(&mut self, name: String, pattern: EmbPattern)
    pub fn get(&self, name: &str) -> Option<&EmbPattern>
    pub fn get_mut(&mut self, name: &str) -> Option<&mut EmbPattern>
    pub fn iter(&self) -> impl Iterator<Item = (&String, &EmbPattern)>
    pub fn len(&self) -> usize
    pub fn is_empty(&self) -> bool
}

// Update readers/writers for multi-pattern formats
pub trait CollectionFormat {
    fn read_collection(file: &mut impl Read) -> Result<EmbPatternCollection>;
    fn write_collection(collection: &EmbPatternCollection, file: &mut impl Write) 
        -> Result<()>;
}
```

---

### 5. Format Auto-Detection 🎯

**Status**: ❌ Missing (Critical for user experience)  
**Impact**: Very High - Eliminates format guessing  
**Complexity**: Medium

**embroidery-rust Design**:

```rust
pub trait PatternReader {
    /// Returns true when the file is able to be loaded by this PatternReader.
    /// Ideally this should inspect the file's magic number, or some metadata
    fn is_loadable(&self, item: &mut dyn Read) -> ReadResult<bool>;
    
    fn read_pattern(&self, item: &mut dyn Read) -> ReadResult<Pattern>;
}
```

**Required Implementation**:

```rust
// In src/formats/io/mod.rs
pub struct FormatDetector;

impl FormatDetector {
    /// Detect format from magic bytes
    pub fn detect_from_content(file: &mut impl Read) -> Result<Format>
    
    /// Detect from file extension (fallback)
    pub fn detect_from_extension(path: &Path) -> Result<Format>
    
    /// Try all readers until one succeeds
    pub fn detect_and_read(file: &mut (impl Read + Seek)) -> Result<EmbPattern>
}

// Magic bytes for formats:
// DST: No magic, but starts with specific header structure
// PES: "#PES" or "#PEC"
// JEF: 0x00 0x00 0x00 0x64 (first 4 bytes)
// HUS/VIP: Specific header signature
// VP3: "%vsm%" or "%vp3%"
```

**Benefits**:

- No need to specify format explicitly
- Better error messages ("not a DST file" vs "failed to parse")
- Batch conversion can auto-detect input formats
- Command-line tools become more user-friendly

---

### 6. Contextual Error Messages 🎯

**Status**: ⚠️ Partial (basic errors exist)  
**Impact**: High - Essential for debugging  
**Complexity**: Medium

**embroidery-rust Design**:

```rust
#[derive(Fail, Debug)]
#[non_exhaustive]
pub enum ReadError {
    #[fail(display = "Invalid format: {}\nError Context(deepest-first):", _0)]
    InvalidFormat(String, Vec<String>),  // Vec<String> = context stack
    
    #[fail(display = "Unexpected End of File: {}\nError Context(deepest-first):", _0)]
    UnexpectedEof(String, #[cause] std::io::Error, Vec<String>),
    
    #[fail(display = "{}\nError Context(deepest-first):", _0)]
    Std(#[cause] StdError, Vec<String>),
}

pub trait ErrorWithContext {
    fn context(&self) -> Vec<String>;
    fn with_additional_context<S: Into<String>>(self, extra: S) -> Self;
    fn without_context(self) -> Self;
}
```

**Macros for Context**:

```rust
// Automatically adds file:line context
maybe_read_with_context!(
    reader.read_exact(&mut buffer),
    "Trying to read header at offset {}", 
    offset
);
```

**Our Current State**:

- Basic error types exist
- No context stack
- No automatic file:line information
- Error messages lack detail

**Required Enhancement**:

```rust
// Add to src/utils/error.rs
impl Error {
    pub fn with_context<S: Into<String>>(self, ctx: S) -> Self
    pub fn context_stack(&self) -> &[String]
}

// Macro for automatic context
#[macro_export]
macro_rules! read_with_context {
    ($expr:expr, $($arg:tt)*) => {
        $expr.map_err(|e| e.with_context(
            format!("{} at {}:{}", format!($($arg)*), file!(), line!())
        ))
    };
}
```

---

### 7. Compression Support (HUS/VIP/HUS6) 🎯

**Status**: ⚠️ Partial (HUS reader exists, but may lack full decompression)  
**Impact**: High - Essential for HUS/VIP format support  
**Complexity**: High

**embroidery-rust Dependencies**:

```toml
archivelib = "^0.1"  # Huffman compression for HUS
```

**Current Status**:

- We have `utils/compress.rs` with Huffman compression
- Used in HUS writer
- May need verification for HUS reader

**Required Verification**:

1. Check if HUS reader properly decompresses all sections:
   - Attribute data (stitch types)
   - X coordinates
   - Y coordinates
2. Verify compression level (Level 4 for HUS)
3. Test with real HUS files containing compressed data

**Enhancement**:

```rust
// In src/utils/compress.rs
pub fn decompress_hus(data: &[u8]) -> Result<Vec<u8>>
pub fn compress_hus(data: &[u8]) -> Result<Vec<u8>>

// Level 4 Huffman as per HUS spec
```

---

### 8. Thread Metadata Enhancement 🎯

**Status**: ⚠️ Partial (basic thread color exists)  
**Impact**: Medium-High - Professional embroidery requires full thread info  
**Complexity**: Low

**embroidery-rust Thread Structure**:

```rust
pub struct Thread {
    pub color: Color,
    pub name: String,
    pub code: String,
    pub manufacturer: Option<String>,
    pub attributes: BTreeMap<String, String>,  // Extensible metadata
}
```

**Our Current Thread** (src/core/thread.rs):

```rust
pub struct EmbThread {
    pub color: u32,  // RGB color
    pub description: String,
    pub catalog_number: String,
    pub brand: String,
}
```

**Enhancement Needed**:

```rust
// Add to EmbThread
pub struct EmbThread {
    pub color: u32,
    pub description: String,
    pub catalog_number: String,
    pub brand: String,
    pub manufacturer: Option<String>,  // NEW
    pub attributes: HashMap<String, String>,  // NEW - Extensible metadata
    pub weight: Option<String>,  // NEW - Thread weight (e.g., "40wt")
    pub type_: Option<String>,  // NEW - Thread type (e.g., "Rayon", "Polyester")
}
```

**Benefits**:

- Store manufacturer-specific thread info
- Support advanced format metadata
- Enable thread library features
- Better color matching

---

## 🟠 HIGH PRIORITY (P1)

### 9. Stitch Distance Calculation 🎯

**Status**: ❌ Missing  
**Impact**: High - Useful for statistics, validation  
**Complexity**: Low

**embroidery-rust Implementation**:

```rust
impl Stitch {
    pub fn distance_to(&self, other: &Self) -> f64 {
        let (dx, dy) = self.relative_to(other);
        ((dx * dx) + (dy * dy)).sqrt()
    }
    
    pub fn relative_to(&self, other: &Self) -> (f64, f64) {
        (self.x - other.x, self.y - other.y)
    }
}
```

**Required Implementation**:

```rust
// Add to src/core/pattern.rs Stitch impl
impl Stitch {
    pub fn distance_to(&self, other: &Stitch) -> f64
    pub fn relative_to(&self, other: &Stitch) -> (f64, f64)
}

// Pattern-level methods
impl EmbPattern {
    pub fn total_stitch_length(&self) -> f64  // Sum of all stitch distances
    pub fn max_stitch_length(&self) -> f64    // Longest single stitch
    pub fn avg_stitch_length(&self) -> f64    // Average stitch length
}
```

---

### 10. Stitch Validation / is_valid() 🎯

**Status**: ⚠️ Partial (some validation exists)  
**Impact**: High - Prevent corrupted patterns  
**Complexity**: Low

**embroidery-rust Implementation**:

```rust
impl Stitch {
    pub fn is_valid(&self) -> bool {
        self.x.is_finite() && self.y.is_finite()
    }
}
```

**Enhancement Needed**:

```rust
// Add to Stitch
impl Stitch {
    pub fn is_valid(&self) -> bool {
        self.x.is_finite() && 
        self.y.is_finite() && 
        !self.x.is_nan() && 
        !self.y.is_nan()
    }
}

// Pattern-level validation
impl EmbPattern {
    pub fn validate_all_stitches(&self) -> Result<()> {
        for (i, stitch) in self.stitches.iter().enumerate() {
            if !stitch.is_valid() {
                return Err(Error::InvalidPattern(
                    format!("Invalid stitch at index {}: ({}, {})", 
                            i, stitch.x, stitch.y)
                ));
            }
        }
        Ok(())
    }
}
```

---

### 11. Color Group / Stitch Group Architecture 🎯

**Status**: ❌ Missing (fundamental architecture difference)  
**Impact**: High - Better organization, format compliance  
**Complexity**: Very High (breaking change)

**embroidery-rust Architecture**:

```rust
Pattern
  ├── color_groups: Vec<ColorGroup>
  │     ├── thread: Option<Thread>
  │     └── stitch_groups: Vec<StitchGroup>
  │           ├── stitches: Vec<Stitch>
  │           ├── trim: bool
  │           └── cut: bool

// Hierarchical structure:
// - Pattern contains multiple ColorGroups (one per thread)
// - ColorGroup contains multiple StitchGroups (separated by trims/cuts)
// - StitchGroup contains stitches (actual needle positions)
```

**Our Current Architecture**:

```rust
EmbPattern
  ├── stitches: Vec<Stitch>  // Flat list
  └── threads: Vec<EmbThread>
```

**Benefits of Color/Stitch Groups**:

1. **Natural grouping**: Stitches grouped by color and continuity
2. **Trim/Cut handling**: Explicit flags instead of command bits
3. **Iteration**: Easy to iterate by color or group
4. **Format compliance**: Maps directly to format structure
5. **Optimization**: Remove duplicate stitches per group

**Migration Strategy** (Non-Breaking):

```rust
// Phase 1: Add optional group structure
pub struct EmbPattern {
    stitches: Vec<Stitch>,  // Keep for backward compatibility
    threads: Vec<EmbThread>,
    color_groups: Option<Vec<ColorGroup>>,  // NEW
}

impl EmbPattern {
    pub fn build_color_groups(&mut self) -> &[ColorGroup]
    pub fn flatten_to_stitches(&mut self) -> &[Stitch]
    pub fn has_groups(&self) -> bool
}

// Phase 2: Prefer groups in new code
// Phase 3: Deprecate flat stitches (major version bump)
```

**Implementation Notes**:

- High complexity due to fundamental architecture change
- Breaking change if done fully
- Consider phased approach with feature flag
- Significant testing required

---

### 12. Build Stitch List / Iteration Helper 🎯

**Status**: ❌ Missing  
**Impact**: Medium-High - Simplifies format writers  
**Complexity**: Medium

**embroidery-rust Implementation**:

```rust
pub enum StitchInfo<'a> {
    Color(&'a Option<Thread>, &'a Stitch),  // Color change
    Cut(&'a Stitch),                        // Cut command
    End(&'a Stitch),                        // End pattern
    Jump(&'a Stitch),                       // Jump stitch
    Stitch(&'a Stitch),                     // Normal stitch
}

pub fn build_stitch_list<'a>(pattern: &'a Pattern) -> Vec<StitchInfo<'a>> {
    // Converts hierarchical pattern to flat command list
    // Automatically inserts color changes
    // Handles jump stitches between groups
    // Ensures proper END command
}
```

**Use Case**:

```rust
// In format writers
for info in build_stitch_list(&pattern) {
    match info {
        StitchInfo::Stitch(s) => write_stitch(s),
        StitchInfo::Jump(s) => write_jump(s),
        StitchInfo::Color(thread, s) => write_color_change(thread, s),
        StitchInfo::Cut(s) => write_cut(s),
        StitchInfo::End(s) => write_end(s),
    }
}
```

**Required Implementation**:

```rust
// In src/utils/processing.rs or src/core/pattern.rs
pub enum StitchCommand<'a> {
    Stitch(&'a Stitch),
    Jump(&'a Stitch),
    ColorChange(&'a EmbThread, &'a Stitch),
    Trim(&'a Stitch),
    Stop(&'a Stitch),
    End(&'a Stitch),
}

impl EmbPattern {
    pub fn iter_commands(&self) -> impl Iterator<Item = StitchCommand>
}
```

---

### 13. Pattern Attributes / Metadata 🎯

**Status**: ⚠️ Partial (basic metadata exists)  
**Impact**: Medium-High - Professional output  
**Complexity**: Low

**embroidery-rust Design**:

```rust
pub enum PatternAttribute {
    Arbitrary(String, String),  // Custom key-value
    Title(String),              // Pattern title
    Author(String),             // Designer name
    Copyright(String),          // Copyright info
}

pub struct Pattern {
    pub name: String,
    pub attributes: Vec<PatternAttribute>,
    // ...
}
```

**Our Current State**:

```rust
// In EmbPattern
pub struct EmbPattern {
    metadata: HashMap<String, String>,  // Generic metadata
    // ...
}
```

**Enhancement Needed**:

```rust
// Add typed attributes
pub enum PatternAttribute {
    Title(String),
    Author(String),
    Copyright(String),
    Date(String),
    Keywords(Vec<String>),
    Description(String),
    Software(String),          // NEW - Creating software
    SoftwareVersion(String),   // NEW - Software version
    StitchCount(usize),        // NEW - Precomputed count
    ThreadCount(usize),        // NEW - Precomputed count
    Custom(String, String),    // Arbitrary metadata
}

impl EmbPattern {
    pub fn get_attribute(&self, attr: &str) -> Option<&PatternAttribute>
    pub fn set_attribute(&mut self, attr: PatternAttribute)
    pub fn title(&self) -> Option<&str>
    pub fn set_title(&mut self, title: impl Into<String>)
    pub fn author(&self) -> Option<&str>
    pub fn set_author(&mut self, author: impl Into<String>)
}
```

---

### 14. Read Macros for Binary Parsing 🎯

**Status**: ❌ Missing  
**Impact**: High - Cleaner reader code, better errors  
**Complexity**: Low

**embroidery-rust Macros**:

```rust
// Read exact bytes with context
read_exact_magic!($reader, [0x00, 0x64]);

// Read integers with automatic EOF handling
read_int!($reader, u16, BigEndian);

// Wrap any read with context
maybe_read_with_context!(
    reader.read_exact(&mut buffer),
    "Reading header at offset {}", offset
);
```

**Required Implementation**:

```rust
// In src/formats/io/macros.rs

#[macro_export]
macro_rules! read_magic {
    ($reader:expr, $expected:expr) => {{
        let expected = $expected;
        let mut actual = vec![0u8; expected.len()];
        $reader.read_exact(&mut actual)
            .map_err(|e| Error::Io(e))?;
        if actual != expected {
            return Err(Error::Parse(format!(
                "Magic bytes mismatch at {}:{}. Expected {:?}, got {:?}",
                file!(), line!(), expected, actual
            )));
        }
        actual
    }};
}

#[macro_export]
macro_rules! read_int {
    ($reader:expr, $type:ty, $endian:ident) => {{
        use byteorder::{$endian, ReadBytesExt};
        $reader.concat!("read_", stringify!($type), "::<", stringify!($endian), ">")()
            .map_err(|e| {
                if e.kind() == std::io::ErrorKind::UnexpectedEof {
                    Error::Parse(format!(
                        "Unexpected EOF reading {} at {}:{}", 
                        stringify!($type), file!(), line!()
                    ))
                } else {
                    Error::Io(e)
                }
            })
    }};
}
```

**Benefits**:

- Cleaner reader code
- Consistent error messages with file:line
- Automatic EOF detection
- Less boilerplate

---

### 15. Palette Color Conversion 🎯

**Status**: ⚠️ Partial (thread palettes exist)  
**Impact**: Medium - Better color accuracy  
**Complexity**: Low

**embroidery-rust Color**:

```rust
impl From<Color> for palette::Srgb {
    fn from(color: Color) -> Self {
        Self::new(
            f32::from(color.red) / 255.,
            f32::from(color.green) / 255.,
            f32::from(color.blue) / 255.,
        )
    }
}
```

**Enhancement Needed**:

```rust
// Add palette dependency
[dependencies]
palette = "0.7"

// In src/core/thread.rs
impl EmbThread {
    pub fn to_srgb(&self) -> palette::Srgb {
        let r = ((self.color >> 16) & 0xFF) as u8;
        let g = ((self.color >> 8) & 0xFF) as u8;
        let b = (self.color & 0xFF) as u8;
        palette::Srgb::new(
            f32::from(r) / 255.0,
            f32::from(g) / 255.0,
            f32::from(b) / 255.0,
        )
    }
    
    pub fn to_lab(&self) -> palette::Lab
    pub fn to_hsl(&self) -> palette::Hsl
    
    pub fn closest_palette_color(&self, palette: &[EmbThread]) -> &EmbThread {
        // Find closest color using deltaE color difference
    }
}
```

---

### 16. Stitch Type Categorization 🎯

**Status**: ❌ Missing  
**Impact**: Medium - Better format handling  
**Complexity**: Low

**embroidery-rust Stitch Types**:

```rust
pub enum HusVipStitchType {
    Normal,
    Jump,
    ColorChange,
    LastStitch,
}
```

**Enhancement Needed**:

```rust
// In src/core/constants.rs or new stitch_types.rs
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StitchType {
    Normal,        // Regular stitch
    Jump,          // Jump (no thread)
    Trim,          // Trim thread
    ColorChange,   // Change to next thread
    Stop,          // Machine stop
    End,           // End of pattern
    SequinEject,   // Sequin placement
    SequinMode,    // Enter sequin mode
}

impl Stitch {
    pub fn stitch_type(&self) -> StitchType {
        let cmd = self.command & COMMAND_MASK;
        match cmd {
            STITCH => StitchType::Normal,
            JUMP => StitchType::Jump,
            TRIM => StitchType::Trim,
            COLOR_CHANGE => StitchType::ColorChange,
            STOP => StitchType::Stop,
            END => StitchType::End,
            SEQUIN_EJECT => StitchType::SequinEject,
            SEQUIN_MODE => StitchType::SequinMode,
            _ => StitchType::Normal,
        }
    }
}
```

---

### 17. Property-Based Testing (proptest) 🎯

**Status**: ❌ Missing  
**Impact**: High - Better test coverage  
**Complexity**: Medium

**embroidery-rust Tests**:

```rust
use proptest::prelude::*;

prop_compose! {
    fn stitch_strategy()
                      (x in -STITCH_MAX..STITCH_MAX, 
                       y in -STITCH_MAX..STITCH_MAX)
                      -> Stitch {
        Stitch {x, y}
    }
}

proptest! {
    #[test]
    fn split_stitches_proptest(
        sg in stitch_group_strategy(100),
        min_x in -STITCH_MAX..0.0,
        max_x in 0.0..STITCH_MAX,
        min_y in -STITCH_MAX..0.0,
        max_y in 0.0..STITCH_MAX
    ) {
        let new_sg = sg.clone().split_stitches(min_x, max_x, min_y, max_y);
        prop_assert!(new_sg.stitches.len() >= sg.stitches.len())
    }
}
```

**Required Implementation**:

```rust
// Add to Cargo.toml
[dev-dependencies]
proptest = "1.0"

// In src/core/pattern.rs tests
mod proptests {
    use super::*;
    use proptest::prelude::*;
    
    prop_compose! {
        fn stitch_strategy()
            (x in -10000.0..10000.0, 
             y in -10000.0..10000.0,
             cmd in 0u32..256u32)
            -> Stitch {
            Stitch { x, y, command: cmd }
        }
    }
    
    proptest! {
        #[test]
        fn translate_preserves_stitch_count(
            pattern in pattern_strategy(),
            dx in -1000.0..1000.0,
            dy in -1000.0..1000.0
        ) {
            let orig_count = pattern.stitches().len();
            let mut translated = pattern.clone();
            translated.translate(dx, dy);
            prop_assert_eq!(translated.stitches().len(), orig_count);
        }
        
        #[test]
        fn bounds_always_valid(pattern in pattern_strategy()) {
            let (min_x, min_y, max_x, max_y) = pattern.bounds();
            prop_assert!(min_x <= max_x);
            prop_assert!(min_y <= max_y);
        }
    }
}
```

---

### 18. Better Thread Display / Debug 🎯

**Status**: ⚠️ Partial  
**Impact**: Low-Medium - Developer experience  
**Complexity**: Very Low

**embroidery-rust Display**:

```rust
impl fmt::Display for Color {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "#{:02X}{:02X}{:02X}", self.red, self.green, self.blue)
    }
}

impl fmt::Display for Stitch {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "({}, {})", self.x, self.y)
    }
}
```

**Enhancement Needed**:

```rust
// In src/core/thread.rs
impl fmt::Display for EmbThread {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let r = (self.color >> 16) & 0xFF;
        let g = (self.color >> 8) & 0xFF;
        let b = self.color & 0xFF;
        write!(f, "#{:02X}{:02X}{:02X}", r, g, b)?;
        if !self.description.is_empty() {
            write!(f, " ({})", self.description)?;
        }
        Ok(())
    }
}

// In src/core/pattern.rs
impl fmt::Display for Stitch {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Stitch({:.2}, {:.2}, cmd=0x{:02X})", 
               self.x, self.y, self.command)
    }
}
```

---

### 19. VF3 Format Support 🎯

**Status**: ❌ Missing  
**Impact**: Medium - Additional format coverage  
**Complexity**: High

**embroidery-rust Support**:

- Has `formats/vf3/` directory
- Implementation incomplete in source

**Research Required**:

- VF3 is Viking/Pfaff format (related to VP3)
- May be older version or variant
- Check if different from VP3

**Action**: Research format specification

---

### 20. Stitch Iterator with Zero Position Handling 🎯

**Status**: ⚠️ Exists but could be enhanced  
**Impact**: Low-Medium  
**Complexity**: Low

**embroidery-rust Pattern**:

```rust
pub const ZERO_STITCH: Stitch = Stitch::new(0.0, 0.0);

pub fn build_stitch_list<'a>(pattern: &'a Pattern) -> Vec<StitchInfo<'a>> {
    let mut last_stitch = &ZERO_STITCH;
    // Automatically handles position tracking
}
```

**Enhancement**:

```rust
// In src/core/pattern.rs
pub const ZERO_POSITION: (f64, f64) = (0.0, 0.0);

impl EmbPattern {
    pub fn iter_with_tracking(&self) -> StitchIterator {
        StitchIterator {
            stitches: &self.stitches,
            current_pos: ZERO_POSITION,
            index: 0,
        }
    }
}

pub struct StitchIterator<'a> {
    stitches: &'a [Stitch],
    current_pos: (f64, f64),
    index: usize,
}

impl<'a> Iterator for StitchIterator<'a> {
    type Item = StitchWithPosition<'a>;
    // Returns stitch with both absolute and relative positions
}
```

---

### 21. Automatic Bounds Calculation 🎯

**Status**: ✅ Exists (get_bounds)  
**Impact**: N/A  
**Complexity**: N/A

**embroidery-rust**:

```rust
pub fn get_bounds(&self) -> (f64, f64, f64, f64) {
    let mut min_x: f64 = f64::NAN;
    // ...uses NAN for initialization, handles empty patterns
}
```

**Our Implementation**:

```rust
pub fn bounds(&self) -> (f64, f64, f64, f64) {
    // ✅ Already implemented correctly
}
```

**No Action Needed** - We already have this!

---

### 22. CUT vs TRIM Distinction 🎯

**Status**: ⚠️ Partial (TRIM exists, CUT may be missing)  
**Impact**: Medium - Format-specific  
**Complexity**: Low

**embroidery-rust Design**:

```rust
pub struct StitchGroup {
    pub stitches: Vec<Stitch>,
    pub trim: bool,  // Thread trim
    pub cut: bool,   // Thread cut (stronger than trim)
}
```

**Difference**:

- **TRIM**: Cut thread but leave tail (common)
- **CUT**: Full cut, no tail (rare, some machines)

**Enhancement Needed**:

```rust
// Add CUT command to constants.rs
pub const CUT: u32 = 0x20;  // New command

// Some formats distinguish:
// - XXX format: 0x88 attribute = CUT
// - HUS format: Separate cut flag
```

---

### 23. Comprehensive Format Testing Suite 🎯

**Status**: ⚠️ Partial (some tests exist)  
**Impact**: High - Quality assurance  
**Complexity**: High (requires test data)

**embroidery-rust Pattern**:

- Uses `tests/` directory with real files
- Roundtrip tests for each format
- Property-based testing

**Required Implementation**:

```rust
// Expand testing/
testing/
  ├── samples/        # Real embroidery files
  │   ├── dst/
  │   ├── pes/
  │   ├── jef/
  │   └── ...
  ├── generated/      # Programmatically created
  └── malformed/      # Edge cases, corruption

// Add comprehensive tests
#[test]
fn test_all_formats_roundtrip() {
    for format in ALL_FORMATS {
        for sample in get_samples(format) {
            let pattern = read(sample, format)?;
            let output = write(pattern, format)?;
            let reread = read(output, format)?;
            assert_patterns_equivalent(pattern, reread);
        }
    }
}
```

---

## 🟡 MEDIUM PRIORITY (P2)

### 24. UTF-8 String Utilities 🎯

**Status**: ❌ Missing  
**Impact**: Low-Medium - Cleaner string handling  
**Complexity**: Low

**embroidery-rust Utils**:

```rust
// Trim C-style null-terminated strings
pub fn c_trim(s: &str) -> &str {
    s.trim_end_matches('\0').trim()
}

// Truncate string to char boundary
pub fn char_truncate(s: &str, max_chars: usize) -> &str {
    match s.char_indices().nth(max_chars) {
        Some((idx, _)) => &s[..idx],
        None => s,
    }
}
```

**Implementation**:

```rust
// In src/utils/str_util.rs (new file)

/// Trim null bytes and whitespace from C-style strings
pub fn c_trim(s: &str) -> &str {
    s.trim_end_matches('\0').trim()
}

/// Truncate string to maximum characters (respects UTF-8 boundaries)
pub fn char_truncate(s: &str, max_chars: usize) -> &str {
    match s.char_indices().nth(max_chars) {
        Some((idx, _)) => &s[..idx],
        None => s,
    }
}

/// Convert null-padded byte array to String
pub fn from_null_padded(bytes: &[u8]) -> String {
    let null_pos = bytes.iter().position(|&b| b == 0)
        .unwrap_or(bytes.len());
    String::from_utf8_lossy(&bytes[..null_pos]).to_string()
}
```

---

### 25. Byte Iterator with Error Handling 🎯

**Status**: ❌ Missing  
**Impact**: Low - Convenience for readers  
**Complexity**: Low

**embroidery-rust Implementation**:

```rust
pub struct ReadByteIterator<T: Read + Sized> {
    reader: Bytes<T>,
    pub closed: bool,
    pub error: Option<Error>,
}

impl<T: Read + Sized> Iterator for ReadByteIterator<T> {
    type Item = u8;
    fn next(&mut self) -> Option<u8> {
        if self.closed { return None; }
        match self.reader.next() {
            Some(Ok(value)) => Some(value),
            Some(Err(error)) => {
                self.error = Some(error);
                self.close();
                None
            },
            None => {
                self.close();
                None
            },
        }
    }
}
```

**Use Case**: Simplify reading byte streams in format readers

---

### 26. Explicit Color vs Thread Terminology 🎯

**Status**: ⚠️ Inconsistent terminology  
**Impact**: Low - Documentation clarity  
**Complexity**: Very Low

**Issue**: We use "thread" for physical thread and "color" for color changes

**Clarification Needed**:

- **Thread**: Physical spool with color, brand, etc.
- **Color**: Just RGB value
- **ColorChange**: Command to switch to next thread

**Action**: Documentation update, consistent naming

---

### 27. Format Registry / Plugin System 🎯

**Status**: ❌ Missing  
**Impact**: Medium - Extensibility  
**Complexity**: High

**embroidery-rust Pattern**:

```rust
pub trait PatternFormat {
    fn name(&self) -> &str;
    fn extensions(&self) -> &[&str];
    fn reader(&self) -> Option<Box<dyn PatternReader>>;
    fn writer(&self) -> Option<Box<dyn PatternWriter>>;
}

fn get_all() -> Vec<Box<dyn PatternFormat>> {
    vec![
        Box::new(DstFormat::default()),
        Box::new(PesFormat::default()),
        // ...
    ]
}
```

**Benefits**:

- Dynamic format discovery
- Plugin support
- Easier testing
- Auto-detect from registry

**Implementation**:

```rust
// In src/formats/registry.rs

pub trait FormatHandler {
    fn name(&self) -> &str;
    fn extensions(&self) -> &[&str];
    fn can_read(&self) -> bool;
    fn can_write(&self) -> bool;
    fn read(&self, file: &mut impl Read, pattern: &mut EmbPattern) -> Result<()>;
    fn write(&self, pattern: &EmbPattern, file: &mut impl Write) -> Result<()>;
}

pub struct FormatRegistry {
    handlers: Vec<Box<dyn FormatHandler>>,
}

impl FormatRegistry {
    pub fn register(&mut self, handler: Box<dyn FormatHandler>)
    pub fn get_handler(&self, name: &str) -> Option<&dyn FormatHandler>
    pub fn detect_format(&self, file: &mut (impl Read + Seek)) -> Result<&dyn FormatHandler>
}
```

---

### 28. Workspace/Cargo Configuration Improvements 🎯

**Status**: ⚠️ Single crate (could be workspace)  
**Impact**: Low - Organization  
**Complexity**: Medium

**embroidery-rust Structure**:

```sh
workspace/
  ├── embroidery-lib/     # Core library
  ├── formats/dst/        # DST format crate
  ├── formats/hus/        # HUS format crate
  └── formats/jef/        # JEF format crate
```

**Benefits**:

- Modular format support
- Optional format features
- Faster compile times (parallel)
- Easier testing

**Consideration**: May be overkill for our size

---

### 29. EditorConfig Support 🎯

**Status**: ❌ Missing  
**Impact**: Very Low - Editor consistency  
**Complexity**: Very Low

**embroidery-rust Has**:

```ini
# .editorconfig
root = true

[*]
charset = utf-8
end_of_line = lf
indent_style = space
indent_size = 4
```

**Action**: Add `.editorconfig` file

---

### 30. Clippy Configuration 🎯

**Status**: ⚠️ Basic (inline warnings)  
**Impact**: Low - Code quality  
**Complexity**: Very Low

**embroidery-rust Has**:

```toml
# .clippy.toml
cognitive-complexity-threshold = 50
```

**Our Enhancement**:

```toml
# .clippy.toml
cognitive-complexity-threshold = 30
doc-valid-idents = ["ButaButi", "DST", "PES", "JEF", "VP3", "HUS", "VIP"]
```

---

### 31. Rustfmt Configuration 🎯

**Status**: ⚠️ Default fmt  
**Impact**: Very Low - Consistency  
**Complexity**: Very Low

**embroidery-rust Has**:

```toml
# .rustfmt.toml
edition = "2018"
```

**Action**: Add `.rustfmt.toml` with project preferences

---

### 32. Fuzzing Tests 🎯

**Status**: ❌ Missing  
**Impact**: Medium - Security, robustness  
**Complexity**: Medium

**Implementation**:

```rust
// Add to Cargo.toml
[dev-dependencies]
cargo-fuzz = "0.11"

// Create fuzz/fuzz_targets/
#[macro_use] extern crate libfuzzer_sys;

fuzz_target!(|data: &[u8]| {
    let mut pattern = EmbPattern::new();
    let _ = dst::read(&mut &data[..], &mut pattern);
});
```

---

### 33. Benchmark Suite 🎯

**Status**: ❌ Missing  
**Impact**: Low-Medium - Performance tracking  
**Complexity**: Medium

**Implementation**:

```rust
// benches/pattern_operations.rs
use criterion::{black_box, criterion_group, criterion_main, Criterion};

fn bench_translate(c: &mut Criterion) {
    let mut pattern = create_large_pattern(10000);
    c.bench_function("translate 10k stitches", |b| {
        b.iter(|| {
            pattern.translate(black_box(10.0), black_box(10.0))
        });
    });
}

criterion_group!(benches, bench_translate);
criterion_main!(benches);
```

---

### 34. Pattern Statistics Dashboard 🎯

**Status**: ⚠️ Basic stats exist  
**Impact**: Low - User feature  
**Complexity**: Low

**Enhancement**:

```rust
pub struct PatternStatistics {
    pub stitch_count: usize,
    pub jump_count: usize,
    pub trim_count: usize,
    pub color_change_count: usize,
    pub total_length_mm: f64,
    pub total_length_inches: f64,
    pub estimated_time_minutes: f64,  // NEW - Based on machine speed
    pub thread_usage: Vec<ThreadUsage>,  // NEW - Per-color usage
    pub density: f64,  // NEW - Stitches per area
}

pub struct ThreadUsage {
    pub thread: EmbThread,
    pub length_mm: f64,
    pub stitch_count: usize,
}
```

---

### 35. VP4 Format Support (Incomplete in embroidery-rust) 🎯

**Status**: ❌ Missing  
**Impact**: Low - Additional format  
**Complexity**: High

**Note**: Even embroidery-rust has incomplete VP4 implementation

**Action**: Research if VP4 is worth implementing

---

## 🟢 LOW PRIORITY (P3)

### 36. GitHub Actions CI/CD 🎯

**Status**: ⚠️ May exist in .github  
**Impact**: Low - Automation  
**Complexity**: Low

**embroidery-rust Has**: `.github/workflows/`

---

### 37. Code Coverage Reports 🎯

**Status**: ❌ Missing  
**Impact**: Low - Quality metrics  
**Complexity**: Low

**Implementation**: tarpaulin or codecov

---

### 38. Changelog 🎯

**Status**: ❌ Missing  
**Impact**: Very Low - Documentation  
**Complexity**: Very Low

**Action**: Add `CHANGELOG.md`

---

### 39. Contribution Guide Enhancement 🎯

**Status**: ✅ Exists (CONTRIBUTING.md)  
**Impact**: N/A  
**Complexity**: N/A

---

### 40. Example Gallery 🎯

**Status**: ⚠️ Basic examples exist  
**Impact**: Low - Documentation  
**Complexity**: Low

**Enhancement**: Add visual examples with SVG output

---

### 41. Python Bindings (PyO3) 🎯

**Status**: ❌ Missing  
**Impact**: Low - Language bindings  
**Complexity**: High

**Future Feature**: Rust library callable from Python

---

### 42. WASM Support 🎯

**Status**: ❌ Missing  
**Impact**: Low - Web use  
**Complexity**: Medium

**Future Feature**: Compile to WebAssembly for browser use

---

### 43. CLI Tool 🎯

**Status**: ❌ Missing  
**Impact**: Low - User tool  
**Complexity**: Medium

**embroidery-rust Has**: Command-line converter in `src/main.rs`

**Implementation**:

```rust
// In bin/butabuti.rs
fn main() {
    // Convert files
    // View statistics
    // Validate patterns
}
```

---

### 44. GUI Application 🎯

**Status**: ❌ Missing  
**Impact**: Very Low - Separate product  
**Complexity**: Very High

**Future**: Desktop app using egui or iced

---

### 45. Online Documentation / Book 🎯

**Status**: ⚠️ Rustdoc exists  
**Impact**: Low - Better docs  
**Complexity**: Medium

**Enhancement**: mdBook or similar for user guide

---

## Implementation Roadmap

### Phase 1: Critical Foundations (P0) - Weeks 1-4

1. Pattern Transformations (rotate, scale, flip)
2. Stitch Splitting / Long Stitch Handling
3. Format Auto-Detection
4. Contextual Error Messages

### Phase 2: High-Value Features (P1) - Weeks 5-8

1. Stitch Distance Calculation
2. Stitch Validation
3. Build Stitch List Iterator
4. Pattern Attributes Enhancement
5. Read Macros for Binary Parsing
6. Property-Based Testing

### Phase 3: Quality Improvements (P1-P2) - Weeks 9-12

1. Thread Metadata Enhancement
2. Palette Color Conversion
3. Stitch Type Categorization
4. Better Display/Debug
5. UTF-8 String Utilities
6. Comprehensive Testing Suite

### Phase 4: Architecture (Long-term)

1. Color Group / Stitch Group Architecture (breaking change)
2. Pattern Collection / Multi-Pattern Files
3. Format Registry / Plugin System

### Phase 5: Polish & Extras (Ongoing)

1. Fuzzing, Benchmarks, Coverage
2. Additional format support (VF3, VP4)
3. CLI tool, documentation improvements

---

## Metrics & Success Criteria

### Coverage Goals

- [ ] All P0 features implemented (8/8)
- [ ] 80% of P1 features implemented (12/15)
- [ ] 50% of P2 features implemented (6/12)
- [ ] Test coverage >85%
- [ ] Zero clippy warnings
- [ ] All format roundtrip tests passing

### Quality Metrics

- [ ] Property-based tests for all transformations
- [ ] Comprehensive error messages with context
- [ ] Full API documentation coverage
- [ ] Real-world format compatibility verified

---

## Notes

1. **Breaking Changes**: Color/Stitch Group architecture is a major refactor
2. **Dependencies**: Some features require new crates (palette, proptest)
3. **Format Support**: Prioritize complete implementation over format count
4. **Testing**: Real embroidery files needed for validation
5. **Community**: Consider feature requests from users

---

**Total Features Identified**: 45  
**Critical (P0)**: 8  
**High (P1)**: 15  
**Medium (P2)**: 12  
**Low (P3)**: 10

**Estimated Total Implementation Time**: 12-16 weeks (full-time)  
**Recommended Team Size**: 2-3 developers
