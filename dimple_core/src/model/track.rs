use serde::Deserialize;
use serde::Serialize;

use crate::collection::Collection;
use crate::model::Entity;
use crate::model::RecordingSource;

use crate::model::Recording;

use super::Entities;

// https://musicbrainz.org/doc/Track
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct Track {
    pub key: Option<String>,

    pub title: String,

    pub length: u32,
    pub number: String,
    pub position: u32,
    pub recording: Recording,
    pub sources: Vec<RecordingSource>,
}

impl Track {
    pub fn list(col: &dyn Collection) -> Box<dyn Iterator<Item = Track>> {
        let iter = col
            .list(&Track::default().entity(), None)
            .map(|m| match m {
                Entities::Track(m) => m,
                _ => panic!(),
            });
        Box::new(iter)
    }
}

impl Entity for Track {
    fn key(&self) -> Option<String> {
        self.key.clone()
    }

    fn set_key(&mut self, key: Option<String>) {
        self.key = key;
    }

    fn entity(&self) -> Entities {
        Entities::Track(self.clone())
    }
}

