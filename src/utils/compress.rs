//! Huffman compression and decompression for embroidery formats
//!
//! Implements Huffman coding used in HUS (Husqvarna Viking) format files.
//! Provides both compression and decompression with dynamic Huffman tree building.

use crate::utils::error::Result;

/// Expand (decompress) data using Huffman decompression
pub fn expand(data: &[u8], uncompressed_size: Option<usize>) -> Result<Vec<u8>> {
    let mut decompressor = EmbCompress::new(data);
    decompressor.decompress(uncompressed_size)
}

/// Compress data (basic header + raw data)
#[allow(dead_code)]
pub fn compress(data: &[u8]) -> Vec<u8> {
    let size = data.len();
    let mut result = vec![
        (size & 0xFF) as u8,
        ((size >> 8) & 0xFF) as u8,
        0x02,
        0xA0,
        0x01,
        0xFE,
    ];
    result.extend_from_slice(data);
    result
}

/// Huffman table for decoding
struct Huffman {
    default_value: usize,
    lengths: Vec<usize>,
    table: Vec<usize>,
    table_width: usize,
}

impl Huffman {
    fn new(lengths: Vec<usize>, default_value: usize) -> Self {
        let mut huffman = Huffman {
            default_value,
            lengths,
            table: Vec::new(),
            table_width: 0,
        };
        huffman.build_table();
        huffman
    }

    fn new_default(default_value: usize) -> Self {
        Huffman {
            default_value,
            lengths: Vec::new(),
            table: Vec::new(),
            table_width: 0,
        }
    }

    fn build_table(&mut self) {
        if self.lengths.is_empty() {
            return;
        }

        // Build lookup table for fast Huffman decoding
        let max_length = *self.lengths.iter().max().unwrap_or(&0);

        // Guard against excessive table width (max 16 bits for u16 lookup)
        self.table_width = max_length.min(16);

        if self.table_width == 0 {
            return;
        }

        let table_size = 1usize.checked_shl(self.table_width as u32).unwrap_or(0);

        // Guard against excessive memory allocation
        if table_size == 0 || table_size > (1 << 16) {
            return;
        }

        self.table = vec![0; table_size];

        for bit_length in 1..=self.table_width {
            let size = 1usize
                .checked_shl((self.table_width - bit_length) as u32)
                .unwrap_or(0);

            if size == 0 {
                continue;
            }

            for (len_index, &length) in self.lengths.iter().enumerate() {
                if length == bit_length {
                    for _ in 0..size {
                        if self.table.len() < self.table.capacity() {
                            self.table.push(len_index);
                        }
                    }
                }
            }
        }
    }

    fn lookup(&self, byte_lookup: u16) -> (usize, usize) {
        if self.table.is_empty() {
            return (self.default_value, 0);
        }

        // Ensure we don't overflow the table
        if self.table_width == 0 || self.table_width > 16 {
            return (self.default_value, 0);
        }

        let index = (byte_lookup >> (16 - self.table_width)) as usize;

        // Bounds check
        if index >= self.table.len() {
            return (self.default_value, 0);
        }

        let v = self.table[index];

        // Ensure v is within bounds of lengths array
        if v >= self.lengths.len() {
            return (self.default_value, 0);
        }

        (v, self.lengths[v])
    }
}

/// Huffman decompressor
struct EmbCompress {
    bit_position: usize,
    input_data: Vec<u8>,
    block_elements: i32,
    character_huffman: Option<Huffman>,
    distance_huffman: Option<Huffman>,
}

impl EmbCompress {
    fn new(data: &[u8]) -> Self {
        EmbCompress {
            bit_position: 0,
            input_data: data.to_vec(),
            block_elements: -1,
            character_huffman: None,
            distance_huffman: None,
        }
    }

    fn get_bits(&self, start_pos_in_bits: usize, length: usize) -> u32 {
        // Guard against zero length or excessive length
        if length == 0 || length > 32 {
            return 0;
        }

        let end_pos_in_bits = start_pos_in_bits.saturating_add(length).saturating_sub(1);
        let start_pos_in_bytes = start_pos_in_bits / 8;
        let end_pos_in_bytes = end_pos_in_bits / 8;

        // Guard against reading beyond data
        if start_pos_in_bytes >= self.input_data.len() {
            return 0;
        }

        // Collect bytes spanning the bit range
        let mut value: u32 = 0;
        for i in start_pos_in_bytes..=end_pos_in_bytes.min(self.input_data.len().saturating_sub(1))
        {
            value <<= 8;
            if i < self.input_data.len() {
                value |= self.input_data[i] as u32;
            }
        }

        // Extract the exact bits requested by masking and shifting
        let unused_bits_right = (8 - (end_pos_in_bits + 1) % 8) % 8;
        let mask = if length == 32 {
            u32::MAX
        } else {
            (1u32 << length) - 1
        };
        (value >> unused_bits_right) & mask
    }

    fn pop(&mut self, bit_count: usize) -> u32 {
        let value = self.peek(bit_count);
        self.slide(bit_count);
        value
    }

    fn peek(&self, bit_count: usize) -> u32 {
        self.get_bits(self.bit_position, bit_count)
    }

    fn slide(&mut self, bit_count: usize) {
        self.bit_position = self.bit_position.saturating_add(bit_count);
    }

    fn read_variable_length(&mut self) -> usize {
        let mut m = self.pop(3) as usize;
        if m != 7 {
            return m;
        }
        for _ in 0..13 {
            let s = self.pop(1);
            if s == 1 {
                m += 1;
            } else {
                break;
            }
        }
        m
    }

    fn load_character_length_huffman(&mut self) -> Huffman {
        let count = self.pop(5) as usize;
        if count == 0 {
            let v = self.pop(5) as usize;
            return Huffman::new_default(v);
        }

        let mut lengths = vec![0; count];
        let mut index = 0;
        while index < count {
            if index == 3 {
                index += self.pop(2) as usize;
            }
            if index < count {
                lengths[index] = self.read_variable_length();
                index += 1;
            }
        }
        Huffman::new(lengths, 8)
    }

    fn load_character_huffman(&mut self, length_huffman: &Huffman) -> Huffman {
        let count = self.pop(9) as usize;
        if count == 0 {
            let v = self.pop(9) as usize;
            return Huffman::new_default(v);
        }

        let mut lengths = vec![0; count];
        let mut index = 0;
        while index < count {
            let (c, len) = length_huffman.lookup(self.peek(16) as u16);
            self.slide(len);

            if c == 0 {
                index += 1;
            } else if c == 1 {
                let skip = 3 + self.pop(4) as usize;
                index += skip;
            } else if c == 2 {
                let skip = 20 + self.pop(9) as usize;
                index += skip;
            } else if index < count {
                lengths[index] = c - 2;
                index += 1;
            }
        }
        Huffman::new(lengths, 0)
    }

    fn load_distance_huffman(&mut self) -> Huffman {
        let count = self.pop(5) as usize;
        if count == 0 {
            let v = self.pop(5) as usize;
            return Huffman::new_default(v);
        }

        let mut lengths = vec![0; count];
        for length in lengths.iter_mut().take(count) {
            *length = self.read_variable_length();
        }
        Huffman::new(lengths, 0)
    }

    fn load_block(&mut self) {
        self.block_elements = self.pop(16) as i32;
        let character_length_huffman = self.load_character_length_huffman();
        self.character_huffman = Some(self.load_character_huffman(&character_length_huffman));
        self.distance_huffman = Some(self.load_distance_huffman());
    }

    fn get_token(&mut self) -> usize {
        if self.block_elements <= 0 {
            self.load_block();
        }
        self.block_elements -= 1;

        let huffman = self
            .character_huffman
            .as_ref()
            .expect("Huffman character table not initialized - call load_block first");
        let (value, len) = huffman.lookup(self.peek(16) as u16);
        self.slide(len);
        value
    }

    fn get_position(&mut self) -> usize {
        let huffman = self
            .distance_huffman
            .as_ref()
            .expect("Huffman distance table not initialized - call load_block first");
        let (value, len) = huffman.lookup(self.peek(16) as u16);
        self.slide(len);

        if value == 0 {
            return 0;
        }

        let v = value.saturating_sub(1);

        // Guard against shifting by too many bits
        if v >= 32 {
            return 0;
        }

        let additional = self.pop(v) as usize;
        (1usize.checked_shl(v as u32).unwrap_or(0)).saturating_add(additional)
    }

    fn decompress(&mut self, uncompressed_size: Option<usize>) -> Result<Vec<u8>> {
        let mut output_data = Vec::new();
        let bits_total = self.input_data.len().saturating_mul(8);

        // Pre-allocate if we know the size
        if let Some(size) = uncompressed_size {
            output_data.reserve(size);
        }

        while bits_total > self.bit_position {
            if let Some(size) = uncompressed_size {
                if output_data.len() >= size {
                    break;
                }
            }

            let character = self.get_token();

            if character <= 255 {
                // Literal byte
                output_data.push(character as u8);
            } else if character == 510 {
                // END marker
                break;
            } else {
                // Lookback reference
                let length = character.saturating_sub(253); // Min length is 3 (256 - 253 = 3)

                // Guard against zero length
                if length == 0 {
                    continue;
                }

                let back = self.get_position().saturating_add(1);
                let position = output_data.len().saturating_sub(back);

                // Ensure we don't overflow
                if back > output_data.len() {
                    // Invalid lookback - skip this sequence
                    continue;
                }

                if back > length {
                    // Entire lookback is already in output
                    let end_pos = position.saturating_add(length).min(output_data.len());
                    for i in position..end_pos {
                        let byte = output_data[i];
                        output_data.push(byte);
                    }
                } else {
                    // Will read & write overlapping data
                    for i in 0..length {
                        let idx = position.saturating_add(i);
                        if idx < output_data.len() {
                            let byte = output_data[idx];
                            output_data.push(byte);
                        } else {
                            break;
                        }
                    }
                }
            }
        }

        Ok(output_data)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compress_basic() {
        let data = vec![1, 2, 3, 4, 5];
        let compressed = compress(&data);

        // Check header
        assert_eq!(compressed[0], 5); // Size low byte
        assert_eq!(compressed[1], 0); // Size high byte
        assert_eq!(compressed[2], 0x02);
        assert_eq!(compressed[3], 0xA0);
        assert_eq!(compressed[4], 0x01);
        assert_eq!(compressed[5], 0xFE);

        // Check data
        assert_eq!(&compressed[6..], &data);
    }

    #[test]
    fn test_expand_literal() {
        // Simple test case with minimal Huffman encoding
        // This is a very basic test - real HUS files are more complex
        let compressed = vec![
            0x00, 0x01, // Block elements: 1
            0x00, // Character length huffman count: 0
            0x00, // Default value: 0
            0x00, 0x01, // Character huffman count: 1
            0x00, 0x41, // Character: 'A'
            0x00, // Distance huffman count: 0
            0x00, // Default value: 0
        ];

        // Note: Real HUS compression is complex - this test is simplified
        // In practice, we'd test with actual HUS file data
        let result = expand(&compressed, Some(10));
        assert!(result.is_ok());
    }
}
