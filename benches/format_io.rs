use butabuti::formats::io::writers::csv::CsvVersion;
use butabuti::formats::io::{readers, writers};
use butabuti::prelude::*;
use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use std::io::Cursor;

// Helper function to create a test pattern
fn create_test_pattern(stitch_count: usize) -> EmbPattern {
    let mut pattern = EmbPattern::new();
    pattern.add_thread(EmbThread::from_string("red").unwrap());
    pattern.add_thread(EmbThread::from_string("blue").unwrap());
    pattern.add_thread(EmbThread::from_string("green").unwrap());

    for i in 0..stitch_count {
        // Add color changes periodically
        if i > 0 && i % (stitch_count / 3) == 0 {
            pattern.color_change(0.0, 0.0);
        }

        let x = (i % 100) as f64 * 10.0;
        let y = (i / 100) as f64 * 10.0;
        pattern.stitch_abs(x, y);

        // Add some jumps
        if i % 50 == 0 {
            pattern.jump(50.0, 0.0);
        }
    }

    pattern
}

// Benchmark: DST format I/O
fn bench_dst_io(c: &mut Criterion) {
    let mut group = c.benchmark_group("dst_format");

    for size in [100, 1000, 5000].iter() {
        let pattern = create_test_pattern(*size);

        // Write benchmark
        group.throughput(Throughput::Elements(*size as u64));
        group.bench_with_input(BenchmarkId::new("write", size), &pattern, |b, pattern| {
            b.iter(|| {
                let mut buffer = Vec::new();
                writers::dst::write(&mut buffer, pattern, false, 0).unwrap();
                black_box(buffer);
            });
        });

        // Read benchmark
        let mut buffer = Vec::new();
        writers::dst::write(&mut buffer, &pattern, false, 0).unwrap();

        group.bench_with_input(BenchmarkId::new("read", size), &buffer, |b, buffer| {
            b.iter(|| {
                let mut cursor = Cursor::new(buffer);
                let pattern = readers::dst::read(&mut cursor, None).unwrap();
                black_box(pattern);
            });
        });

        // Round-trip benchmark
        group.bench_with_input(
            BenchmarkId::new("roundtrip", size),
            &pattern,
            |b, pattern| {
                b.iter(|| {
                    let mut write_buffer = Vec::new();
                    writers::dst::write(&mut write_buffer, pattern, false, 0).unwrap();

                    let mut cursor = Cursor::new(&write_buffer);
                    let read_pattern = readers::dst::read(&mut cursor, None).unwrap();
                    black_box(read_pattern);
                });
            },
        );
    }

    group.finish();
}

// Benchmark: JSON format I/O
fn bench_json_io(c: &mut Criterion) {
    let mut group = c.benchmark_group("json_format");

    for size in [100, 1000, 5000].iter() {
        let pattern = create_test_pattern(*size);

        // Write benchmark
        group.throughput(Throughput::Elements(*size as u64));
        group.bench_with_input(BenchmarkId::new("write", size), &pattern, |b, pattern| {
            b.iter(|| {
                let mut buffer = Vec::new();
                writers::json::write(&mut buffer, pattern).unwrap();
                black_box(buffer);
            });
        });

        // Read benchmark
        let mut buffer = Vec::new();
        writers::json::write(&mut buffer, &pattern).unwrap();

        group.bench_with_input(BenchmarkId::new("read", size), &buffer, |b, buffer| {
            b.iter(|| {
                let mut cursor = Cursor::new(buffer);
                let pattern = readers::json::read(&mut cursor).unwrap();
                black_box(pattern);
            });
        });
    }

    group.finish();
}

// Benchmark: CSV format I/O
fn bench_csv_io(c: &mut Criterion) {
    let mut group = c.benchmark_group("csv_format");

    for size in [100, 1000, 5000].iter() {
        let pattern = create_test_pattern(*size);

        // Write benchmark
        group.throughput(Throughput::Elements(*size as u64));
        group.bench_with_input(BenchmarkId::new("write", size), &pattern, |b, pattern| {
            b.iter(|| {
                let mut buffer = Vec::new();
                writers::csv::write(&mut buffer, pattern, CsvVersion::Default).unwrap();
                black_box(buffer);
            });
        });

        // Read benchmark
        let mut buffer = Vec::new();
        writers::csv::write(&mut buffer, &pattern, CsvVersion::Default).unwrap();

        group.bench_with_input(BenchmarkId::new("read", size), &buffer, |b, buffer| {
            b.iter(|| {
                let mut cursor = Cursor::new(buffer);
                let mut pattern = EmbPattern::new();
                readers::csv::read(&mut cursor, &mut pattern).unwrap();
                black_box(pattern);
            });
        });
    }

    group.finish();
}

// Benchmark: EXP format I/O
fn bench_exp_io(c: &mut Criterion) {
    let mut group = c.benchmark_group("exp_format");

    for size in [100, 1000, 5000].iter() {
        let pattern = create_test_pattern(*size);

        // Write benchmark
        group.throughput(Throughput::Elements(*size as u64));
        group.bench_with_input(BenchmarkId::new("write", size), &pattern, |b, pattern| {
            b.iter(|| {
                let mut buffer = Vec::new();
                writers::exp::write(&mut buffer, pattern).unwrap();
                black_box(buffer);
            });
        });

        // Read benchmark
        let mut buffer = Vec::new();
        writers::exp::write(&mut buffer, &pattern).unwrap();

        group.bench_with_input(BenchmarkId::new("read", size), &buffer, |b, buffer| {
            b.iter(|| {
                let mut cursor = Cursor::new(buffer);
                let pattern = readers::exp::read(&mut cursor).unwrap();
                black_box(pattern);
            });
        });
    }

    group.finish();
}

criterion_group!(
    benches,
    bench_dst_io,
    bench_json_io,
    bench_csv_io,
    bench_exp_io,
);

criterion_main!(benches);
