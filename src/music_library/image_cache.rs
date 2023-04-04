use std::io::Cursor;

use image::{imageops::FilterType, DynamicImage, ImageOutputFormat};
use sled::Tree;

/// Caches and scales downloaded images using Sled
#[derive(Clone)]
pub struct ImageCache {
    tree: Tree,
}

impl ImageCache {
    pub fn new(tree: Tree) -> Self {
        Self { tree }
    }

    /// Stores an image in the cache, making it available for fast retrieval
    /// at any size.
    pub fn insert(&self, id: &str, image: &DynamicImage) {
        // TODO note this will also need to rescale, or better, delete all
        // the previously scaled ones for the same id.
        self.save(id, image);
    }

    /// Get an image from the cache, scaled to the given size. If the image
    /// does not exist at that size the original is scaled. If there is no
    /// original for the id None is returned.
    pub fn get(&self, id: &str, width: u32, height: u32) -> Option<DynamicImage> {
        let scaled_id = format!("{}:{}x{}", id, width, height);
        if let Some(scaled) = self.load(&scaled_id) {
            return Some(scaled);
        }
        if let Some(original) = self.load(id) {
            let scaled = original.resize(width, height, FilterType::CatmullRom);
            self.save(&scaled_id, &scaled);
            return Some(scaled);
        }
        None
    }

    pub fn get_original(&self, id: &str) -> Option<DynamicImage> {
        if let Some(original) = self.load(id) {
            return Some(original);
        }
        None
    }

    fn save(&self, key: &str, image: &DynamicImage) {
        let mut bytes = Vec::new();
        if image
            .write_to(&mut Cursor::new(&mut bytes), ImageOutputFormat::Png)
            .is_ok() {
            self.tree.insert(key, bytes).unwrap();
        }
    }

    fn load(&self, key: &str) -> Option<DynamicImage> {
        if let Ok(Some(bytes)) = self.tree.get(key) {
            if let Ok(image) = image::load_from_memory(&bytes) {
                return Some(image);
            }
        }
        None
    }
}
