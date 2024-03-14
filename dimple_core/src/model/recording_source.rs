use std::collections::HashSet;


use serde::Deserialize;
use serde::Serialize;

use crate::model::KnownId;

use crate::model::Entity;






use super::Entities;

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum RecordingFormat {
    MP3,
    FLAC,
    M4A,
}


#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct RecordingSource {
    pub key: Option<String>,
    pub source_ids: HashSet<String>,
    pub known_ids: HashSet<KnownId>,
    pub format: Option<RecordingFormat>,
    pub extension: Option<String>,
}

impl Entity for RecordingSource {
    fn key(&self) -> Option<String> {
        self.key.clone()
    }

    fn set_key(&mut self, key: Option<String>) {
        self.key = key;
    }

    fn entity(&self) -> Entities {
        Entities::RecordingSource(self.clone())
    }    
}
