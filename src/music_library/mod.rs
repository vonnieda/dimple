use std::sync::Arc;

use image::DynamicImage;

// pub mod navidrome;
pub mod local;
pub mod image_cache;
// pub mod directory_library;

pub trait MusicLibrary {
    /// All of the releases in the library. This function may block for a long
    /// time if resources need to be loaded from disk or network. 
    fn releases(&self) -> Vec<Arc<Release>>;

    /// Add or update a release into the library.
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

#[derive(Default, Clone, Debug)]
pub struct Track {
    pub title: String,
    // pub stream: Option<Arc<dyn Stream>>,
    // pub artists: Vec<Arc<Artist>>,
}

trait Stream {
    fn stream(&self) -> dyn Iterator<Item = u8>;
}

// #[derive(Default, Clone)]
// pub struct Genre {
//     pub name: String,
//     pub cover_art: Option<DynamicImage>,
// }
