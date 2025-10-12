use butabuti::prelude::*;
use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};

// Helper function to create a pattern with N stitches
fn create_pattern(stitch_count: usize) -> EmbPattern {
    let mut pattern = EmbPattern::new();
    pattern.add_thread(EmbThread::from_string("red").unwrap());

    for i in 0..stitch_count {
        let x = (i % 100) as f64 * 10.0;
        let y = (i / 100) as f64 * 10.0;
        pattern.stitch_abs(x, y);
    }

    pattern
}

// Helper function to create a pattern with multiple threads
fn create_multi_thread_pattern(stitch_count: usize, thread_count: usize) -> EmbPattern {
    let mut pattern = EmbPattern::new();

    // Add threads
    let colors = vec![
        "red", "blue", "green", "yellow", "purple", "orange", "pink", "brown",
    ];
    for i in 0..thread_count {
        let color = colors[i % colors.len()];
        pattern.add_thread(EmbThread::from_string(color).unwrap());
    }

    // Add stitches with color changes
    let stitches_per_thread = stitch_count / thread_count.max(1);
    for t in 0..thread_count {
        if t > 0 {
            pattern.color_change(0.0, 0.0);
        }
        for i in 0..stitches_per_thread {
            let x = (i % 50) as f64 * 10.0;
            let y = (i / 50) as f64 * 10.0 + (t as f64 * 100.0);
            pattern.stitch_abs(x, y);
        }
    }

    pattern
}

// Benchmark: Pattern creation
fn bench_pattern_creation(c: &mut Criterion) {
    let mut group = c.benchmark_group("pattern_creation");

    for size in [100, 1000, 10000].iter() {
        group.throughput(Throughput::Elements(*size as u64));
        group.bench_with_input(BenchmarkId::from_parameter(size), size, |b, &size| {
            b.iter(|| create_pattern(black_box(size)));
        });
    }

    group.finish();
}

// Benchmark: Transformations
fn bench_transformations(c: &mut Criterion) {
    let mut group = c.benchmark_group("transformations");

    for size in [100, 1000, 10000].iter() {
        let pattern = create_pattern(*size);

        // Translation
        group.throughput(Throughput::Elements(*size as u64));
        group.bench_with_input(
            BenchmarkId::new("translate", size),
            &pattern,
            |b, pattern| {
                b.iter(|| {
                    let mut p = pattern.clone();
                    p.translate(black_box(10.0), black_box(10.0));
                });
            },
        );

        // Rotation
        group.bench_with_input(BenchmarkId::new("rotate", size), &pattern, |b, pattern| {
            b.iter(|| {
                let mut p = pattern.clone();
                let _ = p.rotate(black_box(45.0));
            });
        });

        // Scaling
        group.bench_with_input(BenchmarkId::new("scale", size), &pattern, |b, pattern| {
            b.iter(|| {
                let mut p = pattern.clone();
                let _ = p.scale(black_box(1.5), black_box(1.5));
            });
        });

        // Flip horizontal
        group.bench_with_input(
            BenchmarkId::new("flip_horizontal", size),
            &pattern,
            |b, pattern| {
                b.iter(|| {
                    let mut p = pattern.clone();
                    p.flip_horizontal();
                });
            },
        );

        // Flip vertical
        group.bench_with_input(
            BenchmarkId::new("flip_vertical", size),
            &pattern,
            |b, pattern| {
                b.iter(|| {
                    let mut p = pattern.clone();
                    p.flip_vertical();
                });
            },
        );
    }

    group.finish();
}

// Benchmark: Stitch splitting
fn bench_stitch_splitting(c: &mut Criterion) {
    let mut group = c.benchmark_group("stitch_splitting");

    for size in [100, 1000, 10000].iter() {
        // Create pattern with long stitches that need splitting
        let mut pattern = EmbPattern::new();
        pattern.add_thread(EmbThread::from_string("red").unwrap());

        for i in 0..*size {
            let x = (i as f64) * 200.0; // Long stitches (200 units = 20mm)
            pattern.stitch_abs(x, 0.0);
        }

        group.throughput(Throughput::Elements(*size as u64));
        group.bench_with_input(BenchmarkId::from_parameter(size), &pattern, |b, pattern| {
            b.iter(|| {
                let mut p = pattern.clone();
                let _ = p.split_long_stitches(black_box(100.0)); // Split at 10mm
            });
        });
    }

    group.finish();
}

// Benchmark: Statistics calculation
fn bench_statistics(c: &mut Criterion) {
    let mut group = c.benchmark_group("statistics");

    for size in [100, 1000, 10000].iter() {
        let pattern = create_multi_thread_pattern(*size, 5);

        group.throughput(Throughput::Elements(*size as u64));
        group.bench_with_input(BenchmarkId::from_parameter(size), &pattern, |b, pattern| {
            b.iter(|| {
                let stats = pattern.calculate_statistics(black_box(800.0));
                black_box(stats);
            });
        });
    }

    group.finish();
}

// Benchmark: Bounds calculation
fn bench_bounds(c: &mut Criterion) {
    let mut group = c.benchmark_group("bounds");

    for size in [100, 1000, 10000].iter() {
        let pattern = create_pattern(*size);

        group.throughput(Throughput::Elements(*size as u64));
        group.bench_with_input(BenchmarkId::from_parameter(size), &pattern, |b, pattern| {
            b.iter(|| {
                let bounds = pattern.bounds();
                black_box(bounds);
            });
        });
    }

    group.finish();
}

// Benchmark: Stitch counting operations
fn bench_stitch_counting(c: &mut Criterion) {
    let mut group = c.benchmark_group("stitch_counting");

    for size in [100, 1000, 10000].iter() {
        let pattern = create_multi_thread_pattern(*size, 5);

        group.throughput(Throughput::Elements(*size as u64));

        group.bench_with_input(
            BenchmarkId::new("count_stitches", size),
            &pattern,
            |b, pattern| {
                b.iter(|| {
                    let count = pattern.count_stitches();
                    black_box(count);
                });
            },
        );

        group.bench_with_input(
            BenchmarkId::new("count_jumps", size),
            &pattern,
            |b, pattern| {
                b.iter(|| {
                    let count = pattern.count_jumps();
                    black_box(count);
                });
            },
        );

        group.bench_with_input(
            BenchmarkId::new("count_trims", size),
            &pattern,
            |b, pattern| {
                b.iter(|| {
                    let count = pattern.count_trims();
                    black_box(count);
                });
            },
        );

        group.bench_with_input(
            BenchmarkId::new("count_color_changes", size),
            &pattern,
            |b, pattern| {
                b.iter(|| {
                    let count = pattern.count_color_changes();
                    black_box(count);
                });
            },
        );
    }

    group.finish();
}

// Benchmark: Length calculations
fn bench_length_calculations(c: &mut Criterion) {
    let mut group = c.benchmark_group("length_calculations");

    for size in [100, 1000, 10000].iter() {
        let pattern = create_pattern(*size);

        group.throughput(Throughput::Elements(*size as u64));

        group.bench_with_input(
            BenchmarkId::new("total_stitch_length", size),
            &pattern,
            |b, pattern| {
                b.iter(|| {
                    let length = pattern.total_stitch_length();
                    black_box(length);
                });
            },
        );

        group.bench_with_input(
            BenchmarkId::new("avg_stitch_length", size),
            &pattern,
            |b, pattern| {
                b.iter(|| {
                    let length = pattern.avg_stitch_length();
                    black_box(length);
                });
            },
        );

        group.bench_with_input(
            BenchmarkId::new("max_stitch_length", size),
            &pattern,
            |b, pattern| {
                b.iter(|| {
                    let length = pattern.max_stitch_length();
                    black_box(length);
                });
            },
        );
    }

    group.finish();
}

// Benchmark: Pattern cloning
fn bench_pattern_clone(c: &mut Criterion) {
    let mut group = c.benchmark_group("pattern_clone");

    for size in [100, 1000, 10000].iter() {
        let pattern = create_pattern(*size);

        group.throughput(Throughput::Elements(*size as u64));
        group.bench_with_input(BenchmarkId::from_parameter(size), &pattern, |b, pattern| {
            b.iter(|| {
                let cloned = pattern.clone();
                black_box(cloned);
            });
        });
    }

    group.finish();
}

// Benchmark: Remove duplicates
fn bench_remove_duplicates(c: &mut Criterion) {
    let mut group = c.benchmark_group("remove_duplicates");

    for size in [100, 1000, 10000].iter() {
        // Create pattern with some duplicates
        let mut pattern = EmbPattern::new();
        pattern.add_thread(EmbThread::from_string("red").unwrap());

        for i in 0..*size {
            let x = (i % 100) as f64 * 10.0;
            let y = (i / 100) as f64 * 10.0;
            pattern.stitch_abs(x, y);

            // Add duplicate every 10 stitches
            if i % 10 == 0 {
                pattern.stitch_abs(x, y);
            }
        }

        group.throughput(Throughput::Elements(*size as u64));
        group.bench_with_input(BenchmarkId::from_parameter(size), &pattern, |b, pattern| {
            b.iter(|| {
                let mut p = pattern.clone();
                p.remove_duplicates();
            });
        });
    }

    group.finish();
}

criterion_group!(
    benches,
    bench_pattern_creation,
    bench_transformations,
    bench_stitch_splitting,
    bench_statistics,
    bench_bounds,
    bench_stitch_counting,
    bench_length_calculations,
    bench_pattern_clone,
    bench_remove_duplicates,
);

criterion_main!(benches);
