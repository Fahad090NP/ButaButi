/// Butabuti CLI - Command-line tool for embroidery file conversion and analysis
///
/// Usage:
///   butabuti convert <input> <output>  - Convert between formats
///   butabuti info <file>                - Display pattern information
///   butabuti validate <file>            - Validate pattern file
///   butabuti batch <input_dir> <output_dir> <format> - Batch convert files
use butabuti::formats::registry::FormatRegistry;
use butabuti::prelude::*;
use butabuti::utils::batch::BatchConverter;
use std::env;
use std::fs::File;
use std::process;

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        print_usage();
        process::exit(1);
    }

    let command = &args[1];

    let result = match command.as_str() {
        "convert" => {
            if args.len() < 4 {
                eprintln!("Error: convert requires <input> and <output> arguments");
                print_usage();
                process::exit(1);
            }
            convert_file(&args[2], &args[3])
        },
        "info" => {
            if args.len() < 3 {
                eprintln!("Error: info requires <file> argument");
                print_usage();
                process::exit(1);
            }
            show_info(&args[2])
        },
        "validate" => {
            if args.len() < 3 {
                eprintln!("Error: validate requires <file> argument");
                print_usage();
                process::exit(1);
            }
            validate_file(&args[2])
        },
        "batch" => {
            if args.len() < 5 {
                eprintln!("Error: batch requires <input_dir> <output_dir> <format> arguments");
                print_usage();
                process::exit(1);
            }
            batch_convert(&args[2], &args[3], &args[4])
        },
        "list-formats" => list_formats(),
        "version" => {
            println!("Butabuti v{}", env!("CARGO_PKG_VERSION"));
            Ok(())
        },
        "help" | "--help" | "-h" => {
            print_usage();
            Ok(())
        },
        _ => {
            eprintln!("Error: Unknown command '{}'", command);
            print_usage();
            process::exit(1);
        },
    };

    if let Err(e) = result {
        eprintln!("Error: {}", e);
        process::exit(1);
    }
}

fn print_usage() {
    println!(
        "Butabuti - Embroidery File Converter v{}",
        env!("CARGO_PKG_VERSION")
    );
    println!();
    println!("USAGE:");
    println!("    butabuti <COMMAND> [ARGS]");
    println!();
    println!("COMMANDS:");
    println!("    convert <input> <output>              Convert embroidery file between formats");
    println!(
        "    info <file>                           Display pattern information and statistics"
    );
    println!("    validate <file>                       Validate embroidery file");
    println!("    batch <input_dir> <output_dir> <fmt>  Batch convert directory of files");
    println!("    list-formats                          List all supported formats");
    println!("    version                               Show version information");
    println!("    help                                  Show this help message");
    println!();
    println!("EXAMPLES:");
    println!("    butabuti convert input.dst output.pes");
    println!("    butabuti info design.dst");
    println!("    butabuti validate pattern.pes");
    println!("    butabuti batch ./input ./output pes");
    println!("    butabuti list-formats");
}

fn convert_file(input: &str, output: &str) -> Result<()> {
    println!("Converting {} -> {}", input, output);

    let registry = FormatRegistry::new();

    // Read input file
    let mut input_file = File::open(input).map_err(Error::Io)?;

    let input_format = registry
        .get_format_from_path(input)
        .ok_or_else(|| Error::UnsupportedFormat(format!("Unknown input format: {}", input)))?;

    if !input_format.can_read {
        return Err(Error::UnsupportedFormat(format!(
            "Format '{}' does not support reading",
            input_format.name
        )));
    }

    println!("  Reading {} format...", input_format.name.to_uppercase());
    let pattern = registry.read_pattern(&mut input_file, input_format.name)?;

    // Write output file
    let output_format = registry
        .get_format_from_path(output)
        .ok_or_else(|| Error::UnsupportedFormat(format!("Unknown output format: {}", output)))?;

    if !output_format.can_write {
        return Err(Error::UnsupportedFormat(format!(
            "Format '{}' does not support writing",
            output_format.name
        )));
    }

    println!("  Writing {} format...", output_format.name.to_uppercase());
    let mut output_file = File::create(output).map_err(Error::Io)?;

    registry.write_pattern(&pattern, &mut output_file, output_format.name)?;

    println!("✓ Conversion complete!");
    println!("  Stitches: {}", pattern.count_stitches());
    println!("  Colors: {}", pattern.threads().len());

    Ok(())
}

fn show_info(filename: &str) -> Result<()> {
    println!("Pattern Information: {}", filename);
    println!("{}", "=".repeat(60));

    let registry = FormatRegistry::new();
    let mut file = File::open(filename).map_err(Error::Io)?;

    let format = registry
        .get_format_from_path(filename)
        .ok_or_else(|| Error::UnsupportedFormat(format!("Unknown format: {}", filename)))?;

    if !format.can_read {
        return Err(Error::UnsupportedFormat(format!(
            "Format '{}' does not support reading",
            format.name
        )));
    }

    let pattern = registry.read_pattern(&mut file, format.name)?;

    // Basic info
    println!("\nBasic Information:");
    println!("  Format: {}", format.name.to_uppercase());
    println!("  Stitches: {}", pattern.count_stitches());
    println!("  Colors: {}", pattern.threads().len());
    println!("  Jumps: {}", pattern.count_jumps());
    println!("  Trims: {}", pattern.count_trims());
    println!("  Color Changes: {}", pattern.count_color_changes());

    // Bounds
    let (min_x, min_y, max_x, max_y) = pattern.bounds();
    let width_mm = (max_x - min_x) / 10.0;
    let height_mm = (max_y - min_y) / 10.0;

    println!("\nDimensions:");
    println!("  Width: {:.2} mm ({:.1} units)", width_mm, max_x - min_x);
    println!("  Height: {:.2} mm ({:.1} units)", height_mm, max_y - min_y);
    println!(
        "  Bounds: ({:.1}, {:.1}) to ({:.1}, {:.1})",
        min_x, min_y, max_x, max_y
    );

    // Stitch statistics
    if pattern.count_stitches() > 0 {
        println!("\nStitch Statistics:");
        println!(
            "  Total Length: {:.2} mm",
            pattern.total_stitch_length() / 10.0
        );
        println!("  Max Stitch: {:.2} mm", pattern.max_stitch_length() / 10.0);
        println!("  Avg Stitch: {:.2} mm", pattern.avg_stitch_length() / 10.0);
    }

    // Thread colors
    if !pattern.threads().is_empty() {
        println!("\nThread Colors:");
        for (i, thread) in pattern.threads().iter().enumerate() {
            println!(
                "  #{}: {} (#{:02X}{:02X}{:02X})",
                i + 1,
                thread.description.as_deref().unwrap_or("Unknown"),
                thread.red(),
                thread.green(),
                thread.blue()
            );
        }
    }

    println!();
    Ok(())
}

fn validate_file(filename: &str) -> Result<()> {
    println!("Validating: {}", filename);

    let registry = FormatRegistry::new();
    let mut file = File::open(filename).map_err(Error::Io)?;

    let format = registry
        .get_format_from_path(filename)
        .ok_or_else(|| Error::UnsupportedFormat(format!("Unknown format: {}", filename)))?;

    if !format.can_read {
        return Err(Error::UnsupportedFormat(format!(
            "Format '{}' does not support reading",
            format.name
        )));
    }

    match registry.read_pattern(&mut file, format.name) {
        Ok(pattern) => {
            println!("✓ File is valid");

            // Check for common issues
            let mut warnings = Vec::new();

            if pattern.count_stitches() == 0 {
                warnings.push("Pattern contains no stitches");
            }

            if pattern.threads().is_empty() {
                warnings.push("Pattern has no thread colors defined");
            }

            if pattern.max_stitch_length() > 127.0 {
                warnings.push("Pattern contains very long stitches (>12.7mm)");
            }

            if !warnings.is_empty() {
                println!("\nWarnings:");
                for warning in warnings {
                    println!("  ⚠ {}", warning);
                }
            } else {
                println!("  No issues found");
            }

            Ok(())
        },
        Err(e) => {
            println!("✗ Validation failed: {}", e);
            Err(e)
        },
    }
}

fn batch_convert(input_dir: &str, output_dir: &str, target_format: &str) -> Result<()> {
    println!("Batch Converting:");
    println!("  Input: {}", input_dir);
    println!("  Output: {}", output_dir);
    println!("  Format: {}", target_format.to_uppercase());
    println!();

    let registry = FormatRegistry::new();

    // Validate target format
    let format_info = registry
        .get_format(target_format)
        .ok_or_else(|| Error::UnsupportedFormat(format!("Unknown format: {}", target_format)))?;

    if !format_info.can_write {
        return Err(Error::UnsupportedFormat(format!(
            "Format '{}' does not support writing",
            target_format
        )));
    }

    // Get all supported input extensions
    let input_extensions: Vec<&str> = registry
        .readable_formats()
        .iter()
        .flat_map(|f| f.extensions.iter().copied())
        .collect();

    let converter = BatchConverter::new()
        .input_dir(input_dir)
        .output_dir(output_dir)
        .target_format(target_format)
        .input_extensions(&input_extensions)
        .overwrite(true)
        .build();

    let results = converter.convert_all()?;

    println!("\n{}", "=".repeat(60));
    results.print_summary();

    Ok(())
}

fn list_formats() -> Result<()> {
    let registry = FormatRegistry::new();

    println!("Supported Embroidery Formats");
    println!("{}", "=".repeat(60));

    println!("\nReadable & Writable Formats:");
    for format in registry.all_formats() {
        if format.can_read && format.can_write {
            let extensions = format.extensions.join(", ");
            println!(
                "  {} - {} ({})",
                format.name.to_uppercase(),
                format.description,
                extensions
            );
        }
    }

    println!("\nRead-Only Formats:");
    let read_only: Vec<_> = registry
        .all_formats()
        .iter()
        .filter(|f| f.can_read && !f.can_write)
        .collect();

    if read_only.is_empty() {
        println!("  None");
    } else {
        for format in read_only {
            let extensions = format.extensions.join(", ");
            println!(
                "  {} - {} ({})",
                format.name.to_uppercase(),
                format.description,
                extensions
            );
        }
    }

    println!("\nWrite-Only Formats:");
    let write_only: Vec<_> = registry
        .all_formats()
        .iter()
        .filter(|f| !f.can_read && f.can_write)
        .collect();

    if write_only.is_empty() {
        println!("  None");
    } else {
        for format in write_only {
            let extensions = format.extensions.join(", ");
            println!(
                "  {} - {} ({})",
                format.name.to_uppercase(),
                format.description,
                extensions
            );
        }
    }

    println!();
    Ok(())
}
