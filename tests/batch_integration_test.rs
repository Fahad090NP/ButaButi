//! Integration test for batch conversion features
//!
//! This test demonstrates the complete workflow of batch conversion
//! and multi-format export capabilities.

use butabuti::prelude::*;
use std::fs;
use std::path::PathBuf;

#[test]
fn test_batch_conversion_workflow() {
    // Create a test pattern
    let mut pattern = EmbPattern::new();
    pattern.add_thread(EmbThread::from_string("red").unwrap());
    pattern.stitch(100.0, 0.0);
    pattern.stitch(0.0, 100.0);
    pattern.stitch(-100.0, 0.0);
    pattern.stitch(0.0, -100.0);
    pattern.trim();
    pattern.end();

    // Create temporary test directory
    let test_dir = PathBuf::from("test_batch_temp");
    let input_dir = test_dir.join("input");
    let output_dir = test_dir.join("output");

    fs::create_dir_all(&input_dir).unwrap();
    fs::create_dir_all(&output_dir).unwrap();

    // Write test files
    let file1 = input_dir.join("test1.json");
    let file2 = input_dir.join("test2.json");

    let mut writer1 = fs::File::create(&file1).unwrap();
    let mut writer2 = fs::File::create(&file2).unwrap();

    butabuti::formats::io::writers::json::write(&mut writer1, &pattern).unwrap();
    butabuti::formats::io::writers::json::write(&mut writer2, &pattern).unwrap();

    // Test batch converter
    let converter = BatchConverter::new()
        .input_dir(&input_dir)
        .output_dir(&output_dir)
        .target_format("csv")
        .input_extensions(&["json"])
        .overwrite(true)
        .parallel(false) // Sequential for deterministic testing
        .build();

    let results = converter.convert_all().unwrap();

    // Verify results
    assert_eq!(results.total_count(), 2);
    assert_eq!(results.success_count(), 2);
    assert_eq!(results.failed_count(), 0);
    assert_eq!(results.success_rate(), 1.0);
    assert!(results.total_output_size() > 0);

    // Verify output files exist
    assert!(output_dir.join("test1.csv").exists());
    assert!(output_dir.join("test2.csv").exists());

    // Cleanup
    fs::remove_dir_all(&test_dir).unwrap();
}

#[test]
fn test_multi_format_export_workflow() {
    // Create a test pattern
    let mut pattern = EmbPattern::new();
    pattern.add_thread(EmbThread::from_string("blue").unwrap());
    pattern.stitch(50.0, 0.0);
    pattern.stitch(0.0, 50.0);
    pattern.end();

    // Create temporary output directory
    let output_dir = PathBuf::from("test_export_temp");
    fs::create_dir_all(&output_dir).unwrap();

    // Test multi-format exporter
    let exporter = MultiFormatExporter::new()
        .output_dir(&output_dir)
        .base_name("test_pattern")
        .formats(&["json", "csv", "txt"])
        .overwrite(true)
        .build();

    let results = exporter.export(&pattern).unwrap();

    // Verify results
    assert_eq!(results.total_count(), 3);
    assert_eq!(results.success_count(), 3);
    assert_eq!(results.failed_count(), 0);
    assert_eq!(results.success_rate(), 1.0);

    // Verify output files exist
    assert!(output_dir.join("test_pattern.json").exists());
    assert!(output_dir.join("test_pattern.csv").exists());
    assert!(output_dir.join("test_pattern.txt").exists());

    // Cleanup
    fs::remove_dir_all(&output_dir).unwrap();
}

#[test]
fn test_conversion_results_statistics() {
    let mut results = ConversionResults::new();

    // Add various results
    results.add(ConversionResult::Success {
        input: PathBuf::from("file1.dst"),
        output: PathBuf::from("file1.pes"),
        duration_ms: 100,
        file_size: 2048,
    });

    results.add(ConversionResult::Success {
        input: PathBuf::from("file2.dst"),
        output: PathBuf::from("file2.pes"),
        duration_ms: 150,
        file_size: 3072,
    });

    results.add(ConversionResult::Failed {
        input: PathBuf::from("bad.dst"),
        error: "Parse error".to_string(),
        duration_ms: 50,
    });

    results.add(ConversionResult::Skipped {
        input: PathBuf::from("exists.dst"),
        reason: "Already exists".to_string(),
    });

    results.set_total_duration(300);

    // Verify statistics
    assert_eq!(results.total_count(), 4);
    assert_eq!(results.success_count(), 2);
    assert_eq!(results.failed_count(), 1);
    assert_eq!(results.skipped_count(), 1);
    assert_eq!(results.success_rate(), 0.5);
    assert_eq!(results.total_output_size(), 5120);
    assert_eq!(results.total_duration_ms(), 300);
}

#[test]
fn test_builder_patterns() {
    // Test BatchConverter builder - just verify it builds
    let _converter = BatchConverter::new()
        .input_dir("./test")
        .output_dir("./output")
        .target_format("pes")
        .input_extensions(&["dst", "jef"])
        .overwrite(true)
        .recursive(true)
        .parallel(false)
        .build();

    // Test MultiFormatExporter builder - just verify it builds
    let _exporter = MultiFormatExporter::new()
        .output_dir("./exports")
        .base_name("design")
        .formats(&["dst", "pes", "jef"])
        .overwrite(true)
        .build();

    // If we get here, both builders worked correctly without panicking
}
