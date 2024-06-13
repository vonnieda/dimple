use std::io::Cursor;

use dimple_core_macro::ModelSupport;
use image::DynamicImage;
use serde::Deserialize;
use serde::Serialize;
use image::ImageFormat;

/// A model for storing an image in Dimple. Not Image because too overloaded.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, Default, ModelSupport)]
pub struct Dimage {
    pub key: Option<String>,

    // TODO replace with blob, unserialized.
    pub data: Vec<u8>,
}
impl Dimage {
    pub fn new(image: &DynamicImage) -> Self {
        let mut pic = Self::default();
        pic.set_image(image);
        pic
    }

    pub fn set_image(&mut self, image: &DynamicImage) {
        let mut cursor = Cursor::new(&mut self.data);
        image.write_to(&mut cursor, ImageFormat::Png).unwrap()
    }

    pub fn get_image(&self) -> DynamicImage {
        image::load_from_memory(&self.data).unwrap()
    }
}
