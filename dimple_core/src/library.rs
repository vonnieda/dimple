use crate::model::{Release, Track, Artist, Genre};

#[derive(Clone, Debug)]
pub enum SearchResult {
    Artist(Artist),
    Genre(Genre),
    Release(Release),
    Track(Track),
}

#[derive(Clone, Debug)]
pub enum KnownId {
    DimpleId(String),
    MBID(String),
}

pub trait Library: Send + Sync {
    /// Get a user friendly display name for the Library.
    fn name(&self) -> String;

    /// Search the library for entities that match the query string. How the
    /// search query is interpreted is left up to the implementation. In
    /// general it should return, at least, matching Artists, Releases,
    /// Genres, and Tracks.
    fn search(&self, query: &str) -> Box<dyn Iterator<Item = SearchResult>>;
}
