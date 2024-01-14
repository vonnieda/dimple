use colored::Colorize;

use crate::model::{DimpleReleaseGroup, DimpleTrack, DimpleArtist, DimpleGenre};

// TODO I think this enum's time is up. Replace with Traits.
#[derive(Clone, Debug)]
pub enum LibraryEntity {
    Artist(DimpleArtist),
    Genre(DimpleGenre),
    Release(DimpleReleaseGroup),
    Track(DimpleTrack),
}

impl LibraryEntity {
    pub fn mbid(&self) -> String {
        match self {
            LibraryEntity::Artist(a) => a.id.clone(),
            LibraryEntity::Genre(_) => todo!(),
            LibraryEntity::Release(r) => r.id.clone(),
            LibraryEntity::Track(_) => todo!(),
        }
    }

    pub fn name(&self) -> String {
        match self {
            LibraryEntity::Artist(a) => a.name.clone(),
            LibraryEntity::Genre(_) => todo!(),
            LibraryEntity::Release(r) => r.title.clone(),
            LibraryEntity::Track(_) => todo!(),
        }
    }
}

pub trait Library: Send + Sync {
    /// Get a user friendly display name for the Library.
    fn name(&self) -> String {
        todo!()
    }

    /// Search the library for entities that match the query string. How the
    /// search query is interpreted is left up to the implementation. In
    /// general it should return, at least, matching Artists, Releases,
    /// Genres, and Tracks.
    fn search(&self, _query: &str) -> Box<dyn Iterator<Item = LibraryEntity>> {
        Box::new(vec![].into_iter())
    }

    /// Fetch a complete record for the given entity. The Library implementation
    /// can choose to use any appropriate fields in the entity as the selection
    /// criteria. Typically mbid or another id will be used. The Library should
    /// return only those fields that are fetched, it should not return fields
    /// that come from the source entity.
    fn fetch(&self, _entity: &LibraryEntity) -> Option<LibraryEntity> {
        None
    }

    // TODO Explore fn list<LibraryEntity>() or e.g. list<Artist>(page)
    fn artists(&self) -> Box<dyn Iterator<Item = DimpleArtist>> {
        Box::new(vec![].into_iter())
    }

    // TODO Eventually this will allow access to more image types.
    fn image(&self, _entity: &LibraryEntity) -> Option<image::DynamicImage> {
        None
    }

    // fn list<T: LibraryEnt + 'static>(&self) -> Box<dyn Iterator<Item = T>>;
}

pub struct LibrarySupport {
}

impl LibrarySupport {
    pub fn log_request(library: &dyn Library, url: &str) {
        log::info!("{} {}", library.name().blue(), url.yellow());
    }
}