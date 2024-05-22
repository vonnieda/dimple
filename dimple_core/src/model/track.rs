use dimple_core_macro::ModelSupport;
use serde::Deserialize;
use serde::Serialize;

use super::KnownIds;
use super::Recording;

// https://musicbrainz.org/doc/Track
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, Default, ModelSupport)]
pub struct Track {
    pub key: Option<String>,
    pub title: Option<String>,
    pub known_ids: KnownIds,

    pub length: Option<u32>,
    pub number: Option<u32>,
    pub position: Option<u32>,

    pub recording: Option<Recording>,
}
