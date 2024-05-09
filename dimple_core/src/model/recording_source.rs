use std::time::SystemTime;

use dimple_core_macro::ModelSupport;
use serde::Deserialize;
use serde::Serialize;

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum RecordingFormat {
    MP3,
    FLAC,
    M4A,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, Default, ModelSupport)]
pub struct RecordingSource {
    pub key: Option<String>,
    pub source_id: String,
    pub format: Option<RecordingFormat>,
    pub extension: Option<String>,
    pub last_modified: Option<SystemTime>,
}
