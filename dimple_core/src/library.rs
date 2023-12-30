use std::{sync::{mpsc::Receiver, Arc, RwLock}, collections::HashSet, fmt::Debug};

use image::DynamicImage;

use crate::model::{Release, Image, Track, Artist, Genre};

#[derive(Clone, Debug)]
pub enum SearchResult {
    Artist(Artist),
    Genre(Genre),
    Image(Image), // TODO no?
    Release(Release),
    Track(Track),
}

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
    fn search(&self, query: &str) -> impl Iterator<Item = SearchResult>;

    // /// Get the list of releases
    // // TODO Ay yo wouldn't just like a shared Vec with a RwLock be way
    // // easier? And just like work for the front end?
    // fn releases(&self) -> Receiver<Release>;

    // fn image(&self, _image: &Image) -> Result<DynamicImage, String>;

    // // TODO I wanted to have this return a Source but I couldn't figure out how.
    // fn stream(&self, _track: &Track) -> Result<Vec<u8>, String>;

    // fn merge_release(&self, _library: &dyn Library, _release: &Release) -> Result<(), String> {
    //     todo!();
    // }
}

// impl Debug for dyn Library {
//     fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // f.write_str(&self.name())
//     }
// }
