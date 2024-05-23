use dimple_core_macro::ModelSupport;
use serde::Deserialize;
use serde::Serialize;

use super::Release;
use super::Track;

// https://musicbrainz.org/doc/Medium
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, Default, ModelSupport)]
pub struct Medium {
    pub key: Option<String>,

    pub title: Option<String>,

    pub disc_count: Option<u32>,
    pub format: Option<String>,
    pub position: Option<u32>,
    pub track_count: Option<u32>,

    #[serde(skip)]
    pub tracks: Vec<Track>,
}
