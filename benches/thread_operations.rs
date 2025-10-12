use butabuti::prelude::*;
use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};

// Benchmark: Thread creation from strings
fn bench_thread_creation(c: &mut Criterion) {
    let mut group = c.benchmark_group("thread_creation");

    let colors = vec![
        "red", "blue", "green", "yellow", "purple", "orange", "pink", "brown", "black", "white",
    ];

    group.bench_function("from_string_named", |b| {
        b.iter(|| {
            for color in &colors {
                let thread = EmbThread::from_string(black_box(color)).unwrap();
                black_box(thread);
            }
        });
    });

    group.bench_function("from_string_hex", |b| {
        b.iter(|| {
            for i in 0..10 {
                let hex = format!("{:06X}", i * 0x111111);
                let thread = EmbThread::from_string(black_box(&hex)).unwrap();
                black_box(thread);
            }
        });
    });

    group.bench_function("new_u32", |b| {
        b.iter(|| {
            for i in 0..10 {
                let thread = EmbThread::new(black_box(i * 0x111111));
                black_box(thread);
            }
        });
    });

    group.finish();
}

// Benchmark: Color distance calculations
fn bench_color_distance(c: &mut Criterion) {
    let mut group = c.benchmark_group("color_distance");

    let thread1 = EmbThread::new(0xFF0000); // Red
    let thread2 = EmbThread::new(0x00FF00); // Green
    let thread3 = EmbThread::new(0x0000FF); // Blue

    group.bench_function("rgb_distance", |b| {
        b.iter(|| {
            let dist1 = thread1.color_distance(black_box(thread2.color));
            let dist2 = thread2.color_distance(black_box(thread3.color));
            let dist3 = thread3.color_distance(black_box(thread1.color));
            black_box((dist1, dist2, dist3));
        });
    });

    group.bench_function("delta_e", |b| {
        b.iter(|| {
            let dist1 = thread1.delta_e(black_box(&thread2));
            let dist2 = thread2.delta_e(black_box(&thread3));
            let dist3 = thread3.delta_e(black_box(&thread1));
            black_box((dist1, dist2, dist3));
        });
    });

    group.finish();
}

// Benchmark: Finding nearest thread in palette
fn bench_find_nearest(c: &mut Criterion) {
    let mut group = c.benchmark_group("find_nearest");

    // Create palettes of different sizes
    for palette_size in [10, 50, 100, 500].iter() {
        let mut palette = Vec::new();
        for i in 0..*palette_size {
            let color = ((i * 255 / palette_size) as u32) * 0x010101;
            palette.push(EmbThread::new(color));
        }

        let test_thread = EmbThread::new(0x7F7F7F); // Gray

        group.throughput(Throughput::Elements(*palette_size as u64));

        group.bench_with_input(
            BenchmarkId::new("rgb_distance", palette_size),
            &palette,
            |b, palette| {
                b.iter(|| {
                    let nearest = test_thread.find_nearest_in_palette(black_box(palette));
                    black_box(nearest);
                });
            },
        );

        group.bench_with_input(
            BenchmarkId::new("delta_e", palette_size),
            &palette,
            |b, palette| {
                b.iter(|| {
                    let nearest = test_thread.find_closest_delta_e(black_box(palette));
                    black_box(nearest);
                });
            },
        );
    }

    group.finish();
}

// Benchmark: Color space conversions
fn bench_color_conversions(c: &mut Criterion) {
    let mut group = c.benchmark_group("color_conversions");

    let thread = EmbThread::new(0xFF8040); // Orange-ish

    group.bench_function("to_srgb", |b| {
        b.iter(|| {
            let srgb = thread.to_srgb();
            black_box(srgb);
        });
    });

    group.bench_function("to_lab", |b| {
        b.iter(|| {
            let lab = thread.to_lab();
            black_box(lab);
        });
    });

    group.bench_function("to_hsl", |b| {
        b.iter(|| {
            let hsl = thread.to_hsl();
            black_box(hsl);
        });
    });

    group.finish();
}

// Benchmark: Thread attribute operations
fn bench_thread_attributes(c: &mut Criterion) {
    let mut group = c.benchmark_group("thread_attributes");

    group.bench_function("set_attribute", |b| {
        b.iter(|| {
            let mut thread = EmbThread::new(0xFF0000);
            thread.set_attribute(black_box("brand"), black_box("Madeira"));
            thread.set_attribute(black_box("catalog"), black_box("1234"));
            thread.set_attribute(black_box("description"), black_box("Bright Red"));
            black_box(thread);
        });
    });

    let mut thread = EmbThread::new(0xFF0000);
    thread.set_attribute("brand", "Madeira");
    thread.set_attribute("catalog", "1234");
    thread.set_attribute("description", "Bright Red");

    group.bench_function("get_attribute", |b| {
        b.iter(|| {
            let brand = thread.get_attribute(black_box("brand"));
            let catalog = thread.get_attribute(black_box("catalog"));
            let description = thread.get_attribute(black_box("description"));
            black_box((brand, catalog, description));
        });
    });

    group.bench_function("clone_with_attributes", |b| {
        b.iter(|| {
            let cloned = thread.clone();
            black_box(cloned);
        });
    });

    group.finish();
}

criterion_group!(
    benches,
    bench_thread_creation,
    bench_color_distance,
    bench_find_nearest,
    bench_color_conversions,
    bench_thread_attributes,
);

criterion_main!(benches);
