use dimple_core::{library::{Library, LibraryEntity}, image_cache::ImageCache, model::DimpleArtist};

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
    pub images: ImageCache,
    _audio: Tree,
}

#[derive(Deserialize, Debug, Serialize)]
pub struct SledLibraryConfig {
    pub ulid: String,
    pub name: String,
}

impl SledLibrary {
    pub fn new(ulid: &str, name: &str) -> Self {
        // TODO magic root path, and remove ulid?
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
            images: ImageCache::new(images),
            _audio: audio,
        }
    }

    fn get_artist(&self, mbid: &str) -> Option<DimpleArtist> {
        assert!(!mbid.is_empty());
        self.artists.get(mbid).ok()?
            .and_then(|v| {
                let bytes = v.as_bytes();
                let json: String = String::from_utf8(bytes.into()).unwrap();
                serde_json::from_str(&json).unwrap()
            })
    }

    pub fn set_artist(&self, a: &DimpleArtist) {
        assert!(!a.id.is_empty());
        serde_json::to_string_pretty(a)
            .map(|json| {
                self.artists.insert(a.id.to_string(), &*json).unwrap()
            })
            .unwrap();
    }

    pub fn set_image(&self, entity: &LibraryEntity, dyn_image: &DynamicImage) {
        self.images.insert(&entity.mbid(), dyn_image);
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

    fn artists(&self) -> Box<dyn Iterator<Item = dimple_core::model::DimpleArtist>> {
        let artists: Vec<DimpleArtist> = self.artists.iter()
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
                let a = self.get_artist(&a.id)?;
                Some(LibraryEntity::Artist(a))
            },
            LibraryEntity::Genre(_) => todo!(),
            LibraryEntity::Release(_) => todo!(),
            LibraryEntity::Track(_) => todo!(),
        }        
    }

    fn image(&self, entity: &LibraryEntity) -> Option<DynamicImage> {
        self.images.get_original(&entity.mbid())
    }
}
