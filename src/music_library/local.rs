use std::io::Cursor;

use image::{ImageOutputFormat, DynamicImage, imageops::FilterType};
use serde::{Deserialize, Serialize};
use sled::Tree;

/// A local music library living in a directory. Stores metadata in TBD
/// and media (art and music) in TBD. Currently exploring Sled.
/// 
/// This music library is how the app stores its local cache of all other
/// libraries and for that reason it is considered the reference implementation.

use crate::MusicLibrary;

use super::Release;

pub struct LocalMusicLibrary {
    db: sled::Db,
    images: ImageCache,
}

impl LocalMusicLibrary {
    pub fn new(path: &str) -> Self {
        let db = sled::open(path).unwrap();
        let images = ImageCache::new(db.open_tree("images").unwrap());
        Self {
            db,
            images,
        }
    }
}

impl MusicLibrary for LocalMusicLibrary {
    fn releases(self: &Self) -> Vec<Release> {
        if let Ok(releases) = self.db.open_tree("releases") {
            let mut results = Vec::new();
            for release in releases.iter() {
                // Grab the serialized data
                let (_key, bin) = release.unwrap();

                // Deserialize
                let internal: InternalRelease = bincode::deserialize(&bin[..]).unwrap();

                // Create release from InternalRelease and pull in image.
                let release = Release {
                    id: internal.id.clone(),
                    title: internal.title.clone(),
                    artist: internal.artist.clone(),
                    cover_image: self.images.get(&internal.id, 200, 200),
                    genre: internal.genre,
                };
                results.push(release);
            }
            return results;
        }
        
        Vec::new()
    }

    fn merge_release(self: &Self, release: &Release) -> Result<Release, String> {
        if let Ok(releases) = self.db.open_tree("releases") {
            // Store the original cover image
            if let Some(cover_image) = &release.cover_image {
                // TODO error checking
                self.images.insert(&release.id, &cover_image);
            }

            // Create a serializable release
            let internal:InternalRelease = InternalRelease::from(release);

            // Store the release
            if let Ok(bin) = bincode::serialize(&internal) {
                releases.insert(&release.id, bin).expect("insert failed");
            }
        }
        // TODO return hydrated object
        return Ok(Release::default());
    }
}

impl From<&Release> for InternalRelease {
    fn from(release: &Release) -> Self {
        InternalRelease {
            id: release.id.clone(),
            title: release.title.clone(),
            artist: release.artist.clone(),
            cover_image_id: Some(release.id.clone()),
            genre: release.genre.clone(),
        }
    }
}

#[derive(Serialize, Deserialize, PartialEq, Debug)]
struct InternalRelease {
    id: String,
    title: String,
    artist: Option<String>,
    cover_image_id: Option<String>,
    genre: Option<String>,
}

/// Caches and scales downloaded images using Sled
pub struct ImageCache {
    tree: Tree,
}

impl ImageCache {
    fn new(tree: Tree) -> Self {
        Self {
            tree,
        }
    }

    /// Stores an image in the cache, making it available for fast retrieval
    /// at any size. 
    pub fn insert(self: &Self, id: &str, image: &DynamicImage) {
        // TODO note this will also need to rescale, or better, delete all
        // the previously scaled ones for the same id.
        self._set(id, image);
    }

    /// Get an image from the cache, scaled to the given size. If the image
    /// does not exist at that size the original is scaled. If there is no
    /// original for the id None is returned. 
    pub fn get(self: &Self, id: &str, width: u32, height: u32) -> Option<DynamicImage> {
        let scaled_id = format!("{}:{}x{}", id, width, height);
        if let Some(scaled) = self._get(&scaled_id) {
            return Some(scaled);
        }
        if let Some(original) = self._get(id) {
            let scaled = original.resize(width, height, FilterType::CatmullRom);
            self._set(&scaled_id, &scaled);
            return Some(scaled);
        }
        None
    }

    fn _set(self: &Self, key: &str, image: &DynamicImage) {
        let mut bytes = Vec::new();
        if image.write_to(&mut Cursor::new(&mut bytes), ImageOutputFormat::Png).is_ok() {
            self.tree.insert(key, bytes).unwrap();
        }
    }

    fn _get(self: &Self, key: &str) -> Option<DynamicImage> {
        if let Ok(value) = self.tree.get(key) {
            if let Some(bytes) = value {
                if let Ok(image) = image::load_from_memory(&bytes) {
                    return Some(image);
                }
            }
        }
        None
    }
}