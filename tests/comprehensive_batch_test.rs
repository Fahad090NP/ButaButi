//! Comprehensive batch conversion tests for ButaButi embroidery library//! Comprehensive batch conversion tests for ButaButi embroidery library

//!//!

//! This test suite demonstrates ALL batch conversion capabilities://! This test suite demonstrates ALL batch conversion capabilities:

//! 1. One pattern → Many formats (MultiFormatExporter)//! 1. One file → Many formats (MultiFormatExporter)

//! 2. Many files → One format (BatchConverter with target_format)//! 2. Many files → Many formats (BatchConverter + MultiFormatExporter)

//! 3. Many files → Many formats (Loop with MultiFormatExporter)//! 3. Many files → One format (BatchConverter with target_format)



use butabuti::prelude::*;use butabuti::prelude::*;

use butabuti::utils::batch::{BatchConverter, MultiFormatExporter, ConversionResult};use butabuti::utils::batch::{BatchConverter, ConversionResult, MultiFormatExporter};

use std::fs;use std::fs;

use std::path::{Path, PathBuf};use std::path::{Path, PathBuf};



/// Helper: Create a test pattern with specified number of stitches/// Helper: Create a test pattern with specified number of stitches

fn create_test_pattern(_name: &str, stitch_count: usize) -> EmbPattern {fn create_test_pattern(name: &str, stitch_count: usize) -> EmbPattern {

    let mut pattern = EmbPattern::new();    let mut pattern = EmbPattern::new();

    

    // Add threads    // Add threads

    pattern.add_thread(EmbThread::from_string("red").unwrap());    pattern.add_thread(EmbThread::from_string("red").unwrap());

    pattern.add_thread(EmbThread::from_string("blue").unwrap());    pattern.add_thread(EmbThread::from_string("blue").unwrap());

    pattern.add_thread(EmbThread::from_string("green").unwrap());    pattern.add_thread(EmbThread::from_string("green").unwrap());

    

    // Create stitches in a circular pattern    // Create stitches in a pattern

    for i in 0..stitch_count {    for i in 0..stitch_count {

        let angle = (i as f64 * 2.0 * std::f64::consts::PI) / stitch_count as f64;        let angle = (i as f64 * 2.0 * std::f64::consts::PI) / stitch_count as f64;

        let x = (angle.cos() * 500.0) as f64;        let x = (angle.cos() * 500.0) as f64;

        let y = (angle.sin() * 500.0) as f64;        let y = (angle.sin() * 500.0) as f64;

        pattern.stitch_abs(x, y);        pattern.stitch_abs(x, y);

        

        // Add color changes periodically        // Add color changes periodically

        if i > 0 && i % (stitch_count / 3) == 0 {        if i > 0 && i % (stitch_count / 3) == 0 {

            pattern.color_change(0.0, 0.0);            pattern.color_change(0.0, 0.0);

        }        }

    }    }

    

    pattern.trim();    pattern.trim();

    pattern.end();    pattern.end();

    pattern    pattern

}}



/// Helper: Setup test directory with sample files/// Helper: Setup test directory with sample files

fn setup_test_files(dir: &Path, file_count: usize) -> Vec<PathBuf> {fn setup_test_files(dir: &Path, file_count: usize) -> Vec<PathBuf> {

    fs::create_dir_all(dir).unwrap();    fs::create_dir_all(dir).unwrap();

    

    let mut files = Vec::new();    let mut files = Vec::new();

    for i in 0..file_count {    for i in 0..file_count {

        let pattern = create_test_pattern(&format!("pattern_{}", i), 50 + i * 10);        let pattern = create_test_pattern(&format!("pattern_{}", i), 50 + i * 10);

        let file_path = dir.join(format!("test_pattern_{}.json", i));        let file_path = dir.join(format!("test_pattern_{}.json", i));

        

        let mut writer = fs::File::create(&file_path).unwrap();        let mut writer = fs::File::create(&file_path).unwrap();

        butabuti::formats::io::writers::json::write(&mut writer, &pattern).unwrap();        butabuti::formats::io::writers::json::write(&mut writer, &pattern).unwrap();

        files.push(file_path);        files.push(file_path);

    }    }

    

    files    files

}}



/// Helper: Cleanup test directory/// Helper: Cleanup test directory

fn cleanup(dir: &Path) {fn cleanup(dir: &Path) {

    if dir.exists() {    if dir.exists() {

        fs::remove_dir_all(dir).ok();        fs::remove_dir_all(dir).ok();

    }    }

}}



/// Test 1: Convert ONE pattern into MANY formats using MultiFormatExporter/// Test 1: Convert ONE file into MANY formats using MultiFormatExporter

/// This demonstrates: 1 → Many#[test]

#[test]fn test_one_file_to_many_formats() {

fn test_one_to_many() {    println!("\n╔════════════════════════════════════════════════════════╗");

    println!("\n╔════════════════════════════════════════════════════════╗");    println!("║  TEST 1: ONE FILE → MANY FORMATS                      ║");

    println!("║  TEST 1: ONE PATTERN → MANY FORMATS                   ║");    println!("║  Using: MultiFormatExporter                           ║");

    println!("║  Using: MultiFormatExporter                           ║");    println!("╚════════════════════════════════════════════════════════╝\n");

    println!("╚════════════════════════════════════════════════════════╝\n");

    let test_dir = PathBuf::from("test_one_to_many_temp");

    let test_dir = PathBuf::from("test_one_to_many_temp");    cleanup(&test_dir);

    cleanup(&test_dir);

        // Create ONE test pattern

    // Create ONE test pattern    let pattern = create_test_pattern("single_test", 100);

    let pattern = create_test_pattern("single_test", 100);

    println!("✓ Created test pattern");

    println!("✓ Created test pattern");    println!("  - Stitches: {}", pattern.count_stitches());

    println!("  - Stitches: {}", pattern.count_stitches());    println!("  - Color changes: {}", pattern.count_color_changes());

    println!("  - Threads: {}", pattern.threads().len());    println!("  - Threads: {}", pattern.threads().len());



    // ALL supported output formats (17 formats)    // ALL supported output formats (17 formats)

    let all_formats = vec![    let all_formats = vec![

        "dst", "pes", "exp", "jef", "vp3", "xxx", "u01", "pec",        "dst", "pes", "exp", "jef", "vp3", "xxx", "u01", "pec", "tbf", "col", "edr", "inf", "json",

        "tbf", "col", "edr", "inf", "json", "csv", "txt", "svg", "gcode"        "csv", "txt", "svg", "gcode",

    ];    ];



    println!("\n🔄 Exporting ONE pattern to {} formats...", all_formats.len());    println!("\n🔄 Exporting to {} formats...", all_formats.len());



    let output_dir = test_dir.join("output");    // Use MultiFormatExporter to export ONE pattern to MANY formats

    let exporter = MultiFormatExporter::new()    let output_dir = test_dir.join("output");

        .output_dir(&output_dir)    let exporter = MultiFormatExporter::new()

        .base_name("exported")        .output_dir(&output_dir)

        .formats(&all_formats)        .base_name("exported_pattern")

        .overwrite(true)        .formats(&all_formats)

        .build();        .overwrite(true)

        .build();

    match exporter.export(&pattern) {

        Ok(results) => {    match exporter.export(&pattern) {

            println!("\n✅ Export completed!");        Ok(results) => {

            results.print_summary();            println!("\n✅ Multi-format export completed!");

            results.print_summary();

            // Verify

            assert_eq!(results.total_count(), all_formats.len());            println!("\n📁 Generated files:");

            assert!(results.success_rate() > 0.85,             let mut format_table = Vec::new();

                "Success rate: {:.1}%", results.success_rate() * 100.0);            for result in results.results() {

                            if let ConversionResult::Success {

            println!("\n✅ CONFIRMED: ONE → MANY ✓");                    output,

        }                    file_size,

        Err(e) => panic!("❌ Export failed: {}", e),                    duration_ms,

    }                    ..

                    } = result

    cleanup(&test_dir);                {

}                    let ext = output.extension().unwrap().to_str().unwrap().to_uppercase();

                    format_table.push((ext, file_size / 1024, duration_ms));

/// Test 2: Convert MANY files into ONE format using BatchConverter                }

/// This demonstrates: Many → 1            }

#[test]

fn test_many_to_one() {            // Sort by format name for consistent output

    println!("\n╔════════════════════════════════════════════════════════╗");            format_table.sort_by(|a, b| a.0.cmp(&b.0));

    println!("║  TEST 2: MANY FILES → ONE FORMAT                      ║");            for (ext, kb, ms) in format_table {

    println!("║  Using: BatchConverter with target_format             ║");                println!("  ✓ {:6} | {:6} KB | {:4} ms", ext, kb, ms);

    println!("╚════════════════════════════════════════════════════════╝\n");            }



    let test_dir = PathBuf::from("test_many_to_one_temp");            // Verify all formats were created

    cleanup(&test_dir);            assert!(

                    results.success_rate() > 0.90,

    let input_dir = test_dir.join("input");                "Success rate too low: {:.1}% (expected >90%)",

                    results.success_rate() * 100.0

    // Create MANY test files            );

    let files = setup_test_files(&input_dir, 5);            assert_eq!(

    println!("📂 Created {} test files", files.len());                results.total_count(),

                all_formats.len(),

    // Convert ALL files to PES format                "Should attempt all {} formats",

    println!("🎯 Converting to PES format...\n");                all_formats.len()

    let output_pes = test_dir.join("output_pes");            );

    let converter = BatchConverter::new()        }

        .input_dir(&input_dir)        Err(e) => panic!("❌ Export failed: {}", e),

        .output_dir(&output_pes)    }

        .target_format("pes")  // ALL → ONE

        .input_extensions(&["json"])    cleanup(&test_dir);

        .overwrite(true)}

        .parallel(true)

        .build();/// Test 2: Convert MANY files into ONE format using BatchConverter

#[test]

    match converter.convert_all() {fn test_many_files_to_one_format() {

        Ok(results) => {    println!("\n╔════════════════════════════════════════════════════════╗");

            println!("✅ PES conversion completed!");    println!("║  TEST 2: MANY FILES → ONE FORMAT                      ║");

            results.print_summary();    println!("║  Using: BatchConverter with target_format             ║");

            assert_eq!(results.total_count(), files.len());    println!("╚════════════════════════════════════════════════════════╝\n");

            assert!(results.success_rate() > 0.9);

        }    let test_dir = Path::new("tests/DST Files");

        Err(e) => panic!("❌ Failed: {}", e),

    }    if !test_dir.exists() {

        println!("⚠️  Test directory not found, skipping test");

    // Convert to another format        return;

    println!("\n🎯 Converting to JEF format...\n");    }

    let output_jef = test_dir.join("output_jef");

    let converter2 = BatchConverter::new()    println!("📂 Source: {}", test_dir.display());

        .input_dir(&input_dir)    println!("🎯 Target format: PES");

        .output_dir(&output_jef)

        .target_format("jef")  // ALL → ONE (different)    // Convert ALL DST files to PES format

        .input_extensions(&["json"])    let converter = BatchConverter::new()

        .overwrite(true)        .input_dir(test_dir)

        .parallel(true)        .output_dir("tests/Output/many_to_one_pes")

        .build();        .target_format("pes") // ALL files → ONE format

        .input_extensions(&["dst"])

    match converter2.convert_all() {        .overwrite(true)

        Ok(results) => {        .parallel(true)

            println!("✅ JEF conversion completed!");        .build();

            results.print_summary();

                match converter.convert_all() {

            println!("\n📊 Summary:");        Ok(results) => {

            println!("  Input files: {}", files.len());            println!("\n✅ Batch conversion completed!");

            println!("  Output formats: 2 (PES, JEF)");            results.print_summary();

            println!("  Total outputs: {}", files.len() * 2);

                        println!("\n📊 Conversion details:");

            println!("\n✅ CONFIRMED: MANY → ONE ✓");            for result in results.results() {

        }                match result {

        Err(e) => panic!("❌ Failed: {}", e),                    ConversionResult::Success {

    }                        input,

                            output,

    cleanup(&test_dir);                        duration_ms,

}                        file_size,

                    } => {

/// Test 3: Convert MANY files into MANY formats                        println!(

/// This demonstrates: Many → Many                            "  ✓ {} → {} ({} KB, {} ms)",

#[test]                            input.file_name().unwrap().to_str().unwrap(),

fn test_many_to_many() {                            output.file_name().unwrap().to_str().unwrap(),

    println!("\n╔════════════════════════════════════════════════════════╗");                            file_size / 1024,

    println!("║  TEST 3: MANY FILES → MANY FORMATS                    ║");                            duration_ms

    println!("║  Using: MultiFormatExporter in loop                   ║");                        );

    println!("╚════════════════════════════════════════════════════════╝\n");                    }

                    ConversionResult::Failed { input, error, .. } => {

    let test_dir = PathBuf::from("test_many_to_many_temp");                        println!("  ✗ {} - Error: {}", input.display(), error);

    cleanup(&test_dir);                    }

                        ConversionResult::Skipped { input, reason } => {

    let input_dir = test_dir.join("input");                        println!("  ⊘ {} - Skipped: {}", input.display(), reason);

                        }

    // Create MANY files                }

    let files = setup_test_files(&input_dir, 4);            }

    println!("📂 Created {} test files", files.len());

                assert!(

    // Target formats                results.success_count() > 0,

    let formats = vec!["pes", "jef", "exp", "json", "csv"];                "No files were converted successfully"

    println!("🎯 Target formats: {:?}\n", formats);            );

        }

    let mut total = 0;        Err(e) => panic!("❌ Batch conversion failed: {}", e),

    let mut success = 0;    }



    // Process EACH file → MANY formats    println!("\n🔄 Now converting same files to JEF format...");

    for (i, file) in files.iter().enumerate() {

        let name = file.file_stem().unwrap().to_str().unwrap();    // Convert ALL DST files to JEF format

        println!("[{}/{}] Processing: {}", i + 1, files.len(), name);    let converter2 = BatchConverter::new()

        .input_dir(test_dir)

        let pattern = {        .output_dir("tests/Output/many_to_one_jef")

            let mut reader = fs::File::open(file).unwrap();        .target_format("jef") // ALL files → ONE format (different)

            butabuti::formats::io::readers::json::read(&mut reader).unwrap()        .input_extensions(&["dst"])

        };        .overwrite(true)

        .parallel(true)

        let exporter = MultiFormatExporter::new()        .build();

            .output_dir(&test_dir.join("output"))

            .base_name(name)    match converter2.convert_all() {

            .formats(&formats)        Ok(results) => {

            .overwrite(true)            println!("✅ JEF conversion completed!");

            .build();            results.print_summary();

            assert!(

        match exporter.export(&pattern) {                results.success_count() > 0,

            Ok(results) => {                "No files were converted to JEF"

                total += results.total_count();            );

                success += results.success_count();        }

                println!("  ✓ {:.0}% success\n", results.success_rate() * 100.0);        Err(e) => panic!("❌ JEF conversion failed: {}", e),

            }    }

            Err(e) => println!("  ✗ Failed: {}\n", e),}

        }

    }/// Test 3: Convert MANY files into MANY formats

#[test]

    println!("╔════════════════════════════════════════════════════════╗");fn test_many_files_to_many_formats() {

    println!("║  SUMMARY: MANY → MANY                                 ║");    println!("\n╔════════════════════════════════════════════════════════╗");

    println!("╚════════════════════════════════════════════════════════╝");    println!("║  TEST 3: MANY FILES → MANY FORMATS                    ║");

    println!("  Conversions: {}/{}", success, total);    println!("║  Using: BatchConverter + MultiFormatExporter loop     ║");

    println!("  Rate: {:.1}%\n", (success as f64 / total as f64) * 100.0);    println!("╚════════════════════════════════════════════════════════╝\n");



    assert_eq!(total, files.len() * formats.len());    let test_dir = Path::new("tests/DST Files");

    assert!(success as f64 / total as f64 > 0.85);

        if !test_dir.exists() {

    println!("✅ CONFIRMED: MANY → MANY ✓");        println!("⚠️  Test directory not found, skipping test");

            return;

    cleanup(&test_dir);    }

}

    // Find all DST files

/// Test 4: All format coverage    let dst_files: Vec<_> = fs::read_dir(test_dir)

#[test]        .expect("Failed to read test directory")

fn test_all_formats() {        .filter_map(|entry| entry.ok())

    println!("\n╔════════════════════════════════════════════════════════╗");        .map(|entry| entry.path())

    println!("║  TEST 4: ALL FORMAT COVERAGE (17 formats)             ║");        .filter(|path| {

    println!("╚════════════════════════════════════════════════════════╝\n");            path.extension()

                .and_then(|ext| ext.to_str())

    let test_dir = PathBuf::from("test_formats_temp");                .map(|ext| ext.eq_ignore_ascii_case("dst"))

    cleanup(&test_dir);                .unwrap_or(false)

            })

    let pattern = create_test_pattern("test", 100);        .collect();



    let all_formats = vec![    if dst_files.is_empty() {

        "dst", "pes", "exp", "jef", "vp3", "xxx", "u01", "pec",        println!("⚠️  No DST files found, skipping test");

        "tbf", "col", "edr", "inf", "json", "csv", "txt", "svg", "gcode"        return;

    ];    }



    let exporter = MultiFormatExporter::new()    println!("📂 Found {} DST files", dst_files.len());

        .output_dir(&test_dir.join("output"))

        .base_name("test")    // Target formats for this test (subset for speed)

        .formats(&all_formats)    let target_formats = vec!["pes", "jef", "exp", "vp3", "json", "csv", "svg"];

        .overwrite(true)    println!("🎯 Target formats: {:?}\n", target_formats);

        .build();

    let mut total_conversions = 0;

    match exporter.export(&pattern) {    let mut successful_conversions = 0;

        Ok(results) => {

            results.print_summary();    // Process EACH file and export to MANY formats

            assert_eq!(results.total_count(), 17);    for (index, dst_file) in dst_files.iter().enumerate().take(3) {

            assert!(results.success_rate() > 0.85);        // Limit to 3 files for test speed

            println!("\n✅ All formats tested!");        let file_name = dst_file.file_stem().unwrap().to_str().unwrap();

        }        println!(

        Err(e) => panic!("Failed: {}", e),            "═══ [{}/{}] Processing: {} ═══",

    }            index + 1,

                dst_files.len().min(3),

    cleanup(&test_dir);            file_name

}        );



/// Test 5: Parallel vs Sequential        // Read the pattern

#[test]        let pattern = match butabuti::formats::io::readers::dst::read(

fn test_parallel_performance() {            &mut std::io::BufReader::new(std::fs::File::open(dst_file).unwrap()),

    println!("\n╔════════════════════════════════════════════════════════╗");            None,

    println!("║  TEST 5: PARALLEL VS SEQUENTIAL                       ║");        ) {

    println!("╚════════════════════════════════════════════════════════╝\n");            Ok(p) => p,

            Err(e) => {

    let test_dir = PathBuf::from("test_perf_temp");                println!("  ❌ Failed to read: {}", e);

    cleanup(&test_dir);                continue;

                }

    let input_dir = test_dir.join("input");        };

    let files = setup_test_files(&input_dir, 5);

        println!("  ✓ Loaded pattern ({} stitches)", pattern.count_stitches());

    // Parallel

    println!("🚀 Parallel:");        // Export to ALL target formats

    let start = std::time::Instant::now();        let exporter = MultiFormatExporter::new()

    let conv1 = BatchConverter::new()            .output_dir("tests/Output/many_to_many")

        .input_dir(&input_dir)            .base_name(file_name)

        .output_dir(&test_dir.join("par"))            .formats(&target_formats)

        .target_format("pes")            .overwrite(true)

        .input_extensions(&["json"])            .build();

        .parallel(true)

        .build();        match exporter.export(&pattern) {

                Ok(results) => {

    let r1 = conv1.convert_all().unwrap();                total_conversions += results.total_count();

    let t1 = start.elapsed();                successful_conversions += results.success_count();

    println!("  {:?} ({:.0}%)\n", t1, r1.success_rate() * 100.0);

                println!(

    // Sequential                    "  ✅ Exported to {} formats ({:.0}% success)",

    println!("🐢 Sequential:");                    results.total_count(),

    let start = std::time::Instant::now();                    results.success_rate() * 100.0

    let conv2 = BatchConverter::new()                );

        .input_dir(&input_dir)

        .output_dir(&test_dir.join("seq"))                // Show what was created

        .target_format("pes")                for result in results.results() {

        .input_extensions(&["json"])                    if let ConversionResult::Success {

        .parallel(false)                        output, file_size, ..

        .build();                    } = result

                        {

    let r2 = conv2.convert_all().unwrap();                        let ext = output.extension().unwrap().to_str().unwrap();

    let t2 = start.elapsed();                        println!("    ✓ {} ({} KB)", ext.to_uppercase(), file_size / 1024);

    println!("  {:?} ({:.0}%)", t2, r2.success_rate() * 100.0);                    }

                }

    assert!(r1.success_rate() > 0.9);            }

    assert!(r2.success_rate() > 0.9);            Err(e) => println!("  ❌ Export failed: {}", e),

            }

    cleanup(&test_dir);        println!();

}    }


    println!("\n╔════════════════════════════════════════════════════════╗");
    println!("║  FINAL SUMMARY: MANY → MANY                           ║");
    println!("╚════════════════════════════════════════════════════════╝");
    println!("  Total conversions: {}", total_conversions);
    println!("  ✓ Successful: {}", successful_conversions);
    println!("  ✗ Failed: {}", total_conversions - successful_conversions);
    println!(
        "  Success rate: {:.1}%\n",
        (successful_conversions as f64 / total_conversions as f64) * 100.0
    );

    assert!(successful_conversions > 0, "No conversions succeeded!");
    assert!(
        successful_conversions as f64 / total_conversions as f64 > 0.8,
        "Success rate too low"
    );
}

/// Test 4: Comprehensive format coverage test
#[test]
fn test_all_format_combinations() {
    println!("\n╔════════════════════════════════════════════════════════╗");
    println!("║  TEST 4: ALL FORMAT COVERAGE                          ║");
    println!("║  Testing all 17 output formats from DST input         ║");
    println!("╚════════════════════════════════════════════════════════╝\n");

    let test_file = Path::new("tests/DST Files/017.DST");

    if !test_file.exists() {
        println!("⚠️  Test file not found, skipping test");
        return;
    }

    let pattern = match butabuti::formats::io::readers::dst::read(
        &mut std::io::BufReader::new(std::fs::File::open(test_file).unwrap()),
        None,
    ) {
        Ok(p) => p,
        Err(e) => {
            println!("❌ Failed to read test file: {}", e);
            return;
        }
    };

    // ALL 17 supported output formats
    let all_formats = vec![
        "dst", "pes", "exp", "jef", "vp3", "xxx", "u01", "pec", "tbf", "col", "edr", "inf", "json",
        "csv", "txt", "svg", "gcode",
    ];

    println!("🎯 Testing {} output formats\n", all_formats.len());

    let exporter = MultiFormatExporter::new()
        .output_dir("tests/Output/format_coverage")
        .base_name("coverage_test")
        .formats(&all_formats)
        .overwrite(true)
        .build();

    match exporter.export(&pattern) {
        Ok(results) => {
            println!("✅ Format coverage test completed!\n");

            // Detailed format breakdown
            println!("📊 Format-by-format results:");
            println!("─────────────────────────────────────────────────");
            for result in results.results() {
                match result {
                    ConversionResult::Success {
                        output,
                        file_size,
                        duration_ms,
                        ..
                    } => {
                        let ext = output.extension().unwrap().to_str().unwrap().to_uppercase();
                        println!(
                            "  ✓ {:6} | {:8} KB | {:4} ms",
                            ext,
                            file_size / 1024,
                            duration_ms
                        );
                    }
                    ConversionResult::Failed { input, error, .. } => {
                        println!(
                            "  ✗ {:6} | Failed: {}",
                            input.extension().unwrap().to_str().unwrap().to_uppercase(),
                            error
                        );
                    }
                    _ => {}
                }
            }

            println!("─────────────────────────────────────────────────");
            results.print_summary();

            // Verify high success rate
            let success_rate = results.success_rate();
            assert!(
                success_rate > 0.85,
                "Format coverage too low: {:.1}% (expected >85%)",
                success_rate * 100.0
            );
        }
        Err(e) => panic!("❌ Format coverage test failed: {}", e),
    }
}

/// Test 5: Performance test with parallel processing
#[test]
fn test_parallel_batch_conversion_performance() {
    println!("\n╔════════════════════════════════════════════════════════╗");
    println!("║  TEST 5: PARALLEL BATCH CONVERSION PERFORMANCE        ║");
    println!("╚════════════════════════════════════════════════════════╝\n");

    let test_dir = Path::new("tests/DST Files");

    if !test_dir.exists() {
        println!("⚠️  Test directory not found, skipping test");
        return;
    }

    println!("⚡ Testing parallel vs sequential conversion...\n");

    // Test with parallel processing
    println!("🚀 Parallel conversion:");
    let start_parallel = std::time::Instant::now();
    let converter_parallel = BatchConverter::new()
        .input_dir(test_dir)
        .output_dir("tests/Output/parallel_test")
        .target_format("pes")
        .input_extensions(&["dst"])
        .overwrite(true)
        .parallel(true) // Enable parallel
        .build();

    let results_parallel = converter_parallel.convert_all().unwrap();
    let duration_parallel = start_parallel.elapsed();

    println!("  ✓ Completed in {:?}", duration_parallel);
    println!(
        "  Success rate: {:.1}%",
        results_parallel.success_rate() * 100.0
    );

    // Test with sequential processing
    println!("\n🐢 Sequential conversion:");
    let start_sequential = std::time::Instant::now();
    let converter_sequential = BatchConverter::new()
        .input_dir(test_dir)
        .output_dir("tests/Output/sequential_test")
        .target_format("pes")
        .input_extensions(&["dst"])
        .overwrite(true)
        .parallel(false) // Disable parallel
        .build();

    let results_sequential = converter_sequential.convert_all().unwrap();
    let duration_sequential = start_sequential.elapsed();

    println!("  ✓ Completed in {:?}", duration_sequential);
    println!(
        "  Success rate: {:.1}%",
        results_sequential.success_rate() * 100.0
    );

    println!("\n📊 Performance comparison:");
    println!("  Parallel:    {:?}", duration_parallel);
    println!("  Sequential:  {:?}", duration_sequential);

    if duration_parallel < duration_sequential {
        let speedup = duration_sequential.as_secs_f64() / duration_parallel.as_secs_f64();
        println!(
            "  ⚡ Speedup: {:.2}x faster with parallel processing",
            speedup
        );
    }
}
