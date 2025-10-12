# ButaButi Benchmarks

This directory contains comprehensive performance benchmarks for the ButaButi embroidery library.

## Running Benchmarks

### Run all benchmarks

```powershell
cargo bench
```

### Run specific benchmark suite

```powershell
cargo bench --bench pattern_operations
cargo bench --bench format_io
cargo bench --bench thread_operations
```

### Filter benchmarks by name

```powershell
# Run only transformation benchmarks
cargo bench transformations

# Run only DST format benchmarks
cargo bench dst_format

# Run specific operation
cargo bench translate
```

## Benchmark Suites

### 1. Pattern Operations (`pattern_operations.rs`)

Tests core pattern manipulation performance:

- **Pattern Creation** - Creating patterns with varying stitch counts (100, 1000, 10000)
- **Transformations** - translate, rotate, scale, flip_horizontal, flip_vertical
- **Stitch Splitting** - Splitting long stitches at various thresholds
- **Statistics** - Calculating comprehensive pattern statistics
- **Bounds Calculation** - Computing pattern boundaries
- **Stitch Counting** - count_stitches, count_jumps, count_trims, count_color_changes
- **Length Calculations** - total_stitch_length, avg_stitch_length, max_stitch_length
- **Pattern Cloning** - Deep copying patterns
- **Remove Duplicates** - Deduplication algorithm performance

### 2. Format I/O (`format_io.rs`)

Tests file format reading/writing performance:

- **DST Format** - Write, Read, Round-trip (100, 1000, 5000 stitches)
- **JSON Format** - Write, Read (100, 1000, 5000 stitches)
- **CSV Format** - Write, Read (100, 1000, 5000 stitches)
- **EXP Format** - Write, Read (100, 1000, 5000 stitches)

Each benchmark measures throughput in stitches per second.

### 3. Thread Operations (`thread_operations.rs`)

Tests thread color manipulation performance:

- **Thread Creation** - from_string (named colors), from_string (hex), new (u32)
- **Color Distance** - RGB distance vs Delta-E calculations
- **Find Nearest** - Finding closest color in palettes (10, 50, 100, 500 threads)
- **Color Conversions** - to_srgb, to_lab, to_hsl
- **Thread Attributes** - set_attribute, get_attribute, cloning with attributes

## Benchmark Results

Results are saved to `target/criterion/` and include:

- **HTML Reports** - Open `target/criterion/report/index.html` in browser
- **CSV Data** - Raw benchmark data for analysis
- **Comparison Reports** - Performance change between runs

## Performance Baselines

Typical results on a modern development machine (for reference):

| Operation | 100 stitches | 1000 stitches | 10000 stitches |
|-----------|-------------|---------------|----------------|
| Pattern Creation | ~5 µs | ~50 µs | ~500 µs |
| Translate | ~1 µs | ~10 µs | ~100 µs |
| Rotate | ~3 µs | ~30 µs | ~300 µs |
| DST Write | ~20 µs | ~200 µs | ~2 ms |
| DST Read | ~15 µs | ~150 µs | ~1.5 ms |
| Statistics | ~2 µs | ~20 µs | ~200 µs |

*Note: Actual results vary based on hardware and system load.*

## Continuous Performance Monitoring

Benchmarks should be run:

- Before major releases
- After performance-critical changes
- To identify regressions
- To validate optimizations

## Adding New Benchmarks

1. Choose appropriate benchmark file (or create new one)
2. Follow existing patterns:
   - Use `black_box()` to prevent over-optimization
   - Set appropriate throughput measurements
   - Test multiple input sizes
   - Group related benchmarks

Example:

```rust
use criterion::{black_box, criterion_group, criterion_main, Criterion};

fn bench_my_function(c: &mut Criterion) {
    c.bench_function("my_function", |b| {
        b.iter(|| {
            let result = my_function(black_box(input));
            black_box(result);
        });
    });
}

criterion_group!(benches, bench_my_function);
criterion_main!(benches);
```

## Dependencies

Benchmarks use [Criterion.rs](https://github.com/bheisler/criterion.rs):

- Statistical analysis of results
- HTML report generation
- Regression detection
- Configurable sampling

See `Cargo.toml` `[dev-dependencies]` for version information.
