//! Binary I/O helper utilities for embroidery file formats
//!
//! Provides ReadHelper and WriteHelper for convenient binary data reading/writing with
//! support for different byte orders, strings, and common embroidery file data structures.

use byteorder::{BigEndian, LittleEndian, ReadBytesExt, WriteBytesExt};
use std::io::{self, Read, Seek, SeekFrom, Write};

/// Helper for reading from binary streams
pub struct ReadHelper<R: Read> {
    reader: R,
}

impl<R: Read> ReadHelper<R> {
    /// Create a new ReadHelper
    pub fn new(reader: R) -> Self {
        Self { reader }
    }

    /// Read exact number of bytes
    pub fn read_bytes(&mut self, count: usize) -> io::Result<Vec<u8>> {
        let mut buffer = vec![0u8; count];
        self.reader.read_exact(&mut buffer)?;
        Ok(buffer)
    }

    /// Read a single byte
    pub fn read_u8(&mut self) -> io::Result<u8> {
        self.reader.read_u8()
    }

    /// Read i8
    pub fn read_i8(&mut self) -> io::Result<i8> {
        self.reader.read_i8()
    }

    /// Read u16 little endian
    pub fn read_u16_le(&mut self) -> io::Result<u16> {
        self.reader.read_u16::<LittleEndian>()
    }

    /// Read u16 big endian
    pub fn read_u16_be(&mut self) -> io::Result<u16> {
        self.reader.read_u16::<BigEndian>()
    }

    /// Read i16 little endian
    pub fn read_i16_le(&mut self) -> io::Result<i16> {
        self.reader.read_i16::<LittleEndian>()
    }

    /// Read i16 big endian
    pub fn read_i16_be(&mut self) -> io::Result<i16> {
        self.reader.read_i16::<BigEndian>()
    }

    /// Read u32 little endian
    pub fn read_u32_le(&mut self) -> io::Result<u32> {
        self.reader.read_u32::<LittleEndian>()
    }

    /// Read u32 big endian
    pub fn read_u32_be(&mut self) -> io::Result<u32> {
        self.reader.read_u32::<BigEndian>()
    }

    /// Read i32 little endian
    pub fn read_i32_le(&mut self) -> io::Result<i32> {
        self.reader.read_i32::<LittleEndian>()
    }

    /// Read i32 big endian
    pub fn read_i32_be(&mut self) -> io::Result<i32> {
        self.reader.read_i32::<BigEndian>()
    }

    /// Read string of specified length
    pub fn read_string(&mut self, length: usize) -> io::Result<String> {
        let bytes = self.read_bytes(length)?;
        Ok(String::from_utf8_lossy(&bytes).to_string())
    }

    /// Read null-terminated string
    pub fn read_cstring(&mut self, max_length: usize) -> io::Result<String> {
        let mut bytes = Vec::new();
        for _ in 0..max_length {
            let b = self.read_u8()?;
            if b == 0 {
                break;
            }
            bytes.push(b);
        }
        Ok(String::from_utf8_lossy(&bytes).to_string())
    }

    /// Get underlying reader
    pub fn into_inner(self) -> R {
        self.reader
    }
}

/// Helper for writing to binary streams
pub struct WriteHelper<W: Write> {
    writer: W,
    bytes_written: usize,
}

impl<W: Write> WriteHelper<W> {
    /// Create a new WriteHelper
    pub fn new(writer: W) -> Self {
        Self {
            writer,
            bytes_written: 0,
        }
    }

    /// Get number of bytes written
    pub fn bytes_written(&self) -> usize {
        self.bytes_written
    }

    /// Write bytes
    pub fn write_bytes(&mut self, data: &[u8]) -> io::Result<()> {
        self.writer.write_all(data)?;
        self.bytes_written = self.bytes_written.saturating_add(data.len());
        Ok(())
    }

    /// Write a single byte
    pub fn write_u8(&mut self, value: u8) -> io::Result<()> {
        self.writer.write_u8(value)?;
        self.bytes_written = self.bytes_written.saturating_add(1);
        Ok(())
    }

    /// Write i8
    pub fn write_i8(&mut self, value: i8) -> io::Result<()> {
        self.writer.write_i8(value)?;
        self.bytes_written = self.bytes_written.saturating_add(1);
        Ok(())
    }

    /// Write u16 little endian
    pub fn write_u16_le(&mut self, value: u16) -> io::Result<()> {
        self.writer.write_u16::<LittleEndian>(value)?;
        self.bytes_written = self.bytes_written.saturating_add(2);
        Ok(())
    }

    /// Write u16 big endian
    pub fn write_u16_be(&mut self, value: u16) -> io::Result<()> {
        self.writer.write_u16::<BigEndian>(value)?;
        self.bytes_written = self.bytes_written.saturating_add(2);
        Ok(())
    }

    /// Write i16 little endian
    pub fn write_i16_le(&mut self, value: i16) -> io::Result<()> {
        self.writer.write_i16::<LittleEndian>(value)?;
        self.bytes_written = self.bytes_written.saturating_add(2);
        Ok(())
    }

    /// Write i16 big endian
    pub fn write_i16_be(&mut self, value: i16) -> io::Result<()> {
        self.writer.write_i16::<BigEndian>(value)?;
        self.bytes_written = self.bytes_written.saturating_add(2);
        Ok(())
    }

    /// Write u32 little endian
    pub fn write_u32_le(&mut self, value: u32) -> io::Result<()> {
        self.writer.write_u32::<LittleEndian>(value)?;
        self.bytes_written = self.bytes_written.saturating_add(4);
        Ok(())
    }

    /// Write u32 big endian
    pub fn write_u32_be(&mut self, value: u32) -> io::Result<()> {
        self.writer.write_u32::<BigEndian>(value)?;
        self.bytes_written = self.bytes_written.saturating_add(4);
        Ok(())
    }

    /// Write i32 little endian
    pub fn write_i32_le(&mut self, value: i32) -> io::Result<()> {
        self.writer.write_i32::<LittleEndian>(value)?;
        self.bytes_written = self.bytes_written.saturating_add(4);
        Ok(())
    }

    /// Write i32 big endian
    pub fn write_i32_be(&mut self, value: i32) -> io::Result<()> {
        self.writer.write_i32::<BigEndian>(value)?;
        self.bytes_written = self.bytes_written.saturating_add(4);
        Ok(())
    }

    /// Write f32 little endian
    pub fn write_f32_le(&mut self, value: f32) -> io::Result<()> {
        self.writer.write_f32::<LittleEndian>(value)?;
        self.bytes_written = self.bytes_written.saturating_add(4);
        Ok(())
    }

    /// Write i24 (3 bytes) little endian
    pub fn write_i24_le(&mut self, value: i32) -> io::Result<()> {
        let bytes = value.to_le_bytes();
        self.writer.write_all(&bytes[0..3])?;
        self.bytes_written = self.bytes_written.saturating_add(3);
        Ok(())
    }

    /// Write string as bytes
    pub fn write_string(&mut self, s: &str) -> io::Result<()> {
        self.writer.write_all(s.as_bytes())?;
        self.bytes_written = self.bytes_written.saturating_add(s.len());
        Ok(())
    }

    /// Write string as UTF-8 bytes (alias for write_string)
    pub fn write_string_utf8(&mut self, s: &str) -> io::Result<()> {
        self.write_string(s)
    }

    /// Write null-terminated string with padding
    pub fn write_cstring(&mut self, s: &str, length: usize) -> io::Result<()> {
        if length == 0 {
            return Ok(());
        }

        let bytes = s.as_bytes();
        let len = bytes.len().min(length.saturating_sub(1));
        self.writer.write_all(&bytes[..len])?;

        // Pad with zeros
        let padding = length.saturating_sub(len);
        for _ in 0..padding {
            self.writer.write_u8(0)?;
        }

        self.bytes_written = self.bytes_written.saturating_add(length);
        Ok(())
    }

    /// Get underlying writer
    pub fn into_inner(self) -> W {
        self.writer
    }

    /// Get mutable reference to underlying writer
    pub fn inner_mut(&mut self) -> &mut W {
        &mut self.writer
    }
}

// Add Seek support for WriteHelper when writer supports it
impl<W: Write + Seek> WriteHelper<W> {
    /// Seek to a position
    pub fn seek(&mut self, pos: SeekFrom) -> io::Result<u64> {
        self.writer.seek(pos)
    }
}
