use serde::Deserialize;
use serde::Serialize;

// TODO feels more like attributed things are their own objects.

/// References
/// https://musicbrainz.org/doc/Artist
/// https://picard-docs.musicbrainz.org/en/appendices/tag_mapping.html
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Default)]
pub struct DimpleArtist {
    pub id: String,
    pub name: String,
    pub disambiguation: String,
    pub summary: Option<Attributed<String>>,
    pub release_groups: Option<Vec<DimpleReleaseGroup>>,
    pub releases: Option<Vec<DimpleRelease>>,
    pub relations: Option<Vec<DimpleRelation>>,
    pub genres: Option<Vec<DimpleGenre>>,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Default)]
#[serde(default)]
pub struct DimpleReleaseGroup {
    pub id: String,
    pub title: String,
    pub disambiguation: String,
    pub summary: Option<Attributed<String>>,
    pub primary_type: String,
    pub first_release_date: String,
    pub relations: Option<Vec<DimpleRelation>>,
    pub genres: Option<Vec<DimpleGenre>>,
    pub releases: Option<Vec<DimpleRelease>>,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Default)]
#[serde(default)]
pub struct DimpleRelease {
    pub id: String,
    pub title: String,
    pub disambiguation: String,
    pub summary: Option<Attributed<String>>,
    pub primary_type: String,
    pub first_release_date: String,
    pub relations: Option<Vec<DimpleRelation>>,
    pub genres: Option<Vec<DimpleGenre>>,
    // pub releases: Option<Vec<DimpleReleaseGroup>>,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Default)]
pub struct DimpleGenre {
    // pub id: String,
    pub name: String,
    pub count: u32,
    pub description: Option<Attributed<String>>,
}

#[derive(Default, Debug, Clone, Serialize, Eq, Hash, PartialEq, Deserialize)]
pub struct DimpleTrack {
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
/// TODO just move all this into the Attributed
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Default)]
pub struct Attributed<T> {
    pub value: T,

    pub text: String,
    pub url: String,
    pub license: String,
    pub copyright_holder: String,
}