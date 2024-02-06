use std::time::Instant;

use colored::Colorize;

use crate::model::{DimpleReleaseGroup, DimpleArtist, DimpleGenre, DimpleRelease, DimpleRecording, DimpleRecordingSource};

#[derive(Clone, Debug)]
pub enum Model {
    Artist(DimpleArtist),
    Genre(DimpleGenre),
    ReleaseGroup(DimpleReleaseGroup),
    Release(DimpleRelease),
    Recording(DimpleRecording),
}

impl Model {
    pub fn id(&self) -> String {
        match self {
            Model::Artist(a) => a.id.clone(),
            Model::ReleaseGroup(r) => r.id.clone(),
            Model::Release(r) => r.id.clone(),
            Model::Recording(r) => r.id.clone(),
            Model::Genre(g) => g.name.clone(),
        }
    }

    pub fn name(&self) -> String {
        match self {
            Model::Artist(a) => a.name.clone(),
            Model::ReleaseGroup(r) => r.title.clone(),
            Model::Release(r) => r.title.clone(),
            Model::Recording(r) => r.title.clone(),
            Model::Genre(g) => g.name.clone(),
        }
    }
}

pub trait Library: Send + Sync {
    /// Get a user friendly display name for the Library.
    fn name(&self) -> String;

    /// Search the library for entities that match the query string. How the
    /// search query is interpreted is left up to the implementation. In
    /// general it should return, at least, matching Artists, Releases,
    /// Genres, and Tracks.
    fn search(&self, _query: &str) -> Box<dyn Iterator<Item = Model>> {
        Box::new(vec![].into_iter())
    }

    /// Fetch a complete record for the given entity. The Library implementation
    /// can choose to use any appropriate fields in the entity as the selection
    /// criteria. Typically mbid or another id will be used. The Library should
    /// return only those fields that are fetched, it should not return fields
    /// that come from the source entity.
    fn fetch(&self, _entity: &Model) -> Option<Model> {
        None
    }

    // fn artists(&self) -> Box<dyn Iterator<Item = DimpleArtist>> {
    //     Box::new(vec![].into_iter())
    // }

    // TODO Eventually this will allow access to more image types.
    fn image(&self, _entity: &Model) -> Option<image::DynamicImage> {
        None
    }

    /// Return zero or more sources for the given recording. 
    fn sources(&self, _recording: &DimpleRecording) -> Box<dyn Iterator<Item = DimpleRecordingSource>> {
        Box::new(vec![].into_iter())
    }

    /// Returns an iterator over all of the objects of the type of the entity.
    /// This is used to list all artists, releases, release-groups, etc.
    fn list(&self, _entity: &Model) -> Box<dyn Iterator<Item = Model>> {
        Box::new(vec![].into_iter())
    }
}

pub struct LibrarySupport {
}

pub struct RequestToken {
    library_name: String,
    start_time: Instant,
    url: String,
}

impl LibrarySupport {
    pub fn start_request(library: &dyn Library, url: &str) -> RequestToken {
        RequestToken {
            library_name: library.name().to_owned(),
            start_time: Instant::now(),
            url: url.to_owned(),
        }
    }

    pub fn end_request(token: RequestToken, status_code: Option<u16>, length: Option<u64>) {
        log::info!("{} [{:?}] {}ms {:?} {}", 
            token.library_name.blue(), 
            status_code, 
            token.start_time.elapsed().as_millis(), 
            length,
            token.url.yellow());
    }
}