// Fuzz Tests for ButaButi Format Readers
//
// These tests use property-based testing via proptest to generate
// random input data and test format readers for robustness.
//
// Run with: cargo test --test fuzz_formats

use butabuti::formats::io::readers;
use butabuti::prelude::*;
use proptest::prelude::*;
use std::io::Cursor;

// Helper to create random bytes
fn random_bytes() -> impl Strategy<Value = Vec<u8>> {
    prop::collection::vec(any::<u8>(), 0..10000)
}

// Helper to create semi-valid DST header
fn dst_like_bytes() -> impl Strategy<Value = Vec<u8>> {
    prop::collection::vec(any::<u8>(), 512..2048).prop_map(|mut data| {
        // Set first bytes to look like a DST file
        if data.len() >= 512 {
            data[0] = 0x4C; // 'L'
            data[1] = 0x41; // 'A'
        }
        data
    })
}

// Test DST reader with random data
proptest! {
    #[test]
    fn fuzz_dst_reader_random(data in random_bytes()) {
        let mut cursor = Cursor::new(&data);

        // Should not panic, even with random data
        let _ = readers::dst::read(&mut cursor, None);
    }

    #[test]
    fn fuzz_dst_reader_semi_valid(data in dst_like_bytes()) {
        let mut cursor = Cursor::new(&data);

        // Should not panic with semi-valid headers
        let _ = readers::dst::read(&mut cursor, None);
    }
}

// Helper to create JSON-like bytes
fn json_like_bytes() -> impl Strategy<Value = Vec<u8>> {
    prop::collection::vec(any::<u8>(), 10..5000).prop_map(|mut data| {
        // Ensure it starts with '{' to look like JSON
        if !data.is_empty() {
            data[0] = b'{';
        }
        data
    })
}

proptest! {
    #[test]
    fn fuzz_json_reader_random(data in random_bytes()) {
        let mut cursor = Cursor::new(&data);

        // Should not panic, may return error
        let _ = readers::json::read(&mut cursor);
    }

    #[test]
    fn fuzz_json_reader_semi_valid(data in json_like_bytes()) {
        let mut cursor = Cursor::new(&data);

        // Should handle malformed JSON gracefully
        let _ = readers::json::read(&mut cursor);
    }
}

// Helper to create CSV-like bytes
fn csv_like_bytes() -> impl Strategy<Value = String> {
    prop::collection::vec((0i32..1000, 0i32..1000, 0u32..255), 0..100).prop_map(|rows| {
        rows.iter()
            .map(|(x, y, cmd)| format!("{},{},{}\n", x, y, cmd))
            .collect::<String>()
    })
}

proptest! {
    #[test]
    fn fuzz_csv_reader_random(data in random_bytes()) {
        let mut pattern = EmbPattern::new();
        let mut cursor = Cursor::new(&data);

        // Should not panic with random data
        let _ = readers::csv::read(&mut cursor, &mut pattern);
    }

    #[test]
    fn fuzz_csv_reader_semi_valid(data in csv_like_bytes()) {
        let mut pattern = EmbPattern::new();
        let mut cursor = Cursor::new(data.as_bytes());

        // Should parse valid CSV structure
        let _ = readers::csv::read(&mut cursor, &mut pattern);
    }
}

// Helper to create EXP-like bytes
fn exp_like_bytes() -> impl Strategy<Value = Vec<u8>> {
    prop::collection::vec(any::<u8>(), 100..2000)
}

proptest! {
    #[test]
    fn fuzz_exp_reader_random(data in random_bytes()) {
        let mut cursor = Cursor::new(&data);

        // Should not panic
        let _ = readers::exp::read(&mut cursor);
    }

    #[test]
    fn fuzz_exp_reader_semi_valid(data in exp_like_bytes()) {
        let mut cursor = Cursor::new(&data);

        // Should handle gracefully
        let _ = readers::exp::read(&mut cursor);
    }
}

// Helper to create JEF-like bytes
fn jef_like_bytes() -> impl Strategy<Value = Vec<u8>> {
    prop::collection::vec(any::<u8>(), 200..3000).prop_map(|mut data| {
        // Set JEF signature
        if data.len() >= 4 {
            data[0] = 0x00;
            data[1] = 0x00;
            data[2] = 0x00;
            data[3] = 0x00;
        }
        data
    })
}

proptest! {
    #[test]
    fn fuzz_jef_reader_random(data in random_bytes()) {
        let mut cursor = Cursor::new(data);

        // Should not panic
        let _ = readers::jef::read(&mut cursor, None);
    }

    #[test]
    fn fuzz_jef_reader_semi_valid(data in jef_like_bytes()) {
        let mut cursor = Cursor::new(&data);

        // Should handle semi-valid JEF data
        let _ = readers::jef::read(&mut cursor, None);
    }
}

// Test thread parsing with random strings
proptest! {
    #[test]
    fn fuzz_thread_from_string(s in ".*") {
        // Should not panic with any string
        let _ = EmbThread::from_string(&s);
    }

    #[test]
    fn fuzz_thread_color_values(color in any::<u32>()) {
        // Should handle any u32 color value
        let thread = EmbThread::new(color);

        // Verify it doesn't panic on color operations
        let _ = thread.to_srgb();
        let _ = thread.to_lab();
        let _ = thread.to_hsl();
    }
}

// Test pattern operations with random coordinates
proptest! {
    #[test]
    fn fuzz_pattern_coordinates(
        x in -10000.0f64..10000.0,
        y in -10000.0f64..10000.0
    ) {
        let mut pattern = EmbPattern::new();
        pattern.add_thread(EmbThread::new(0xFF0000));

        // Should handle any finite coordinates
        if x.is_finite() && y.is_finite() {
            pattern.stitch_abs(x, y);

            // Verify operations don't panic
            let _ = pattern.bounds();
            let _ = pattern.calculate_statistics(800.0);
        }
    }

    #[test]
    fn fuzz_pattern_transformations(
        angle in -360.0f64..360.0,
        scale_x in 0.1f64..10.0,
        scale_y in 0.1f64..10.0
    ) {
        let mut pattern = EmbPattern::new();
        pattern.add_thread(EmbThread::new(0xFF0000));
        pattern.stitch_abs(100.0, 100.0);
        pattern.stitch_abs(200.0, 200.0);

        // Test transformations don't panic
        let mut p1 = pattern.clone();
        let _ = p1.rotate(angle);

        let mut p2 = pattern.clone();
        let _ = p2.scale(scale_x, scale_y);

        let mut p3 = pattern.clone();
        p3.flip_horizontal();

        let mut p4 = pattern.clone();
        p4.flip_vertical();
    }
}

// Test COL format with random data
proptest! {
    #[test]
    fn fuzz_col_reader_random(data in random_bytes()) {
        let mut pattern = EmbPattern::new();
        let mut cursor = Cursor::new(&data);

        // Should not panic
        let _ = readers::col::read(&mut cursor, &mut pattern);
    }
}

// Test EDR format with random data
proptest! {
    #[test]
    fn fuzz_edr_reader_random(data in random_bytes()) {
        let mut pattern = EmbPattern::new();
        let mut cursor = Cursor::new(&data);

        // Should not panic
        let _ = readers::edr::read(&mut cursor, &mut pattern);
    }
}

// Test INF format with random data
proptest! {
    #[test]
    fn fuzz_inf_reader_random(data in random_bytes()) {
        let mut pattern = EmbPattern::new();
        let mut cursor = Cursor::new(&data);

        // Should not panic
        let _ = readers::inf::read(&mut cursor, &mut pattern);
    }
}

// Test TBF format with random data
proptest! {
    #[test]
    fn fuzz_tbf_reader_random(data in random_bytes()) {
        let mut pattern = EmbPattern::new();
        let mut cursor = Cursor::new(&data);

        // Should not panic
        let _ = readers::tbf::read(&mut cursor, &mut pattern);
    }
}

// Test U01 format with random data
proptest! {
    #[test]
    fn fuzz_u01_reader_random(data in random_bytes()) {
        let mut pattern = EmbPattern::new();
        let mut cursor = Cursor::new(&data);

        // Should not panic
        let _ = readers::u01::read(&mut cursor, &mut pattern);
    }
}

// Test XXX format with random data
proptest! {
    #[test]
    fn fuzz_xxx_reader_random(data in random_bytes()) {
        let mut pattern = EmbPattern::new();
        let mut cursor = Cursor::new(&data);

        // Should not panic
        let _ = readers::xxx::read(&mut cursor, &mut pattern);
    }
}

// Test GCODE format with random data
proptest! {
    #[test]
    fn fuzz_gcode_reader_random(data in random_bytes()) {
        let mut pattern = EmbPattern::new();
        let mut cursor = Cursor::new(&data);

        // Should not panic
        let _ = readers::gcode::read(&mut cursor, &mut pattern);
    }
}

// Test stitch splitting with random parameters
proptest! {
    #[test]
    fn fuzz_stitch_splitting(max_length in 1.0f64..1000.0) {
        let mut pattern = EmbPattern::new();
        pattern.add_thread(EmbThread::new(0xFF0000));

        // Add some long stitches
        pattern.stitch_abs(0.0, 0.0);
        pattern.stitch_abs(1000.0, 1000.0);
        pattern.stitch_abs(2000.0, 0.0);

        // Should not panic with any positive max length
        let _ = pattern.split_long_stitches(max_length);
    }
}

// Test color distance calculations
proptest! {
    #[test]
    fn fuzz_color_distance(color1 in any::<u32>(), color2 in any::<u32>()) {
        let thread1 = EmbThread::new(color1);
        let thread2 = EmbThread::new(color2);

        // Should not panic
        let _ = thread1.color_distance(color2);
        let _ = thread1.delta_e(&thread2);
    }
}

// Test pattern validation
proptest! {
    #[test]
    fn fuzz_pattern_validation(stitch_count in 0usize..1000) {
        let mut pattern = EmbPattern::new();
        pattern.add_thread(EmbThread::new(0xFF0000));

        for i in 0..stitch_count {
            pattern.stitch_abs((i as f64) * 10.0, (i as f64) * 10.0);
        }

        // Validation should not panic
        let _ = pattern.validate();
    }
}
