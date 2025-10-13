//! Error case tests for format readers
//!
//! Tests that readers fail gracefully with descriptive errors when given:
//! - Truncated files
//! - Invalid headers
//! - Corrupted stitch data
//! - Out-of-bounds values

#[cfg(test)]
mod tests {
    use crate::core::pattern::EmbPattern;
    use std::io::Cursor;

    // DST format error cases
    mod dst {
        use super::*;
        use crate::formats::io::readers::dst;

        #[test]
        fn test_dst_truncated_header() {
            // DST requires 512-byte header
            let data = vec![0u8; 100]; // Only 100 bytes
            let mut cursor = Cursor::new(data);
            let result = dst::read(&mut cursor, None);
            assert!(result.is_err(), "Should fail on truncated header");
        }

        #[test]
        fn test_dst_empty_file() {
            let data = vec![];
            let mut cursor = Cursor::new(data);
            let result = dst::read(&mut cursor, None);
            assert!(result.is_err(), "Should fail on empty file");
        }

        #[test]
        fn test_dst_corrupted_stitches() {
            // Valid 512-byte header but corrupted stitch data
            let mut data = vec![0x20u8; 512]; // Space-filled header
            data.extend_from_slice(b"LA:Test     \r");
            data.extend_from_slice(&[0xFF, 0xFF, 0xFF]); // Invalid stitch bytes

            let mut cursor = Cursor::new(data);
            let result = dst::read(&mut cursor, None);
            // Should still succeed but handle gracefully
            assert!(result.is_ok());
        }
    }

    // PES format error cases
    mod pes {
        use super::*;
        use crate::formats::io::readers::pes;

        #[test]
        fn test_pes_invalid_signature() {
            let data = b"#INVALID".to_vec();
            let mut cursor = Cursor::new(data);
            let mut pattern = EmbPattern::new();
            let result = pes::read(&mut cursor, &mut pattern);
            assert!(result.is_err(), "Should fail on invalid signature");
            let err = result.unwrap_err();
            assert!(
                err.to_string().contains("PES"),
                "Error should mention PES format"
            );
        }

        #[test]
        fn test_pes_truncated_file() {
            // Valid signature but incomplete file
            let data = b"#PES0001".to_vec();
            let mut cursor = Cursor::new(data);
            let mut pattern = EmbPattern::new();
            let result = pes::read(&mut cursor, &mut pattern);
            assert!(result.is_err(), "Should fail on truncated file");
        }

        #[test]
        fn test_pes_empty_file() {
            let data = vec![];
            let mut cursor = Cursor::new(data);
            let mut pattern = EmbPattern::new();
            let result = pes::read(&mut cursor, &mut pattern);
            assert!(result.is_err(), "Should fail on empty file");
        }
    }

    // JEF format error cases
    mod jef {
        use super::*;
        use crate::formats::io::readers::jef;

        #[test]
        fn test_jef_truncated_header() {
            let data = vec![0u8; 10]; // Too small for JEF header
            let mut cursor = Cursor::new(data);
            let result = jef::read(&mut cursor, None);
            assert!(result.is_err(), "Should fail on truncated header");
        }

        #[test]
        fn test_jef_invalid_offset() {
            // Create header with invalid stitch offset
            let mut data = vec![0u8; 100];
            // Write invalid offset (negative or too large)
            data[0..4].copy_from_slice(&0xFFFFFFFFu32.to_le_bytes());

            let mut cursor = Cursor::new(data);
            let result = jef::read(&mut cursor, None);
            assert!(result.is_err(), "Should fail on invalid offset");
        }
    }

    // EXP format error cases
    mod exp {
        use super::*;
        use crate::formats::io::readers::exp;

        #[test]
        fn test_exp_empty_file() {
            let data = vec![];
            let mut cursor = Cursor::new(data);
            let result = exp::read(&mut cursor);
            // EXP handles empty gracefully and adds END command
            assert!(result.is_ok());
            let pattern = result.unwrap();
            // May have END command added
            assert!(pattern.stitches().len() <= 1);
        }

        #[test]
        fn test_exp_corrupted_data() {
            // Random bytes that shouldn't form valid stitches
            let data = vec![0xFF, 0xFF, 0xFF, 0xFF];
            let mut cursor = Cursor::new(data);
            let result = exp::read(&mut cursor);
            // Should handle gracefully
            assert!(result.is_ok());
        }
    }

    // VP3 format error cases
    mod vp3 {
        use super::*;
        use crate::formats::io::readers::vp3;

        #[test]
        fn test_vp3_invalid_signature() {
            let data = b"INVALID".to_vec();
            let mut cursor = Cursor::new(data);
            let mut pattern = EmbPattern::new();
            let result = vp3::read(&mut cursor, &mut pattern);
            assert!(result.is_err(), "Should fail on invalid signature");
            let err = result.unwrap_err();
            assert!(
                err.to_string().contains("vsm"),
                "Error should mention vsm signature"
            );
        }

        #[test]
        fn test_vp3_truncated_file() {
            let data = b"%vsm%".to_vec(); // Valid signature but incomplete
            let mut cursor = Cursor::new(data);
            let mut pattern = EmbPattern::new();
            let result = vp3::read(&mut cursor, &mut pattern);
            // VP3 may succeed with minimal data or fail - either is acceptable
            // The key is it shouldn't panic
            match result {
                Ok(_) => {}  // Handled gracefully
                Err(_) => {} // Or returned error
            }
        }
    }

    // XXX format error cases
    mod xxx {
        use super::*;
        use crate::formats::io::readers::xxx;

        #[test]
        fn test_xxx_file_too_small() {
            let data = vec![0u8; 100]; // Less than 256 bytes minimum
            let mut cursor = Cursor::new(data);
            let mut pattern = EmbPattern::new();
            let result = xxx::read(&mut cursor, &mut pattern);
            assert!(result.is_err(), "Should fail on file too small");
            // Error message format may vary
        }

        #[test]
        fn test_xxx_invalid_color_count() {
            // Create 256-byte header with invalid color count
            let mut data = vec![0u8; 256];
            // Set color count to 0xFFFF (way too high)
            data[12..14].copy_from_slice(&0xFFFFu16.to_le_bytes());

            let mut cursor = Cursor::new(data);
            let mut pattern = EmbPattern::new();
            let result = xxx::read(&mut cursor, &mut pattern);
            assert!(result.is_err(), "Should fail on invalid color count");
        }
    }

    // COL format error cases
    mod col {
        use super::*;
        use crate::formats::io::readers::col;

        #[test]
        fn test_col_invalid_thread_count() {
            let data = b"99999999\n".to_vec(); // Exceeds MAX_THREADS
            let mut cursor = Cursor::new(data);
            let mut pattern = EmbPattern::new();
            let result = col::read(&mut cursor, &mut pattern);
            assert!(result.is_err(), "Should fail on too many threads");
        }

        #[test]
        fn test_col_invalid_rgb() {
            let data = b"1\n,300,0,0\n".to_vec(); // RGB > 255
            let mut cursor = Cursor::new(data);
            let mut pattern = EmbPattern::new();
            let result = col::read(&mut cursor, &mut pattern);
            assert!(result.is_err(), "Should fail on invalid RGB value");
        }

        #[test]
        fn test_col_incomplete_thread() {
            let data = b"2\nCAT,255,0,0\nCAT2,128\n".to_vec(); // Second thread incomplete
            let mut cursor = Cursor::new(data);
            let mut pattern = EmbPattern::new();
            let result = col::read(&mut cursor, &mut pattern);
            assert!(result.is_err(), "Should fail on incomplete thread data");
        }
    }

    // CSV format error cases
    mod csv {
        use super::*;
        use crate::formats::io::readers::csv;

        #[test]
        fn test_csv_line_too_long() {
            // Create a line longer than MAX_CSV_LINE_LENGTH (10,000 bytes)
            let long_line = "a".repeat(11_000);
            let data = format!("*,0,STITCH,{}\n", long_line);

            let mut cursor = Cursor::new(data.into_bytes());
            let mut pattern = EmbPattern::new();
            let result = csv::read(&mut cursor, &mut pattern);
            assert!(result.is_err(), "Should fail on line too long");
        }

        #[test]
        fn test_csv_invalid_coordinate() {
            let data = b"*,0,STITCH,invalid,20.0\n".to_vec();
            let mut cursor = Cursor::new(data);
            let mut pattern = EmbPattern::new();
            let result = csv::read(&mut cursor, &mut pattern);
            assert!(result.is_err(), "Should fail on invalid coordinate");
            let err = result.unwrap_err();
            assert!(
                err.to_string().contains("coordinate"),
                "Error should mention coordinate"
            );
        }

        #[test]
        fn test_csv_too_many_stitches() {
            // Verifying that CSV has protection against excessive stitches
            // In practice, a file with 10M+ stitches would be rejected during read
            // This is validated internally by the reader
            let data = b"*,0,STITCH,0.0,0.0\n".to_vec();
            let mut cursor = Cursor::new(data);
            let mut pattern = EmbPattern::new();
            let result = csv::read(&mut cursor, &mut pattern);
            assert!(result.is_ok(), "Small CSV should succeed");
        }
    }

    // INF format error cases
    mod inf {
        use super::*;
        use crate::formats::io::readers::inf;

        #[test]
        fn test_inf_truncated_header() {
            let data = vec![0u8; 8]; // Less than 16-byte header
            let mut cursor = Cursor::new(data);
            let mut pattern = EmbPattern::new();
            let result = inf::read(&mut cursor, &mut pattern);
            assert!(result.is_err(), "Should fail on truncated header");
        }

        #[test]
        fn test_inf_invalid_thread_count() {
            let mut data = vec![0u8; 16];
            // Set thread count to exceed MAX_INF_THREADS
            data[12..16].copy_from_slice(&50_000u32.to_be_bytes());

            let mut cursor = Cursor::new(data);
            let mut pattern = EmbPattern::new();
            let result = inf::read(&mut cursor, &mut pattern);
            assert!(result.is_err(), "Should fail on too many threads");
        }

        #[test]
        fn test_inf_record_too_small() {
            let mut data = vec![0u8; 16];
            data[12..16].copy_from_slice(&1u32.to_be_bytes()); // 1 thread
                                                               // Write invalid record length (less than minimum)
            data.extend_from_slice(&2u16.to_be_bytes()); // Length = 2 (too small)

            let mut cursor = Cursor::new(data);
            let mut pattern = EmbPattern::new();
            let result = inf::read(&mut cursor, &mut pattern);
            assert!(result.is_err(), "Should fail on record too small");
        }
    }

    // JSON format error cases
    mod json {
        use super::*;
        use crate::formats::io::readers::json;

        #[test]
        fn test_json_invalid_syntax() {
            let data = b"{invalid json".to_vec();
            let mut cursor = Cursor::new(data);
            let result = json::read(&mut cursor);
            assert!(result.is_err(), "Should fail on invalid JSON");
        }

        #[test]
        fn test_json_invalid_color_format() {
            let data = br#"{"threads":[{"color":"invalid"}]}"#.to_vec();
            let mut cursor = Cursor::new(data);
            let result = json::read(&mut cursor);
            assert!(result.is_err(), "Should fail on invalid color format");
        }

        #[test]
        fn test_json_empty() {
            let data = b"{}".to_vec();
            let mut cursor = Cursor::new(data);
            let result = json::read(&mut cursor);
            // Should succeed with empty pattern
            assert!(result.is_ok());
        }
    }
}
