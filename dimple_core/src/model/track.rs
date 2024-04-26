use dimple_core_macro::ModelSupport;
use serde::Deserialize;
use serde::Serialize;

// https://musicbrainz.org/doc/Track
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, Default, ModelSupport)]
pub struct Track {
    pub key: Option<String>,

    pub title: Option<String>,

    pub length: Option<u32>,
    pub number: Option<String>,
    pub position: Option<u32>,
}
