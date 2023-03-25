use std::{sync::Arc, fmt::Debug};

use image::DynamicImage;

pub mod local;
pub mod image_cache;
pub mod navidrome;
// pub mod directory_library;

pub trait Library {
    fn releases(&self) -> Vec<Arc<Release>>;

    fn merge_release(&self, _release: &Release) -> Result<(), String> {
        todo!();
    }
}

#[derive(Default, Clone)]
pub struct Release {
    // TODO Remove after converting to trait. Implementations should handle
    // their own IDs. 
    pub id: String,
    pub title: String,
    pub artist: Option<String>,
    pub cover_art: Option<Arc<dyn Image>>,
    pub genre: Option<String>,
    pub tracks: Vec<Arc<Track>>,
}

pub trait Image {
    fn scaled(&self, width: u32, height: u32) -> Option<DynamicImage>;
    fn original(&self) -> Option<DynamicImage>;
}

#[derive(Default, Clone)]
pub struct Track {
    pub title: String,
    pub stream: Option<Arc<dyn Stream>>,
    // pub artists: Vec<Arc<Artist>>,
}

pub trait Stream {
    // TODO length?
    // TODO size?
    // This is just goes away and Track becomes a trait with a function for this.
    fn stream(&self) -> Vec<u8>;
}

#[derive(Default, Clone)]
pub struct Genre {
    pub name: String,
    pub cover_art: Option<Arc<dyn Image>>,
}

impl Debug for Track {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Track").field("title", &self.title).finish()
    }
}

// #[derive(Default, Clone, Debug)]
// pub struct Artist {
//     pub id: String,
//     pub name: String,
//     // pub cover_art: Option<Arc<dyn ScaledImage>>,
//     releases: Vec<Arc<Release>>,
// }

