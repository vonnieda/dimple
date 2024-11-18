use std::io::Cursor;

use dimple_core_macro::ModelSupport;
use image::imageops::FilterType;
use image::DynamicImage;
use serde::Deserialize;
use serde::Serialize;
use image::ImageFormat;
use fast_image_resize::Resizer;


// https://fanart.tv/music-fanart/
#[derive(Clone, Debug, Serialize, Deserialize, Eq, PartialEq, Hash, Default)]
pub enum DimageKind {
    #[default]
    Album,
    Artist,
    Background,
    Banner,
    ClearLogo,
}

/// A model for storing an image in Dimple. Not Image because too overloaded.
#[derive(Clone, Debug, Serialize, Deserialize, Default, PartialEq, Eq, Hash, ModelSupport)]
pub struct Dimage {
    pub key: Option<String>,
    pub data: Vec<u8>,
    pub kind: DimageKind,
    pub width: u32,
    pub height: u32,
    // An tiny (4x4) image meant to be used as a scaled up placeholder.
    pub placeholder_data: Vec<u8>,
}

impl Dimage {
    pub fn new(image: &DynamicImage) -> Self {
        let mut pic = Self::default();
        pic.set_image(image);
        pic
    }

    pub fn set_image(&mut self, image: &DynamicImage) {
        let mut cursor = Cursor::new(&mut self.data);
        image.write_to(&mut cursor, ImageFormat::Png).unwrap();
        self.width = image.width();
        self.height = image.height();

        let thumb = resize(image, 4, 4);
        let mut cursor = Cursor::new(&mut self.placeholder_data);
        thumb.write_to(&mut cursor, ImageFormat::Png).unwrap();
    }

    pub fn get_image(&self) -> DynamicImage {
        image::load_from_memory(&self.data).unwrap()
    }

    pub fn get_placeholder(&self, width: u32, height: u32)  -> DynamicImage {
        let image = image::load_from_memory(&self.placeholder_data).unwrap();
        image.resize(width, height, FilterType::Gaussian)
    }
}

pub fn resize(image: &DynamicImage, width: u32, height: u32) -> DynamicImage {
    let src_image = image;

    let mut dst_image = DynamicImage::new(width, height, 
        src_image.color());

    let mut resizer = Resizer::new();
    resizer.resize(src_image, &mut dst_image, None).unwrap();

    dst_image
}