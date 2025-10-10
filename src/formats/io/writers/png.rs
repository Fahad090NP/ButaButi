//! PNG raster image format writer for embroidery patterns
//!
//! Renders embroidery patterns to PNG images with anti-aliased line rendering,
//! gradient shading, and optional dimension guides. Manual PNG encoding without dependencies.

use crate::core::pattern::EmbPattern;
use crate::core::thread::EmbThread;
use crate::utils::error::Result;
use std::io::Write;

/// PNG writer settings
#[derive(Debug, Clone)]
pub struct PngSettings {
    /// Enable fancy gradient shading
    pub fancy: bool,
    /// Background color (default: white)
    pub background: Option<EmbThread>,
    /// Line width in pixels (default: 3)
    pub line_width: usize,
    /// Draw dimension guides (default: false)
    pub guides: bool,
}

impl Default for PngSettings {
    fn default() -> Self {
        Self {
            fancy: false,
            background: Some(EmbThread::from_rgb(255, 255, 255)),
            line_width: 3,
            guides: false,
        }
    }
}

/// Write pattern as PNG image
pub fn write(pattern: &EmbPattern, file: &mut impl Write, settings: &PngSettings) -> Result<()> {
    // Get pattern bounds
    let (min_x, min_y, max_x, max_y) = pattern.bounds();

    // Check if pattern is empty
    if pattern.stitches().is_empty() {
        // Empty pattern - write minimal valid PNG
        let empty_png = create_png(&[255, 255, 255, 255], 1, 1);
        file.write_all(&empty_png)?;
        return Ok(());
    }

    let width = ((max_x - min_x) as usize) + 3;
    let height = ((max_y - min_y) as usize) + 3;

    // Create drawing buffer
    let mut buffer = PngBuffer::new(width, height);
    buffer.set_fancy(settings.fancy);
    buffer.set_line_width(settings.line_width);

    // Set background
    if let Some(bg) = &settings.background {
        buffer.background(bg.red(), bg.green(), bg.blue(), 255);
    }

    // Offset coordinates to origin
    let offset_x = -min_x;
    let offset_y = -min_y;

    // Draw all stitch blocks
    for (block, thread) in pattern.get_as_stitchblock() {
        buffer.set_color(thread.red(), thread.green(), thread.blue(), 255);

        let mut last_x: Option<i32> = None;
        let mut last_y: Option<i32> = None;

        for (x, y) in block {
            let px = (x + offset_x) as i32;
            let py = (y + offset_y) as i32;

            if let (Some(lx), Some(ly)) = (last_x, last_y) {
                buffer.draw_line(lx, ly, px, py);
            }

            last_x = Some(px);
            last_y = Some(py);
        }
    }

    // Draw guides if requested
    if settings.guides {
        draw_guides(&mut buffer, min_x, min_y, width, height);
    }

    // Encode as PNG and write
    let png_data = create_png(&buffer.buf, buffer.width, buffer.height);
    file.write_all(&png_data)?;
    Ok(())
}

/// PNG image buffer with anti-aliased drawing
struct PngBuffer {
    width: usize,
    height: usize,
    buf: Vec<u8>, // RGBA format
    line_width: usize,
    fancy: bool,
    // Current drawing color
    red: u8,
    green: u8,
    blue: u8,
    alpha: u8,
    distance_from_black: f64,
    // Gradient parameters
    gradient_shade_ends: f64,
    gradient_shade_edge: f64,
    gradient_shade_center: f64,
}

impl PngBuffer {
    fn new(width: usize, height: usize) -> Self {
        Self {
            width,
            height,
            buf: vec![0; 4 * width * height],
            line_width: 3,
            fancy: false,
            red: 0,
            green: 0,
            blue: 0,
            alpha: 255,
            distance_from_black: 0.0,
            gradient_shade_ends: 0.65,
            gradient_shade_edge: 1.1,
            gradient_shade_center: 1.55,
        }
    }

    fn set_fancy(&mut self, fancy: bool) {
        self.fancy = fancy;
    }

    fn set_line_width(&mut self, width: usize) {
        self.line_width = width;
    }

    fn background(&mut self, red: u8, green: u8, blue: u8, alpha: u8) {
        for i in (0..self.buf.len()).step_by(4) {
            self.buf[i] = red;
            self.buf[i + 1] = green;
            self.buf[i + 2] = blue;
            self.buf[i + 3] = alpha;
        }
    }

    fn set_color(&mut self, red: u8, green: u8, blue: u8, alpha: u8) {
        self.red = red;
        self.green = green;
        self.blue = blue;
        self.alpha = alpha;

        // Calculate distance from black for gradient
        let r = red as f64;
        let g = green as f64;
        let b = blue as f64;
        let rmean = r / 2.0;
        self.distance_from_black = ((((512.0 + rmean) * r * r) / 256.0)
            + 4.0 * g * g
            + (((767.0 - rmean) * b * b) / 256.0))
            .sqrt();
    }

    fn gradient(&self, position: f64) -> f64 {
        if position <= 0.40 {
            let amount = position / 0.40;
            let v = amount * (self.gradient_shade_edge - self.gradient_shade_ends)
                + self.gradient_shade_ends;
            v.clamp(0.0, 1.0)
        } else if position <= 0.50 {
            let amount = (position - 0.40) / 0.10;
            let v = amount * (self.gradient_shade_center - self.gradient_shade_edge)
                + self.gradient_shade_edge;
            v.clamp(0.0, 1.0)
        } else if position <= 0.70 {
            let amount = (position - 0.50) / 0.20;
            let v = amount * (self.gradient_shade_edge - self.gradient_shade_center)
                + self.gradient_shade_center;
            v.clamp(0.0, 1.0)
        } else {
            let amount = (position - 0.70) / 0.30;
            let v = amount * (self.gradient_shade_ends - self.gradient_shade_edge)
                + self.gradient_shade_edge;
            v.clamp(0.0, 1.0)
        }
    }

    fn plot(&mut self, x: i32, y: i32, value: f64) {
        if x < 0 || y < 0 {
            return;
        }

        let x = (x as usize) + 1;
        let y = (y as usize) + 1;

        if x >= self.width || y >= self.height {
            return;
        }

        let idx = (y * self.width + x) * 4;

        // Calculate color with gradient
        let (mut r, mut g, mut b) = if self.distance_from_black < 15.0 {
            // Make black have highlights by using dark gray
            (35.0 * value, 35.0 * value, 35.0 * value)
        } else {
            (
                self.red as f64 * value,
                self.green as f64 * value,
                self.blue as f64 * value,
            )
        };

        // Clamp values
        r = r.clamp(0.0, 255.0);
        g = g.clamp(0.0, 255.0);
        b = b.clamp(0.0, 255.0);

        // Alpha blending
        let background_a = self.buf[idx + 3];
        let a = if background_a != 0 && self.alpha != 255 {
            let s_alpha = self.alpha as f64 / 255.0;
            let s_bg_alpha = background_a as f64 / 255.0;
            let one_minus_s_alpha = 1.0 - s_alpha;

            let bg_r = self.buf[idx] as f64;
            let bg_g = self.buf[idx + 1] as f64;
            let bg_b = self.buf[idx + 2] as f64;

            r = r * s_alpha + one_minus_s_alpha * bg_r;
            g = g * s_alpha + one_minus_s_alpha * bg_g;
            b = b * s_alpha + one_minus_s_alpha * bg_b;

            ((s_alpha + one_minus_s_alpha * s_bg_alpha) * 255.0) as u8
        } else {
            self.alpha
        };

        self.buf[idx] = r as u8;
        self.buf[idx + 1] = g as u8;
        self.buf[idx + 2] = b as u8;
        self.buf[idx + 3] = a.saturating_sub(4); // Slight transparency for background tint
    }

    fn draw_line(&mut self, x0: i32, y0: i32, x1: i32, y1: i32) {
        // Bresenham line drawing with anti-aliasing
        let dx = (x1 - x0).abs();
        let dy = (y1 - y0).abs();
        let sx = if x0 < x1 { 1 } else { -1 };
        let sy = if y0 < y1 { 1 } else { -1 };

        let mut x = x0;
        let mut y = y0;
        let mut i = 0;
        let max_pos = dx.max(dy);

        if dx > dy {
            let mut err = dx / 2;
            while x != x1 {
                self.line_for_point(x, y, false, max_pos, i);
                err -= dy;
                if err < 0 {
                    y += sy;
                    err += dx;
                }
                x += sx;
                i += 1;
            }
        } else {
            let mut err = dy / 2;
            while y != y1 {
                self.line_for_point(x, y, true, max_pos, i);
                err -= dx;
                if err < 0 {
                    x += sx;
                    err += dy;
                }
                y += sy;
                i += 1;
            }
        }

        // Draw final point
        self.line_for_point(x, y, dx <= dy, max_pos, i);
    }

    fn line_for_point(&mut self, x: i32, y: i32, vertical: bool, max_pos: i32, index: i32) {
        let w = self.line_width as i32;
        let left = w / 2;
        let right = w - left;

        let value = if self.fancy && max_pos > 0 {
            self.gradient(index as f64 / max_pos as f64)
        } else {
            1.0
        };

        if vertical {
            for pos in -left..right {
                self.plot(x + pos, y, value);
            }
        } else {
            for pos in -left..right {
                self.plot(x, y + pos, value);
            }
        }
    }
}

/// Draw dimension guides on the buffer
fn draw_guides(buffer: &mut PngBuffer, min_x: f64, min_y: f64, width: usize, height: usize) {
    buffer.set_color(0, 0, 0, 255);
    let original_width = buffer.line_width;
    buffer.set_line_width(1);

    let min_x = min_x as i32;
    let min_y = min_y as i32;
    let points = 50i32; // Grid spacing

    // Vertical guides
    let mut x = points - (min_x % points);
    while x < (width as i32 - 30) {
        if x >= 30 {
            buffer.draw_line(x, 0, x, 30);
        }
        x += points;
    }

    // Horizontal guides
    let mut y = points - (min_y % points);
    while y < (height as i32 - 30) {
        if y >= 30 {
            buffer.draw_line(0, y, 30, y);
        }
        y += points;
    }

    buffer.set_line_width(original_width);
}

/// Create PNG file from RGBA buffer
fn create_png(buf: &[u8], width: usize, height: usize) -> Vec<u8> {
    // Add filter byte (0x00 = no filter) to each scanline
    let mut raw_data = Vec::new();
    for y in 0..height {
        raw_data.push(0x00); // Filter type: None
        let start = y * width * 4;
        let end = start + width * 4;
        raw_data.extend_from_slice(&buf[start..end]);
    }

    // Compress with zlib
    let compressed = compress_zlib(&raw_data);

    // Build PNG chunks
    let mut png = Vec::new();

    // PNG signature
    png.extend_from_slice(b"\x89PNG\r\n\x1a\n");

    // IHDR chunk
    let mut ihdr = Vec::new();
    ihdr.extend_from_slice(&(width as u32).to_be_bytes());
    ihdr.extend_from_slice(&(height as u32).to_be_bytes());
    ihdr.push(8); // Bit depth
    ihdr.push(6); // Color type: RGBA
    ihdr.push(0); // Compression method
    ihdr.push(0); // Filter method
    ihdr.push(0); // Interlace method
    png.extend_from_slice(&png_chunk(b"IHDR", &ihdr));

    // IDAT chunk
    png.extend_from_slice(&png_chunk(b"IDAT", &compressed));

    // IEND chunk
    png.extend_from_slice(&png_chunk(b"IEND", &[]));

    png
}

/// Create a PNG chunk with CRC
fn png_chunk(tag: &[u8], data: &[u8]) -> Vec<u8> {
    let mut chunk = Vec::new();

    // Length
    chunk.extend_from_slice(&(data.len() as u32).to_be_bytes());

    // Type + Data
    chunk.extend_from_slice(tag);
    chunk.extend_from_slice(data);

    // CRC
    let crc = crc32(&chunk[4..]); // CRC of type + data
    chunk.extend_from_slice(&crc.to_be_bytes());

    chunk
}

/// CRC32 calculation for PNG
fn crc32(data: &[u8]) -> u32 {
    const CRC_TABLE: [u32; 256] = generate_crc_table();

    let mut crc = 0xFFFFFFFF_u32;
    for &byte in data {
        let index = ((crc ^ byte as u32) & 0xFF) as usize;
        crc = (crc >> 8) ^ CRC_TABLE[index];
    }
    crc ^ 0xFFFFFFFF
}

/// Generate CRC32 lookup table
const fn generate_crc_table() -> [u32; 256] {
    let mut table = [0u32; 256];
    let mut n = 0;
    while n < 256 {
        let mut c = n as u32;
        let mut k = 0;
        while k < 8 {
            if c & 1 != 0 {
                c = 0xEDB88320 ^ (c >> 1);
            } else {
                c >>= 1;
            }
            k += 1;
        }
        table[n] = c;
        n += 1;
    }
    table
}

/// Simple zlib compression (deflate with zlib wrapper)
fn compress_zlib(data: &[u8]) -> Vec<u8> {
    let mut result = Vec::new();

    // Zlib header (CMF + FLG)
    result.push(0x78); // CMF: deflate, 32K window
    result.push(0x9C); // FLG: default compression

    // Deflate compressed data (using uncompressed blocks for simplicity)
    let mut pos = 0;
    while pos < data.len() {
        let chunk_size = (data.len() - pos).min(65535);
        let is_final = pos + chunk_size >= data.len();

        // Block header
        result.push(if is_final { 0x01 } else { 0x00 });

        // Length (little-endian)
        result.extend_from_slice(&(chunk_size as u16).to_le_bytes());

        // One's complement of length
        result.extend_from_slice(&(!chunk_size as u16).to_le_bytes());

        // Data
        result.extend_from_slice(&data[pos..pos + chunk_size]);

        pos += chunk_size;
    }

    // Adler-32 checksum
    let adler = adler32(data);
    result.extend_from_slice(&adler.to_be_bytes());

    result
}

/// Adler-32 checksum for zlib
fn adler32(data: &[u8]) -> u32 {
    let mut a = 1u32;
    let mut b = 0u32;

    for &byte in data {
        a = (a + byte as u32) % 65521;
        b = (b + a) % 65521;
    }

    (b << 16) | a
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::constants::*;

    #[test]
    fn test_write_png_basic() {
        let mut pattern = EmbPattern::new();
        pattern.add_thread(EmbThread::from_rgb(255, 0, 0));
        pattern.add_stitch_absolute(STITCH, 0.0, 0.0);
        pattern.add_stitch_absolute(STITCH, 100.0, 100.0);
        pattern.add_stitch_absolute(END, 100.0, 100.0);

        let mut output = Vec::new();
        let settings = PngSettings::default();
        write(&pattern, &mut output, &settings).unwrap();

        // Verify PNG signature
        assert_eq!(&output[0..8], b"\x89PNG\r\n\x1a\n");
    }

    #[test]
    fn test_png_with_fancy() {
        let mut pattern = EmbPattern::new();
        pattern.add_thread(EmbThread::from_rgb(0, 255, 0));
        pattern.add_stitch_absolute(STITCH, 0.0, 0.0);
        pattern.add_stitch_absolute(STITCH, 50.0, 50.0);
        pattern.add_stitch_absolute(END, 50.0, 50.0);

        let mut output = Vec::new();
        let settings = PngSettings {
            fancy: true,
            ..Default::default()
        };
        write(&pattern, &mut output, &settings).unwrap();

        assert_eq!(&output[0..8], b"\x89PNG\r\n\x1a\n");
    }

    #[test]
    fn test_png_empty_pattern() {
        let pattern = EmbPattern::new();
        let mut output = Vec::new();
        let settings = PngSettings::default();
        write(&pattern, &mut output, &settings).unwrap();

        // Should produce valid PNG even with no stitches
        assert_eq!(&output[0..8], b"\x89PNG\r\n\x1a\n");
    }
}
