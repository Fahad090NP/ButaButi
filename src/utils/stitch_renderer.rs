//! Realistic stitch rendering utilities
//!
//! Provides functions to render embroidery stitches with realistic appearance
//! using the stitch.svg icon as a template. Supports color replacement and
//! rotation for accurate stitch visualization.

use crate::core::thread::EmbThread;

/// Stitch icon SVG template (embedded from assets/icons/stitch.svg)
///
/// This is a gradient-filled stitch icon that can be colorized by replacing
/// the #808080ff color with thread colors. The gradient provides realistic
/// 3D appearance with opacity variations.
const STITCH_SVG_TEMPLATE: &str = include_str!("../../assets/icons/stitch.svg");

/// Extract the stitch symbol definition from the template SVG
///
/// This parses the stitch.svg file and extracts just the path/gradient definitions
/// that can be embedded as a `<symbol>` in a larger SVG document.
///
/// # Returns
///
/// SVG symbol definition ready to be inserted into `<defs>` section
pub fn get_stitch_symbol_definition() -> String {
    // Extract the core SVG content (between <svg> and </svg>)
    // This includes defs and the actual stitch path elements

    // For now, return a simplified version
    // TODO: Parse the full template and extract gradients + paths

    String::from(
        r#"<symbol id="stitch" viewBox="0 0 9.6619425 2.240238">
    <defs>
        <linearGradient id="stitchGradient">
            <stop style="stop-color:THREAD_COLOR;stop-opacity:1;" offset="0" />
            <stop style="stop-color:THREAD_COLOR;stop-opacity:0.6;" offset="0.5" />
            <stop style="stop-color:THREAD_COLOR;stop-opacity:1;" offset="1" />
        </linearGradient>
    </defs>
    <!-- Stitch paths here -->
</symbol>"#,
    )
}

/// Create a colorized stitch symbol definition for a specific thread
///
/// Replaces the default gray color (#808080ff) with the thread's actual color
/// in all gradient definitions.
///
/// # Arguments
///
/// * `thread` - Thread color to use for the stitch
/// * `symbol_id` - Unique ID for this symbol (e.g., "stitch_0" for thread 0)
///
/// # Returns
///
/// SVG symbol definition with thread color applied
pub fn create_colored_stitch_symbol(thread: &EmbThread, symbol_id: &str) -> String {
    let thread_color = thread.hex_color();

    // Replace the default gray color with thread color
    // This is a simplified version - full implementation would parse and modify
    // all gradient stop colors in the template

    let symbol = STITCH_SVG_TEMPLATE
        .replace("#808080ff", &thread_color)
        .replace("#808080", &thread_color); // Handle both with and without alpha

    // Wrap in symbol definition
    // Extract just the content between <svg> tags
    if let Some(start) = symbol.find("<defs") {
        if let Some(end) = symbol.find("</svg>") {
            let content = &symbol[start..end];
            return format!(
                r#"<symbol id="{}" viewBox="0 0 9.6619425 2.240238">{}</symbol>"#,
                symbol_id, content
            );
        }
    }

    // Fallback to empty symbol if parsing fails
    format!(
        r#"<symbol id="{}" viewBox="0 0 9.6619425 2.240238"></symbol>"#,
        symbol_id
    )
}

/// Calculate the angle (in degrees) between two stitch points
///
/// Used to rotate the stitch icon to match the stitch direction.
///
/// # Arguments
///
/// * `x1, y1` - Starting point coordinates
/// * `x2, y2` - Ending point coordinates
///
/// # Returns
///
/// Angle in degrees (0-360), where 0° is horizontal right
pub fn calculate_stitch_angle(x1: f32, y1: f32, x2: f32, y2: f32) -> f32 {
    let dy = y2 - y1;
    let dx = x2 - x1;

    // atan2 returns radians, convert to degrees
    let angle_rad = dy.atan2(dx);
    let mut angle_deg = angle_rad.to_degrees();

    // Normalize to 0-360 range
    if angle_deg < 0.0 {
        angle_deg += 360.0;
    }

    angle_deg
}

/// Generate SVG `<use>` element for a stitch at a specific position
///
/// Creates a reference to a stitch symbol with rotation and positioning.
///
/// # Arguments
///
/// * `symbol_id` - ID of the stitch symbol to reference
/// * `x, y` - Position to place the stitch
/// * `angle` - Rotation angle in degrees
///
/// # Returns
///
/// SVG `<use>` element string
pub fn create_stitch_use_element(symbol_id: &str, x: f32, y: f32, angle: f32) -> String {
    format!(
        "<use xlink:href=\"#{}\" x=\"{}\" y=\"{}\" transform=\"rotate({} {} {})\" />",
        symbol_id, x, y, angle, x, y
    )
}

/// Render quality options for realistic stitch rendering
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum StitchRenderQuality {
    /// Simple paths with solid stroke (current default)
    #[default]
    Low,
    /// Colored paths with rounded caps
    Medium,
    /// Realistic stitch icons with gradients
    High,
    /// 3D-effect stitches with shadows and highlights
    Ultra,
}

impl StitchRenderQuality {
    /// Get stroke width for simple path rendering
    pub fn stroke_width(&self) -> f32 {
        match self {
            StitchRenderQuality::Low => 3.0,
            StitchRenderQuality::Medium => 4.0,
            StitchRenderQuality::High => 5.0,
            StitchRenderQuality::Ultra => 6.0,
        }
    }

    /// Whether to use realistic stitch icons
    pub fn use_stitch_icons(&self) -> bool {
        matches!(self, StitchRenderQuality::High | StitchRenderQuality::Ultra)
    }

    /// Whether to add shadow effects
    pub fn use_shadows(&self) -> bool {
        matches!(self, StitchRenderQuality::Ultra)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::thread::EmbThread;

    #[test]
    fn test_calculate_stitch_angle() {
        // Horizontal right (0°)
        assert!((calculate_stitch_angle(0.0, 0.0, 10.0, 0.0) - 0.0).abs() < 0.01);

        // Vertical down (90°)
        assert!((calculate_stitch_angle(0.0, 0.0, 0.0, 10.0) - 90.0).abs() < 0.01);

        // Horizontal left (180°)
        assert!((calculate_stitch_angle(0.0, 0.0, -10.0, 0.0) - 180.0).abs() < 0.01);

        // Vertical up (270°)
        assert!((calculate_stitch_angle(0.0, 0.0, 0.0, -10.0) - 270.0).abs() < 0.01);

        // Diagonal (45°)
        assert!((calculate_stitch_angle(0.0, 0.0, 10.0, 10.0) - 45.0).abs() < 0.01);
    }

    #[test]
    fn test_create_colored_stitch_symbol() {
        let thread = EmbThread::from_rgb(255, 0, 0);
        let symbol = create_colored_stitch_symbol(&thread, "stitch_0");

        // Should contain thread color
        assert!(symbol.contains("#ff0000") || symbol.contains("#FF0000"));

        // Should have symbol ID
        assert!(symbol.contains("id=\"stitch_0\""));

        // Should be a valid symbol element
        assert!(symbol.starts_with("<symbol"));
        assert!(symbol.ends_with("</symbol>"));
    }

    #[test]
    fn test_create_stitch_use_element() {
        let use_elem = create_stitch_use_element("stitch_0", 100.0, 200.0, 45.0);

        assert!(use_elem.contains("xlink:href=\"#stitch_0\""));
        assert!(use_elem.contains("x=\"100\""));
        assert!(use_elem.contains("y=\"200\""));
        assert!(use_elem.contains("rotate(45 100 200)"));
    }

    #[test]
    fn test_stitch_render_quality() {
        assert_eq!(StitchRenderQuality::default(), StitchRenderQuality::Low);

        assert!(!StitchRenderQuality::Low.use_stitch_icons());
        assert!(StitchRenderQuality::High.use_stitch_icons());
        assert!(StitchRenderQuality::Ultra.use_stitch_icons());

        assert!(!StitchRenderQuality::High.use_shadows());
        assert!(StitchRenderQuality::Ultra.use_shadows());
    }
}
