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

    pub fn load_images(&self, mut release: Release) -> Release {
        let art = release.art.par_iter().map(|image| {
            let mut image = image.clone();
            image.original = self.images.get_original(&image.url).unwrap();
            debug!("Loaded image {}x{} from {}", 
                image.original.width(), 
                image.original.height(), 
                image.url);
            image
        })
        .collect::<Vec<Image>>();
        release.art = art;
        release
    }
}

impl Library for LocalMusicLibrary {
    fn releases(&self) -> Result<Vec<Release>, String> {
        let releases = self.releases
            .iter()
            .par_bridge()
            // Load JSON and parse into Releases.
            .map(|kv| {
                // TODO error handling
                let (_key, value) = kv.unwrap();
                return serde_json::from_slice(&value).unwrap();
            })
            .map(|release: Release| {
                self.load_images(release)
            })
            .collect::<Vec<Release>>();
        Ok(releases)
    }

    fn merge_release(self: &Self, release: &Release) -> Result<(), String> {
        // Store Release art
        for image in &release.art {
            let url = &image.url;
            let image = &image.original;
            debug!("Storing image for {} at {}", release.title, url);
            self.images.insert(url, image);
        }

        // Store Release
        let json = serde_json::to_vec(release).unwrap();
        self.releases.insert(&release.url, json).unwrap();
        Ok(())
    }
}
