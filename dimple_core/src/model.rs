use serde::Deserialize;
use serde::Serialize;

// Notes as I think about the data model
// How would I search for eminem, get artist, album, and track results, 
// merge those into the local, and push the changes up to the UI.

#[derive(Clone, Debug, Serialize, Deserialize, Default, PartialEq, Eq)]
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
        if let Some(artist) = self.artists.first() {
            return artist.name.clone();
        }
        "".to_string()
    }
}

/// Loosely modeled on the MusicBrainz Artist entity
/// https://musicbrainz.org/doc/Artist
/// https://picard-docs.musicbrainz.org/en/appendices/tag_mapping.html
#[derive(Default, Clone, Debug, Serialize, Deserialize)]
pub struct Artist {
    pub url: String,
    pub name: String,
    pub art: Vec<Image>,
    #[serde(default)]
    pub genres: Vec<Genre>,
} 

impl Eq for Artist {}

impl PartialEq for Artist {
    fn eq(&self, other: &Self) -> bool {
        self.url == other.url
    }
}

impl std::hash::Hash for Artist {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.url.hash(state);
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
    pub url: String,
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

