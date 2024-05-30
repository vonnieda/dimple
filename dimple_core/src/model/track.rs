use dimple_core_macro::ModelSupport;
use serde::Deserialize;
use serde::Serialize;

use super::ArtistCredit;
use super::Genre;
use super::KnownIds;
use super::Medium;
use super::Recording;

// https://musicbrainz.org/doc/Track
// > This entity is not visible to users on its own, only in the context of a
// release. It has an MBID, and contains a link to a recording, a title, 
// artist credit and position on its associated medium. 
// In the schema image it also has a medium (ref)
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, Default, ModelSupport)]
pub struct Track {
    pub key: Option<String>,
    pub title: Option<String>,
    pub known_ids: KnownIds,

    pub length: Option<u32>,
    pub number: Option<u32>,
    pub position: Option<u32>,

    #[serde(skip)]
    pub recording: Recording,
    #[serde(skip)]
    pub genres: Vec<Genre>,
    #[serde(skip)]
    pub artist_credits: Vec<ArtistCredit>,
}
