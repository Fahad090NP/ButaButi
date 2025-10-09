//! Husqvarna Viking SHV thread color palette
//!
//! Contains 43 predefined thread colors used in SHV embroidery files,
//! with RGB values and catalog numbers for Husqvarna Viking machines.

use crate::core::thread::EmbThread;

/// Get the SHV (Husqvarna Viking SHV) thread palette
pub fn get_thread_set() -> Vec<EmbThread> {
    vec![
        EmbThread::from_rgb(0, 0, 0)
            .with_description("Black")
            .with_catalog_number("0")
            .with_brand("Shv"),
        EmbThread::from_rgb(0, 0, 255)
            .with_description("Blue")
            .with_catalog_number("1")
            .with_brand("Shv"),
        EmbThread::from_rgb(51, 204, 102)
            .with_description("Green")
            .with_catalog_number("2")
            .with_brand("Shv"),
        EmbThread::from_rgb(255, 0, 0)
            .with_description("Red")
            .with_catalog_number("3")
            .with_brand("Shv"),
        EmbThread::from_rgb(255, 0, 255)
            .with_description("Purple")
            .with_catalog_number("4")
            .with_brand("Shv"),
        EmbThread::from_rgb(255, 255, 0)
            .with_description("Yellow")
            .with_catalog_number("5")
            .with_brand("Shv"),
        EmbThread::from_rgb(127, 127, 127)
            .with_description("Gray")
            .with_catalog_number("6")
            .with_brand("Shv"),
        EmbThread::from_rgb(51, 154, 255)
            .with_description("Light Blue")
            .with_catalog_number("7")
            .with_brand("Shv"),
        EmbThread::from_rgb(0, 255, 0)
            .with_description("Light Green")
            .with_catalog_number("8")
            .with_brand("Shv"),
        EmbThread::from_rgb(255, 127, 0)
            .with_description("Orange")
            .with_catalog_number("9")
            .with_brand("Shv"),
        EmbThread::from_rgb(255, 160, 180)
            .with_description("Pink")
            .with_catalog_number("10")
            .with_brand("Shv"),
        EmbThread::from_rgb(153, 75, 0)
            .with_description("Brown")
            .with_catalog_number("11")
            .with_brand("Shv"),
        EmbThread::from_rgb(255, 255, 255)
            .with_description("White")
            .with_catalog_number("12")
            .with_brand("Shv"),
        EmbThread::from_rgb(0, 0, 0)
            .with_description("Black")
            .with_catalog_number("13")
            .with_brand("Shv"),
        EmbThread::from_rgb(0, 0, 0)
            .with_description("Black")
            .with_catalog_number("14")
            .with_brand("Shv"),
        EmbThread::from_rgb(0, 0, 0)
            .with_description("Black")
            .with_catalog_number("15")
            .with_brand("Shv"),
        EmbThread::from_rgb(0, 0, 0)
            .with_description("Black")
            .with_catalog_number("16")
            .with_brand("Shv"),
        EmbThread::from_rgb(0, 0, 0)
            .with_description("Black")
            .with_catalog_number("17")
            .with_brand("Shv"),
        EmbThread::from_rgb(0, 0, 0)
            .with_description("Black")
            .with_catalog_number("18")
            .with_brand("Shv"),
        EmbThread::from_rgb(255, 127, 127)
            .with_description("Light Red")
            .with_catalog_number("19")
            .with_brand("Shv"),
        EmbThread::from_rgb(255, 127, 255)
            .with_description("Light Purple")
            .with_catalog_number("20")
            .with_brand("Shv"),
        EmbThread::from_rgb(255, 255, 153)
            .with_description("Light Yellow")
            .with_catalog_number("21")
            .with_brand("Shv"),
        EmbThread::from_rgb(192, 192, 192)
            .with_description("Light Gray")
            .with_catalog_number("22")
            .with_brand("Shv"),
        EmbThread::from_rgb(0, 0, 0)
            .with_description("Black")
            .with_catalog_number("23")
            .with_brand("Shv"),
        EmbThread::from_rgb(0, 0, 0)
            .with_description("Black")
            .with_catalog_number("24")
            .with_brand("Shv"),
        EmbThread::from_rgb(255, 165, 65)
            .with_description("Light Orange")
            .with_catalog_number("25")
            .with_brand("Shv"),
        EmbThread::from_rgb(255, 204, 204)
            .with_description("Light Pink")
            .with_catalog_number("26")
            .with_brand("Shv"),
        EmbThread::from_rgb(175, 90, 10)
            .with_description("Light Brown")
            .with_catalog_number("27")
            .with_brand("Shv"),
        EmbThread::from_rgb(0, 0, 0)
            .with_description("Black")
            .with_catalog_number("28")
            .with_brand("Shv"),
        EmbThread::from_rgb(0, 0, 0)
            .with_description("Black")
            .with_catalog_number("29")
            .with_brand("Shv"),
        EmbThread::from_rgb(0, 0, 0)
            .with_description("Black")
            .with_catalog_number("30")
            .with_brand("Shv"),
        EmbThread::from_rgb(0, 0, 0)
            .with_description("Black")
            .with_catalog_number("31")
            .with_brand("Shv"),
        EmbThread::from_rgb(0, 0, 0)
            .with_description("Black")
            .with_catalog_number("32")
            .with_brand("Shv"),
        EmbThread::from_rgb(0, 0, 127)
            .with_description("Dark Blue")
            .with_catalog_number("33")
            .with_brand("Shv"),
        EmbThread::from_rgb(0, 127, 0)
            .with_description("Dark Green")
            .with_catalog_number("34")
            .with_brand("Shv"),
        EmbThread::from_rgb(127, 0, 0)
            .with_description("Dark Red")
            .with_catalog_number("35")
            .with_brand("Shv"),
        EmbThread::from_rgb(127, 0, 127)
            .with_description("Dark Purple")
            .with_catalog_number("36")
            .with_brand("Shv"),
        EmbThread::from_rgb(200, 200, 0)
            .with_description("Dark Yellow")
            .with_catalog_number("37")
            .with_brand("Shv"),
        EmbThread::from_rgb(60, 60, 60)
            .with_description("Dark Gray")
            .with_catalog_number("38")
            .with_brand("Shv"),
        EmbThread::from_rgb(0, 0, 0)
            .with_description("Black")
            .with_catalog_number("39")
            .with_brand("Shv"),
        EmbThread::from_rgb(0, 0, 0)
            .with_description("Black")
            .with_catalog_number("40")
            .with_brand("Shv"),
        EmbThread::from_rgb(232, 63, 0)
            .with_description("Dark Orange")
            .with_catalog_number("41")
            .with_brand("Shv"),
        EmbThread::from_rgb(255, 102, 122)
            .with_description("Dark Pink")
            .with_catalog_number("42")
            .with_brand("Shv"),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_shv_palette_size() {
        let palette = get_thread_set();
        assert_eq!(palette.len(), 43);
    }

    #[test]
    fn test_shv_first_thread() {
        let palette = get_thread_set();
        let black = &palette[0];
        assert_eq!(black.red(), 0);
        assert_eq!(black.green(), 0);
        assert_eq!(black.blue(), 0);
    }

    #[test]
    fn test_shv_branded_threads() {
        let palette = get_thread_set();
        for thread in palette {
            assert_eq!(thread.brand, Some("Shv".to_string()));
        }
    }
}
