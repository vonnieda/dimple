use std::collections::HashSet;
use std::time::Instant;

use serde::Deserialize;
use serde::Serialize;

use crate::collection::Collection;

/// References
/// https://musicbrainz.org/doc/Artist
/// https://picard-docs.musicbrainz.org/en/appendices/tag_mapping.html

// https://musicbrainz.org/doc/Artist
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct Artist {
    pub key: Option<String>,
    pub name: Option<String>,
    pub source_ids: HashSet<String>,
    pub known_ids: HashSet<KnownId>,
    pub disambiguation: Option<String>,
    pub summary: Option<String>,
    pub links: HashSet<String>,

    pub country: Option<String>,
}

// https://musicbrainz.org/doc/ReleaseGroup
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(default)]
pub struct ReleaseGroup {
    pub key: Option<String>,
    pub title: Option<String>,
    pub source_ids: HashSet<String>,
    pub known_ids: HashSet<KnownId>,
    pub disambiguation: Option<String>,
    pub summary: Option<String>,
    pub links: HashSet<String>,

    pub first_release_date: Option<String>,
    pub primary_type: Option<String>,
}




// https://musicbrainz.org/doc/Release
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(default)]
pub struct Release {
    pub key: Option<String>,
    pub title: Option<String>,
    pub source_ids: HashSet<String>,
    pub known_ids: HashSet<KnownId>,
    pub disambiguation: Option<String>,
    pub summary: Option<String>,
    pub links: HashSet<String>,

    pub barcode: Option<String>,
    pub country: Option<String>,
    pub date: Option<String>, // TODO should be chronos, probably.
    pub packaging: Option<String>,
    pub status: Option<String>,
}




// https://musicbrainz.org/doc/Medium
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct Medium {
    pub title: String,

    pub disc_count: u32,
    pub format: String,
    pub position: u32,
    pub track_count: u32,
    pub tracks: Vec<Track>,
}




// https://musicbrainz.org/doc/Track
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct Track {
    pub title: String,

    pub length: u32,
    pub number: String,
    pub position: u32,
    pub recording: Recording,
    pub sources:  Vec<RecordingSource>,
}




// https://musicbrainz.org/doc/Recording
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct Recording {
    pub key: Option<String>,
    pub title: Option<String>,
    pub source_ids: HashSet<String>,
    pub known_ids: HashSet<KnownId>,
    pub disambiguation: Option<String>,
    pub summary: Option<String>,
    pub links: HashSet<String>,

    pub annotation: Option<String>,
    pub length: Option<u32>,

    pub isrcs: HashSet<String>,
}


#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum RecordingFormat {
    MP3,
    FLAC,
    M4A,
}


#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct RecordingSource {
    pub key: Option<String>,
    pub source_ids: HashSet<String>,
    pub known_ids: HashSet<KnownId>,
    pub format: Option<RecordingFormat>,
    pub extension: Option<String>,
}




#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct Genre {
    pub key: Option<String>,
    pub name: Option<String>,
    pub source_ids: HashSet<String>,
    pub known_ids: HashSet<KnownId>,
    pub disambiguation: Option<String>,
    pub summary: Option<String>,
    pub links: HashSet<String>,
}


#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct MediaFile {
    pub key: Option<String>,
    pub url: String,
    // pub created_at: Instant,
    // pub modified_at: Instant,

    pub artist: Option<String>,
    pub album: Option<String>,
    pub album_artist: Option<String>,
    pub title: Option<String>,
    pub genre: Option<String>,

    pub recording_mbid: Option<String>,
    pub release_track_mbid: Option<String>,
    pub album_mbid: Option<String>,
    pub artist_mbid: Option<String>,
    pub album_artist_mbid: Option<String>,
    pub mb_album_type: Option<String>,
    pub mb_album_comment: Option<String>,
}


#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct Playlist {
    pub name: String,
}




#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, Default, Hash)]
pub struct UrlRelation {
    pub id: String,
    pub resource: String,
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




impl Artist {
    pub fn list(col: &dyn Collection) -> Box<dyn Iterator<Item = Artist>> {
        let iter = col.list(&Artist::default().entity(), None)
            .map(|m| match m {
                Entities::Artist(a) => a,
                _ => panic!(),
            });
        Box::new(iter)
    }

    pub fn get(key: &str, lib: &dyn Collection) -> Option<Self> {
        let ent = Artist {
            key: Some(key.to_string()),
            ..Default::default()
        }.entity();
        match lib.fetch(&ent) {
            Some(Entities::Artist(a)) => Some(a),
            _ => todo!()
        }
    }

    pub fn entity(&self) -> Entities {
        Entities::Artist(self.clone())
    }

    pub fn search(query: &str, lib: &dyn Collection) -> Box<dyn Iterator<Item = Artist>> {
        let iter = lib.search(query)
            .filter_map(|m| match m {
                Entities::Artist(a) => Some(a),
                _ => None,
            });
        Box::new(iter)
    }

    pub fn release_groups(&self, lib: &dyn Collection) -> Box<dyn Iterator<Item = ReleaseGroup>> {
        let iter = lib.list(&ReleaseGroup::default().entity(), Some(&self.entity()));
        let iter = iter.map(|r| match r {
            Entities::ReleaseGroup(r) => r,
            _ => panic!(),
        }); 
        Box::new(iter)    
    }

    pub fn releases(&self, lib: &dyn Collection) -> Box<dyn Iterator<Item = Release>> {
        let iter = lib.list(&Release::default().entity(), Some(&self.entity()));
        let iter = iter.map(|r| match r {
            Entities::Release(r) => r,
            _ => panic!(),
        }); 
        Box::new(iter)    
    }

    pub fn recordings(&self, lib: &dyn Collection) -> Box<dyn Iterator<Item = Recording>> {
        let iter = lib.list(&Recording::default().entity(), Some(&self.entity()));
        let iter = iter.map(|r| match r {
            Entities::Recording(r) => r,
            _ => panic!(),
        }); 
        Box::new(iter)    
    }

    pub fn genres(&self, lib: &dyn Collection) -> Box<dyn Iterator<Item = Genre>> {
        let iter = lib.list(&Genre::default().entity(), Some(&self.entity()));
        let iter = iter.map(|r| match r {
            Entities::Genre(r) => r,
            _ => panic!(),
        }); 
        Box::new(iter)    
    }
}




impl ReleaseGroup {
    pub fn entity(&self) -> Entities {
        Entities::ReleaseGroup(self.clone())
    }

    pub fn get(key: &str, lib: &dyn Collection) -> Option<Self> {
        let ent = ReleaseGroup {
            key: Some(key.to_string()),
            ..Default::default()
        }.entity();
        match lib.fetch(&ent) {
            Some(Entities::ReleaseGroup(r)) => Some(r),
            _ => todo!()
        }
    }

    pub fn list(col: &dyn Collection) -> Box<dyn Iterator<Item = ReleaseGroup>> {
        let iter = col.list(&ReleaseGroup::default().entity(), None)
            .map(|m| match m {
                Entities::ReleaseGroup(a) => a,
                _ => panic!(),
            });
        Box::new(iter)
    }

    pub fn releases(&self, lib: &dyn Collection) -> Box<dyn Iterator<Item = Release>> {
        let iter = lib.list(&Release::default().entity(), Some(&self.entity()));
        let iter = iter.map(|r| match r {
            Entities::Release(r) => r,
            _ => panic!(),
        }); 
        Box::new(iter)    
    }

    pub fn genres(&self, lib: &dyn Collection) -> Box<dyn Iterator<Item = Genre>> {
        let iter = lib.list(&Genre::default().entity(), Some(&self.entity()));
        let iter = iter.map(|r| match r {
            Entities::Genre(r) => r,
            _ => panic!(),
        }); 
        Box::new(iter)    
    }

    pub fn artists(&self, lib: &dyn Collection) -> Box<dyn Iterator<Item = Artist>> {
        let iter = lib.list(&Artist::default().entity(), Some(&self.entity()));
        let iter = iter.map(|r| match r {
            Entities::Artist(r) => r,
            _ => panic!(),
        }); 
        Box::new(iter)    
    }
}



impl Release {
    pub fn get(key: &str, lib: &dyn Collection) -> Option<Self> {
        let ent = Release {
            key: Some(key.to_string()),
            ..Default::default()
        }.entity();
        match lib.fetch(&ent) {
            Some(Entities::Release(r)) => Some(r),
            _ => todo!()
        }
    }

    pub fn entity(&self) -> Entities {
        Entities::Release(self.clone())
    }

    pub fn recordings(&self, lib: &dyn Collection) -> Box<dyn Iterator<Item = Recording>> {
        let iter = lib.list(&Recording::default().entity(), Some(&self.entity()));
        let iter = iter.map(|r| match r {
            Entities::Recording(r) => r,
            _ => panic!(),
        }); 
        Box::new(iter)    
    }

    pub fn genres(&self, lib: &dyn Collection) -> Box<dyn Iterator<Item = Genre>> {
        let iter = lib.list(&Genre::default().entity(), Some(&self.entity()));
        let iter = iter.map(|r| match r {
            Entities::Genre(r) => r,
            _ => panic!(),
        }); 
        Box::new(iter)    
    }

    pub fn artists(&self, lib: &dyn Collection) -> Box<dyn Iterator<Item = Artist>> {
        let iter = lib.list(&Artist::default().entity(), Some(&self.entity()));
        let iter = iter.map(|r| match r {
            Entities::Artist(r) => r,
            _ => panic!(),
        }); 
        Box::new(iter)    
    }
}



impl Recording {
    pub fn list(col: &dyn Collection) -> Box<dyn Iterator<Item = Recording>> {
        let iter = col.list(&Recording::default().entity(), None)
            .map(|m| match m {
                Entities::Recording(a) => a,
                _ => panic!(),
            });
        Box::new(iter)
    }

    pub fn entity(&self) -> Entities {
        Entities::Recording(self.clone())
    }

    pub fn sources(&self, lib: &dyn Collection) -> Box<dyn Iterator<Item = RecordingSource>> {
        let iter = lib.list(&RecordingSource::default().entity(), Some(&self.entity()));
        let iter = iter.map(|r| match r {
            Entities::RecordingSource(r) => r,
            _ => panic!(),
        }); 
        Box::new(iter)    
    }
}

impl RecordingSource {
    pub fn entity(&self) -> Entities {
        Entities::RecordingSource(self.clone())
    }
}



impl Genre {
    pub fn get(key: &str, lib: &dyn Collection) -> Option<Self> {
        let ent = Self {
            key: Some(key.to_string()),
            ..Default::default()
        }.entity();
        match lib.fetch(&ent) {
            Some(Entities::Genre(g)) => Some(g),
            _ => todo!()
        }
    }

    pub fn entity(&self) -> Entities {
        Entities::Genre(self.clone())
    }

    pub fn recordings(&self, lib: &dyn Collection) -> Box<dyn Iterator<Item = Recording>> {
        let iter = lib.list(&Recording::default().entity(), Some(&self.entity()));
        let iter = iter.map(|r| match r {
            Entities::Recording(r) => r,
            _ => panic!(),
        }); 
        Box::new(iter)    
    }

    pub fn releases(&self, lib: &dyn Collection) -> Box<dyn Iterator<Item = Release>> {
        let iter = lib.list(&Release::default().entity(), Some(&self.entity()));
        let iter = iter.map(|r| match r {
            Entities::Release(r) => r,
            _ => panic!(),
        }); 
        Box::new(iter)    
    }

    pub fn artists(&self, lib: &dyn Collection) -> Box<dyn Iterator<Item = Artist>> {
        let iter = lib.list(&Artist::default().entity(), Some(&self.entity()));
        let iter = iter.map(|r| match r {
            Entities::Artist(r) => r,
            _ => panic!(),
        }); 
        Box::new(iter)    
    }
}





#[derive(Clone, Debug, Serialize, Deserialize, Eq, PartialEq)]
pub enum Entities {
    Artist(Artist),
    Genre(Genre),
    ReleaseGroup(ReleaseGroup),
    Release(Release),
    Recording(Recording),
    RecordingSource(RecordingSource),
    MediaFile(MediaFile),
}

/**
 * All this repetition seems dumb. Save me, movie reference!
 */
impl Entities {
    pub fn key(&self) -> Option<String> {
        match self {
            Entities::Artist(a) => a.key.clone(),
            Entities::ReleaseGroup(r) => r.key.clone(),
            Entities::Release(r) => r.key.clone(),
            Entities::Recording(r) => r.key.clone(),
            Entities::RecordingSource(r) => r.key.clone(),
            Entities::Genre(g) => g.key.clone(),
            Entities::MediaFile(f) => f.key.clone(),
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
            Entities::MediaFile(m) => m.key = key,
        }
    }

    pub fn mbid(&self) -> Option<String> {
        let known_ids = match self {
            Entities::Artist(a) => a.known_ids.clone(),
            Entities::Release(r) => r.known_ids.clone(),
            Entities::ReleaseGroup(r) => r.known_ids.clone(),
            Entities::Recording(r) => r.known_ids.clone(),
            _ => todo!(),
        };
        for id in known_ids {
            if let KnownId::MusicBrainzId(mbid) = id {
                return Some(mbid.to_string())
            }
        }
        None
    }

    pub fn type_name(&self) -> &str {
        match self {
            Entities::Artist(_) => "artist",
            Entities::ReleaseGroup(_) => "release_group",
            Entities::Release(_) => "release",
            Entities::Recording(_) => "recording",
            Entities::RecordingSource(_) => "recording_source",
            Entities::Genre(_) => "genre",
            Entities::MediaFile(_) => "media_file",
        }
    }

    pub fn known_ids(&self) -> HashSet<KnownId> {
        match self {
            Entities::Artist(a) => a.known_ids.clone(),
            Entities::Release(a) => a.known_ids.clone(),
            Entities::ReleaseGroup(a) => a.known_ids.clone(),
            Entities::Genre(g) => g.known_ids.clone(),
            Entities::Recording(r) => r.known_ids.clone(),
            Entities::RecordingSource(r) => r.known_ids.clone(),
            Entities::MediaFile(f) => Default::default(),
        }
    }

    pub fn source_ids(&self) -> HashSet<String> {
        match self {
            Entities::Artist(a) => a.source_ids.clone(),
            Entities::Release(a) => a.source_ids.clone(),
            Entities::ReleaseGroup(a) => a.source_ids.clone(),
            Entities::Genre(g) => g.source_ids.clone(),
            Entities::Recording(r) => r.source_ids.clone(),
            Entities::RecordingSource(r) => r.source_ids.clone(),
            Entities::MediaFile(_) => Default::default(),
        }
    }

    pub fn name(&self) -> Option<String> {
        match self {
            Entities::Artist(a) => a.name.clone(),
            Entities::ReleaseGroup(r) => r.title.clone(),
            Entities::Release(r) => r.title.clone(),
            Entities::Recording(r) => r.title.clone(),
            Entities::RecordingSource(r) => r.name().clone(),
            Entities::Genre(g) => g.name.clone(),
            Entities::MediaFile(f) => Some(f.url.clone()),
        }
    }

    pub fn disambiguation(&self) -> Option<String> {
        match self {
            Entities::Artist(a) => a.disambiguation.clone(),
            Entities::ReleaseGroup(r) => r.disambiguation.clone(),
            Entities::Release(r) => r.disambiguation.clone(),
            Entities::Recording(r) => r.disambiguation.clone(),
            Entities::RecordingSource(r) => r.disambiguation().clone(),
            Entities::Genre(g) => g.disambiguation.clone(),
            Entities::MediaFile(_) => None,
        }
    }
}

pub trait Entity {
    fn key(&self) -> Option<String>;
    fn set_key(&mut self, key: Option<String>);
    fn name(&self) -> Option<String>;
    fn source_ids(&self) -> HashSet<String>;
    fn known_ids(&self) -> HashSet<KnownId>;
    fn disambiguation(&self) -> Option<String>;
    fn summary(&self) -> Option<String>;
    fn links(&self) -> HashSet<String>;
    fn similarity(&self, other: &Self) -> f32 {
        0.0
    }

    fn mbid(&self) -> Option<String> {
        self.known_ids().iter().find_map(|id| match id {
            KnownId::MusicBrainzId(mbid) => Some(mbid.to_string()),
            _ => None,
        })
    }
} 

impl Entity for Artist {
    fn key(&self) -> Option<String> {
        self.key.clone()
    }

    fn set_key(&mut self, key: Option<String>) {
        self.key = key;
    }

    fn name(&self) -> Option<String> {
        self.name.clone()
    }

    fn source_ids(&self) -> HashSet<String> {
        self.source_ids.clone()
    }

    fn known_ids(&self) -> HashSet<KnownId> {
        self.known_ids.clone()
    }

    fn disambiguation(&self) -> Option<String> {
        self.disambiguation.clone()
    }

    fn summary(&self) -> Option<String> {
        self.summary.clone()
    }

    fn links(&self) -> HashSet<String> {
        self.links.clone()
    }
}

impl Entity for Genre {
    fn key(&self) -> Option<String> {
        self.key.clone()
    }

    fn set_key(&mut self, key: Option<String>) {
        self.key = key;
    }

    fn name(&self) -> Option<String> {
        self.name.clone()
    }

    fn source_ids(&self) -> HashSet<String> {
        self.source_ids.clone()
    }

    fn known_ids(&self) -> HashSet<KnownId> {
        self.known_ids.clone()
    }

    fn disambiguation(&self) -> Option<String> {
        self.disambiguation.clone()
    }

    fn summary(&self) -> Option<String> {
        self.summary.clone()
    }

    fn links(&self) -> HashSet<String> {
        self.links.clone()
    }
}

impl Entity for Release {
    fn key(&self) -> Option<String> {
        self.key.clone()
    }

    fn set_key(&mut self, key: Option<String>) {
        self.key = key;
    }

    fn name(&self) -> Option<String> {
        self.title.clone()
    }

    fn source_ids(&self) -> HashSet<String> {
        self.source_ids.clone()
    }

    fn known_ids(&self) -> HashSet<KnownId> {
        self.known_ids.clone()
    }

    fn disambiguation(&self) -> Option<String> {
        self.disambiguation.clone()
    }

    fn summary(&self) -> Option<String> {
        self.summary.clone()
    }

    fn links(&self) -> HashSet<String> {
        self.links.clone()
    }
}

impl Entity for ReleaseGroup {
    fn key(&self) -> Option<String> {
        self.key.clone()
    }

    fn set_key(&mut self, key: Option<String>) {
        self.key = key;
    }

    fn name(&self) -> Option<String> {
        self.title.clone()
    }

    fn source_ids(&self) -> HashSet<String> {
        self.source_ids.clone()
    }

    fn known_ids(&self) -> HashSet<KnownId> {
        self.known_ids.clone()
    }

    fn disambiguation(&self) -> Option<String> {
        self.disambiguation.clone()
    }

    fn summary(&self) -> Option<String> {
        self.summary.clone()
    }

    fn links(&self) -> HashSet<String> {
        self.links.clone()
    }
}

impl Entity for Recording {
    fn key(&self) -> Option<String> {
        self.key.clone()
    }

    fn set_key(&mut self, key: Option<String>) {
        self.key = key;
    }

    fn name(&self) -> Option<String> {
        self.title.clone()
    }

    fn source_ids(&self) -> HashSet<String> {
        self.source_ids.clone()
    }

    fn known_ids(&self) -> HashSet<KnownId> {
        self.known_ids.clone()
    }

    fn disambiguation(&self) -> Option<String> {
        self.disambiguation.clone()
    }

    fn summary(&self) -> Option<String> {
        self.summary.clone()
    }

    fn links(&self) -> HashSet<String> {
        self.links.clone()
    }
}

impl Entity for RecordingSource {
    fn key(&self) -> Option<String> {
        self.key.clone()
    }

    fn set_key(&mut self, key: Option<String>) {
        self.key = key;
    }

    fn name(&self) -> Option<String> {
        Some(format!("{:?}", self))
    }

    fn source_ids(&self) -> HashSet<String> {
        self.source_ids.clone()
    }

    fn known_ids(&self) -> HashSet<KnownId> {
        self.known_ids.clone()
    }

    fn disambiguation(&self) -> Option<String> {
        Default::default()
    }

    fn summary(&self) -> Option<String> {
        Default::default()
    }

    fn links(&self) -> HashSet<String> {
        Default::default()
    }
}
