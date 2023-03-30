use crossbeam::channel::{unbounded, Receiver};
use image::DynamicImage;
use log::{debug};

/// A local music library living in a directory. Stores data with Sled.
/// Faster than remote, but slower than memory. This is how the app stores
/// the combined library from all the remotes.

use sled::Tree;
use super::{Release, image_cache::ImageCache, Library, Image};

pub struct LocalLibrary {
    releases: Tree,
    images: ImageCache,
    _audio: Tree,
}

impl LocalLibrary {
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

impl Library for LocalLibrary {
    fn name(&self) -> String {
        return "Local".to_string();
    }
    
    fn releases(&self) -> Receiver<Release> {
        let (sender, receiver) = unbounded::<Release>();
        let releases = self.releases.iter();
        std::thread::spawn(move || {
            releases
                .map(|kv| {
                    // TODO error handling
                    let (_key, value) = kv.unwrap();
                    return serde_json::from_slice(&value).unwrap();
                })
                .for_each(|release| {
                    sender.send(release).unwrap();
                });
        });

        return receiver;
    }

    fn image(&self, image: &Image) -> Result<DynamicImage, String> {
        self.images.get_original(&image.url)
            .map_or(Err("".to_string()), |image| Ok(image))
    }

    fn stream(&self, _track: &super::Track, _sink: &rodio::Sink) -> Result<(), String> {
        Err("Not yet implemented".to_string())
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
