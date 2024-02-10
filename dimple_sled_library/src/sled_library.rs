use std::borrow::Borrow;

use dimple_core::{collection::Collection, image_cache::ImageCache, model::Artist};
use dimple_core::model::{Model, Recording, RecordingSource, Release, ReleaseGroup};

use image::{DynamicImage, EncodableLayout};
use serde::{Deserialize, Serialize};

use sled::{IVec, Tree};

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
    sources: Tree,
    _audio: Tree,
}

#[derive(Deserialize, Debug, Serialize)]
pub struct SledLibraryConfig {
    pub path: String,
}

impl SledLibrary {
    pub fn new(path: &str) -> Self {
        let db = sled::open(path).unwrap();
        let release_groups = db.open_tree("release_groups").unwrap();
        let releases = db.open_tree("releases").unwrap();
        let images = db.open_tree("images").unwrap();
        let audio = db.open_tree("audio").unwrap();
        let artists = db.open_tree("artists").unwrap();
        let recordings = db.open_tree("recordings").unwrap();
        let sources = db.open_tree("sources").unwrap();
        Self { 
            path: path.to_string(),
            release_groups,
            releases,
            artists,
            images: ImageCache::new(images),
            recordings,
            sources,
            _audio: audio,
        }
    }

    /// Creates and/or updates the model in storage, along with relations. 
    /// If the model already exists by key, the existing one is read, the new
    /// one is merged into it, and the result is saved. If with_relations_to
    /// is included, a relation is also created and/or updated between the two
    /// models.
    /// 
    /// For instance, merge(ReleaseGroup, Artist) will merge the ReleaseGroup
    /// into storage and create a relation between the ReleaseGroup and Artist
    /// such that subsequently calling list(ReleaseGroup, Artist) will return
    /// a Vec containing the merged ReleaseGroup.
    pub fn merge(&self, model: &Model, with_relation_to: Option<&Model>) -> anyhow::Result<()> {
        fn insert<T>(tree: &Tree, key: &str, value: T) -> anyhow::Result<()>
            where T: Serialize {

            // TODO lookup the old and merge it and return that instead
            // let old = 

            let json = serde_json::to_string(&value)?;
            let _ = tree.insert(key, &*json)?;
            Ok(())
        }
    
        match model {
            Model::Artist(a) => insert(&self.artists, &a.key, a),
            Model::ReleaseGroup(a) => insert(&self.release_groups, &a.key, a),
            Model::Release(a) => insert(&self.releases, &a.key, a),
            Model::Recording(a) => insert(&self.recordings, &a.key, a),
            _ => todo!()
        }
    }

    pub fn set_image(&self, entity: &Model, dyn_image: &DynamicImage) {
        self.images.insert(&entity.key(), dyn_image);
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

    fn list(&self, of_type: &Model, related_to: Option<&Model>) -> Box<dyn Iterator<Item = Model>> {
        fn deserialize<'a, T>(v: &'a IVec) -> T where T: Deserialize<'a> {
            serde_json::from_slice(v.borrow()).unwrap()
        }

        match (of_type, related_to) {
            (Model::Artist(_), None) => {
                let iter = self.artists.iter()
                    .map(|t| deserialize(&t.unwrap().1))
                    .map(Model::Artist);
                Box::new(iter)
            },
            (Model::Recording(_), None) => {
                let iter = self.recordings.iter()
                    .map(|t| deserialize(&t.unwrap().1))
                    .map(Model::Recording);
                Box::new(iter)
            },
            (Model::ReleaseGroup(_), Some(Model::Artist(a))) => {
                // TODO this is temp code that doesn't work cause it's not filtering.
                let iter = self.release_groups.iter()
                    .map(|t| deserialize(&t.unwrap().1))
                    .map(Model::ReleaseGroup);
                Box::new(iter)
            },
            (Model::Release(_), Some(Model::Artist(a))) => {
                // TODO this is temp code that doesn't work cause it's not filtering.
                let iter = self.releases.iter()
                    .map(|t| deserialize(&t.unwrap().1))
                    .map(Model::Release);
                Box::new(iter)
            },
            (Model::Recording(_), Some(Model::Release(a))) => {
                // TODO this is temp code that doesn't work cause it's not filtering.
                let iter = self.recordings.iter()
                    .map(|t| deserialize(&t.unwrap().1))
                    .map(Model::Recording);
                Box::new(iter)
            },
            (Model::RecordingSource(_), Some(Model::Recording(a))) => {
                // TODO this is temp code that doesn't work cause it's not filtering.
                let iter = self.sources.iter()
                    .map(|t| deserialize(&t.unwrap().1))
                    .map(Model::RecordingSource);
                Box::new(iter)
            },
            _ => todo!("{:?} {:?}", of_type, related_to),
        }
    }

    fn fetch(&self, entity: &Model) -> Option<Model> {
        fn get<T>(tree: &Tree, key: &str) -> Option<T> where T: for<'a> Deserialize<'a> {
            let value = tree.get(key).ok()??;
            let bytes = value.as_bytes();
            let json = String::from_utf8(bytes.into()).ok()?;
            serde_json::from_str(&json).ok()?
        }
    
        match entity {
            Model::Artist(a) => {
                get::<Artist>(&self.artists, &a.key).map(|a| a.entity())
            },
            Model::ReleaseGroup(r) => {
                get::<ReleaseGroup>(&self.release_groups, &r.key).map(|a| a.entity())
            },
            Model::Release(r) => {
                get::<Release>(&self.releases, &r.key).map(|a| a.entity())
            },
            Model::Recording(r) => {
                get::<Recording>(&self.recordings, &r.key).map(|a| a.entity())
            },
            Model::RecordingSource(r) => {
                get::<RecordingSource>(&self.sources, &r.key).map(|a| a.entity())
            },
            Model::Genre(_) => todo!(),
        }        
    }

    fn image(&self, entity: &Model) -> Option<DynamicImage> {
        self.images.get_original(&entity.key())
    }
}
