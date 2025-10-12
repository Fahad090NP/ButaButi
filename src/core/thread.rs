//! Thread color management and color utilities
//!
//! Provides the `EmbThread` type for representing thread colors with RGB values,
//! catalog numbers, and descriptions. Includes named color support and color distance calculations.

use crate::utils::error::{Error, Result};
use lazy_static::lazy_static;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Embroidery thread with color and metadata
///
/// Provides comprehensive thread information including color, brand, catalog number,
/// weight, and extensible metadata via the `attributes` field.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmbThread {
    /// Thread color in RGB format (0xRRGGBB)
    pub color: u32,

    /// Thread description/name (e.g., "Red", "Cardinal Red")
    pub description: Option<String>,

    /// Catalog/ID number (e.g., "1234", "5005")
    pub catalog_number: Option<String>,

    /// Additional details (deprecated - use attributes instead)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub details: Option<String>,

    /// Brand/manufacturer (e.g., "Brother", "Madeira", "Isacord")
    pub brand: Option<String>,

    /// Chart reference (e.g., "DMC", "Anchor")
    pub chart: Option<String>,

    /// Thread weight (e.g., "40wt", "60wt", "12wt")
    pub weight: Option<String>,

    /// Extensible metadata for custom thread properties
    ///
    /// Use this for any additional thread information like:
    /// - "type": "rayon", "polyester", "cotton"
    /// - "sheen": "high", "medium", "matte"
    /// - "thickness": "0.25mm"
    /// - "manufacturer_code": "XYZ123"
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub attributes: HashMap<String, String>,
}

impl EmbThread {
    /// Create a new thread with a given color
    ///
    /// # Example
    ///
    /// ```
    /// use rusty_petal::thread::EmbThread;
    ///
    /// let thread = EmbThread::new(0xFF0000); // Red
    /// ```
    pub fn new(color: u32) -> Self {
        Self {
            color: color & 0xFFFFFF,
            description: None,
            catalog_number: None,
            details: None,
            brand: None,
            chart: None,
            weight: None,
            attributes: HashMap::new(),
        }
    }

    /// Create a thread from a color string (hex or named color)
    ///
    /// # Example
    ///
    /// ```
    /// use rusty_petal::thread::EmbThread;
    ///
    /// let thread1 = EmbThread::from_string("red").unwrap();
    /// let thread2 = EmbThread::from_string("#FF0000").unwrap();
    /// let thread3 = EmbThread::from_string("ff0000").unwrap();
    /// ```
    pub fn from_string(color_str: &str) -> Result<Self> {
        let color = parse_color_string(color_str)?;
        Ok(Self::new(color))
    }

    /// Create a thread from RGB values
    pub fn from_rgb(r: u8, g: u8, b: u8) -> Self {
        Self::new(color_rgb(r, g, b))
    }

    /// Set color from hex string
    pub fn set_hex_color(&mut self, hex_string: &str) -> Result<()> {
        self.color = parse_color_hex(hex_string)?;
        Ok(())
    }

    /// Get color as hex string
    pub fn hex_color(&self) -> String {
        format!("#{:02x}{:02x}{:02x}", self.red(), self.green(), self.blue())
    }

    /// Get red component (0-255)
    pub fn red(&self) -> u8 {
        ((self.color >> 16) & 0xFF) as u8
    }

    /// Get green component (0-255)
    pub fn green(&self) -> u8 {
        ((self.color >> 8) & 0xFF) as u8
    }

    /// Get blue component (0-255)
    pub fn blue(&self) -> u8 {
        (self.color & 0xFF) as u8
    }

    /// Get opaque color with alpha channel
    pub fn opaque_color(&self) -> u32 {
        0xFF000000 | self.color
    }

    /// Find the nearest color in a palette
    ///
    /// Returns the index of the closest matching thread
    pub fn find_nearest_color_index(&self, palette: &[EmbThread]) -> Option<usize> {
        find_nearest_color_index(self.color, palette)
    }

    /// Calculate color distance to another color
    ///
    /// Returns the perceptual distance as a floating point value
    pub fn color_distance(&self, other_color: u32) -> f64 {
        color_distance(self.color, other_color) as f64
    }

    /// Builder method: set description
    pub fn with_description(mut self, desc: impl Into<String>) -> Self {
        self.description = Some(desc.into());
        self
    }

    /// Builder method: set catalog number
    pub fn with_catalog_number(mut self, catalog: impl Into<String>) -> Self {
        self.catalog_number = Some(catalog.into());
        self
    }

    /// Builder method: set brand
    pub fn with_brand(mut self, brand: impl Into<String>) -> Self {
        self.brand = Some(brand.into());
        self
    }

    /// Builder method: set chart
    pub fn with_chart(mut self, chart: impl Into<String>) -> Self {
        self.chart = Some(chart.into());
        self
    }

    /// Builder method: set weight
    pub fn with_weight(mut self, weight: impl Into<String>) -> Self {
        self.weight = Some(weight.into());
        self
    }

    /// Builder method: add a custom attribute
    ///
    /// # Example
    ///
    /// ```
    /// use rusty_petal::thread::EmbThread;
    ///
    /// let thread = EmbThread::new(0xFF0000)
    ///     .with_attribute("type", "polyester")
    ///     .with_attribute("sheen", "high");
    /// ```
    pub fn with_attribute(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.attributes.insert(key.into(), value.into());
        self
    }

    /// Get an attribute value by key
    pub fn get_attribute(&self, key: &str) -> Option<&str> {
        self.attributes.get(key).map(|s| s.as_str())
    }

    /// Set an attribute
    pub fn set_attribute(&mut self, key: impl Into<String>, value: impl Into<String>) {
        self.attributes.insert(key.into(), value.into());
    }

    /// Remove an attribute
    pub fn remove_attribute(&mut self, key: &str) -> Option<String> {
        self.attributes.remove(key)
    }

    /// Check if thread has an attribute
    pub fn has_attribute(&self, key: &str) -> bool {
        self.attributes.contains_key(key)
    }

    /// Get all attribute keys
    pub fn attribute_keys(&self) -> impl Iterator<Item = &String> {
        self.attributes.keys()
    }

    /// Find nearest matching thread in a palette using fuzzy color matching
    ///
    /// Returns the index of the closest matching thread based on perceptual color distance
    ///
    /// # Example
    ///
    /// ```
    /// use rusty_petal::thread::EmbThread;
    ///
    /// let my_thread = EmbThread::new(0xFF0055);
    /// let palette = vec![
    ///     EmbThread::new(0xFF0000).with_description("Red"),
    ///     EmbThread::new(0x00FF00).with_description("Green"),
    /// ];
    ///
    /// let nearest = my_thread.find_nearest_in_palette(&palette);
    /// assert_eq!(nearest, Some(0)); // Closest to red
    /// ```
    pub fn find_nearest_in_palette(&self, palette: &[EmbThread]) -> Option<usize> {
        find_nearest_color_index(self.color, palette)
    }

    /// Find nearest matching thread with a maximum color distance threshold
    ///
    /// Returns the index only if the closest match is within the threshold
    ///
    /// # Example
    ///
    /// ```
    /// use rusty_petal::thread::EmbThread;
    ///
    /// let my_thread = EmbThread::new(0xFF0055);
    /// let palette = vec![
    ///     EmbThread::new(0xFF0000).with_description("Red"),
    /// ];
    ///
    /// // Match if within threshold
    /// assert!(my_thread.find_nearest_within_threshold(&palette, 10000).is_some());
    ///
    /// // No match if threshold too strict
    /// assert!(my_thread.find_nearest_within_threshold(&palette, 10).is_none());
    /// ```
    pub fn find_nearest_within_threshold(
        &self,
        palette: &[EmbThread],
        threshold: u32,
    ) -> Option<usize> {
        if palette.is_empty() {
            return None;
        }

        let mut closest_index = 0;
        let mut closest_distance = u32::MAX;

        for (i, thread) in palette.iter().enumerate() {
            let dist = color_distance(self.color, thread.color);
            if dist < closest_distance {
                closest_distance = dist;
                closest_index = i;

                if dist == 0 {
                    return Some(closest_index); // Perfect match
                }
            }
        }

        if closest_distance <= threshold {
            Some(closest_index)
        } else {
            None
        }
    }

    /// Convert thread color to sRGB color space (0.0-1.0 range)
    ///
    /// Returns a palette::Srgb color for color space conversions and accurate color matching.
    ///
    /// # Example
    ///
    /// ```
    /// use butabuti::prelude::*;
    ///
    /// let thread = EmbThread::from_string("red").unwrap();
    /// let srgb = thread.to_srgb();
    /// ```
    pub fn to_srgb(&self) -> palette::Srgb {
        let r = ((self.color >> 16) & 0xFF) as u8;
        let g = ((self.color >> 8) & 0xFF) as u8;
        let b = (self.color & 0xFF) as u8;
        palette::Srgb::new(
            f32::from(r) / 255.0,
            f32::from(g) / 255.0,
            f32::from(b) / 255.0,
        )
    }

    /// Convert thread color to LAB color space for perceptually uniform color matching
    ///
    /// LAB color space is better for color distance calculations as it's designed to be
    /// perceptually uniform (equal distances = equal perceptual differences).
    ///
    /// # Example
    ///
    /// ```
    /// use butabuti::prelude::*;
    ///
    /// let thread = EmbThread::from_string("red").unwrap();
    /// let lab = thread.to_lab();
    /// ```
    pub fn to_lab(&self) -> palette::Lab {
        use palette::FromColor;
        palette::Lab::from_color(self.to_srgb())
    }

    /// Convert thread color to HSL (Hue, Saturation, Lightness) color space
    ///
    /// HSL is useful for color manipulation and generating complementary colors.
    ///
    /// # Example
    ///
    /// ```
    /// use butabuti::prelude::*;
    ///
    /// let thread = EmbThread::from_string("red").unwrap();
    /// let hsl = thread.to_hsl();
    /// ```
    pub fn to_hsl(&self) -> palette::Hsl {
        use palette::FromColor;
        palette::Hsl::from_color(self.to_srgb())
    }

    /// Calculate perceptually accurate color distance using DeltaE (CIE76 formula)
    ///
    /// Returns the perceptual color difference between this thread and another.
    /// Lower values mean more similar colors. DeltaE < 1.0 is imperceptible to humans.
    ///
    /// # Example
    ///
    /// ```
    /// use butabuti::prelude::*;
    ///
    /// let red = EmbThread::from_string("FF0000").unwrap();
    /// let dark_red = EmbThread::from_string("CC0000").unwrap();
    /// let blue = EmbThread::from_string("0000FF").unwrap();
    ///
    /// let dist1 = red.delta_e(&dark_red);
    /// let dist2 = red.delta_e(&blue);
    /// assert!(dist1 < dist2); // Red is closer to dark red than to blue
    /// ```
    pub fn delta_e(&self, other: &EmbThread) -> f32 {
        use palette::color_difference::EuclideanDistance;
        self.to_lab().distance(other.to_lab())
    }

    /// Find the closest matching thread in a palette using perceptually accurate DeltaE
    ///
    /// Uses LAB color space and DeltaE for better color matching than simple RGB distance.
    ///
    /// # Example
    ///
    /// ```
    /// use butabuti::prelude::*;
    ///
    /// let my_thread = EmbThread::from_string("FF0055").unwrap();
    /// let palette = vec![
    ///     EmbThread::from_string("FF0000").unwrap(), // Red
    ///     EmbThread::from_string("00FF00").unwrap(), // Green
    /// ];
    ///
    /// let (index, distance) = my_thread.find_closest_delta_e(&palette).unwrap();
    /// assert_eq!(index, 0); // Closest to red
    /// ```
    pub fn find_closest_delta_e(&self, palette: &[EmbThread]) -> Option<(usize, f32)> {
        if palette.is_empty() {
            return None;
        }

        let my_lab = self.to_lab();
        let mut closest_index = 0;
        let mut closest_distance = f32::MAX;

        for (i, thread) in palette.iter().enumerate() {
            let dist = {
                use palette::color_difference::EuclideanDistance;
                my_lab.distance(thread.to_lab())
            };
            if dist < closest_distance {
                closest_distance = dist;
                closest_index = i;

                if dist == 0.0 {
                    return Some((closest_index, 0.0)); // Perfect match
                }
            }
        }

        Some((closest_index, closest_distance))
    }
}

impl Default for EmbThread {
    fn default() -> Self {
        Self::new(0x000000)
    }
}

impl PartialEq for EmbThread {
    fn eq(&self, other: &Self) -> bool {
        self.color == other.color
    }
}

impl Eq for EmbThread {}

impl std::fmt::Display for EmbThread {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // Start with RGB hex value (always shown)
        write!(f, "Thread({})", self.hex_color())?;

        // Add description if available
        if let Some(ref desc) = self.description {
            write!(f, " - {}", desc)?;
        }

        // Add brand and catalog number if available
        if let (Some(ref brand), Some(ref catalog)) = (&self.brand, &self.catalog_number) {
            write!(f, " [{} #{}]", brand, catalog)?;
        } else if let Some(ref brand) = self.brand {
            write!(f, " [{}]", brand)?;
        } else if let Some(ref catalog) = self.catalog_number {
            write!(f, " [#{}]", catalog)?;
        }

        Ok(())
    }
}

// Color utility functions

/// Convert RGB components to a single u32 color value
pub fn color_rgb(r: u8, g: u8, b: u8) -> u32 {
    ((r as u32) << 16) | ((g as u32) << 8) | (b as u32)
}

/// Parse hex color string (with or without #)
pub fn parse_color_hex(hex_string: &str) -> Result<u32> {
    let h = hex_string.trim_start_matches('#');
    let size = h.len();

    match size {
        6 | 8 => u32::from_str_radix(&h[..6], 16)
            .map_err(|_| Error::InvalidColor(format!("Invalid hex color: {}", hex_string))),
        3 | 4 => {
            let chars: Vec<char> = h.chars().collect();
            let expanded = format!(
                "{}{}{}{}{}{}",
                chars[0], chars[0], chars[1], chars[1], chars[2], chars[2]
            );
            u32::from_str_radix(&expanded, 16)
                .map_err(|_| Error::InvalidColor(format!("Invalid hex color: {}", hex_string)))
        },
        _ => Err(Error::InvalidColor(format!(
            "Invalid hex color length: {}",
            hex_string
        ))),
    }
}

/// Parse a color string (hex or named color)
pub fn parse_color_string(color: &str) -> Result<u32> {
    if color == "random" {
        use std::collections::hash_map::RandomState;
        use std::hash::BuildHasher;
        return Ok((RandomState::new().hash_one(std::time::SystemTime::now()) as u32) & 0xFFFFFF);
    }

    // Try hex color first if it starts with # or is exactly 6 or 3 characters of hex digits
    if color.starts_with('#') {
        return parse_color_hex(color);
    }

    // Check if it looks like a hex string (3 or 6 hex digits)
    if (color.len() == 3 || color.len() == 6) && color.chars().all(|c| c.is_ascii_hexdigit()) {
        return parse_color_hex(color);
    }

    // Try named color
    NAMED_COLORS
        .get(color.to_lowercase().as_str())
        .copied()
        .ok_or_else(|| Error::InvalidColor(format!("Unknown color name: {}", color)))
}

/// Calculate color distance using the red-mean formula
///
/// This provides a perceptually better distance metric than simple Euclidean distance.
/// See: <https://www.compuphase.com/cmetric.htm>
pub fn color_distance(color1: u32, color2: u32) -> u32 {
    let r1 = ((color1 >> 16) & 0xFF) as i32;
    let g1 = ((color1 >> 8) & 0xFF) as i32;
    let b1 = (color1 & 0xFF) as i32;

    let r2 = ((color2 >> 16) & 0xFF) as i32;
    let g2 = ((color2 >> 8) & 0xFF) as i32;
    let b2 = (color2 & 0xFF) as i32;

    color_distance_components(r1, g1, b1, r2, g2, b2)
}

/// Calculate color distance from RGB components
pub fn color_distance_components(r1: i32, g1: i32, b1: i32, r2: i32, g2: i32, b2: i32) -> u32 {
    // Red-mean formula: weights R/B based on average red value for better perceptual accuracy
    let red_mean = (r1 + r2) / 2;
    let r = r1 - r2;
    let g = g1 - g2;
    let b = b1 - b2;

    // Use bit shifts instead of division and saturating operations
    let r_component = ((512 + red_mean) * r * r) >> 8;
    let g_component = 4 * g * g;
    let b_component = ((767 - red_mean) * b * b) >> 8;

    // Use saturating_add to prevent overflow
    r_component
        .saturating_add(g_component)
        .saturating_add(b_component) as u32
}

/// Find the nearest color in a palette
pub fn find_nearest_color_index(color: u32, palette: &[EmbThread]) -> Option<usize> {
    if palette.is_empty() {
        return None;
    }

    let mut closest_index = 0;
    let mut closest_distance = u32::MAX;

    for (i, thread) in palette.iter().enumerate() {
        let dist = color_distance(color, thread.color);
        if dist < closest_distance {
            closest_distance = dist;
            closest_index = i;

            // Perfect match - early exit optimization
            if dist == 0 {
                break;
            }
        }
    }

    Some(closest_index)
}

// X11/CSS/SVG Named colors
lazy_static! {
    static ref NAMED_COLORS: HashMap<&'static str, u32> = {
        let mut m = HashMap::new();
        m.insert("aliceblue", color_rgb(240, 248, 255));
        m.insert("antiquewhite", color_rgb(250, 235, 215));
        m.insert("aqua", color_rgb(0, 255, 255));
        m.insert("aquamarine", color_rgb(127, 255, 212));
        m.insert("azure", color_rgb(240, 255, 255));
        m.insert("beige", color_rgb(245, 245, 220));
        m.insert("bisque", color_rgb(255, 228, 196));
        m.insert("black", color_rgb(0, 0, 0));
        m.insert("blanchedalmond", color_rgb(255, 235, 205));
        m.insert("blue", color_rgb(0, 0, 255));
        m.insert("blueviolet", color_rgb(138, 43, 226));
        m.insert("brown", color_rgb(165, 42, 42));
        m.insert("burlywood", color_rgb(222, 184, 135));
        m.insert("cadetblue", color_rgb(95, 158, 160));
        m.insert("chartreuse", color_rgb(127, 255, 0));
        m.insert("chocolate", color_rgb(210, 105, 30));
        m.insert("coral", color_rgb(255, 127, 80));
        m.insert("cornflowerblue", color_rgb(100, 149, 237));
        m.insert("cornsilk", color_rgb(255, 248, 220));
        m.insert("crimson", color_rgb(220, 20, 60));
        m.insert("cyan", color_rgb(0, 255, 255));
        m.insert("darkblue", color_rgb(0, 0, 139));
        m.insert("darkcyan", color_rgb(0, 139, 139));
        m.insert("darkgoldenrod", color_rgb(184, 134, 11));
        m.insert("darkgray", color_rgb(169, 169, 169));
        m.insert("darkgreen", color_rgb(0, 100, 0));
        m.insert("darkgrey", color_rgb(169, 169, 169));
        m.insert("darkkhaki", color_rgb(189, 183, 107));
        m.insert("darkmagenta", color_rgb(139, 0, 139));
        m.insert("darkolivegreen", color_rgb(85, 107, 47));
        m.insert("darkorange", color_rgb(255, 140, 0));
        m.insert("darkorchid", color_rgb(153, 50, 204));
        m.insert("darkred", color_rgb(139, 0, 0));
        m.insert("darksalmon", color_rgb(233, 150, 122));
        m.insert("darkseagreen", color_rgb(143, 188, 143));
        m.insert("darkslateblue", color_rgb(72, 61, 139));
        m.insert("darkslategray", color_rgb(47, 79, 79));
        m.insert("darkslategrey", color_rgb(47, 79, 79));
        m.insert("darkturquoise", color_rgb(0, 206, 209));
        m.insert("darkviolet", color_rgb(148, 0, 211));
        m.insert("deeppink", color_rgb(255, 20, 147));
        m.insert("deepskyblue", color_rgb(0, 191, 255));
        m.insert("dimgray", color_rgb(105, 105, 105));
        m.insert("dimgrey", color_rgb(105, 105, 105));
        m.insert("dodgerblue", color_rgb(30, 144, 255));
        m.insert("firebrick", color_rgb(178, 34, 34));
        m.insert("floralwhite", color_rgb(255, 250, 240));
        m.insert("forestgreen", color_rgb(34, 139, 34));
        m.insert("fuchsia", color_rgb(255, 0, 255));
        m.insert("gainsboro", color_rgb(220, 220, 220));
        m.insert("ghostwhite", color_rgb(248, 248, 255));
        m.insert("gold", color_rgb(255, 215, 0));
        m.insert("goldenrod", color_rgb(218, 165, 32));
        m.insert("gray", color_rgb(128, 128, 128));
        m.insert("grey", color_rgb(128, 128, 128));
        m.insert("green", color_rgb(0, 128, 0));
        m.insert("greenyellow", color_rgb(173, 255, 47));
        m.insert("honeydew", color_rgb(240, 255, 240));
        m.insert("hotpink", color_rgb(255, 105, 180));
        m.insert("indianred", color_rgb(205, 92, 92));
        m.insert("indigo", color_rgb(75, 0, 130));
        m.insert("ivory", color_rgb(255, 255, 240));
        m.insert("khaki", color_rgb(240, 230, 140));
        m.insert("lavender", color_rgb(230, 230, 250));
        m.insert("lavenderblush", color_rgb(255, 240, 245));
        m.insert("lawngreen", color_rgb(124, 252, 0));
        m.insert("lemonchiffon", color_rgb(255, 250, 205));
        m.insert("lightblue", color_rgb(173, 216, 230));
        m.insert("lightcoral", color_rgb(240, 128, 128));
        m.insert("lightcyan", color_rgb(224, 255, 255));
        m.insert("lightgoldenrodyellow", color_rgb(250, 250, 210));
        m.insert("lightgray", color_rgb(211, 211, 211));
        m.insert("lightgreen", color_rgb(144, 238, 144));
        m.insert("lightgrey", color_rgb(211, 211, 211));
        m.insert("lightpink", color_rgb(255, 182, 193));
        m.insert("lightsalmon", color_rgb(255, 160, 122));
        m.insert("lightseagreen", color_rgb(32, 178, 170));
        m.insert("lightskyblue", color_rgb(135, 206, 250));
        m.insert("lightslategray", color_rgb(119, 136, 153));
        m.insert("lightslategrey", color_rgb(119, 136, 153));
        m.insert("lightsteelblue", color_rgb(176, 196, 222));
        m.insert("lightyellow", color_rgb(255, 255, 224));
        m.insert("lime", color_rgb(0, 255, 0));
        m.insert("limegreen", color_rgb(50, 205, 50));
        m.insert("linen", color_rgb(250, 240, 230));
        m.insert("magenta", color_rgb(255, 0, 255));
        m.insert("maroon", color_rgb(128, 0, 0));
        m.insert("mediumaquamarine", color_rgb(102, 205, 170));
        m.insert("mediumblue", color_rgb(0, 0, 205));
        m.insert("mediumorchid", color_rgb(186, 85, 211));
        m.insert("mediumpurple", color_rgb(147, 112, 219));
        m.insert("mediumseagreen", color_rgb(60, 179, 113));
        m.insert("mediumslateblue", color_rgb(123, 104, 238));
        m.insert("mediumspringgreen", color_rgb(0, 250, 154));
        m.insert("mediumturquoise", color_rgb(72, 209, 204));
        m.insert("mediumvioletred", color_rgb(199, 21, 133));
        m.insert("midnightblue", color_rgb(25, 25, 112));
        m.insert("mintcream", color_rgb(245, 255, 250));
        m.insert("mistyrose", color_rgb(255, 228, 225));
        m.insert("moccasin", color_rgb(255, 228, 181));
        m.insert("navajowhite", color_rgb(255, 222, 173));
        m.insert("navy", color_rgb(0, 0, 128));
        m.insert("oldlace", color_rgb(253, 245, 230));
        m.insert("olive", color_rgb(128, 128, 0));
        m.insert("olivedrab", color_rgb(107, 142, 35));
        m.insert("orange", color_rgb(255, 165, 0));
        m.insert("orangered", color_rgb(255, 69, 0));
        m.insert("orchid", color_rgb(218, 112, 214));
        m.insert("palegoldenrod", color_rgb(238, 232, 170));
        m.insert("palegreen", color_rgb(152, 251, 152));
        m.insert("paleturquoise", color_rgb(175, 238, 238));
        m.insert("palevioletred", color_rgb(219, 112, 147));
        m.insert("papayawhip", color_rgb(255, 239, 213));
        m.insert("peachpuff", color_rgb(255, 218, 185));
        m.insert("peru", color_rgb(205, 133, 63));
        m.insert("pink", color_rgb(255, 192, 203));
        m.insert("plum", color_rgb(221, 160, 221));
        m.insert("powderblue", color_rgb(176, 224, 230));
        m.insert("purple", color_rgb(128, 0, 128));
        m.insert("red", color_rgb(255, 0, 0));
        m.insert("rosybrown", color_rgb(188, 143, 143));
        m.insert("royalblue", color_rgb(65, 105, 225));
        m.insert("saddlebrown", color_rgb(139, 69, 19));
        m.insert("salmon", color_rgb(250, 128, 114));
        m.insert("sandybrown", color_rgb(244, 164, 96));
        m.insert("seagreen", color_rgb(46, 139, 87));
        m.insert("seashell", color_rgb(255, 245, 238));
        m.insert("sienna", color_rgb(160, 82, 45));
        m.insert("silver", color_rgb(192, 192, 192));
        m.insert("skyblue", color_rgb(135, 206, 235));
        m.insert("slateblue", color_rgb(106, 90, 205));
        m.insert("slategray", color_rgb(112, 128, 144));
        m.insert("slategrey", color_rgb(112, 128, 144));
        m.insert("snow", color_rgb(255, 250, 250));
        m.insert("springgreen", color_rgb(0, 255, 127));
        m.insert("steelblue", color_rgb(70, 130, 180));
        m.insert("tan", color_rgb(210, 180, 140));
        m.insert("teal", color_rgb(0, 128, 128));
        m.insert("thistle", color_rgb(216, 191, 216));
        m.insert("tomato", color_rgb(255, 99, 71));
        m.insert("turquoise", color_rgb(64, 224, 208));
        m.insert("violet", color_rgb(238, 130, 238));
        m.insert("wheat", color_rgb(245, 222, 179));
        m.insert("white", color_rgb(255, 255, 255));
        m.insert("whitesmoke", color_rgb(245, 245, 245));
        m.insert("yellow", color_rgb(255, 255, 0));
        m.insert("yellowgreen", color_rgb(154, 205, 50));
        m
    };
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_color_rgb() {
        assert_eq!(color_rgb(255, 0, 0), 0xFF0000);
        assert_eq!(color_rgb(0, 255, 0), 0x00FF00);
        assert_eq!(color_rgb(0, 0, 255), 0x0000FF);
    }

    #[test]
    fn test_parse_hex() {
        assert_eq!(parse_color_hex("#FF0000").unwrap(), 0xFF0000);
        assert_eq!(parse_color_hex("FF0000").unwrap(), 0xFF0000);
        assert_eq!(parse_color_hex("#F00").unwrap(), 0xFF0000);
        assert_eq!(parse_color_hex("F00").unwrap(), 0xFF0000);
    }

    #[test]
    fn test_named_colors() {
        assert_eq!(parse_color_string("red").unwrap(), 0xFF0000);
        assert_eq!(parse_color_string("green").unwrap(), 0x008000);
        assert_eq!(parse_color_string("blue").unwrap(), 0x0000FF);
    }

    #[test]
    fn test_thread_creation() {
        let thread = EmbThread::new(0xFF0000);
        assert_eq!(thread.color, 0xFF0000);
        assert_eq!(thread.red(), 255);
        assert_eq!(thread.green(), 0);
        assert_eq!(thread.blue(), 0);
    }

    #[test]
    fn test_thread_from_string() {
        let thread = EmbThread::from_string("red").unwrap();
        assert_eq!(thread.color, 0xFF0000);
    }

    #[test]
    fn test_color_distance() {
        let d1 = color_distance(0xFF0000, 0xFF0000);
        assert_eq!(d1, 0);

        let d2 = color_distance(0xFF0000, 0x00FF00);
        assert!(d2 > 0);
    }

    // Display trait tests
    #[test]
    fn test_thread_display_basic() {
        let thread = EmbThread::new(0xFF0000);
        assert_eq!(thread.to_string(), "Thread(#ff0000)");
    }

    #[test]
    fn test_thread_display_with_description() {
        let thread = EmbThread::new(0xFF0000).with_description("Red Thread");
        assert_eq!(thread.to_string(), "Thread(#ff0000) - Red Thread");
    }

    #[test]
    fn test_thread_display_with_brand() {
        let thread = EmbThread::new(0x0000FF).with_brand("Isacord");
        assert_eq!(thread.to_string(), "Thread(#0000ff) [Isacord]");
    }

    #[test]
    fn test_thread_display_with_catalog() {
        let thread = EmbThread::new(0x00FF00).with_catalog_number("1234");
        assert_eq!(thread.to_string(), "Thread(#00ff00) [#1234]");
    }

    #[test]
    fn test_thread_display_with_brand_and_catalog() {
        let thread = EmbThread::new(0xFFFF00)
            .with_brand("Madeira")
            .with_catalog_number("5678");
        assert_eq!(thread.to_string(), "Thread(#ffff00) [Madeira #5678]");
    }

    #[test]
    fn test_thread_display_full() {
        let thread = EmbThread::new(0xFF00FF)
            .with_description("Magenta")
            .with_brand("Sulky")
            .with_catalog_number("1109");
        assert_eq!(
            thread.to_string(),
            "Thread(#ff00ff) - Magenta [Sulky #1109]"
        );
    }

    #[test]
    fn test_thread_display_black() {
        let thread = EmbThread::new(0x000000).with_description("Black");
        assert_eq!(thread.to_string(), "Thread(#000000) - Black");
    }

    #[test]
    fn test_thread_display_white() {
        let thread = EmbThread::new(0xFFFFFF).with_description("White");
        assert_eq!(thread.to_string(), "Thread(#ffffff) - White");
    }

    // Attribute tests
    #[test]
    fn test_thread_with_attribute() {
        let thread = EmbThread::new(0xFF0000)
            .with_attribute("type", "polyester")
            .with_attribute("sheen", "high");

        assert_eq!(thread.get_attribute("type"), Some("polyester"));
        assert_eq!(thread.get_attribute("sheen"), Some("high"));
        assert_eq!(thread.get_attribute("nonexistent"), None);
    }

    #[test]
    fn test_thread_set_attribute() {
        let mut thread = EmbThread::new(0xFF0000);
        thread.set_attribute("type", "cotton");
        thread.set_attribute("weight", "40wt");

        assert_eq!(thread.get_attribute("type"), Some("cotton"));
        assert_eq!(thread.get_attribute("weight"), Some("40wt"));
    }

    #[test]
    fn test_thread_remove_attribute() {
        let mut thread = EmbThread::new(0xFF0000).with_attribute("type", "rayon");

        assert!(thread.has_attribute("type"));
        assert_eq!(thread.remove_attribute("type"), Some("rayon".to_string()));
        assert!(!thread.has_attribute("type"));
        assert_eq!(thread.remove_attribute("type"), None);
    }

    #[test]
    fn test_thread_attribute_keys() {
        let thread = EmbThread::new(0xFF0000)
            .with_attribute("type", "polyester")
            .with_attribute("sheen", "high")
            .with_attribute("thickness", "0.25mm");

        let keys: Vec<&String> = thread.attribute_keys().collect();
        assert_eq!(keys.len(), 3);
        assert!(keys.contains(&&"type".to_string()));
        assert!(keys.contains(&&"sheen".to_string()));
        assert!(keys.contains(&&"thickness".to_string()));
    }

    #[test]
    fn test_thread_with_weight() {
        let thread = EmbThread::new(0xFF0000).with_weight("40wt");
        assert_eq!(thread.weight, Some("40wt".to_string()));
    }

    #[test]
    fn test_thread_attributes_empty_by_default() {
        let thread = EmbThread::new(0xFF0000);
        assert_eq!(thread.attributes.len(), 0);
        assert!(!thread.has_attribute("any_key"));
    }

    // Fuzzy matching tests
    #[test]
    fn test_find_nearest_in_palette() {
        let my_thread = EmbThread::new(0xFF0055); // Reddish
        let palette = vec![
            EmbThread::new(0xFF0000).with_description("Red"),
            EmbThread::new(0x00FF00).with_description("Green"),
            EmbThread::new(0x0000FF).with_description("Blue"),
        ];

        let nearest = my_thread.find_nearest_in_palette(&palette);
        assert_eq!(nearest, Some(0)); // Closest to red
    }

    #[test]
    fn test_find_nearest_perfect_match() {
        let my_thread = EmbThread::new(0x00FF00);
        let palette = vec![
            EmbThread::new(0xFF0000).with_description("Red"),
            EmbThread::new(0x00FF00).with_description("Green"),
            EmbThread::new(0x0000FF).with_description("Blue"),
        ];

        let nearest = my_thread.find_nearest_in_palette(&palette);
        assert_eq!(nearest, Some(1)); // Perfect match with green
    }

    #[test]
    fn test_find_nearest_empty_palette() {
        let my_thread = EmbThread::new(0xFF0000);
        let palette: Vec<EmbThread> = vec![];

        let nearest = my_thread.find_nearest_in_palette(&palette);
        assert_eq!(nearest, None);
    }

    #[test]
    fn test_find_nearest_within_threshold() {
        let my_thread = EmbThread::new(0xFF0055);
        let palette = vec![
            EmbThread::new(0xFF0000).with_description("Red"),
            EmbThread::new(0x00FF00).with_description("Green"),
        ];

        // Should match red within generous threshold
        assert!(my_thread
            .find_nearest_within_threshold(&palette, 100000)
            .is_some());

        // Should not match with very strict threshold
        assert!(my_thread
            .find_nearest_within_threshold(&palette, 10)
            .is_none());
    }

    #[test]
    fn test_find_nearest_within_threshold_perfect_match() {
        let my_thread = EmbThread::new(0xFF0000);
        let palette = vec![EmbThread::new(0xFF0000).with_description("Red")];

        // Perfect match works even with threshold 0
        assert_eq!(
            my_thread.find_nearest_within_threshold(&palette, 0),
            Some(0)
        );
    }

    #[test]
    fn test_find_nearest_within_threshold_empty_palette() {
        let my_thread = EmbThread::new(0xFF0000);
        let palette: Vec<EmbThread> = vec![];

        assert_eq!(
            my_thread.find_nearest_within_threshold(&palette, 1000),
            None
        );
    }

    #[test]
    fn test_thread_metadata_combination() {
        let thread = EmbThread::new(0xFF0000)
            .with_description("Cardinal Red")
            .with_brand("Madeira")
            .with_catalog_number("1147")
            .with_weight("40wt")
            .with_attribute("type", "polyester")
            .with_attribute("sheen", "trilobal");

        assert_eq!(thread.description, Some("Cardinal Red".to_string()));
        assert_eq!(thread.brand, Some("Madeira".to_string()));
        assert_eq!(thread.catalog_number, Some("1147".to_string()));
        assert_eq!(thread.weight, Some("40wt".to_string()));
        assert_eq!(thread.get_attribute("type"), Some("polyester"));
        assert_eq!(thread.get_attribute("sheen"), Some("trilobal"));
    }

    #[test]
    fn test_thread_serialization_with_attributes() {
        let thread = EmbThread::new(0xFF0000)
            .with_description("Red")
            .with_attribute("type", "polyester");

        let json = serde_json::to_string(&thread).unwrap();
        let deserialized: EmbThread = serde_json::from_str(&json).unwrap();

        assert_eq!(deserialized.color, thread.color);
        assert_eq!(deserialized.description, thread.description);
        assert_eq!(deserialized.get_attribute("type"), Some("polyester"));
    }

    #[test]
    fn test_to_srgb() {
        let thread = EmbThread::new(0xFF8040); // Orange-ish color
        let srgb = thread.to_srgb();

        // Check RGB components (0.0-1.0 range)
        assert!((srgb.red - 1.0).abs() < 0.01); // 0xFF -> 1.0
        assert!((srgb.green - 0.502).abs() < 0.01); // 0x80 -> ~0.502
        assert!((srgb.blue - 0.251).abs() < 0.01); // 0x40 -> ~0.251
    }

    #[test]
    fn test_to_srgb_pure_colors() {
        let red = EmbThread::new(0xFF0000);
        let green = EmbThread::new(0x00FF00);
        let blue = EmbThread::new(0x0000FF);
        let black = EmbThread::new(0x000000);
        let white = EmbThread::new(0xFFFFFF);

        let srgb_red = red.to_srgb();
        assert!((srgb_red.red - 1.0).abs() < 0.01);
        assert!(srgb_red.green < 0.01);
        assert!(srgb_red.blue < 0.01);

        let srgb_green = green.to_srgb();
        assert!(srgb_green.red < 0.01);
        assert!((srgb_green.green - 1.0).abs() < 0.01);
        assert!(srgb_green.blue < 0.01);

        let srgb_blue = blue.to_srgb();
        assert!(srgb_blue.red < 0.01);
        assert!(srgb_blue.green < 0.01);
        assert!((srgb_blue.blue - 1.0).abs() < 0.01);

        let srgb_black = black.to_srgb();
        assert!(srgb_black.red < 0.01);
        assert!(srgb_black.green < 0.01);
        assert!(srgb_black.blue < 0.01);

        let srgb_white = white.to_srgb();
        assert!((srgb_white.red - 1.0).abs() < 0.01);
        assert!((srgb_white.green - 1.0).abs() < 0.01);
        assert!((srgb_white.blue - 1.0).abs() < 0.01);
    }

    #[test]
    fn test_to_lab() {
        let red = EmbThread::new(0xFF0000);
        let lab = red.to_lab();

        // LAB color space: L is lightness (0-100), a and b are color opponents
        // Red should have positive L and positive a (red-green axis)
        assert!(lab.l > 0.0 && lab.l <= 100.0);
        assert!(lab.a > 0.0); // Positive a means red
    }

    #[test]
    fn test_to_hsl() {
        let red = EmbThread::new(0xFF0000);
        let hsl = red.to_hsl();

        // HSL: Hue (0-360), Saturation (0-1), Lightness (0-1)
        // Pure red should have hue ~0, high saturation, medium lightness
        assert!(hsl.hue.into_positive_degrees() < 10.0 || hsl.hue.into_positive_degrees() > 350.0);
        assert!(hsl.saturation > 0.9); // High saturation for pure color
    }

    #[test]
    fn test_delta_e_identical_colors() {
        let thread1 = EmbThread::new(0xFF0000);
        let thread2 = EmbThread::new(0xFF0000);

        let distance = thread1.delta_e(&thread2);
        assert!(distance < 0.1); // Should be very close to 0
    }

    #[test]
    fn test_delta_e_similar_colors() {
        let red = EmbThread::new(0xFF0000);
        let dark_red = EmbThread::new(0xCC0000);
        let light_red = EmbThread::new(0xFF3333);

        let dist1 = red.delta_e(&dark_red);
        let dist2 = red.delta_e(&light_red);

        // Both should be relatively close
        assert!(dist1 < 50.0);
        assert!(dist2 < 50.0);
    }

    #[test]
    fn test_delta_e_different_colors() {
        let red = EmbThread::new(0xFF0000);
        let blue = EmbThread::new(0x0000FF);

        let distance = red.delta_e(&blue);

        // Red and blue should be very different
        assert!(distance > 100.0);
    }

    #[test]
    fn test_delta_e_ordering() {
        let red = EmbThread::new(0xFF0000);
        let orange = EmbThread::new(0xFF8000);
        let yellow = EmbThread::new(0xFFFF00);
        let blue = EmbThread::new(0x0000FF);

        // Red should be closer to orange than to yellow
        let dist_to_orange = red.delta_e(&orange);
        let dist_to_yellow = red.delta_e(&yellow);
        let dist_to_blue = red.delta_e(&blue);

        assert!(dist_to_orange < dist_to_yellow);
        assert!(dist_to_yellow < dist_to_blue);
    }

    #[test]
    fn test_find_closest_delta_e_empty_palette() {
        let thread = EmbThread::new(0xFF0000);
        let palette: Vec<EmbThread> = vec![];

        assert!(thread.find_closest_delta_e(&palette).is_none());
    }

    #[test]
    fn test_find_closest_delta_e_single_color() {
        let my_color = EmbThread::new(0xFF0055);
        let palette = vec![EmbThread::new(0xFF0000)];

        let result = my_color.find_closest_delta_e(&palette);
        assert!(result.is_some());

        let (index, distance) = result.unwrap();
        assert_eq!(index, 0);
        assert!(distance > 0.0); // Not identical but close
        assert!(distance < 50.0); // Should be reasonably close
    }

    #[test]
    fn test_find_closest_delta_e_multiple_colors() {
        let my_color = EmbThread::new(0xFF0055);
        let palette = vec![
            EmbThread::new(0xFF0000), // Red - closest
            EmbThread::new(0x00FF00), // Green - far
            EmbThread::new(0x0000FF), // Blue - far
        ];

        let (index, distance) = my_color.find_closest_delta_e(&palette).unwrap();
        assert_eq!(index, 0); // Should match red
        assert!(distance < 50.0); // Should be close to red
    }

    #[test]
    fn test_find_closest_delta_e_perfect_match() {
        let my_color = EmbThread::new(0xFF0000);
        let palette = vec![
            EmbThread::new(0x00FF00), // Green
            EmbThread::new(0xFF0000), // Exact match
            EmbThread::new(0x0000FF), // Blue
        ];

        let (index, distance) = my_color.find_closest_delta_e(&palette).unwrap();
        assert_eq!(index, 1); // Should match exact color
        assert!(distance < 0.1); // Distance should be nearly zero
    }

    #[test]
    fn test_find_closest_delta_e_vs_rgb_distance() {
        // Test that DeltaE gives better perceptual results than simple RGB distance
        let my_color = EmbThread::new(0x808080); // Gray

        let palette = vec![
            EmbThread::new(0x707070), // Slightly darker gray
            EmbThread::new(0xFF0000), // Red (same distance in some dimensions)
        ];

        let (index, _) = my_color.find_closest_delta_e(&palette).unwrap();
        assert_eq!(index, 0); // Should prefer the gray, not red
    }

    #[test]
    fn test_color_conversion_roundtrip() {
        let original = EmbThread::new(0xFF8040);

        // Convert to LAB and back via sRGB
        let srgb = original.to_srgb();
        let lab = original.to_lab();
        let hsl = original.to_hsl();

        // All conversions should produce valid values
        assert!(srgb.red >= 0.0 && srgb.red <= 1.0);
        assert!(srgb.green >= 0.0 && srgb.green <= 1.0);
        assert!(srgb.blue >= 0.0 && srgb.blue <= 1.0);

        assert!(lab.l >= 0.0 && lab.l <= 100.0);

        assert!(hsl.saturation >= 0.0 && hsl.saturation <= 1.0);
        assert!(hsl.lightness >= 0.0 && hsl.lightness <= 1.0);
    }
}
