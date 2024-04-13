use dimple_core_macro::ModelSupport;
use image::DynamicImage;
use image::ImageDecoder;
use image::ImageEncoder;
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
    pub fn set_image(&mut self, image: Option<&DynamicImage>) {
    }

    pub fn get_image(&self) -> Option<DynamicImage> {
        if self.compressed_image.is_none() {
            return None
        }
        ImageDecoder::
    }
}

