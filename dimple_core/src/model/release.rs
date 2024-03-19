use std::collections::HashSet;


use serde::Deserialize;
use serde::Serialize;

use crate::model::KnownId;
use crate::model::Entity;

use crate::model::Artist;
use crate::model::Recording;
use crate::model::Genre;
use crate::collection::Collection;

use super::Entities;


// https://musicbrainz.org/doc/Release
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(default)]
pub struct Release {
    pub key: Option<String>,
    pub title: Option<String>,
    pub source_ids: HashSet<String>,
    pub known_ids: HashSet<KnownId>,
    pub disambiguation: Option<String>,
    pub summary: Option<String>,
    pub links: HashSet<String>,

    pub barcode: Option<String>,
    pub country: Option<String>,
    pub date: Option<String>, // TODO should be chronos, probably.
    pub packaging: Option<String>,
    pub status: Option<String>,
}

impl Release {
    pub fn get(key: &str, lib: &dyn Collection) -> Option<Self> {
        let ent = Release {
            key: Some(key.to_string()),
            ..Default::default()
        }.entity();
        match lib.fetch(&ent) {
            Some(Entities::Release(r)) => Some(r),
            _ => todo!()
        }
    }

    pub fn list(col: &dyn Collection) -> Box<dyn Iterator<Item = Release>> {
        let iter = col
            .list(&Release::default().entity(), None)
            .map(|m| match m {
                Entities::Release(m) => m,
                _ => panic!(),
            });
        Box::new(iter)
    }

    pub fn recordings(&self, lib: &dyn Collection) -> Box<dyn Iterator<Item = Recording>> {
        let iter = lib.list(&Recording::default().entity(), Some(&self.entity()));
        let iter = iter.map(|r| match r {
            Entities::Recording(r) => r,
            _ => panic!(),
        }); 
        Box::new(iter)    
    }

    pub fn genres(&self, lib: &dyn Collection) -> Box<dyn Iterator<Item = Genre>> {
        let iter = lib.list(&Genre::default().entity(), Some(&self.entity()));
        let iter = iter.map(|r| match r {
            Entities::Genre(r) => r,
            _ => panic!(),
        }); 
        Box::new(iter)    
    }

    pub fn artists(&self, lib: &dyn Collection) -> Box<dyn Iterator<Item = Artist>> {
        let iter = lib.list(&Artist::default().entity(), Some(&self.entity()));
        let iter = iter.map(|r| match r {
            Entities::Artist(r) => r,
            _ => panic!(),
        }); 
        Box::new(iter)    
    }
}

impl Entity for Release {
    fn key(&self) -> Option<String> {
        self.key.clone()
    }

    fn set_key(&mut self, key: Option<String>) {
        self.key = key;
    }

    fn entity(&self) -> Entities {
        Entities::Release(self.clone())
    }   
}
