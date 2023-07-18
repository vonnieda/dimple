/// Objects all have a URL. The URL must uniquely identify that object and
/// it should, at least, have a scheme that matches the library it
/// came from. In other words, given the same library config, it should be
/// possible to re-load the object from the library with the same URL.

use std::{fmt::Debug, sync::mpsc::Receiver};

use image::DynamicImage;
use serde::{Serialize, Deserialize};

use self::{navidrome_library::NavidromeConfig, local_library::LocalConfig};

pub mod local_library;
pub mod image_cache;
pub mod navidrome_library;
pub mod memory_library;

pub trait Library: Send + Sync {
    fn name(&self) -> String;

    fn releases(&self) -> Receiver<Release>;

    fn image(&self, _image: &Image) -> Result<DynamicImage, String>;

    // TODO I wanted to have this return a Source but I couldn't figure out how.
    fn stream(&self, _track: &Track) -> Result<Vec<u8>, String>;

    fn merge_release(&self, _library: &dyn Library, _release: &Release) -> Result<(), String> {
        todo!();
    }
}

impl Debug for dyn Library {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.name())
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct Release {
    pub url: String,
    pub title: String,
    pub artists: Vec<Artist>,
    pub art: Vec<Image>,
    pub genres: Vec<Genre>,
    pub tracks: Vec<Track>,
}

impl Release {
    pub fn artist(&self) -> String {
        self.artists.first().unwrap().name.to_string()
    }
}

#[derive(Default, Clone, Debug, PartialEq, Eq, Serialize, Deserialize, Hash)]
pub struct Artist {
    pub url: String,
    pub name: String,
    pub art: Vec<Image>,
    #[serde(default)]
    pub genres: Vec<Genre>,
}

// TODO I think this is gonna need a way to get back to the release. Giving
// more credence to everything just having an ID and the vectors being
// vectors of IDs.
#[derive(Default, Debug, Clone, Serialize, Eq, Hash, PartialEq, Deserialize)]
pub struct Track {
    pub url: String,
    pub title: String,
    pub art: Vec<Image>,
    #[serde(default)]
    pub artists: Vec<Artist>,
    #[serde(default)]
    pub genres: Vec<Genre>,
}

#[derive(Default, Clone, Debug, PartialEq, Eq, Serialize, Deserialize, Hash)]
pub struct Genre {
    pub url: String,
    pub name: String,
    pub art: Vec<Image>,
}

#[derive(Default, Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Playlist {
    pub url: String,
    pub name: String,
    pub art: Vec<Image>,
}

#[derive(Default, Clone, Debug, PartialEq, Eq, Serialize, Deserialize, Hash)]
pub struct Image {
    pub url: String,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(tag = "type")]
pub enum LibraryConfig {
    Navidrome(NavidromeConfig),
    Local(LocalConfig),
}

pub trait HasArtwork {
    fn art(&self) -> Vec<Image>;
}

impl HasArtwork for Release {
    fn art(&self) -> Vec<Image> {
        self.art.clone()
    }
}

impl HasArtwork for Artist {
    fn art(&self) -> Vec<Image> {
        self.art.clone()
    }
}

impl HasArtwork for Genre {
    fn art(&self) -> Vec<Image> {
        self.art.clone()
    }
}

impl HasArtwork for Playlist {
    fn art(&self) -> Vec<Image> {
        self.art.clone()
    }
}

impl HasArtwork for Track {
    fn art(&self) -> Vec<Image> {
        self.art.clone()
    }
}

