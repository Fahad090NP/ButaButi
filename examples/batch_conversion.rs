//! Example: Batch conversion of embroidery files
//!
//! This example demonstrates how to convert multiple embroidery files
//! from one format to another in a single operation.

use butabuti::prelude::*;
use std::path::PathBuf;

fn main() -> Result<()> {
    println!("=== Batch Conversion Example ===\n");

    // Example 1: Convert all DST files in a directory to PES format
    println!("Example 1: Converting all DST files to PES format");
    println!("------------------------------------------------");

    let converter = BatchConverter::new()
        .input_dir("tests/DST Files")
        .output_dir("tests/Output/batch_pes")
        .target_format("pes")
        .input_extensions(&["dst"])
        .overwrite(true)
        .parallel(true)
        .build();

    match converter.convert_all() {
        Ok(results) => {
            println!("\n✓ Batch conversion completed!");
            results.print_summary();

            // Show individual results
            println!("\nDetailed Results:");
            for result in results.results() {
                match result {
                    ConversionResult::Success {
                        input,
                        output,
                        duration_ms,
                        file_size,
                    } => {
                        println!(
                            "  ✓ {} -> {} ({} KB, {} ms)",
                            input.display(),
                            output.display(),
                            file_size / 1024,
                            duration_ms
                        );
                    }
                    ConversionResult::Failed { input, error, .. } => {
                        println!("  ✗ {} - Error: {}", input.display(), error);
                    }
                    ConversionResult::Skipped { input, reason } => {
                        println!("  ⊘ {} - Skipped: {}", input.display(), reason);
                    }
                }
            }
        }
        Err(e) => {
            eprintln!("Error during batch conversion: {}", e);
            return Err(e);
        }
    }

    println!("\n");

    // Example 2: Convert specific files to multiple formats
    println!("Example 2: Converting specific files to JEF format");
    println!("--------------------------------------------------");

    let specific_files = vec![
        PathBuf::from("tests/DST Files/017.DST"),
        PathBuf::from("tests/DST Files/018.DST"),
    ];

    let converter2 = BatchConverter::new()
        .input_files(&specific_files)
        .output_dir("tests/Output/batch_jef")
        .target_format("jef")
        .overwrite(true)
        .parallel(false) // Sequential processing
        .build();

    match converter2.convert_all() {
        Ok(results) => {
            println!("\n✓ Batch conversion completed!");
            results.print_summary();
        }
        Err(e) => {
            eprintln!("Error: {}", e);
        }
    }

    println!("\n");

    // Example 3: Recursive directory scanning
    println!("Example 3: Recursive directory conversion");
    println!("------------------------------------------");

    let converter3 = BatchConverter::new()
        .input_dir("tests")
        .output_dir("tests/Output/recursive")
        .target_format("exp")
        .input_extensions(&["dst", "pes"])
        .recursive(true)
        .overwrite(true)
        .build();

    match converter3.convert_all() {
        Ok(results) => {
            println!("\n✓ Recursive conversion completed!");
            println!("Success rate: {:.1}%", results.success_rate() * 100.0);
            println!(
                "Processed {} files ({} succeeded, {} failed, {} skipped)",
                results.total_count(),
                results.success_count(),
                results.failed_count(),
                results.skipped_count()
            );
        }
        Err(e) => {
            eprintln!("Error: {}", e);
        }
    }

    Ok(())
}
