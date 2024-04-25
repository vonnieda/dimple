use std::io::Cursor;

use dimple_core_macro::ModelSupport;
use image::DynamicImage;
use image::ImageOutputFormat;
use serde::Deserialize;
use serde::Serialize;

/// A model for storing an image in Dimple. Not Image because too overloaded.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, Default, ModelSupport)]
pub struct Picture {
    pub key: Option<String>,
    compressed_image: Vec<u8>,
}

impl Picture {
    pub fn new(image: &DynamicImage) -> Self {
        let mut pic = Self::default();
        pic.set_image(image);
        pic
    }

    pub fn set_image(&mut self, image: &DynamicImage) {
        let mut cursor = Cursor::new(&mut self.compressed_image);
        image.write_to(&mut cursor, ImageOutputFormat::Png).unwrap()
    }

    pub fn get_image(&self) -> DynamicImage {
        image::load_from_memory(&self.compressed_image).unwrap()
    }
}
