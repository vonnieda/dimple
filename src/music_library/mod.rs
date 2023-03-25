use std::{sync::Arc, fmt::Debug};

use image::DynamicImage;

pub mod local;
pub mod image_cache;
// pub mod navidrome;
// pub mod directory_library;

pub trait MusicLibrary {
    /// All of the releases in the library. This function may block for a long
    /// time if resources need to be loaded from disk or network. 
    fn releases(&self) -> Vec<Arc<Release>>;

    /// Add or update a release into the library. For now, all metadata and
    /// audio data is copied. This includes all of the tracks in the release.
    /// TODO I think probably in the future we specify a range of tracks
    /// to download? Sometimes you might just want one song from a release -
    /// like in the case of a song being included in a playlist.
    fn merge_release(&self, _release: &Release) -> Result<(), String> {
        todo!();
    }
}

#[derive(Default, Clone)]
pub struct MemoryMusicLibrary {
    releases: Vec<Arc<Release>>,
}

impl MusicLibrary for MemoryMusicLibrary {
    fn releases(&self) -> Vec<Arc<Release>> {
        return self.releases.clone();
    }

    fn merge_release(&self, _release: &Release) -> Result<(), String> {
        Ok(())
    }
}

#[derive(Default, Clone)]
pub struct Release {
    // TODO think about removing this. I don't think I'm using it and it would
    // be better if implementations create their own so that there aren't conflicts.
    pub id: String,
    pub title: String,
    pub artist: Option<String>,
    pub cover_art: Option<Arc<dyn ScaledImage>>,
    pub genre: Option<String>,
    pub tracks: Vec<Arc<Track>>,
}

pub trait ScaledImage {
    fn scaled(&self, width: u32, height: u32) -> Option<DynamicImage>;
    fn original(&self) -> Option<DynamicImage>;
}

// #[derive(Default, Clone, Debug)]
// pub struct Artist {
//     pub id: String,
//     pub name: String,
//     // pub cover_art: Option<Arc<dyn ScaledImage>>,
//     releases: Vec<Arc<Release>>,
// }

#[derive(Default, Clone)]
pub struct Track {
    pub title: String,
    pub stream: Option<Arc<dyn Stream>>,
    // pub artists: Vec<Arc<Artist>>,
}

impl Debug for Track {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Track").field("title", &self.title).finish()
    }
}

pub trait Stream {
    /// For now I think it makes sense to treat these as a stream of bytes of
    /// the original file, and then the player will handle decoding and all
    /// that. Maybe eventually it becomes a MediaSource from Symphonia or something.
    fn stream(&self) -> Vec<u8>;
}

// #[derive(Default, Clone)]
// pub struct Genre {
//     pub name: String,
//     pub cover_art: Option<DynamicImage>,
// }
