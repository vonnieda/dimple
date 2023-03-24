use std::sync::Arc;

use image::DynamicImage;
/// A local music library living in a directory. Stores data with Sled.
/// Faster than remote, but slower than memory. This is how the app stores
/// the combined library from all the remotes.

use rayon::prelude::*;
use serde::{Deserialize, Serialize};
use super::{Release, image_cache::ImageCache, MusicLibrary, ScaledImage};

pub struct LocalMusicLibrary {
    db: sled::Db,
    images: Arc<ImageCache>,
}

impl LocalMusicLibrary {
    pub fn new(path: &str) -> Self {
        let db = sled::open(path).unwrap();
        let images = ImageCache::new(db.open_tree("images").unwrap());
        Self { db, images: Arc::new(images) }
    }
}

#[derive(Clone)]
struct LocalScaledImage {
    images: Arc<ImageCache>,
    id: String,
}

impl ScaledImage for LocalScaledImage {
    fn scaled(&self, width: u32, height: u32) -> Option<DynamicImage> {
        self.images.get(&self.id, width, height)
    }

    fn original(&self) -> Option<DynamicImage> {
        self.images.get_original(&self.id)
    }
}

impl MusicLibrary for LocalMusicLibrary {
    fn releases(&self) -> Vec<Arc<Release>> {
        let internal_releases: Vec<InternalRelease> = self
            .db
            .open_tree("releases")
            .unwrap()
            .iter()
            .par_bridge()
            .map(|kv| {
                let (_key, bin) = kv.unwrap();
                bincode::deserialize(&bin[..]).unwrap()
            })
            .collect();

        // TODO I think this can be done further in parallel by doing the
        // deserialization here.
        // TODO removed the parallization here temporary cause I can't get it
        // to work with all my custom types.
        let releases: Vec<Arc<Release>> = internal_releases
            .iter()
            // .par_iter()
            .map(|internal_release| Arc::new(Release {
                id: internal_release.id.clone(),
                title: internal_release.title.clone(),
                artist: internal_release.artist.clone(),
                cover_art: match &internal_release.cover_image_id { 
                    Some(cover_art_id) => Some(Arc::new(LocalScaledImage { 
                        images: self.images.clone(),
                        id: cover_art_id.clone(), 
                    })),
                    None => None,
                },
                genre: internal_release.genre.clone(),
                tracks: Vec::new(),
            }))
            .collect();

        return releases;
    }

    fn merge_release(self: &Self, release: &Release) -> Result<(), String> {
        if let Ok(releases) = self.db.open_tree("releases") {
            // If there is cover art, store it.
            let cover_art_id = release.cover_art.as_ref()
                .map_or(None, |cover_art| cover_art.original())
                .map_or(None, |original| {
                    self.images.insert(&release.id, &original);
                    return Some(release.id.clone());
            });

            // Create serializable version.
            let internal_release = InternalRelease {
                id: release.id.clone(),
                title: release.title.clone(),
                artist: release.artist.clone(),
                cover_image_id: cover_art_id,
                genre: release.genre.clone(),
            };

            // Store the release.
            if let Ok(bin) = bincode::serialize(&internal_release) {
                releases.insert(&release.id, bin).expect("insert failed");
            }
        }
        // TODO return hydrated object
        return Ok(());
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
