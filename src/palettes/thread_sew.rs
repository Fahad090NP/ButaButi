//! Janome SEW thread color palette
//!
//! Contains 79 predefined thread colors used in SEW embroidery files,
//! with RGB values and catalog numbers for Janome sewing machines.

use crate::core::thread::EmbThread;

/// Get the SEW (Janome Sewing Machine) thread palette
pub fn get_thread_set() -> Vec<EmbThread> {
    vec![
        EmbThread::from_rgb(0, 0, 0)
            .with_description("Unknown")
            .with_catalog_number("0")
            .with_brand("Sew"),
        EmbThread::from_rgb(0, 0, 0)
            .with_description("Black")
            .with_catalog_number("1")
            .with_brand("Sew"),
        EmbThread::from_rgb(255, 255, 255)
            .with_description("White")
            .with_catalog_number("2")
            .with_brand("Sew"),
        EmbThread::from_rgb(255, 255, 23)
            .with_description("Sunflower")
            .with_catalog_number("3")
            .with_brand("Sew"),
        EmbThread::from_rgb(250, 160, 96)
            .with_description("Hazel")
            .with_catalog_number("4")
            .with_brand("Sew"),
        EmbThread::from_rgb(92, 118, 73)
            .with_description("Green Dust")
            .with_catalog_number("5")
            .with_brand("Sew"),
        EmbThread::from_rgb(64, 192, 48)
            .with_description("Green")
            .with_catalog_number("6")
            .with_brand("Sew"),
        EmbThread::from_rgb(101, 194, 200)
            .with_description("Sky")
            .with_catalog_number("7")
            .with_brand("Sew"),
        EmbThread::from_rgb(172, 128, 190)
            .with_description("Purple")
            .with_catalog_number("8")
            .with_brand("Sew"),
        EmbThread::from_rgb(245, 188, 203)
            .with_description("Pink")
            .with_catalog_number("9")
            .with_brand("Sew"),
        EmbThread::from_rgb(255, 0, 0)
            .with_description("Red")
            .with_catalog_number("10")
            .with_brand("Sew"),
        EmbThread::from_rgb(192, 128, 0)
            .with_description("Brown")
            .with_catalog_number("11")
            .with_brand("Sew"),
        EmbThread::from_rgb(0, 0, 240)
            .with_description("Blue")
            .with_catalog_number("12")
            .with_brand("Sew"),
        EmbThread::from_rgb(228, 195, 93)
            .with_description("Gold")
            .with_catalog_number("13")
            .with_brand("Sew"),
        EmbThread::from_rgb(165, 42, 42)
            .with_description("Dark Brown")
            .with_catalog_number("14")
            .with_brand("Sew"),
        EmbThread::from_rgb(213, 176, 212)
            .with_description("Pale Violet")
            .with_catalog_number("15")
            .with_brand("Sew"),
        EmbThread::from_rgb(252, 242, 148)
            .with_description("Pale Yellow")
            .with_catalog_number("16")
            .with_brand("Sew"),
        EmbThread::from_rgb(240, 208, 192)
            .with_description("Pale Pink")
            .with_catalog_number("17")
            .with_brand("Sew"),
        EmbThread::from_rgb(255, 192, 0)
            .with_description("Peach")
            .with_catalog_number("18")
            .with_brand("Sew"),
        EmbThread::from_rgb(201, 164, 128)
            .with_description("Beige")
            .with_catalog_number("19")
            .with_brand("Sew"),
        EmbThread::from_rgb(155, 61, 75)
            .with_description("Wine Red")
            .with_catalog_number("20")
            .with_brand("Sew"),
        EmbThread::from_rgb(160, 184, 204)
            .with_description("Pale Sky")
            .with_catalog_number("21")
            .with_brand("Sew"),
        EmbThread::from_rgb(127, 194, 28)
            .with_description("Yellow Green")
            .with_catalog_number("22")
            .with_brand("Sew"),
        EmbThread::from_rgb(185, 185, 185)
            .with_description("Silver Grey")
            .with_catalog_number("23")
            .with_brand("Sew"),
        EmbThread::from_rgb(160, 160, 160)
            .with_description("Grey")
            .with_catalog_number("24")
            .with_brand("Sew"),
        EmbThread::from_rgb(152, 214, 189)
            .with_description("Pale Aqua")
            .with_catalog_number("25")
            .with_brand("Sew"),
        EmbThread::from_rgb(184, 240, 240)
            .with_description("Baby Blue")
            .with_catalog_number("26")
            .with_brand("Sew"),
        EmbThread::from_rgb(54, 139, 160)
            .with_description("Powder Blue")
            .with_catalog_number("27")
            .with_brand("Sew"),
        EmbThread::from_rgb(79, 131, 171)
            .with_description("Bright Blue")
            .with_catalog_number("28")
            .with_brand("Sew"),
        EmbThread::from_rgb(56, 106, 145)
            .with_description("Slate Blue")
            .with_catalog_number("29")
            .with_brand("Sew"),
        EmbThread::from_rgb(0, 32, 107)
            .with_description("Nave Blue")
            .with_catalog_number("30")
            .with_brand("Sew"),
        EmbThread::from_rgb(229, 197, 202)
            .with_description("Salmon Pink")
            .with_catalog_number("31")
            .with_brand("Sew"),
        EmbThread::from_rgb(249, 103, 107)
            .with_description("Coral")
            .with_catalog_number("32")
            .with_brand("Sew"),
        EmbThread::from_rgb(227, 49, 31)
            .with_description("Burnt Orange")
            .with_catalog_number("33")
            .with_brand("Sew"),
        EmbThread::from_rgb(226, 161, 136)
            .with_description("Cinnamon")
            .with_catalog_number("34")
            .with_brand("Sew"),
        EmbThread::from_rgb(181, 148, 116)
            .with_description("Umber")
            .with_catalog_number("35")
            .with_brand("Sew"),
        EmbThread::from_rgb(228, 207, 153)
            .with_description("Blonde")
            .with_catalog_number("36")
            .with_brand("Sew"),
        EmbThread::from_rgb(225, 203, 0)
            .with_description("Sunflower")
            .with_catalog_number("37")
            .with_brand("Sew"),
        EmbThread::from_rgb(225, 173, 212)
            .with_description("Orchid Pink")
            .with_catalog_number("38")
            .with_brand("Sew"),
        EmbThread::from_rgb(195, 0, 126)
            .with_description("Peony Purple")
            .with_catalog_number("39")
            .with_brand("Sew"),
        EmbThread::from_rgb(128, 0, 75)
            .with_description("Burgundy")
            .with_catalog_number("40")
            .with_brand("Sew"),
        EmbThread::from_rgb(160, 96, 176)
            .with_description("Royal Purple")
            .with_catalog_number("41")
            .with_brand("Sew"),
        EmbThread::from_rgb(192, 64, 32)
            .with_description("Cardinal Red")
            .with_catalog_number("42")
            .with_brand("Sew"),
        EmbThread::from_rgb(202, 224, 192)
            .with_description("Opal Green")
            .with_catalog_number("43")
            .with_brand("Sew"),
        EmbThread::from_rgb(137, 152, 86)
            .with_description("Moss Green")
            .with_catalog_number("44")
            .with_brand("Sew"),
        EmbThread::from_rgb(0, 170, 0)
            .with_description("Meadow Green")
            .with_catalog_number("45")
            .with_brand("Sew"),
        EmbThread::from_rgb(33, 138, 33)
            .with_description("Dark Green")
            .with_catalog_number("46")
            .with_brand("Sew"),
        EmbThread::from_rgb(93, 174, 148)
            .with_description("Aquamarine")
            .with_catalog_number("47")
            .with_brand("Sew"),
        EmbThread::from_rgb(76, 191, 143)
            .with_description("Emerald Green")
            .with_catalog_number("48")
            .with_brand("Sew"),
        EmbThread::from_rgb(0, 119, 114)
            .with_description("Peacock Green")
            .with_catalog_number("49")
            .with_brand("Sew"),
        EmbThread::from_rgb(112, 112, 112)
            .with_description("Dark Grey")
            .with_catalog_number("50")
            .with_brand("Sew"),
        EmbThread::from_rgb(242, 255, 255)
            .with_description("Ivory White")
            .with_catalog_number("51")
            .with_brand("Sew"),
        EmbThread::from_rgb(177, 88, 24)
            .with_description("Hazel")
            .with_catalog_number("52")
            .with_brand("Sew"),
        EmbThread::from_rgb(203, 138, 7)
            .with_description("Toast")
            .with_catalog_number("53")
            .with_brand("Sew"),
        EmbThread::from_rgb(247, 146, 123)
            .with_description("Salmon")
            .with_catalog_number("54")
            .with_brand("Sew"),
        EmbThread::from_rgb(152, 105, 45)
            .with_description("Cocoa Brown")
            .with_catalog_number("55")
            .with_brand("Sew"),
        EmbThread::from_rgb(162, 113, 72)
            .with_description("Sienna")
            .with_catalog_number("56")
            .with_brand("Sew"),
        EmbThread::from_rgb(123, 85, 74)
            .with_description("Sepia")
            .with_catalog_number("57")
            .with_brand("Sew"),
        EmbThread::from_rgb(79, 57, 70)
            .with_description("Dark Sepia")
            .with_catalog_number("58")
            .with_brand("Sew"),
        EmbThread::from_rgb(82, 58, 151)
            .with_description("Violet Blue")
            .with_catalog_number("59")
            .with_brand("Sew"),
        EmbThread::from_rgb(0, 0, 160)
            .with_description("Blue Ink")
            .with_catalog_number("60")
            .with_brand("Sew"),
        EmbThread::from_rgb(0, 150, 222)
            .with_description("Solar Blue")
            .with_catalog_number("61")
            .with_brand("Sew"),
        EmbThread::from_rgb(178, 221, 83)
            .with_description("Green Dust")
            .with_catalog_number("62")
            .with_brand("Sew"),
        EmbThread::from_rgb(250, 143, 187)
            .with_description("Crimson")
            .with_catalog_number("63")
            .with_brand("Sew"),
        EmbThread::from_rgb(222, 100, 158)
            .with_description("Floral Pink")
            .with_catalog_number("64")
            .with_brand("Sew"),
        EmbThread::from_rgb(181, 80, 102)
            .with_description("Wine")
            .with_catalog_number("65")
            .with_brand("Sew"),
        EmbThread::from_rgb(94, 87, 71)
            .with_description("Olive Drab")
            .with_catalog_number("66")
            .with_brand("Sew"),
        EmbThread::from_rgb(76, 136, 31)
            .with_description("Meadow")
            .with_catalog_number("67")
            .with_brand("Sew"),
        EmbThread::from_rgb(228, 220, 121)
            .with_description("Canary Yellow")
            .with_catalog_number("68")
            .with_brand("Sew"),
        EmbThread::from_rgb(203, 138, 26)
            .with_description("Toast")
            .with_catalog_number("69")
            .with_brand("Sew"),
        EmbThread::from_rgb(198, 170, 66)
            .with_description("Beige")
            .with_catalog_number("70")
            .with_brand("Sew"),
        EmbThread::from_rgb(236, 176, 44)
            .with_description("Honey Dew")
            .with_catalog_number("71")
            .with_brand("Sew"),
        EmbThread::from_rgb(248, 128, 64)
            .with_description("Tangerine")
            .with_catalog_number("72")
            .with_brand("Sew"),
        EmbThread::from_rgb(255, 229, 5)
            .with_description("Ocean Blue")
            .with_catalog_number("73")
            .with_brand("Sew"),
        EmbThread::from_rgb(250, 122, 122)
            .with_description("Sepia")
            .with_catalog_number("74")
            .with_brand("Sew"),
        EmbThread::from_rgb(107, 224, 0)
            .with_description("Royal Purple")
            .with_catalog_number("75")
            .with_brand("Sew"),
        EmbThread::from_rgb(56, 108, 174)
            .with_description("Yellow Ocher")
            .with_catalog_number("76")
            .with_brand("Sew"),
        EmbThread::from_rgb(208, 186, 176)
            .with_description("Beige Grey")
            .with_catalog_number("77")
            .with_brand("Sew"),
        EmbThread::from_rgb(227, 190, 129)
            .with_description("Bamboo")
            .with_catalog_number("78")
            .with_brand("Sew"),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sew_palette_size() {
        let palette = get_thread_set();
        assert_eq!(palette.len(), 79);
    }

    #[test]
    fn test_sew_first_thread() {
        let palette = get_thread_set();
        let unknown = &palette[0];
        assert_eq!(unknown.red(), 0);
        assert_eq!(unknown.green(), 0);
        assert_eq!(unknown.blue(), 0);
    }

    #[test]
    fn test_sew_branded_threads() {
        let palette = get_thread_set();
        for thread in palette {
            assert_eq!(thread.brand, Some("Sew".to_string()));
        }
    }
}
