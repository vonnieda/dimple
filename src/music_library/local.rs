use crossbeam::channel::{unbounded, Receiver};
use image::DynamicImage;
use log::{debug};

use serde::{Deserialize, Serialize};
/// A local music library living in a directory. Stores data with Sled.
/// Faster than remote, but slower than memory. This is how the app stores
/// the combined library from all the remotes.

use sled::Tree;
use super::{Release, image_cache::ImageCache, Library, Image};

pub struct LocalLibrary {
    _ulid: String,
    name: String,
    releases: Tree,
    images: ImageCache,
    _audio: Tree,
}

#[derive(Deserialize, Debug, Serialize)]
pub struct LocalConfig {
    pub ulid: String,
    pub name: String,
    pub site: String,
    pub username: String,
    pub password: String,
}

impl LocalLibrary {
    pub fn new(ulid: &str, name: &str) -> Self {
        let db = sled::open(format!("data/{}", ulid)).unwrap();
        let releases = db.open_tree("releases").unwrap();
        let images = db.open_tree("images").unwrap();
        let audio = db.open_tree("audio").unwrap();
        Self { 
            _ulid: String::from(ulid),
            name: String::from(name),
            releases,
            images: ImageCache::new(images),
            _audio: audio,
        }
    }

    pub fn from_config(config: LocalConfig) -> Self {
        Self::new(&config.ulid, &config.name)
    }
}

impl Library for LocalLibrary {
    fn name(&self) -> String {
        self.name.to_string()
    }
    
    fn releases(&self) -> Receiver<Release> {
        let (sender, receiver) = unbounded::<Release>();
        let releases = self.releases.iter();
        std::thread::spawn(move || {
            releases
                .map(|kv| {
                    // TODO error handling
                    let (_key, value) = kv.unwrap();
                    serde_json::from_slice(&value).unwrap()
                })
                .for_each(|release| {
                    sender.send(release).unwrap();
                });
        });

        receiver
    }

    fn image(&self, image: &Image) -> Result<DynamicImage, String> {
        self.images.get_original(&image.url)
            .map_or(Err("".to_string()), Ok)
    }

    fn stream(&self, _track: &super::Track, _sink: &rodio::Sink) -> Result<(), String> {
        Err("Not yet implemented".to_string())
    }

    fn merge_release(&self, library: &dyn Library, release: &Release) -> Result<(), String> {
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
