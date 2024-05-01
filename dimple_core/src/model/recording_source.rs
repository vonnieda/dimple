use dimple_core_macro::ModelSupport;
use serde::Deserialize;
use serde::Serialize;

use crate::db::Db;

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
}

impl RecordingSource {
    // TODO this really exposes the need for a real database interface. whine.
    pub fn find_by_source_id(db: &dyn Db, source_id: &str) -> Option<RecordingSource> {
        db.list(&Self::default().model(), None).unwrap()
            .map(Into::<RecordingSource>::into)
            .find(|rec_source| rec_source.source_id == source_id)
    }
}