use dimple_core::{library::{Collection, Model}, image_cache::ImageCache, model::{DimpleArtist, DimpleReleaseGroup, DimpleRelease, DimpleRecording}};

use image::{DynamicImage, EncodableLayout};
use serde::{Deserialize, Serialize};

use sled::Tree;

#[derive(Debug)]

/// A local music library living in a directory. Stores data with Sled.
/// Faster than remote, but slower than memory. This is how the app stores
/// the combined library from all the remotes. Object are serialized as JSON,
/// media is stored raw.
pub struct SledLibrary {
    path: String,
    artists: Tree,
    release_groups: Tree,
    releases: Tree,
    pub images: ImageCache,
    recordings: Tree,
    _audio: Tree,
}

#[derive(Deserialize, Debug, Serialize)]
pub struct SledLibraryConfig {
    pub path: String,
}

impl SledLibrary {
    pub fn new(path: &str) -> Self {
        // TODO magic root path, and remove ulid?
        let db = sled::open(path).unwrap();
        let release_groups = db.open_tree("release_groups").unwrap();
        let releases = db.open_tree("releases").unwrap();
        let images = db.open_tree("images").unwrap();
        let audio = db.open_tree("audio").unwrap();
        let artists = db.open_tree("artists").unwrap();
        let recordings = db.open_tree("recordings").unwrap();
        Self { 
            path: path.to_string(),
            release_groups,
            releases,
            artists,
            images: ImageCache::new(images),
            recordings,
            _audio: audio,
        }
    }

    // TODO for now assuming storing does not modify the object, so no need
    // to return one. Might change. 
    pub fn store(&self, entity: &Model) {
        match entity {
            Model::Artist(a) => {
                self.set_artist(a)
            }
            Model::ReleaseGroup(r) => {
                self.set_release_group(r)
            }
            Model::Release(r) => {
                self.set_release(r)
            }
            Model::Recording(r) => {
                self.set_recording(r)
            }
            _ => todo!()
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
        serde_json::to_string(a)
            .map(|json| {
                self.artists.insert(a.id.to_string(), &*json).unwrap()
            })
            .unwrap();
    }

    fn get_release_group(&self, mbid: &str) -> Option<DimpleReleaseGroup> {
        assert!(!mbid.is_empty());
        self.release_groups.get(mbid).ok()?
            .and_then(|v| {
                let bytes = v.as_bytes();
                let json: String = String::from_utf8(bytes.into()).unwrap();
                serde_json::from_str(&json).unwrap()
            })
    }

    fn get_release(&self, mbid: &str) -> Option<DimpleRelease> {
        assert!(!mbid.is_empty());
        self.releases.get(mbid).ok()?
            .and_then(|v| {
                let bytes = v.as_bytes();
                let json: String = String::from_utf8(bytes.into()).unwrap();
                serde_json::from_str(&json).unwrap()
            })
    }

    fn get_recording(&self, mbid: &str) -> Option<DimpleRecording> {
        assert!(!mbid.is_empty());
        self.recordings.get(mbid).ok()?
            .and_then(|v| {
                let bytes = v.as_bytes();
                let json: String = String::from_utf8(bytes.into()).unwrap();
                serde_json::from_str(&json).unwrap()
            })
    }

    pub fn set_release_group(&self, a: &DimpleReleaseGroup) {
        assert!(!a.id.is_empty());
        serde_json::to_string(a)
            .map(|json| {
                self.release_groups.insert(a.id.to_string(), &*json).unwrap()
            })
            .unwrap();
    }

    pub fn set_release(&self, a: &DimpleRelease) {
        assert!(!a.id.is_empty());
        serde_json::to_string(a)
            .map(|json| {
                self.releases.insert(a.id.to_string(), &*json).unwrap()
            })
            .unwrap();
    }

    pub fn set_recording(&self, a: &DimpleRecording) {
        assert!(!a.id.is_empty());
        serde_json::to_string(a)
            .map(|json| {
                self.recordings.insert(a.id.to_string(), &*json).unwrap()
            })
            .unwrap();
    }

    pub fn set_image(&self, entity: &Model, dyn_image: &DynamicImage) {
        self.images.insert(&entity.id(), dyn_image);
    }
}

impl Collection for SledLibrary {
    fn name(&self) -> String {
        format!("SledLibrary({})", self.path)
    }

    fn search(&self, _query: &str) -> Box<dyn Iterator<Item = Model>> {
        let v: Vec<Model> = vec![];
        Box::new(v.into_iter())
    }    

    fn list(&self, entity: &Model) -> Box<dyn Iterator<Item = Model>> {
        let entities = match entity {
            Model::Artist(_) => {
                self.artists.iter()
                .map(|t| {
                    let (_k, v) = t.unwrap();
                    let bytes = v.as_bytes();
                    let json: String = String::from_utf8(bytes.into()).unwrap();
                    serde_json::from_str(&json).unwrap()
                })
                .map(Model::Artist)
                .collect()
            }
            _ => vec![],
        };

        Box::new(entities.into_iter())
    }

    fn fetch(&self, entity: &Model) -> Option<Model> {
        match entity {
            Model::Artist(a) => {
                let a = self.get_artist(&a.id)?;
                Some(Model::Artist(a))
            },
            Model::ReleaseGroup(r) => {
                let r = self.get_release_group(&r.id)?;
                Some(Model::ReleaseGroup(r))
            },
            Model::Release(r) => {
                let r = self.get_release(&r.id)?;
                Some(Model::Release(r))
            },
            Model::Recording(r) => {
                let r = self.get_recording(&r.id)?;
                Some(Model::Recording(r))
            },
            Model::Genre(_) => todo!(),
        }        
    }

    fn image(&self, entity: &Model) -> Option<DynamicImage> {
        self.images.get_original(&entity.id())
    }
}
