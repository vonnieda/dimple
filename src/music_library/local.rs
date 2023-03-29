use image::DynamicImage;
use log::{debug};

/// A local music library living in a directory. Stores data with Sled.
/// Faster than remote, but slower than memory. This is how the app stores
/// the combined library from all the remotes.

use rayon::prelude::*;
use sled::Tree;
use super::{Release, image_cache::ImageCache, Library, Image};

pub struct LocalMusicLibrary {
    releases: Tree,
    images: ImageCache,
    _audio: Tree,
}

impl LocalMusicLibrary {
    pub fn new(path: &str) -> Self {
        let db = sled::open(path).unwrap();
        let releases = db.open_tree("releases").unwrap();
        let images = db.open_tree("images").unwrap();
        let audio = db.open_tree("audio").unwrap();
        Self { 
            releases,
            images: ImageCache::new(images),
            _audio: audio,
        }
    }
}

impl Library for LocalMusicLibrary {
    fn releases(&self) -> Result<Vec<Release>, String> {
        let releases = self.releases
            .iter()
            .par_bridge()
            .map(|kv| {
                // TODO error handling
                let (_key, value) = kv.unwrap();
                return serde_json::from_slice(&value).unwrap();
            })
            .collect::<Vec<Release>>();
        Ok(releases)
    }

    fn image(&self, image: &Image) -> Result<DynamicImage, String> {
        self.images.get_original(&image.url)
            .map_or(Err("".to_string()), |image| Ok(image))
    }

    fn merge_release(self: &Self, library: &dyn Library, release: &Release) -> Result<(), String> {
        // Store Release art
        for image in &release.art {
            if let Ok(dynamic_image) = library.image(image) {
                let url = &image.url;
                debug!("Storing image for {} at {}", release.title, url);
                self.images.insert(url, &dynamic_image);
            }
        }

        // Store Release
        let json = serde_json::to_vec(release).unwrap();
        self.releases.insert(&release.url, json).unwrap();
        Ok(())
    }
}
