pub mod navidrome;
pub mod example;

use egui_extras::RetainedImage;
use std::sync::Arc;

// #[derive(Default)]
// pub struct MusicLibrary {
//     pub releases: Vec<Release>,
//     pub playlists: Vec<Playlist>,
//     pub images: Vec<Arc<RetainedImage>>,
// }

pub trait MusicLibrary {
    fn ping(self: &Self) -> Result<(), String>;

    fn releases(self: &Self) -> &[Release];

    // pub releases: Vec<Release>,
    // pub playlists: Vec<Playlist>,
    // pub images: Vec<Arc<RetainedImage>>,
    // fn playlists() -> Vec<Playlist>;
}

#[derive(Default)]
pub struct Playlist {
    pub name: String,
    pub items: Vec<Release>,
    pub cover_image: Option<Arc<RetainedImage>>,
}

#[derive(Default)]
pub struct Release {
    pub title: String,
    pub artist: String,
    pub release_year: u32,
    pub cover_image: Option<Arc<RetainedImage>>,
}

#[derive(Default)]
pub struct Artist {
    pub name: String,
    pub cover_image: Option<Arc<RetainedImage>>,
}

#[derive(Default)]
pub struct Track {
    pub title: String,
    pub lyrics: Lyrics,
    pub cover_image: Option<Arc<RetainedImage>>,
}

#[derive(Default)]
pub struct Lyrics {
    pub lyrics: String,
}

