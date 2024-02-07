use image::DynamicImage;
use serde::Deserialize;
use serde::Serialize;

use crate::collection::Collection;

/// References
/// https://musicbrainz.org/doc/Artist
/// https://picard-docs.musicbrainz.org/en/appendices/tag_mapping.html




// https://musicbrainz.org/doc/Artist
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Default)]
pub struct Artist {
    pub key: String,
    pub name: Option<String>,

    pub disambiguation: Option<String>,
    pub summary: Option<String>,
    pub genres: Vec<Genre>,
    pub relations: Vec<Relation>,
}




// https://musicbrainz.org/doc/ReleaseGroup
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Default)]
#[serde(default)]
pub struct ReleaseGroup {
    pub key: String,
    pub title: String,

    pub artists: Vec<Artist>,
    pub disambiguation: String,
    pub first_release_date: String,
    pub genres: Vec<Genre>,
    pub primary_type: String,
    pub relations: Vec<Relation>,
    pub releases: Vec<Release>,
    pub summary: String,
}




// https://musicbrainz.org/doc/Release
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Default)]
#[serde(default)]
pub struct Release {
    pub key: String,
    pub title: String,

    pub artists: Vec<Artist>,
    pub barcode: String,
    pub country: String,
    pub date: String,
    pub disambiguation: String,
    pub genres: Vec<Genre>,
    pub media: Vec<Medium>,
    pub packaging: String,
    pub relations: Vec<Relation>,
    pub release_group: ReleaseGroup,
    pub status: String,
    pub summary: String,
}




// https://musicbrainz.org/doc/Medium
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Default)]
pub struct Medium {
    pub title: String,

    pub disc_count: u32,
    pub format: String,
    pub position: u32,
    pub track_count: u32,
    pub tracks: Vec<Track>,
}




// https://musicbrainz.org/doc/Track
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Default)]
pub struct Track {
    pub key: String,
    pub title: String,

    pub length: u32,
    pub number: String,
    pub position: u32,
    pub recording: Recording,
}




// https://musicbrainz.org/doc/Recording
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Default)]
pub struct Recording {
    pub key: String,
    pub title: String,

    pub annotation: String,
    pub disambiguation: String,
    pub length: u32,
    pub summary: String,

    pub isrcs: Vec<String>,
    pub relations: Vec<Relation>,
    pub releases: Vec<Release>,
    pub artist_credits: Vec<Artist>,

    pub genres: Vec<Genre>,
}




#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Default)]
pub struct RecordingSource {
    pub recording_id: String,
}




#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Default)]
pub struct Genre {
    pub key: String,

    pub name: String,
    pub count: u32,
    pub summary: String,

    #[serde(default)]
    pub fetched: bool,
}




#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Default)]
pub struct Playlist {
    pub name: String,
}




#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct Relation {
    pub content: RelationContent,
}




#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub enum RelationContent {
    Url(UrlRelation),
}




#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Default)]
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




impl Artist {
    pub fn from_id(id: &str) -> Self {
        Self {
            key: id.to_string(),
            ..Default::default()
        }
    }

    pub fn list(col: &dyn Collection) -> Box<dyn Iterator<Item = Artist>> {
        let iter = col.list(&Artist::default().entity(), None)
            .map(|m| match m {
                Model::Artist(a) => a,
                _ => panic!(),
            });
        Box::new(iter)
    }

    pub fn get(id: &str, lib: &dyn Collection) -> Option<Self> {
        match lib.fetch(&Model::Artist(Self::from_id(id))) {
            Some(Model::Artist(o)) => Some(o),
            _ => todo!()
        }
    }

    pub fn fetch(&self, lib: &dyn Collection) -> Option<Self> {
        Self::get(&self.key, lib)
    }

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
}




impl Release {
    pub fn from_id(id: &str) -> Self {
        Self {
            key: id.to_string(),
            ..Default::default()
        }
    }

    pub fn get(id: &str, lib: &dyn Collection) -> Option<Self> {
        match lib.fetch(&Model::Release(Self::from_id(id))) {
            Some(Model::Release(o)) => Some(o),
            _ => todo!()
        }
    }

    pub fn fetch(&self, lib: &dyn Collection) -> Option<Self> {
        Self::get(&self.key, lib)
    }
}




impl ReleaseGroup {
    pub fn from_id(id: &str) -> Self {
        Self {
            key: id.to_string(),
            ..Default::default()
        }
    }

    pub fn get(id: &str, lib: &dyn Collection) -> Option<Self> {
        match lib.fetch(&Model::ReleaseGroup(Self::from_id(id))) {
            Some(Model::ReleaseGroup(o)) => Some(o),
            _ => todo!()
        }
    }

    pub fn fetch(&self, lib: &dyn Collection) -> Option<Self> {
        Self::get(&self.key, lib)
    }

    pub fn entity(&self) -> Model {
        Model::ReleaseGroup(self.clone())
    }

    pub fn image(&self, lib: &dyn Collection) -> Option<DynamicImage> {
        lib.image(&self.entity())
    }
}




impl Recording {
    pub fn from_id(id: &str) -> Self {
        Self {
            key: id.to_string(),
            ..Default::default()
        }
    }

    pub fn list(col: &dyn Collection) -> Box<dyn Iterator<Item = Recording>> {
        let iter = col.list(&Recording::default().entity(), None)
            .map(|m| match m {
                Model::Recording(a) => a,
                _ => panic!(),
            });
        Box::new(iter)
    }

    pub fn get(id: &str, lib: &dyn Collection) -> Option<Self> {
        match lib.fetch(&Model::Recording(Self::from_id(id))) {
            Some(Model::Recording(o)) => Some(o),
            _ => todo!()
        }
    }

    pub fn entity(&self) -> Model {
        Model::Recording(self.clone())
    }

    pub fn fetch(&self, lib: &dyn Collection) -> Option<Self> {
        Self::get(&self.key, lib)
    }
}


#[derive(Clone, Debug)]
pub enum Model {
    Artist(Artist),
    Genre(Genre),
    ReleaseGroup(ReleaseGroup),
    Release(Release),
    Recording(Recording),
}


impl Model {
    pub fn key(&self) -> String {
        match self {
            Model::Artist(a) => a.key.clone(),
            Model::ReleaseGroup(r) => r.key.clone(),
            Model::Release(r) => r.key.clone(),
            Model::Recording(r) => r.key.clone(),
            Model::Genre(g) => g.name.clone(),
        }
    }
}
