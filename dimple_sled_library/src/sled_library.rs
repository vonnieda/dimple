use dimple_core::{library::{Library, LibraryEntity}, image_cache::ImageCache, model::Artist};

use image::codecs::qoi;
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

    pub fn get_artist(&self, id: &str) -> Option<Artist> {
        if id.is_empty() {
            return None
        }
        self.artists
            .get(id)
            .ok() // TODO log the error?
            .and_then(|v| serde_json::from_slice(&v.unwrap()).ok())
    }

    pub fn get_artist_by_mbid(&self, mbid: Option<String>) -> Option<Artist> {
        mbid.as_ref()?;
        // TODO slow
        self.artists()
            .find(|a| a.mbid == mbid)
    }

    pub fn set_artist(&self, a: &Artist) {
        assert!(!a.id.is_empty());
        serde_json::to_vec(a)
            .ok()
            .and_then(|json| self.artists.insert(a.id.clone(), json).ok());
    }

    // /// Ah, maybe merge is a trait of Artist, etc.?
    // fn merge_artists(a: &Artist, b: &Artist) -> Artist {

    // }
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
                let (k, v) = t.unwrap();
                serde_json::from_slice(&v).unwrap()
            })
            .collect();

        Box::new(artists.into_iter())
    }

    // fn list<T: dimple_core::library::LibraryEnt + 'static>(&self) -> Box<dyn Iterator<Item = T>> {
    //     Box::new(std::iter::empty())
    // }
}
