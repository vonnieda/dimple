use std::collections::HashSet;


use serde::Deserialize;
use serde::Serialize;

use crate::model::KnownId;

use crate::model::Entity;

use crate::model::RecordingSource;


use crate::collection::Collection;

use super::Entities;

// https://musicbrainz.org/doc/Recording
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct Recording {
    pub key: Option<String>,
    pub title: Option<String>,
    pub source_ids: HashSet<String>,
    pub known_ids: HashSet<KnownId>,
    pub disambiguation: Option<String>,
    pub summary: Option<String>,
    pub links: HashSet<String>,

    pub annotation: Option<String>,
    pub length: Option<u32>,

    pub isrcs: HashSet<String>,
}

impl Recording {
    pub fn list(col: &dyn Collection) -> Box<dyn Iterator<Item = Recording>> {
        let iter = col.list(&Recording::default().entity(), None)
            .map(|m| match m {
                Entities::Recording(a) => a,
                _ => panic!(),
            });
        Box::new(iter)
    }

    pub fn entity(&self) -> Entities {
        Entities::Recording(self.clone())
    }

    pub fn sources(&self, lib: &dyn Collection) -> Box<dyn Iterator<Item = RecordingSource>> {
        let iter = lib.list(&RecordingSource::default().entity(), Some(&self.entity()));
        let iter = iter.map(|r| match r {
            Entities::RecordingSource(r) => r,
            _ => panic!(),
        }); 
        Box::new(iter)    
    }
}

impl Entity for Recording {
    fn key(&self) -> Option<String> {
        self.key.clone()
    }

    fn set_key(&mut self, key: Option<String>) {
        self.key = key;
    }

    fn entity(&self) -> Entities {
        Entities::Recording(self.clone())
    }    
}

