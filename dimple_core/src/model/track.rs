use dimple_core_macro::ModelSupport;
use serde::Deserialize;
use serde::Serialize;

use crate::model::RecordingSource;

use crate::model::Recording;

use super::Model;

// https://musicbrainz.org/doc/Track
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, Default, ModelSupport)]
pub struct Track {
    pub key: Option<String>,

    pub title: String,

    pub length: u32,
    pub number: String,
    pub position: u32,
    pub recording: Recording,
    pub sources: Vec<RecordingSource>,
}
