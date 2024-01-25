use std::time::Instant;

use colored::Colorize;

use crate::model::{DimpleReleaseGroup, DimpleArtist, DimpleGenre, DimpleRelease, DimpleRecording, DimpleRecordingSource};

#[derive(Clone, Debug)]
pub enum DimpleEntity {
    Artist(DimpleArtist),
    Genre(DimpleGenre),
    ReleaseGroup(DimpleReleaseGroup),
    Release(DimpleRelease),
    Recording(DimpleRecording),
}

impl DimpleEntity {
    pub fn id(&self) -> String {
        match self {
            DimpleEntity::Artist(a) => a.id.clone(),
            DimpleEntity::ReleaseGroup(r) => r.id.clone(),
            DimpleEntity::Release(r) => r.id.clone(),
            DimpleEntity::Recording(r) => r.id.clone(),
            DimpleEntity::Genre(g) => g.name.clone(),
        }
    }

    pub fn name(&self) -> String {
        match self {
            DimpleEntity::Artist(a) => a.name.clone(),
            DimpleEntity::ReleaseGroup(r) => r.title.clone(),
            DimpleEntity::Release(r) => r.title.clone(),
            DimpleEntity::Recording(r) => r.title.clone(),
            DimpleEntity::Genre(g) => g.name.clone(),
        }
    }
}

// TODO This becomes Source, I think. Or Provider.
pub trait Library: Send + Sync {
    /// Get a user friendly display name for the Library.
    fn name(&self) -> String;

    /// Search the library for entities that match the query string. How the
    /// search query is interpreted is left up to the implementation. In
    /// general it should return, at least, matching Artists, Releases,
    /// Genres, and Tracks.
    fn search(&self, _query: &str) -> Box<dyn Iterator<Item = DimpleEntity>> {
        Box::new(vec![].into_iter())
    }

    /// Fetch a complete record for the given entity. The Library implementation
    /// can choose to use any appropriate fields in the entity as the selection
    /// criteria. Typically mbid or another id will be used. The Library should
    /// return only those fields that are fetched, it should not return fields
    /// that come from the source entity.
    fn fetch(&self, _entity: &DimpleEntity) -> Option<DimpleEntity> {
        None
    }

    // fn artists(&self) -> Box<dyn Iterator<Item = DimpleArtist>> {
    //     Box::new(vec![].into_iter())
    // }

    // TODO Eventually this will allow access to more image types.
    fn image(&self, _entity: &DimpleEntity) -> Option<image::DynamicImage> {
        None
    }

    /// Return zero or more sources for the given recording. 
    fn sources(&self, _recording: &DimpleRecording) -> Box<dyn Iterator<Item = DimpleRecordingSource>> {
        Box::new(vec![].into_iter())
    }

    /// Returns an iterator over all of the objects of the type of the entity.
    /// This is used to list all artists, releases, release-groups, etc.
    fn list(&self, _entity: &DimpleEntity) -> Box<dyn Iterator<Item = DimpleEntity>> {
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