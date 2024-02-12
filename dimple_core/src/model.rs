use std::collections::HashSet;
use std::mem::discriminant;

use image::DynamicImage;
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
    pub date: Option<String>, // TODO should be Instant but need to think about serialization
    pub packaging: Option<String>,
    pub status: Option<String>,
}




// // https://musicbrainz.org/doc/Medium
// #[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, Default)]
// pub struct Medium {
//     pub title: String,

//     pub disc_count: u32,
//     pub format: String,
//     pub position: u32,
//     pub track_count: u32,
//     pub tracks: Vec<Track>,
// }




// // https://musicbrainz.org/doc/Track
// #[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, Default)]
// pub struct Track {
//     pub key: String,
//     pub title: String,

//     pub length: u32,
//     pub number: String,
//     pub position: u32,
//     pub recording: Recording,
// }




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




#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct RecordingSource {
    pub key: Option<String>,
    pub source_ids: HashSet<String>,
    pub known_ids: HashSet<KnownId>,
}




#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct Genre {
    pub key: Option<String>,

    pub name: String,
    pub count: u32,
    pub summary: String,
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



#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum KnownId {
    MusicBrainzId(String),
    DiscogsId,
    LastFmId,
    WikidataId,
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
                Model::Artist(a) => a,
                _ => panic!(),
            });
        Box::new(iter)
    }

    // pub fn get(id: &str, lib: &dyn Collection) -> Option<Self> {
    //     match lib.fetch(&Model::Artist(Self::from_id(id))) {
    //         Some(Model::Artist(o)) => Some(o),
    //         _ => todo!()
    //     }
    // }

    // pub fn fetch(&self, lib: &dyn Collection) -> Option<Self> {
    //     Self::get(&self.key, lib)
    // }

    pub fn entity(&self) -> Model {
        Model::Artist(self.clone())
    }

    pub fn release_groups(&self, lib: &dyn Collection) -> Box<dyn Iterator<Item = ReleaseGroup>> {
        let iter = lib.list(&ReleaseGroup::default().entity(), Some(&self.entity()));
        let iter = iter.map(|r| match r {
            Model::ReleaseGroup(r) => r,
            _ => panic!(),
        }); 
        Box::new(iter)    
    }

    pub fn releases(&self, lib: &dyn Collection) -> Box<dyn Iterator<Item = Release>> {
        let iter = lib.list(&Release::default().entity(), Some(&self.entity()));
        let iter = iter.map(|r| match r {
            Model::Release(r) => r,
            _ => panic!(),
        }); 
        Box::new(iter)    
    }

    pub fn search(query: &str, lib: &dyn Collection) -> Box<dyn Iterator<Item = Artist>> {
        let iter = lib.search(query)
            .filter_map(|m| match m {
                Model::Artist(a) => Some(a),
                _ => None,
            });
        Box::new(iter)
    }
    
    pub fn known_id(&self, of_type: &KnownId) -> Option<KnownId> {
        for id in &self.known_ids {
            if discriminant(id) == discriminant(of_type) {
                return Some(id.clone())
            }
        }
        None
    }
}




impl Release {
    // pub fn from_id(id: &str) -> Self {
    //     Self {
    //         key: id.to_string(),
    //         ..Default::default()
    //     }
    // }

    // pub fn get(id: &str, lib: &dyn Collection) -> Option<Self> {
    //     match lib.fetch(&Model::Release(Self::from_id(id))) {
    //         Some(Model::Release(o)) => Some(o),
    //         _ => todo!()
    //     }
    // }

    // pub fn fetch(&self, lib: &dyn Collection) -> Option<Self> {
    //     Self::get(&self.key, lib)
    // }

    pub fn entity(&self) -> Model {
        Model::Release(self.clone())
    }

    pub fn recordings(&self, lib: &dyn Collection) -> Box<dyn Iterator<Item = Recording>> {
        let iter = lib.list(&Recording::default().entity(), Some(&self.entity()));
        let iter = iter.map(|r| match r {
            Model::Recording(r) => r,
            _ => panic!(),
        }); 
        Box::new(iter)    
    }

    pub fn known_id(&self, of_type: &KnownId) -> Option<KnownId> {
        for id in &self.known_ids {
            if discriminant(id) == discriminant(of_type) {
                return Some(id.clone())
            }
        }
        None
    }
}




impl ReleaseGroup {
    // pub fn from_id(id: &str) -> Self {
    //     Self {
    //         key: id.to_string(),
    //         ..Default::default()
    //     }
    // }

    // pub fn get(id: &str, lib: &dyn Collection) -> Option<Self> {
    //     match lib.fetch(&Model::ReleaseGroup(Self::from_id(id))) {
    //         Some(Model::ReleaseGroup(o)) => Some(o),
    //         _ => todo!()
    //     }
    // }

    // pub fn fetch(&self, lib: &dyn Collection) -> Option<Self> {
    //     Self::get(&self.key, lib)
    // }

    pub fn entity(&self) -> Model {
        Model::ReleaseGroup(self.clone())
    }

    pub fn image(&self, lib: &dyn Collection) -> Option<DynamicImage> {
        lib.image(&self.entity())
    }
}




impl Recording {
    // pub fn from_id(id: &str) -> Self {
    //     Self {
    //         key: id.to_string(),
    //         ..Default::default()
    //     }
    // }

    pub fn list(col: &dyn Collection) -> Box<dyn Iterator<Item = Recording>> {
        let iter = col.list(&Recording::default().entity(), None)
            .map(|m| match m {
                Model::Recording(a) => a,
                _ => panic!(),
            });
        Box::new(iter)
    }

    // pub fn get(id: &str, lib: &dyn Collection) -> Option<Self> {
    //     match lib.fetch(&Model::Recording(Self::from_id(id))) {
    //         Some(Model::Recording(o)) => Some(o),
    //         _ => todo!()
    //     }
    // }

    pub fn entity(&self) -> Model {
        Model::Recording(self.clone())
    }

    // pub fn fetch(&self, lib: &dyn Collection) -> Option<Self> {
    //     Self::get(&self.key, lib)
    // }

    pub fn sources(&self, lib: &dyn Collection) -> Box<dyn Iterator<Item = RecordingSource>> {
        let iter = lib.list(&RecordingSource::default().entity(), Some(&self.entity()));
        let iter = iter.map(|r| match r {
            Model::RecordingSource(r) => r,
            _ => panic!(),
        }); 
        Box::new(iter)    
    }

    pub fn known_id(&self, of_type: &KnownId) -> Option<KnownId> {
        for id in &self.known_ids {
            if discriminant(id) == discriminant(of_type) {
                return Some(id.clone())
            }
        }
        None
    }
}

impl RecordingSource {
    pub fn entity(&self) -> Model {
        Model::RecordingSource(self.clone())
    }
}

#[derive(Clone, Debug)]
pub enum Model {
    Artist(Artist),
    Genre(Genre),
    ReleaseGroup(ReleaseGroup),
    Release(Release),
    Recording(Recording),
    RecordingSource(RecordingSource),
}


impl Model {
    pub fn key(&self) -> Option<String> {
        match self {
            Model::Artist(a) => a.key.clone(),
            Model::ReleaseGroup(r) => r.key.clone(),
            Model::Release(r) => r.key.clone(),
            Model::Recording(r) => r.key.clone(),
            Model::RecordingSource(r) => r.key.clone(),
            Model::Genre(g) => g.key.clone(),
        }
    }

    pub fn mbid(&self) -> Option<String> {
        let known_ids = match self {
            Model::Artist(a) => a.known_ids.clone(),
            Model::Release(r) => r.known_ids.clone(),
            Model::ReleaseGroup(r) => r.known_ids.clone(),
            Model::Recording(r) => r.known_ids.clone(),
            _ => todo!(),
        };
        for id in known_ids {
            if let KnownId::MusicBrainzId(mbid) = id {
                return Some(mbid.to_string())
            }
        }
        None
    }
}
