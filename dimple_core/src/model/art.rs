use std::io::Cursor;

use dimple_core_macro::ModelSupport;
use image::DynamicImage;
use image::ImageOutputFormat;
use serde::Deserialize;
use serde::Serialize;

use super::Model;

// https://musicbrainz.org/doc/Artist
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, Default, ModelSupport)]
pub struct Art {
    pub key: Option<String>,
    compressed_image: Vec<u8>,
}

impl Art {
    pub fn set_image(&mut self, image: &DynamicImage) {
        let mut cursor = Cursor::new(&mut self.compressed_image);
        image.write_to(&mut cursor, ImageOutputFormat::Png).unwrap()
    }

    pub fn get_image(&self) -> DynamicImage {
        image::load_from_memory(&self.compressed_image).unwrap()
    }
}
