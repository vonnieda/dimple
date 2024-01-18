use std::error::Error;

use image::DynamicImage;
use serde::Deserialize;
use serde::Serialize;

use crate::library::Library;
use crate::library::LibraryEntity;

// TODO feels more like attributed things are their own objects and not fields
// on structs that may also be attributed.

/// References
/// https://musicbrainz.org/doc/Artist
/// https://picard-docs.musicbrainz.org/en/appendices/tag_mapping.html
/// These objects all closely map Musicbrainz objects and were heavily
/// lifted from musicbrainz_rs. 

// ReleseGroup -> Release -> Media -> Track -> Recording


// https://musicbrainz.org/doc/Artist
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Default)]
pub struct DimpleArtist {
    pub id: String,
    pub name: String,
    pub disambiguation: String,
    pub summary: String,
    pub release_groups: Vec<DimpleReleaseGroup>,
    pub relations: Vec<DimpleRelation>,
    pub genres: Vec<DimpleGenre>,
    #[serde(default)]
    pub fetched: bool,
}

// https://musicbrainz.org/doc/ReleaseGroup
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Default)]
#[serde(default)]
pub struct DimpleReleaseGroup {
    pub id: String,
    pub title: String,
    pub disambiguation: String,
    pub summary: String,
    pub primary_type: String,
    pub first_release_date: String,
    pub relations: Vec<DimpleRelation>,
    pub genres: Vec<DimpleGenre>,
    pub releases: Vec<DimpleRelease>,
    pub artists: Vec<DimpleArtist>,
    #[serde(default)]
    pub fetched: bool,
}

// https://musicbrainz.org/doc/Release
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Default)]
#[serde(default)]
pub struct DimpleRelease {
    pub id: String,
    pub title: String,
    pub disambiguation: String,
    pub summary: String,
    pub relations: Vec<DimpleRelation>,
    pub genres: Vec<DimpleGenre>,
    pub artists: Vec<DimpleArtist>,
    pub status: String,
    pub date: String,
    pub packaging: String,
    pub country: String,
    pub barcode: String,
    pub asin: String,
    pub release_group: DimpleReleaseGroup,
    #[serde(default)]
    pub fetched: bool,
    pub media: Vec<DimpleMedium>,
}

// https://musicbrainz.org/doc/Medium
#[derive(Default, Debug, Clone, Serialize, Eq, Hash, PartialEq, Deserialize)]
pub struct DimpleMedium {
    pub title: String,
    pub position: u32,
    pub track_count: u32,
    pub disc_count: u32,
    pub format_id: String,
    pub format: String,
    pub tracks: Vec<DimpleTrack>,
    #[serde(default)]
    pub fetched: bool,
}

// https://musicbrainz.org/doc/Track
#[derive(Default, Debug, Clone, Serialize, Eq, Hash, PartialEq, Deserialize)]
pub struct DimpleTrack {
    pub recording: DimpleRecording,
    pub title: String,
    pub number: String,
    pub length: u32,
    pub position: u32,
    pub id: String,
    #[serde(default)]
    pub fetched: bool,
}

// https://musicbrainz.org/doc/Track
#[derive(Default, Debug, Clone, Serialize, Eq, Hash, PartialEq, Deserialize)]
pub struct DimpleRecording {
    pub id: String,
    pub title: String,
    pub length: u32,
    pub disambiguation: String,
    pub annotation: String,
    #[serde(default)]
    pub fetched: bool,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Default)]
pub struct DimpleGenre {
    // pub id: String,
    pub name: String,
    pub count: u32,
    pub summary: String,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct DimpleRelation {
    pub content: DimpleRelationContent,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub enum DimpleRelationContent {
    Url(DimpleUrl),
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Default)]
pub struct DimpleUrl {
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

impl DimpleArtist {
    pub fn from_id(id: &str) -> Self {
        Self {
            id: id.to_string(),
            ..Default::default()
        }
    }

    pub fn get(id: &str, lib: &dyn Library) -> Option<Self> {
        match lib.fetch(&LibraryEntity::Artist(Self::from_id(id))) {
            Some(LibraryEntity::Artist(o)) => Some(o),
            _ => todo!()
        }
    }

    pub fn fetch(&self, lib: &dyn Library) -> Option<Self> {
        Self::get(&self.id, lib)
    }
}

impl DimpleRelease {
    pub fn from_id(id: &str) -> Self {
        Self {
            id: id.to_string(),
            ..Default::default()
        }
    }

    pub fn get(id: &str, lib: &dyn Library) -> Option<Self> {
        match lib.fetch(&LibraryEntity::Release(Self::from_id(id))) {
            Some(LibraryEntity::Release(o)) => Some(o),
            _ => todo!()
        }
    }

    pub fn fetch(&self, lib: &dyn Library) -> Option<Self> {
        Self::get(&self.id, lib)
    }
}

impl DimpleReleaseGroup {
    pub fn from_id(id: &str) -> Self {
        Self {
            id: id.to_string(),
            ..Default::default()
        }
    }

    pub fn get(id: &str, lib: &dyn Library) -> Option<Self> {
        match lib.fetch(&LibraryEntity::ReleaseGroup(Self::from_id(id))) {
            Some(LibraryEntity::ReleaseGroup(o)) => Some(o),
            _ => todo!()
        }
    }

    pub fn fetch(&self, lib: &dyn Library) -> Option<Self> {
        Self::get(&self.id, lib)
    }

    pub fn entity(&self) -> LibraryEntity {
        LibraryEntity::ReleaseGroup(self.clone())
    }

    pub fn image(&self, lib: &dyn Library) -> Option<DynamicImage> {
        lib.image(&self.entity())
    }
}

