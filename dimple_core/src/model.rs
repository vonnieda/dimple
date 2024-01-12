use serde::Deserialize;
use serde::Serialize;

/// References
/// https://musicbrainz.org/doc/Artist
/// https://picard-docs.musicbrainz.org/en/appendices/tag_mapping.html
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Default)]
pub struct Artist {
    pub mb: MusicBrainzArtist,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Default)]
pub struct MusicBrainzArtist {
    pub id: String,
    pub name: String,
    pub disambiguation: String,

    // TODO do we actually need options here?
    pub release_groups: Option<Vec<MusicbrainzReleaseGroup>>,
    pub relations: Option<Vec<MusicBrainzRelation>>,
    pub genres: Option<Vec<MusicBrainzGenre>>,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct MusicBrainzRelation {
    pub content: MusicBrainzRelationContent,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub enum MusicBrainzRelationContent {
    Url(MusicBrainzUrlRelation),
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Default)]
pub struct MusicBrainzUrlRelation {
    pub resource: String,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Default)]
pub struct MusicBrainzGenre {
    pub id: String,
    pub name: String,
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

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct Release {
    pub mb: MusicbrainzReleaseGroup,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct MusicbrainzReleaseGroup {
    pub id: String,
    pub title: String,
}

impl Release {
    pub fn mbid(&self) -> String {
        self.mb.id.clone()
    }

    pub fn title(&self) -> String {
        self.mb.title.clone()
    }
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

