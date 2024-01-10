use dimple_core::{library::{Library, LibraryEntity}, image_cache::ImageCache, model::Artist};

use image::{DynamicImage, EncodableLayout};
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

    fn get_artist(&self, mbid: &str) -> Option<Artist> {
        assert!(!mbid.is_empty());
        self.artists.get(mbid).ok()?
            .and_then(|v| {
                let bytes = v.as_bytes();
                let json: String = String::from_utf8(bytes.into()).unwrap();
                serde_json::from_str(&json).unwrap()
            })
            .and_then(|a: Option<Artist>| {
                log::info!("fetched {:?}", a.clone().unwrap().mb.release_groups.map(|r| r.len()));
                a
            })
    }

    pub fn set_artist(&self, a: &Artist) {
        assert!(!a.mbid().is_empty());
        log::info!("storing {:?}", a.clone().mb.release_groups.map(|r| r.len()));
        serde_json::to_string_pretty(a)
            .map(|json| {
                self.artists.insert(a.mbid(), &*json).unwrap()
            })
            .unwrap();
    }

    pub fn set_image(&self, entity: &LibraryEntity, dyn_image: &DynamicImage) {
        self._images.insert(&entity.mbid(), dyn_image);
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
                let bytes = v.as_bytes();
                let json: String = String::from_utf8(bytes.into()).unwrap();
                serde_json::from_str(&json).unwrap()
            })
            .collect();

        Box::new(artists.into_iter())
    }

    fn fetch(&self, entity: &LibraryEntity) -> Option<LibraryEntity> {
        match entity {
            LibraryEntity::Artist(a) => {
                let a = self.get_artist(&a.mbid())?;
                Some(LibraryEntity::Artist(a))
            },
            LibraryEntity::Genre(_) => todo!(),
            LibraryEntity::Release(_) => todo!(),
            LibraryEntity::Track(_) => todo!(),
        }        
    }

    fn image(&self, entity: &LibraryEntity) -> Option<DynamicImage> {
        self._images.get_original(&entity.mbid())
    }
}
