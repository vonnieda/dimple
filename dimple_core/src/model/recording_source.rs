use std::collections::HashSet;

use dimple_core_macro::ModelSupport;
use serde::Deserialize;
use serde::Serialize;

use crate::model::KnownId;

use super::Model;

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum RecordingFormat {
    MP3,
    FLAC,
    M4A,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, Default, ModelSupport)]
pub struct RecordingSource {
    pub key: Option<String>,
    pub source_ids: HashSet<String>,
    pub known_ids: HashSet<KnownId>,
    pub format: Option<RecordingFormat>,
    pub extension: Option<String>,
}
