use std::collections::HashSet;

use serde::Deserialize;
use serde::Serialize;

pub mod artist;
pub mod genre;
pub mod media_file;
pub mod medium;
pub mod playlist;
pub mod recording_source;
pub mod recording;
pub mod release_group;
pub mod release;
pub mod track;

pub use artist::Artist;
pub use genre::Genre;
pub use media_file::MediaFile;
pub use medium::Medium;
pub use playlist::Playlist;
pub use recording_source::RecordingSource;
pub use recording::Recording;
pub use release_group::ReleaseGroup;
pub use release::Release;
pub use track::Track;

pub trait Entity {
    fn key(&self) -> Option<String>;
    fn set_key(&mut self, key: Option<String>);
    fn entity(&self) -> Entities;

    fn known_ids(&self) -> HashSet<KnownId> {
        Default::default()
    }

    fn mbid(&self) -> Option<String> {
        self.known_ids().iter().find_map(|id| match id {
            KnownId::MusicBrainzId(mbid) => Some(mbid.to_string()),
            _ => None,
        })
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, Eq, PartialEq)]
pub enum Entities {
    Artist(Artist),
    ReleaseGroup(ReleaseGroup),
    Release(Release),
    Track(Track),
    Medium(Medium),
    MediaFile(MediaFile),
    Recording(Recording),
    RecordingSource(RecordingSource),
    Genre(Genre),
}

impl Entities {
    // TODO belongs in db impl
    pub fn type_name(&self) -> &str {
        match self {
            Entities::Artist(_) => "artist",
            Entities::ReleaseGroup(_) => "release_group",
            Entities::Release(_) => "release",
            Entities::Recording(_) => "recording",
            Entities::RecordingSource(_) => "recording_source",
            Entities::Genre(_) => "genre",
            Entities::MediaFile(_) => "media_file",
            Entities::Track(_) => "track",
            Entities::Medium(_) => "medium",
        }
    }

    pub fn key(&self) -> Option<String> {
        match self {
            Entities::Artist(a) => a.key.clone(),
            Entities::ReleaseGroup(r) => r.key.clone(),
            Entities::Release(r) => r.key.clone(),
            Entities::Recording(r) => r.key.clone(),
            Entities::RecordingSource(r) => r.key.clone(),
            Entities::Genre(g) => g.key.clone(),
            Entities::MediaFile(f) => Some(f.key.clone()),
            Entities::Track(t) => t.key.clone(),
            Entities::Medium(m) => m.key.clone(),
        }
    }

    pub fn set_key(&mut self, key: Option<String>) {
        match self {
            Entities::Artist(m) => m.key = key,
            Entities::ReleaseGroup(m) => m.key = key,
            Entities::Release(m) => m.key = key,
            Entities::Genre(m) => m.key = key,
            Entities::Recording(m) => m.key = key,
            Entities::RecordingSource(m) => m.key = key,
            Entities::MediaFile(m) => m.key = key.unwrap(),
            Entities::Track(t) => t.key = key,
            Entities::Medium(m) => m.key = key,
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, Default, Hash)]
pub struct UrlRelation {
    pub id: String,
    pub resource: String,
}

// I think this becomes a struct 
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum KnownId {
    MusicBrainzId(String),
    DiscogsId(String),
    LastFmId(String),
    WikidataId(String),
    SpotifyId,
    DeezerId,
    TidalId,
    YouTubeId,
    ItunesStoreId,
    AppleMusicId, // TODO same as above?
    QobuzId,
    BandcampUrl,
    SoundCloud,

    // https://musicbrainz.org/doc/Barcode
    Barcode,

    // https://musicbrainz.org/doc/ISRC
    ISRC,

    // https://musicbrainz.org/doc/ASIN
    ASIN,

    AcoustId,
    AcoustIdFingerprint,
}

/// An attribution for a piece of data. Indicates where the data was sourced,
/// who owns it, and under what license it is being used.
/// DimpleAttribution {
///     text: "Wikipedia content provided under the terms of the Creative Commons BY-SA license",
///     url: "https://en.wikipedia.org/wiki/Brutus_%28Belgian_band%29"
///     license: "CC-BY-SA"
///     copyright_holder: "WikiCommons"
/// }
/// TODO read https://wiki.creativecommons.org/wiki/Best_practices_for_attribution#This_is_a_great_attribution
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Default)]
pub struct Attributed<T> {
    pub value: T,

    pub text: String,
    pub url: String,
    pub license: String,
    pub copyright_holder: String,
}

