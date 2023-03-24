use std::sync::Arc;

use image::DynamicImage;

pub mod navidrome;
pub mod local;
pub mod image_cache;

pub trait MusicLibrary {
    /// All of the releases in the library. This function may block for a long
    /// time if resources need to be loaded from disk or network. 
    fn releases(&self) -> Vec<Arc<Release>>;

    /// Add or update a release into the library. Returns the merged release.
    /// If data in the existing release and the new one differ the new data
    /// are preferred.
    fn merge_release(&self, _release: &Release) -> Result<(), String> {
        Err("not implemented".to_string())
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

    fn merge_release(&self, release: &Release) -> Result<(), String> {
        Ok(())
    }
}

#[derive(Default, Clone)]
pub struct Release {
    pub id: String,
    pub title: String,
    pub artist: Option<String>,
    pub cover_art: Option<DynamicImage>,
    pub genre: Option<String>,
    pub tracks: Vec<Track>,
}

#[derive(Default, Clone)]
pub struct Artist {
    pub id: String,
    pub name: String,
    pub cover_art: Option<DynamicImage>,
    // releases: Vec<Release>,
}

#[derive(Default, Clone)]
pub struct Track {
    pub title: String,
    pub stream: Vec<u8>,
    pub artists: Vec<Artist>,
}

#[derive(Default, Clone)]
pub struct Genre {
    pub name: String,
    pub cover_art: Option<DynamicImage>,
}
