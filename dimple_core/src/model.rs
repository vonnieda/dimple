use serde::Deserialize;
use serde::Serialize;

/// References
/// https://musicbrainz.org/doc/Artist
/// https://picard-docs.musicbrainz.org/en/appendices/tag_mapping.html
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Default)]
pub struct DimpleArtist {
    pub id: String,
    pub name: String,
    pub disambiguation: String,
    pub bio: Option<Attributed<String>>,

    // TODO do we actually need options here?
    // One benefit is it's easier to serde smaller objects.
    // But an empty vec is nothing
    // And also, I think I'd generally treat and empty vec and a
    // None the same, so why add the additional bs?
    pub release_groups: Option<Vec<DimpleReleaseGroup>>,
    pub relations: Option<Vec<DimpleRelation>>,
    pub genres: Option<Vec<DimpleGenre>>,
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

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Default)]
pub struct DimpleGenre {
    // pub id: String,
    pub name: String,
    pub count: u32,
    pub description: Option<Attributed<String>>,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Default)]
pub struct DimpleReleaseGroup {
    pub id: String,
    pub title: String,
}

#[derive(Default, Debug, Clone, Serialize, Eq, Hash, PartialEq, Deserialize)]
pub struct DimpleTrack {
}

/// An attribution for a piece of data. Indicates where the data was sourced,
/// who owns it, and under what license it is being used.
/// DimpleAttribution {
///     text: "Wikipedia content provided under the terms of the Creative Commons BY-SA license",
///     url: "https://en.wikipedia.org/wiki/Brutus_%28Belgian_band%29"
///     license: "CC-BY-SA"
///     copyright_holder: "WikiCommons"
/// }
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Default)]
pub struct DimpleAttribution {
    pub text: String,
    pub url: String,
    pub license: String,
    pub copyright_holder: String,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Default)]
pub struct Attributed<T> {
    pub value: T,
    pub attribution: DimpleAttribution,
}