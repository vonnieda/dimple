use serde::Deserialize;
use serde::Serialize;

/// References
/// https://musicbrainz.org/doc/Artist
/// https://picard-docs.musicbrainz.org/en/appendices/tag_mapping.html

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Default)]
pub struct Artist {
    pub mb: musicbrainz_rs::entity::artist::Artist,
}

impl Artist {
    pub fn with_mbid(mbid: &str) -> Self {
        let mut a = Self::default();
        a.mb.id = mbid.to_string();
        a
    }

    pub fn mbid(&self) -> String {
        self.mb.id.clone()
    }

    pub fn name(&self) -> String {
        self.mb.name.clone()
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, Default, PartialEq, Eq)]
pub struct Release {
    // pub id: String,
    // pub mbid: String,
    // pub title: String,
    // #[serde(default)]
    // pub artists: Vec<Artist>,
    // #[serde(default)]
    // pub genres: Vec<Genre>,
    // #[serde(default)]
    // pub tracks: Vec<Track>,
}

// The Deezer version of a Track https://developers.deezer.com/api/track
// includes a detailed Artist object, but just one, and a detail album
// Object.
#[derive(Default, Debug, Clone, Serialize, Eq, Hash, PartialEq, Deserialize)]
pub struct Track {
    // pub url: String,
    // pub title: String,
    // #[serde(default)]
    // pub artists: Vec<Artist>,
    // #[serde(default)]
    // pub genres: Vec<Genre>,
}

#[derive(Default, Clone, Debug, PartialEq, Eq, Serialize, Deserialize, Hash)]
pub struct Genre {
    pub url: String,
    pub name: String,
}

#[derive(Default, Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Playlist {
    pub url: String,
    pub name: String,
}

