use crate::model::{Release, Track, Artist, Genre};

#[derive(Clone, Debug)]
pub enum LibraryEntity {
    Artist(Artist),
    Genre(Genre),
    Release(Release),
    Track(Track),
}

pub trait Library: Send + Sync {
    /// Get a user friendly display name for the Library.
    fn name(&self) -> String;

    /// Search the library for entities that match the query string. How the
    /// search query is interpreted is left up to the implementation. In
    /// general it should return, at least, matching Artists, Releases,
    /// Genres, and Tracks.
    fn search(&self, query: &str) -> Box<dyn Iterator<Item = LibraryEntity>>;

    /// Fetch a complete record for the given entity. The Library implementation
    /// can choose to use any appropriate fields in the entity as the selection
    /// criteria. Typically mbid or another id will be used.
    fn fetch(&self, _entity: &LibraryEntity) -> Option<LibraryEntity> {
        todo!()
    }

    // TODO Explore fn list(LibraryEntity....type?) or maybe fn list<T>
    fn artists(&self) -> Box<dyn Iterator<Item = Artist>>;

    // TODO Eventually this will allow access to more image types.
    fn image(&self, entity: &LibraryEntity) -> Option<image::DynamicImage>;

    // fn list<T: LibraryEnt + 'static>(&self) -> Box<dyn Iterator<Item = T>>;
}
