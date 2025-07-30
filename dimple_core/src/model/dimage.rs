use std::{fmt::Debug, io::Cursor};

use image::imageops::FilterType;
use image::DynamicImage;
use image::ImageFormat;
use fast_image_resize::Resizer;
use serde::{Deserialize, Serialize};
use sha2::{Digest as _, Sha256};

/// A model for storing an image in Dimple. Not Image because too overloaded.
#[derive(Clone, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Dimage {
    pub id: Option<String>,

    pub kind: Option<DimageKind>,
    pub width: u32,
    pub height: u32,
    pub png_thumbnail: Vec<u8>,
    pub png_data: Vec<u8>,
    pub sha256: String,
}

impl Debug for Dimage {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Dimage")
            .field("id", &self.id)
            .field("kind", &self.kind)
            .field("width", &self.width)
            .field("height", &self.height)
            .field("png_thumbnail.len()", &self.png_thumbnail.len())
            .field("png_data.len()", &self.png_data.len())
            .field("sha256", &self.sha256)
            .finish()
    }
}

impl Dimage {
    pub fn new(image: &DynamicImage) -> Self {
        let mut pic = Self::default();
        pic.set_image(image);
        pic
    }

    pub fn set_image(&mut self, image: &DynamicImage) {
        let mut cursor = Cursor::new(&mut self.png_data);
        image.write_to(&mut cursor, ImageFormat::Png).unwrap();
        self.width = image.width();
        self.height = image.height();

        let thumb = resize(image, 4, 4);
        let mut cursor = Cursor::new(&mut self.png_thumbnail);
        thumb.write_to(&mut cursor, ImageFormat::Png).unwrap();

        self.sha256 = calculate_sha256(&self.png_data);
    }

    pub fn get_image(&self) -> DynamicImage {
        image::load_from_memory(&self.png_data).unwrap()
    }

    pub fn get_thumbnail(&self, width: u32, height: u32)  -> DynamicImage {
        let image = image::load_from_memory(&self.png_thumbnail).unwrap();
        image.resize(width, height, FilterType::Gaussian)
    }
}

fn calculate_sha256(data: &Vec<u8>) -> String {
    let mut hasher = Sha256::new();
    hasher.update(data);
    let result = hasher.finalize();
    format!("{:x}", result)
}            

pub fn resize(image: &DynamicImage, width: u32, height: u32) -> DynamicImage {
    let src_image = image;

    let mut dst_image = DynamicImage::new(width, height, 
        src_image.color());

    let mut resizer = Resizer::new();
    resizer.resize(src_image, &mut dst_image, None).unwrap();

    dst_image
}


/// This list is based on and gives thanks to:
// https://wiki.fanart.tv/ImageTypes/Music/hdmusiclogo/
// https://fanart.tv/music-fanart/
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, Hash, Default)]
pub enum DimageKind {
    #[default]
    MusicArtistThumb, // 1000x1000
    MusicHdClearLogo, // 800x310
    MusicAlbumCover, // 1000x1000
    MusicCdArt, // 1000x1000
    MusicArtistBackground, // 1920x1080
    MusicBanner, // 1000x185
    MusicRecordLabel, // 400x270
}

impl From<&DynamicImage> for Dimage {
    fn from(value: &DynamicImage) -> Self {
        Dimage::new(value)
    }
}

#[cfg(test)]
mod tests {
    use crate::{library::Library};

    use super::Dimage;

    #[test]
    fn library_crud() {
        let library = Library::open_memory();
        let mut dimage = Dimage::default();
        let dymage = image::open("tests/data/smol.png").unwrap();
        dimage.set_image(&dymage);
        let dimage = library.save(&dimage);
        assert!(dimage.id.is_some());
        assert!(dimage.png_data.len() > 0);
        assert!(dimage.png_thumbnail.len() > 0);
        assert_eq!(dimage.width, 21);
        assert_eq!(dimage.height, 20);
        assert!(dimage.get_image().width() == dimage.width);
        assert!(dimage.get_image().height() == dimage.height);
        assert!(dimage.get_thumbnail(4, 4).width() == 4);
        assert!(dimage.get_thumbnail(4, 4).height() == 4);
    }
}