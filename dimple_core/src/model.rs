use serde::Deserialize;
use serde::Serialize;
use ulid::Ulid;

use crate::library::LibraryEnt;

/// References
/// https://musicbrainz.org/doc/Artist
/// https://picard-docs.musicbrainz.org/en/appendices/tag_mapping.html

#[derive(Clone, Debug, Serialize, Deserialize, Default, PartialEq, Eq)]
pub struct Release {
    pub id: String,
    pub mbid: String,
    pub title: String,
    pub artists: Vec<Artist>,
    pub art: Vec<Image>,
    pub genres: Vec<Genre>,
    pub tracks: Vec<Track>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Artist {
    pub id: String,    
    pub mbid: Option<String>,

    pub name: String,
    pub art: Vec<Image>,
    pub genres: Vec<Genre>,
}

impl Default for Artist {
    fn default() -> Self {
        Artist {
            id: Ulid::new().to_string(),
            mbid: None,

            name: "".to_string(),
            art: vec![],
            genres: vec![],
        }
    }
}

impl Eq for Artist {}

impl PartialEq for Artist {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl std::hash::Hash for Artist {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.id.hash(state);
    }
}

// The Deezer version of a Track https://developers.deezer.com/api/track
// includes a detailed Artist object, but just one, and a detail album
// Object.
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
    pub id: String,
}

// TODO Maybe just Artwork, or Art.
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

impl LibraryEnt for Artist {

}

impl LibraryEnt for Release {

}

impl LibraryEnt for Genre {

}

impl LibraryEnt for Image {

}

