//! Husqvarna Viking HUS thread color palette
//!
//! Contains 29 predefined thread colors used in HUS embroidery files,
//! with RGB values, descriptions, and catalog numbers.

use crate::core::thread::EmbThread;

/// Get the HUS (Husqvarna Viking) thread palette
pub fn get_thread_set() -> Vec<EmbThread> {
    vec![
        EmbThread::from_string("#000000")
            .expect("HUS palette: invalid hex color #000000")
            .with_description("Black")
            .with_catalog_number("026")
            .with_brand("Hus"),
        EmbThread::from_string("#0000e7")
            .expect("HUS palette: invalid hex color #0000e7")
            .with_description("Blue")
            .with_catalog_number("005")
            .with_brand("Hus"),
        EmbThread::from_string("#00c600")
            .expect("HUS palette: invalid hex color #00c600")
            .with_description("Green")
            .with_catalog_number("002")
            .with_brand("Hus"),
        EmbThread::from_string("#ff0000")
            .expect("HUS palette: invalid hex color #ff0000")
            .with_description("Red")
            .with_catalog_number("014")
            .with_brand("Hus"),
        EmbThread::from_string("#840084")
            .expect("HUS palette: invalid hex color #840084")
            .with_description("Purple")
            .with_catalog_number("008")
            .with_brand("Hus"),
        EmbThread::from_string("#ffff00")
            .expect("HUS palette: invalid hex color #ffff00")
            .with_description("Yellow")
            .with_catalog_number("020")
            .with_brand("Hus"),
        EmbThread::from_string("#848484")
            .expect("HUS palette: invalid hex color #848484")
            .with_description("Grey")
            .with_catalog_number("024")
            .with_brand("Hus"),
        EmbThread::from_string("#8484e7")
            .expect("HUS palette: invalid hex color #8484e7")
            .with_description("Light Blue")
            .with_catalog_number("006")
            .with_brand("Hus"),
        EmbThread::from_string("#00ff84")
            .expect("HUS palette: invalid hex color #00ff84")
            .with_description("Light Green")
            .with_catalog_number("003")
            .with_brand("Hus"),
        EmbThread::from_string("#ff7b31")
            .expect("HUS palette: invalid hex color #ff7b31")
            .with_description("Orange")
            .with_catalog_number("017")
            .with_brand("Hus"),
        EmbThread::from_string("#ff8ca5")
            .expect("HUS palette: invalid hex color #ff8ca5")
            .with_description("Pink")
            .with_catalog_number("011")
            .with_brand("Hus"),
        EmbThread::from_string("#845200")
            .expect("HUS palette: invalid hex color #845200")
            .with_description("Brown")
            .with_catalog_number("028")
            .with_brand("Hus"),
        EmbThread::from_string("#ffffff")
            .expect("HUS palette: invalid hex color #ffffff")
            .with_description("White")
            .with_catalog_number("022")
            .with_brand("Hus"),
        EmbThread::from_string("#000084")
            .expect("HUS palette: invalid hex color #000084")
            .with_description("Dark Blue")
            .with_catalog_number("004")
            .with_brand("Hus"),
        EmbThread::from_string("#008400")
            .expect("HUS palette: invalid hex color #008400")
            .with_description("Dark Green")
            .with_catalog_number("001")
            .with_brand("Hus"),
        EmbThread::from_string("#7b0000")
            .expect("HUS palette: invalid hex color #7b0000")
            .with_description("Dark Red")
            .with_catalog_number("013")
            .with_brand("Hus"),
        EmbThread::from_string("#ff6384")
            .expect("HUS palette: invalid hex color #ff6384")
            .with_description("Light Red")
            .with_catalog_number("015")
            .with_brand("Hus"),
        EmbThread::from_string("#522952")
            .expect("HUS palette: invalid hex color #522952")
            .with_description("Dark Purple")
            .with_catalog_number("007")
            .with_brand("Hus"),
        EmbThread::from_string("#ff00ff")
            .expect("HUS palette: invalid hex color #ff00ff")
            .with_description("Light Purple")
            .with_catalog_number("009")
            .with_brand("Hus"),
        EmbThread::from_string("#ffde00")
            .expect("HUS palette: invalid hex color #ffde00")
            .with_description("Dark Yellow")
            .with_catalog_number("019")
            .with_brand("Hus"),
        EmbThread::from_string("#ffff9c")
            .expect("HUS palette: invalid hex color #ffff9c")
            .with_description("Light Yellow")
            .with_catalog_number("021")
            .with_brand("Hus"),
        EmbThread::from_string("#525252")
            .expect("HUS palette: invalid hex color #525252")
            .with_description("Dark Grey")
            .with_catalog_number("025")
            .with_brand("Hus"),
        EmbThread::from_string("#d6d6d6")
            .expect("HUS palette: invalid hex color #d6d6d6")
            .with_description("Light Grey")
            .with_catalog_number("023")
            .with_brand("Hus"),
        EmbThread::from_string("#ff5208")
            .expect("HUS palette: invalid hex color #ff5208")
            .with_description("Dark Orange")
            .with_catalog_number("016")
            .with_brand("Hus"),
        EmbThread::from_string("#ff9c5a")
            .expect("HUS palette: invalid hex color #ff9c5a")
            .with_description("Light Orange")
            .with_catalog_number("018")
            .with_brand("Hus"),
        EmbThread::from_string("#ff52b5")
            .expect("HUS palette: invalid hex color #ff52b5")
            .with_description("Dark Pink")
            .with_catalog_number("010")
            .with_brand("Hus"),
        EmbThread::from_string("#ffc6de")
            .expect("HUS palette: invalid hex color #ffc6de")
            .with_description("Light Pink")
            .with_catalog_number("012")
            .with_brand("Hus"),
        EmbThread::from_string("#523100")
            .expect("HUS palette: invalid hex color #523100")
            .with_description("Dark Brown")
            .with_catalog_number("027")
            .with_brand("Hus"),
        EmbThread::from_string("#b5a584")
            .expect("HUS palette: invalid hex color #b5a584")
            .with_description("Light Brown")
            .with_catalog_number("029")
            .with_brand("Hus"),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hus_palette_size() {
        let palette = get_thread_set();
        assert_eq!(palette.len(), 29);
    }

    #[test]
    fn test_hus_first_thread() {
        let palette = get_thread_set();
        let black = &palette[0];
        assert_eq!(black.red(), 0);
        assert_eq!(black.green(), 0);
        assert_eq!(black.blue(), 0);
    }

    #[test]
    fn test_hus_branded_threads() {
        let palette = get_thread_set();
        for thread in palette {
            assert_eq!(thread.brand, Some("Hus".to_string()));
        }
    }
}
