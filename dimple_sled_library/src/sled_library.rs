use dimple_core::{library::{Library, LibraryEntity}, image_cache::ImageCache, model::Artist};

use image::DynamicImage;
use serde::{Deserialize, Serialize};

use sled::Tree;

#[derive(Debug)]

/// A local music library living in a directory. Stores data with Sled.
/// Faster than remote, but slower than memory. This is how the app stores
/// the combined library from all the remotes. Object are serialized as JSON,
/// media is stored raw.
pub struct SledLibrary {
    _ulid: String,
    name: String,
    artists: Tree,
    _releases: Tree,
    _images: ImageCache,
    _audio: Tree,
}

#[derive(Deserialize, Debug, Serialize)]
pub struct SledLibraryConfig {
    pub ulid: String,
    pub name: String,
}

impl SledLibrary {
    pub fn new(ulid: &str, name: &str) -> Self {
        // TODO magic
        let db = sled::open(format!("data/{}", ulid)).unwrap();
        let releases = db.open_tree("releases").unwrap();
        let images = db.open_tree("images").unwrap();
        let audio = db.open_tree("audio").unwrap();
        let artists = db.open_tree("artists").unwrap();
        Self { 
            _ulid: String::from(ulid),
            name: String::from(name),
            _releases: releases,
            artists,
            _images: ImageCache::new(images),
            _audio: audio,
        }
    }

    pub fn get_artist_by_id(&self, id: &str) -> Option<Artist> {
        if id.is_empty() {
            return None
        }
        self.artists
            .get(id)
            .ok() // TODO log the error?
            .and_then(|v| {
                v.and_then(|ivec| {
                    serde_json::from_slice(&ivec).ok()
                })
            })
    }

    pub fn get_artist_by_mbid(&self, mbid: String) -> Option<Artist> {
        // TODO slow
        self.artists()
            .find(|a| a.mbid() == mbid)
    }

    pub fn set_artist(&self, a: &Artist) {
        assert!(!a.id().is_empty());
        serde_json::to_vec(a)
            .ok()
            .and_then(|json| self.artists.insert(a.id(), json).ok());
    }

    pub fn set_image(&self, entity: &LibraryEntity, dyn_image: &DynamicImage) {
        match entity {
            LibraryEntity::Artist(a) => {
                self._images.insert(&a.id(), dyn_image);
            },
            LibraryEntity::Genre(_) => todo!(),
            LibraryEntity::Release(_) => todo!(),
            LibraryEntity::Track(_) => todo!(),
        }
    }
}

impl From<SledLibraryConfig> for SledLibrary {
    fn from(config: SledLibraryConfig) -> Self {
        Self::new(&config.ulid, &config.name)
    }
}

impl Library for SledLibrary {
    fn name(&self) -> String {
        self.name.to_string()
    }

    fn search(&self, _query: &str) -> Box<dyn Iterator<Item = dimple_core::library::LibraryEntity>> {
        let v: Vec<LibraryEntity> = vec![];
        Box::new(v.into_iter())
    }    

    fn artists(&self) -> Box<dyn Iterator<Item = dimple_core::model::Artist>> {
        let artists: Vec<Artist> = self.artists.iter()
            .map(|t| {
                let (_k, v) = t.unwrap();
                serde_json::from_slice(&v).unwrap()
            })
            .collect();

        Box::new(artists.into_iter())
    }

    fn image(&self, entity: &LibraryEntity) -> Option<DynamicImage> {
        match entity {
            LibraryEntity::Artist(a) => {
                self._images.get_original(&a.id())
            },
            LibraryEntity::Genre(_) => todo!(),
            LibraryEntity::Release(_) => todo!(),
            LibraryEntity::Track(_) => todo!(),
        }
    }
}
