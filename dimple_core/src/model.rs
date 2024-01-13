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

    // TODO do we actually need options here?
    // One benefit is it's easier to serde smaller objects.
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
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Default)]
pub struct DimpleReleaseGroup {
    pub id: String,
    pub title: String,
}

#[derive(Default, Debug, Clone, Serialize, Eq, Hash, PartialEq, Deserialize)]
pub struct DimpleTrack {
}

impl DimpleArtist {
    pub fn mbid(&self) -> String {
        self.id.clone()
    }
}

impl DimpleReleaseGroup {
    pub fn mbid(&self) -> String {
        self.id.clone()
    }
}